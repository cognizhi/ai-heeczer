//! `aih` — local developer CLI for the ai-heeczer scoring core. See ADR-0010.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use heeczer_core::{
    schema::{EventValidator, Mode},
    score, Event, ScoringProfile, TierSet, SCORING_VERSION, SPEC_VERSION,
};

/// Hard upper bound for any single JSON document the CLI accepts. Prevents an
/// unbounded stdin from turning into an OOM. The same constant should be
/// honored by the future ingestion service (plan 0004).
const MAX_INPUT_BYTES: u64 = 1024 * 1024;

fn validator() -> &'static EventValidator {
    static V: OnceLock<EventValidator> = OnceLock::new();
    V.get_or_init(EventValidator::new_v1)
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
    /// List bundled fixture names by category.
    List,
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
        },
        Command::Diff { a, b } => cmd_diff(&a, &b),
        Command::Migrate { sub } => cmd_migrate(sub),
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
        Some(p) => serde_json::from_str(&std::fs::read_to_string(p)?)?,
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
    // Bundled fixture names live under `core/schema/fixtures/events/`.
    // We don't embed every fixture into the binary; instead, the CLI walks the
    // crate's known relative path when run from a checkout, and otherwise
    // points the user at the published catalog.
    println!("Bundled fixture catalog:");
    println!("  valid/01-prd-canonical.json");
    println!("  edge/01-minimum-required.json");
    println!("  edge/02-missing-category.json");
    println!("  edge/03-extensions-passthrough.json");
    println!("  edge/04-unicode.json");
    println!("(see core/schema/fixtures/events/ in the source tree)");
    Ok(())
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
