//! `aih` — local developer CLI for the ai-heeczer scoring core. See ADR-0010.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use heeczer_core::{
    schema::{EventValidator, Mode, ProfileValidator},
    score, Event, ScoringProfile, TierSet, SCORING_VERSION, SPEC_VERSION,
};
use include_dir::{include_dir, Dir};

/// Bundled fixture tree, embedded at compile time (PRD §12.21, ADR-0010).
/// The path is relative to this Cargo manifest. Using a manifest-relative
/// include keeps `cargo install --path core/heeczer-cli` working.
static FIXTURES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../schema/fixtures");

/// Hard upper bound for any single JSON document the CLI accepts. Prevents an
/// unbounded stdin from turning into an OOM. The same constant should be
/// honored by the future ingestion service (plan 0004).
const MAX_INPUT_BYTES: u64 = 1024 * 1024;

fn validator() -> &'static EventValidator {
    static V: OnceLock<EventValidator> = OnceLock::new();
    V.get_or_init(EventValidator::new_v1)
}

fn profile_validator() -> &'static ProfileValidator {
    static V: OnceLock<ProfileValidator> = OnceLock::new();
    V.get_or_init(ProfileValidator::new_v1)
}

#[derive(Debug, Parser)]
#[command(name = "aih", version, about = "ai-heeczer local developer CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate or score against the canonical schema.
    Schema {
        #[command(subcommand)]
        sub: SchemaCmd,
    },
    /// Run the scoring engine on an event.
    Score(ScoreArgs),
    /// Inspect bundled golden fixtures.
    Fixtures {
        #[command(subcommand)]
        sub: FixturesCmd,
    },
    /// Diff two ScoreResult JSONs and exit non-zero if they differ.
    Diff { a: PathBuf, b: PathBuf },
    /// Apply storage migrations.
    Migrate {
        #[command(subcommand)]
        sub: MigrateCmd,
    },
    /// Validate a scoring profile or tier set against its JSON schema.
    Validate {
        #[command(subcommand)]
        sub: ValidateCmd,
    },
    /// Benchmark `score()` over N iterations of a fixture event.
    Bench(BenchArgs),
    /// Read-only replay: re-score a persisted event and diff against the
    /// latest persisted score row. Never inserts a new score row
    /// (row-inserting replays live in the dashboard test-orchestration view
    /// per ADR-0012).
    Replay(ReplayArgs),
    /// Print engine and CLI version metadata.
    Version,
}

#[derive(Debug, Subcommand)]
enum SchemaCmd {
    /// Validate a JSON event against `event.v1.json`. Pass `-` to read stdin.
    Validate { input: String },
}

#[derive(Debug, Subcommand)]
enum FixturesCmd {
    /// List bundled fixture names.
    List,
    /// Print the canonical bundled fixture body to stdout.
    Show {
        /// Fixture name relative to the bundled root, e.g. `valid/01-prd-canonical.json`.
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum MigrateCmd {
    /// Apply pending migrations.
    Up {
        /// Database URL, e.g. `sqlite:///tmp/aih.sqlite?mode=rwc`.
        #[arg(long, default_value = "sqlite::memory:")]
        database_url: String,
    },
    /// Print the current migration version.
    Status {
        #[arg(long)]
        database_url: String,
    },
    /// Verify the database has all expected migrations applied.
    Verify {
        #[arg(long)]
        database_url: String,
    },
}

#[derive(Debug, Parser)]
struct ScoreArgs {
    /// Path to the event JSON, or `-` for stdin.
    input: String,
    /// Optional scoring profile JSON path; defaults to embedded `default.v1.json`.
    #[arg(long)]
    profile: Option<PathBuf>,
    /// Optional tier set JSON path; defaults to embedded `default.v1.json`.
    #[arg(long)]
    tiers: Option<PathBuf>,
    /// Override the resolved tier id.
    #[arg(long)]
    tier: Option<String>,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
    /// Print the explainability trace as a human-formatted, multi-line view
    /// (PRD §16 / ADR-0010 Phase 2 `aih score --detail`).
    #[arg(long)]
    detail: bool,
}

#[derive(Debug, Subcommand)]
enum ValidateCmd {
    /// Validate a scoring-profile JSON against `scoring_profile.v1.json`.
    Profile { input: String },
    /// Validate a tier-set JSON against `tier_set.v1.json` (schema not yet
    /// shipped; surface reserved per ADR-0010 Phase 2).
    Tier { input: String },
}

#[derive(Debug, Parser)]
struct BenchArgs {
    /// Path to the event JSON to benchmark, or `-` for stdin.
    #[arg(long, default_value = "-")]
    fixture: String,
    /// Number of iterations.
    #[arg(long, default_value_t = 1000)]
    iter: u32,
    /// Optional p95 budget in milliseconds; non-zero exit if exceeded.
    #[arg(long)]
    budget_ms: Option<f64>,
}

#[derive(Debug, Parser)]
struct ReplayArgs {
    /// Database URL, e.g. `sqlite:///tmp/aih.sqlite`.
    #[arg(long)]
    database_url: String,
    /// Workspace id (defaults to `default`).
    #[arg(long, default_value = "default")]
    workspace: String,
    /// Persisted event id to re-score.
    #[arg(long)]
    event_id: String,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    Pretty,
    Table,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Schema { sub } => match sub {
            SchemaCmd::Validate { input } => cmd_schema_validate(&input),
        },
        Command::Score(args) => cmd_score(&args),
        Command::Fixtures { sub } => match sub {
            FixturesCmd::List => cmd_fixtures_list(),
            FixturesCmd::Show { name } => cmd_fixtures_show(&name),
        },
        Command::Diff { a, b } => cmd_diff(&a, &b),
        Command::Migrate { sub } => cmd_migrate(sub),
        Command::Validate { sub } => cmd_validate(sub),
        Command::Bench(args) => cmd_bench(&args),
        Command::Replay(args) => cmd_replay(&args),
        Command::Version => cmd_version(),
    }
}

fn read_input(arg: &str) -> Result<String> {
    if arg == "-" {
        let mut s = String::new();
        std::io::stdin()
            .lock()
            .take(MAX_INPUT_BYTES + 1)
            .read_to_string(&mut s)
            .context("reading stdin")?;
        if s.len() as u64 > MAX_INPUT_BYTES {
            bail!("input larger than {MAX_INPUT_BYTES} bytes");
        }
        Ok(s)
    } else {
        let meta = std::fs::metadata(arg).with_context(|| format!("stat {arg}"))?;
        if meta.len() > MAX_INPUT_BYTES {
            bail!("input file {arg} is larger than {MAX_INPUT_BYTES} bytes");
        }
        std::fs::read_to_string(arg).with_context(|| format!("reading {arg}"))
    }
}

fn cmd_schema_validate(input: &str) -> Result<()> {
    let body = read_input(input)?;
    validator()
        .validate_str(&body, Mode::Strict)
        .map_err(|e| anyhow::anyhow!("schema validation failed: {e}"))?;
    println!("ok");
    Ok(())
}

fn cmd_score(args: &ScoreArgs) -> Result<()> {
    let body = read_input(&args.input)?;
    let value: serde_json::Value = serde_json::from_str(&body).context("parsing event JSON")?;
    validator()
        .validate(&value, Mode::Strict)
        .map_err(|e| anyhow::anyhow!("schema validation failed: {e}"))?;
    let event: Event = serde_json::from_value(value).context("materialising Event")?;

    let profile = match &args.profile {
        Some(p) => {
            let body = std::fs::read_to_string(p)
                .with_context(|| format!("reading profile {}", p.display()))?;
            profile_validator()
                .validate_str(&body, Mode::Strict)
                .map_err(|e| anyhow::anyhow!("scoring profile schema validation failed: {e}"))?;
            serde_json::from_str(&body).context("materialising ScoringProfile")?
        }
        None => ScoringProfile::default_v1(),
    };
    let tiers = match &args.tiers {
        Some(p) => serde_json::from_str(&std::fs::read_to_string(p)?)?,
        None => TierSet::default_v1(),
    };
    let result = score(&event, &profile, &tiers, args.tier.as_deref())
        .map_err(|e| anyhow::anyhow!("scoring failed: {e}"))?;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if args.detail {
        write_detail(&mut out, &result)?;
        return Ok(());
    }
    match args.format {
        OutputFormat::Json => {
            serde_json::to_writer(&mut out, &result)?;
            writeln!(&mut out)?;
        }
        OutputFormat::Pretty => {
            serde_json::to_writer_pretty(&mut out, &result)?;
            writeln!(&mut out)?;
        }
        OutputFormat::Table => {
            writeln!(
                &mut out,
                "scoring_version  {}\nspec_version     {}\nprofile          {}\ncategory         {}\ntier             {} (x{})\nminutes          {}\nhours            {}\ndays             {}\nfec              {} {}\nconfidence       {} ({:?})\nsummary          {}",
                result.scoring_version,
                result.spec_version,
                result.scoring_profile,
                result.category,
                result.tier.id,
                result.tier.multiplier,
                result.final_estimated_minutes,
                result.estimated_hours,
                result.estimated_days,
                result.financial_equivalent_cost,
                result.tier.currency,
                result.confidence_score,
                result.confidence_band,
                result.human_summary,
            )?;
        }
    }
    Ok(())
}

fn cmd_fixtures_list() -> Result<()> {
    // Bundled fixtures are embedded at compile time via `include_dir!` so
    // installed binaries work regardless of the source-tree location.
    println!("Bundled fixture catalog:");
    let mut paths: Vec<&str> = bundled_fixture_paths().collect();
    paths.sort_unstable();
    for p in paths {
        println!("  {p}");
    }
    Ok(())
}

fn cmd_fixtures_show(name: &str) -> Result<()> {
    let normalized = name.trim_start_matches("./").trim_start_matches('/');
    let file = FIXTURES
        .get_file(normalized)
        .with_context(|| format!("fixture not found: {normalized} (try `aih fixtures list`)"))?;
    let body = file.contents_utf8().context("fixture is not valid UTF-8")?;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    out.write_all(body.as_bytes())?;
    if !body.ends_with('\n') {
        writeln!(&mut out)?;
    }
    Ok(())
}

fn bundled_fixture_paths() -> impl Iterator<Item = &'static str> {
    fn walk<'a>(dir: &'a include_dir::Dir<'a>, out: &mut Vec<&'a str>) {
        for f in dir.files() {
            out.push(f.path().to_str().expect("fixture path is UTF-8"));
        }
        for d in dir.dirs() {
            walk(d, out);
        }
    }
    let mut buf = Vec::new();
    walk(&FIXTURES, &mut buf);
    buf.into_iter()
}

fn cmd_diff(a: &PathBuf, b: &PathBuf) -> Result<()> {
    let av: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(a)?)?;
    let bv: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(b)?)?;
    if av == bv {
        println!("equal");
        Ok(())
    } else {
        bail!("ScoreResult JSONs differ");
    }
}

fn cmd_migrate(sub: MigrateCmd) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        match sub {
            MigrateCmd::Up { database_url } => {
                let pool = heeczer_storage::sqlite::open(&database_url).await?;
                heeczer_storage::sqlite::migrate(&pool).await?;
                let v = heeczer_storage::sqlite::current_version(&pool)
                    .await?
                    .unwrap_or(-1);
                println!("migrated to {v}");
                Ok(())
            }
            MigrateCmd::Status { database_url } => {
                let pool = heeczer_storage::sqlite::open(&database_url).await?;
                let v = heeczer_storage::sqlite::current_version(&pool).await?;
                println!("{v:?}");
                Ok(())
            }
            MigrateCmd::Verify { database_url } => {
                let pool = heeczer_storage::sqlite::open(&database_url).await?;
                heeczer_storage::sqlite::migrate(&pool).await?;
                println!("ok");
                Ok(())
            }
        }
    })
}

fn cmd_version() -> Result<()> {
    println!(
        "{} {} (scoring_version={} spec_version={})",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        SCORING_VERSION,
        SPEC_VERSION,
    );
    Ok(())
}

fn write_detail<W: Write>(out: &mut W, r: &heeczer_core::ScoreResult) -> Result<()> {
    writeln!(out, "Explainability trace (PRD §16)")?;
    writeln!(out, "  scoring_version : {}", r.scoring_version)?;
    writeln!(out, "  spec_version    : {}", r.spec_version)?;
    writeln!(out, "  profile         : {}", r.scoring_profile)?;
    writeln!(
        out,
        "  category        : {} (×{})",
        r.category, r.category_multiplier
    )?;
    writeln!(out, "  BCU breakdown")?;
    writeln!(out, "    tokens     : {}", r.bcu_breakdown.tokens)?;
    writeln!(out, "    duration   : {}", r.bcu_breakdown.duration)?;
    writeln!(out, "    steps      : {}", r.bcu_breakdown.steps)?;
    writeln!(out, "    tools      : {}", r.bcu_breakdown.tools)?;
    writeln!(out, "    artifacts  : {}", r.bcu_breakdown.artifacts)?;
    writeln!(out, "    output     : {}", r.bcu_breakdown.output)?;
    writeln!(out, "    review     : {}", r.bcu_breakdown.review)?;
    writeln!(out, "    total      : {}", r.bcu_breakdown.total())?;
    writeln!(out, "  context multipliers")?;
    writeln!(out, "    retry         : ×{}", r.context_multiplier.retry)?;
    writeln!(
        out,
        "    ambiguity     : ×{}",
        r.context_multiplier.ambiguity
    )?;
    writeln!(out, "    risk          : ×{}", r.context_multiplier.risk)?;
    writeln!(
        out,
        "    human_in_loop : ×{}",
        r.context_multiplier.human_in_loop
    )?;
    writeln!(out, "    outcome       : ×{}", r.context_multiplier.outcome)?;
    writeln!(
        out,
        "    product       : ×{}",
        r.context_multiplier.product()
    )?;
    writeln!(out, "  baseline_minutes : {}", r.baseline_human_minutes)?;
    writeln!(
        out,
        "  tier             : {} (×{} @ {} {}/h)",
        r.tier.id, r.tier.multiplier, r.tier.hourly_rate, r.tier.currency
    )?;
    writeln!(out, "  final_minutes    : {}", r.final_estimated_minutes)?;
    writeln!(out, "  final_hours      : {}", r.estimated_hours)?;
    writeln!(out, "  final_days       : {}", r.estimated_days)?;
    writeln!(
        out,
        "  fec              : {} {}",
        r.financial_equivalent_cost, r.tier.currency
    )?;
    writeln!(
        out,
        "  confidence       : {} ({:?})",
        r.confidence_score, r.confidence_band
    )?;
    writeln!(out, "  summary          : {}", r.human_summary)?;
    Ok(())
}

fn cmd_validate(sub: ValidateCmd) -> Result<()> {
    match sub {
        ValidateCmd::Profile { input } => {
            let body = read_input(&input)?;
            profile_validator()
                .validate_str(&body, Mode::Strict)
                .map_err(|e| anyhow::anyhow!("scoring profile schema validation failed: {e}"))?;
            // Round-trip through the typed struct so deny_unknown_fields fires too.
            let _: ScoringProfile =
                serde_json::from_str(&body).context("materialising ScoringProfile")?;
            println!("ok");
            Ok(())
        }
        ValidateCmd::Tier { input: _ } => {
            // The tier-set JSON Schema (`tier_set.v1.json`) is not yet shipped.
            // Surface is reserved by ADR-0010 Phase 2 so downstream tooling can
            // depend on the command path now and the schema can land later
            // without a breaking CLI change.
            bail!(
                "tier-set schema not yet shipped (ADR-0010 Phase 2); \
                 the `aih validate tier` surface is reserved \
                 and will activate once `core/schema/tier_set.v1.json` lands"
            );
        }
    }
}

fn load_event_from_path(path: &str) -> Result<Event> {
    let body = read_input(path)?;
    let value: serde_json::Value = serde_json::from_str(&body).context("parsing event JSON")?;
    validator()
        .validate(&value, Mode::Strict)
        .map_err(|e| anyhow::anyhow!("schema validation failed: {e}"))?;
    serde_json::from_value(value).context("materialising Event")
}

fn cmd_bench(args: &BenchArgs) -> Result<()> {
    if args.iter == 0 {
        bail!("--iter must be at least 1");
    }
    let event = load_event_from_path(&args.fixture)?;
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();

    // Warm-up to amortise allocator and code-cache effects so the first sample
    // does not skew the percentiles.
    let _ = score(&event, &profile, &tiers, None)
        .map_err(|e| anyhow::anyhow!("scoring failed in warm-up: {e}"))?;

    let mut samples: Vec<Duration> = Vec::with_capacity(args.iter as usize);
    for _ in 0..args.iter {
        let t0 = Instant::now();
        let _ = score(&event, &profile, &tiers, None)
            .map_err(|e| anyhow::anyhow!("scoring failed: {e}"))?;
        samples.push(t0.elapsed());
    }
    samples.sort_unstable();
    let p50 = samples[args.iter as usize / 2];
    let p95 = samples[((args.iter as usize) * 95 / 100).min(args.iter as usize - 1)];
    let p99 = samples[((args.iter as usize) * 99 / 100).min(args.iter as usize - 1)];
    let p50_ms = p50.as_secs_f64() * 1000.0;
    let p95_ms = p95.as_secs_f64() * 1000.0;
    let p99_ms = p99.as_secs_f64() * 1000.0;
    println!(
        "score() iter={} p50={:.4}ms p95={:.4}ms p99={:.4}ms",
        args.iter, p50_ms, p95_ms, p99_ms
    );
    if let Some(budget) = args.budget_ms {
        if p95_ms > budget {
            bail!("p95 {p95_ms:.4}ms exceeds budget {budget:.4}ms");
        }
    }
    Ok(())
}

fn cmd_replay(args: &ReplayArgs) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let pool = heeczer_storage::sqlite::open(&args.database_url).await?;
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT payload FROM aih_events WHERE workspace_id = ?1 AND event_id = ?2",
        )
        .bind(&args.workspace)
        .bind(&args.event_id)
        .fetch_optional(&pool)
        .await?;
        let payload = row.map(|(p,)| p).with_context(|| {
            format!(
                "no aih_events row for workspace={} event_id={}",
                args.workspace, args.event_id
            )
        })?;

        let value: serde_json::Value =
            serde_json::from_str(&payload).context("aih_events.payload is not valid JSON")?;
        validator()
            .validate(&value, Mode::Strict)
            .map_err(|e| anyhow::anyhow!("persisted event no longer matches schema: {e}"))?;
        let event: Event = serde_json::from_value(value).context("materialising Event")?;
        let profile = ScoringProfile::default_v1();
        let tiers = TierSet::default_v1();
        let result = score(&event, &profile, &tiers, None)
            .map_err(|e| anyhow::anyhow!("scoring failed: {e}"))?;

        let prior: Option<(String, String)> = sqlx::query_as(
            "SELECT result_json, scoring_version FROM aih_scores
             WHERE workspace_id = ?1 AND event_id = ?2
             ORDER BY created_at DESC LIMIT 1",
        )
        .bind(&args.workspace)
        .bind(&args.event_id)
        .fetch_optional(&pool)
        .await?;

        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        serde_json::to_writer_pretty(&mut out, &result)?;
        writeln!(&mut out)?;
        match prior {
            None => writeln!(&mut out, "no prior score row; replay is read-only")?,
            Some((prior_json, prior_ver)) => {
                let prior_value: serde_json::Value = serde_json::from_str(&prior_json)
                    .context("aih_scores.result_json is not valid JSON")?;
                let live_value = serde_json::to_value(&result)?;
                if prior_value == live_value {
                    writeln!(&mut out, "match: live score equals prior {prior_ver} row")?;
                } else {
                    writeln!(
                        &mut out,
                        "drift: live score differs from prior {prior_ver} row"
                    )?;
                    bail!("score drift detected");
                }
            }
        }
        Ok(())
    })
}
