# Product Requirements Document
## ai-heeczer
**AI Human Equivalent Effort and Cost Analyzer**

**Document Version:** v1.3  
**Status:** Draft  
**Owner:** Product / Founding Team  
**Primary Audience:** Product, Engineering, Platform, Finance, AI Enablement  
**Repository Use:** Source-of-truth PRD for implementation planning, architecture, task breakdown, release automation, and AI-assisted development in GitHub/Copilot

---

## 1. Executive Summary

Organizations are rapidly adopting agentic AI frameworks such as Google ADK, PydanticAI, LangGraph, and similar orchestration stacks. While these systems generate abundant telemetry such as token usage, latency, retries, tool calls, and workflow steps, most organizations still lack a credible way to translate that telemetry into business value.

Technical metrics alone do not answer leadership’s real questions:

- How much human effort did this AI workflow replace or accelerate?
- What is the estimated person-hour or person-day equivalent of this AI work?
- How much labor-equivalent cost did this workflow save?
- Which workflows, teams, or users create the most measurable AI value?

**ai-heeczer** is a cross-language telemetry-to-value analytics framework that converts agentic AI execution metadata into **Human Equivalent Effort (HEE)** and **Financial Equivalent Cost (FEC)**. It provides a deterministic, explainable, configurable, and auditable scoring engine that estimates how much human effort a task would have required under defined human role tiers such as Junior Engineer, Senior Engineer, or Principal Engineer.

The product is designed for lightweight instrumentation, cross-language consistency, enterprise configuration, and operational reporting. Its core purpose is to become the translation layer between AI runtime activity and business ROI.

This revision also formalizes **release engineering, repository standards, TDD, contract precision, CI quality gates, multi-ecosystem publishing, Makefile support, and README completeness** as first-class product requirements rather than incidental engineering choices.

---

## 2. Problem Statement

Today, organizations can measure model consumption, infrastructure cost, and runtime performance. They cannot reliably measure the human-equivalent value created by agentic AI.

This creates several business and operational problems:

1. AI ROI remains vague and difficult to defend.
2. Leadership cannot compare AI platform spend against labor-equivalent outcomes.
3. Teams cannot identify which workflows create the highest practical value.
4. Finance cannot model internal chargeback or cost allocation against AI usage.
5. Product and platform teams cannot benchmark whether an agent meaningfully replaces, accelerates, or merely assists human work.
6. Different frameworks emit different telemetry, resulting in inconsistent reporting.
7. Existing estimates are often ad hoc, non-repeatable, and vulnerable to inflated claims.
8. Multi-language SDK projects often drift in behavior if release, versioning, and contract governance are weak.
9. Repositories become hard to contribute to when local setup, build, test, release, and publish instructions are fragmented or inconsistent.

Without a standardized translation model and a disciplined delivery framework, organizations are left with anecdotal narratives instead of evidence-backed analytics.

---

## 3. Product Vision

To become the universal standard for translating agentic AI execution into human-equivalent effort and labor-equivalent financial value across programming languages, frameworks, and enterprise environments.

---

## 4. Product Goals

### 4.1 Primary Goals
1. Provide a deterministic and explainable scoring engine that estimates human-equivalent effort from AI task telemetry.
2. Support consistent behavior across JavaScript, TypeScript, Python, Go, Rust, and Java.
3. Allow organizations to configure role tiers, labor rates, scoring weights, and category-specific baselines.
4. Provide a dashboard that helps engineering, finance, and leadership understand AI-generated effort and value.
5. Support two ingestion execution modes: native in-process execution and isolated image-based ingestion service execution.
6. Make the isolated image-based ingestion service the default production and reference deployment mode.
7. Provide a separately containerized dashboard for operational and business reporting, including queue and processing status visibility.
8. Ship with production-grade CI/CD that automatically versions and releases artifacts once all required quality gates pass.
9. Publish synchronized releases to each supported ecosystem repository from one governed release process.
10. Make the repository easy to understand, build, test, and release via a comprehensive README and standardized Makefile targets.
11. Enforce test-driven development and precise contracts so scoring semantics remain stable and reproducible.

### 4.2 Secondary Goals
1. Establish a canonical event schema for telemetry ingestion.
2. Provide adapters for popular agentic frameworks.
3. Support calibration against real-world task baselines.
4. Ensure every score is versioned, auditable, and reproducible.
5. Enable future ecosystem integrations such as Langfuse and OpenTelemetry bridges.

---

## 5. Non-Goals

ai-heeczer is not intended to be:

- an agent orchestration framework
- a general observability platform
- a payroll, ERP, or accounting ledger
- a model quality benchmark suite
- a prompt storage platform
- a replacement for APM, tracing, or runtime debugging tools

It focuses specifically on **translating AI execution telemetry into human-equivalent effort and cost metrics**.

---

## 6. Product Principles

1. **Deterministic by default**  
   Core scoring must be formula-based, explainable, reproducible, and versioned.

2. **Configurable without becoming arbitrary**  
   Organizations can tune the model, but defaults must remain sensible, documented, and safe.

3. **Privacy-safe by design**  
   Prompt content and model outputs are excluded by default.

4. **Operational isolation by default**  
   Production ingestion should default to an isolated containerized service rather than sharing application process boundaries.

5. **Low-friction adoption**  
   Developers should be able to instrument ai-heeczer in minutes and switch between native and image-based processing through configuration.

6. **Cross-language parity**  
   Equivalent inputs must yield equivalent outputs across supported SDKs.

7. **Confidence-aware reporting**  
   The product must communicate uncertainty and avoid false precision.

8. **Auditability is mandatory**  
   Score version, tier version, rate version, and configuration changes must be traceable.

9. **Release discipline is part of the product**  
   Versioning, changelogs, CI gates, and ecosystem publishing are product requirements, not optional repo hygiene.

10. **TDD over retrofit testing**  
   Requirements, contracts, fixtures, and acceptance tests must be written before or alongside implementation, never treated as cleanup.

11. **Executable developer experience**  
   README, Makefile, examples, and CI must agree with one another and remain runnable.

---

## 7. Target Users and Personas

### Engineering Leadership
Needs evidence that AI investment produces meaningful labor-equivalent leverage.

### Platform Engineering
Needs an SDK and runtime architecture that is easy to embed and safe for production systems.

### AI Product Teams
Need to compare value generated by different agents, workflows, and user-facing capabilities.

### Finance / Cost Governance
Needs labor-equivalent cost estimates for reporting, ROI analysis, and chargeback modeling.

### Consulting / Delivery Leads
Need client-friendly reporting that translates AI activity into business-relevant value.

### Individual Contributors
Want visibility into their productivity amplification through AI-enabled workflows.

### Open Source Maintainers / Internal Repo Owners
Need a repo that is easy to build, test, release, and consume across ecosystems without undocumented tribal knowledge.

---

## 8. Jobs To Be Done

When an organization runs agentic AI in production, they want to:

- quantify how much human-equivalent work AI completed
- estimate effort saved in minutes, hours, and days
- compare value across workflows, teams, and business units
- estimate labor-equivalent financial value
- understand the confidence and assumptions behind each estimate
- audit how every score was calculated
- calibrate the scoring system to their own role definitions and labor structure
- release changes safely without manual versioning drift
- publish updated SDKs and artifacts consistently across ecosystems
- onboard contributors quickly with one obvious source of truth for setup and execution

---

## 9. Core Product Scope

ai-heeczer consists of eight product surfaces:

1. **Cross-Language Telemetry SDKs**  
   Lightweight SDKs for JS/TS, Python, Go, Rust, and Java.

2. **Core Scoring Engine**  
   Deterministic engine producing HEE, FEC, confidence score, and explainability trace.

3. **heeczer Ingestion Service Image**  
   A containerized ingestion service that receives events through API, hook, or queue-driven delivery, manages incoming queues, processes events, updates the database, and responds to clients.

4. **Storage and Adapters Layer**  
   Database abstraction with SQLite for local use and PostgreSQL for production.

5. **Dashboard and Admin Console**  
   A separately containerized web UI for metrics, drill-downs, configuration, governance, and queue/status observability.

6. **Framework Integrations**  
   Adapters for major frameworks such as LangGraph, Google ADK, and PydanticAI.

7. **Release and Distribution Tooling**  
   GitHub Actions-based semantic versioning and automated publish flows for all supported ecosystems.

8. **Developer Experience Surface**  
   Comprehensive README, examples, Makefile, local scripts, test fixtures, and contribution standards.

---

## 10. Key Use Cases

### Use Case 1: Monthly ROI Reporting
A consulting team uses ai-heeczer to show a client that AI workflows delivered the equivalent of hundreds of mid-level engineering hours in a given month.

### Use Case 2: Feature Prioritization
A product team compares multiple AI features and finds that one agent creates far more labor-equivalent value than others.

### Use Case 3: Internal Chargeback
A platform team allocates central AI platform cost based on the labor-equivalent value delivered to business units.

### Use Case 4: Team-Level Benchmarking
Leadership identifies which teams are capturing meaningful productivity gains from AI-assisted workflows.

### Use Case 5: Calibration Against Human Baselines
A company tunes the scoring model using sample tasks with known human effort ranges.

### Use Case 6: Automated Multi-Ecosystem Release
A maintainer merges a PR with conventional commit messages. GitHub Actions runs lint, unit tests, integration tests, contract tests, benchmark smoke tests, UI tests where applicable, and required security checks. If all required gates pass, the pipeline computes the next semantic version, generates a changelog, tags the repo, and publishes updated packages to npm, PyPI, the Go module ecosystem through versioned module tags, crates.io, and Maven Central.

### Use Case 7: Fast Local Development
A new contributor clones the repo, runs `make bootstrap` and `make test`, and gets the project working locally without having to memorize language-specific commands.

---

## 11. Definitions

### Human Equivalent Effort (HEE)
Estimated human effort required to achieve the same or substantially equivalent task outcome under a defined human tier.

### Financial Equivalent Cost (FEC)
Estimated labor-equivalent cost associated with that effort, based on configured rates.

### Base Cognitive Units (BCU)
Internal normalized effort units used before applying role and contextual adjustments.

### Human Tier
A role definition such as Junior Engineer, Senior Engineer, Business Analyst, or Principal Engineer, with associated productivity and cost characteristics.

### Confidence Score
A numeric indicator from 0.0 to 1.0 representing how trustworthy an estimate is based on telemetry completeness and calibration strength.

### Explainability Trace
A structured breakdown showing exactly how the score was computed.

### Contract
A precise, versioned definition of expected behavior, structure, and outputs between components, SDKs, APIs, schemas, database adapters, and UI consumers.

### Required Tests
The minimum quality gates that must pass before a change is mergeable or releasable. These include all mandatory unit, integration, contract, and repository-defined validation stages.

---

## 12. Functional Requirements

### 12.1 Event Ingestion
The product shall ingest standardized JSON events from SDKs, HTTP endpoints, hooks, queue consumers, or batch replay jobs.

The product shall support two selectable execution modes:
- **Native mode:** ingestion and scoring occur in the host application process
- **Image mode:** ingestion is delegated to the containerized heeczer ingestion service

Image mode shall become the default production configuration and reference architecture at production-readiness GA. MVP may ship with native mode as the primary GA path and image mode as preview.

### 12.2 Event Validation
The product shall validate payloads against a canonical versioned schema and reject malformed events with actionable errors.

### 12.3 Scoring
The system shall compute:
- estimated human minutes
- estimated human hours
- estimated human days
- financial equivalent cost
- confidence score
- explainability trace
- score version metadata

### 12.4 Queueing and Processing
In image mode, the ingestion service shall:
- receive incoming events via API, hook, or queue
- enqueue and prioritize work
- validate and normalize payloads
- process scoring asynchronously or synchronously as configured
- persist raw and computed data
- expose client responses and processing outcomes
- maintain queue and job statuses for observability

### 12.5 Persistence
The product shall persist:
- raw normalized events
- computed scores
- scoring profile version
- tier version
- rate version
- queue/job records where image mode is used
- derived aggregates
- audit log records

### 12.6 Visualization
The dashboard shall display:
- total AI tasks
- total HEE
- total FEC
- trends over time
- leaderboards by user, team, project, framework, and category
- confidence distribution
- event-level explainability drill-down
- incoming queue depth
- queue item statuses
- processing failures, retries, and dead-letter counts where applicable

### 12.7 Configuration
Admins shall be able to manage:
- processing mode selection: native or image
- queue and processing configuration for image mode
- ingestion endpoint and hook configuration

- role tiers
- productivity multipliers
- labor rates
- scoring weights
- category multipliers
- confidence penalties
- currency and financial settings
- workspace and project overrides

### 12.8 Calibration
The product shall support model calibration using known benchmark tasks and expected human-effort ranges.

### 12.9 Governance
The system shall provide audit logs for all configuration changes and re-scoring events.

### 12.10 Semantic Version and Release Automation
The repository shall provide a GitHub Actions CI/CD pipeline that:
- computes and applies semantic versions automatically
- derives changelogs from commit history or release metadata
- blocks release unless all required checks pass
- tags the repository with the released version
- publishes release artifacts to supported package ecosystems
- records publish intent and publish status in a release manifest
- marks a release complete only when all required publish stages succeed
- supports retry, resume, and operator intervention for partial publish failures without minting a new version
- records release provenance and build metadata

### 12.11 Multi-Ecosystem Publishing
The release process shall publish or update the standard distribution channel for each supported language ecosystem:
- npm for JS/TS
- PyPI for Python
- versioned Go module tags on the canonical module path, consumable through the Go module ecosystem including proxy-compatible clients
- crates.io for Rust
- Maven Central for Java
- GitHub Releases for source artifacts, checksums, changelog, and container references

### 12.12 README and Onboarding
The repository shall include a comprehensive root README that covers:
- what ai-heeczer is
- architecture overview
- supported languages and frameworks
- quickstart
- local development setup
- Makefile targets
- how to run tests
- how release/versioning works
- package publishing behavior
- examples
- security and privacy stance
- contribution guidance
- support and roadmap links

### 12.13 Makefile Support
The repository shall include a Makefile that provides a stable cross-platform command surface for common tasks including:
- bootstrap
- format
- lint
- unit-test
- integration-test
- contract-test
- ui-test
- test
- build
- release-dry-run
- docs
- clean

### 12.14 Test-Driven Development
All implementation must follow TDD discipline:
- requirements and contracts must be defined before implementation
- unit tests must accompany all logic changes
- integration tests must cover cross-component behavior
- contract tests must validate schema and cross-language parity
- UI tests must be used where dashboard behavior or user workflows are in scope
- no feature is complete without corresponding acceptance tests

### 12.15 Precise Scope and Contract Definition
Every major module shall have explicit scope boundaries and testable contracts, including:
- canonical event schema contract
- scoring engine input/output contract
- SDK API contract across all languages
- storage adapter contract
- HTTP ingest API contract
- dashboard data contract
- release artifact contract
- Makefile target contract

### 12.16 API Versioning
The HTTP ingestion API and SDK transport contract shall be explicitly versioned via URL prefix (`/v1/`, `/v2/`) and a `spec_version` field on the payload. Breaking API changes shall introduce a new major version, run in parallel for at least one minor release before deprecation, and emit a deprecation header on the prior version. Backward-compatible changes are minor; bug fixes are patch.

### 12.17 Data Retention and Deletion
The product shall provide:
- configurable per-workspace retention windows for raw events, scores, and aggregates
- a hard-deletion API and CLI for GDPR/CCPA-style subject deletion that removes raw events, scores, and audit-log identifiers while preserving anonymized aggregates
- a tombstone record for any deleted event so re-scoring jobs do not silently re-create them
- documented defaults (recommended: raw 90 days, scores 1 year, aggregates indefinite) overridable per deployment

### 12.18 Rate Limiting and Request Quotas
The ingestion service shall enforce:
- per-API-key request rate limits (token bucket, configurable)
- per-workspace daily event quotas
- maximum payload size limits (default 64 KiB per event, 1 MiB per batch)
- structured 429 responses including `Retry-After` and quota headers

### 12.19 Idempotency
All ingestion endpoints shall honor `event_id` as the primary idempotency key per Section 19.4. Clients may additionally supply an `Idempotency-Key` HTTP header for batch endpoints; the response of a previously processed key shall be replayed verbatim within the configured idempotency retention window (default 24 hours).

### 12.20 Database Schema Migrations
The project shall use a versioned, forward-only migration tool with one migration history table (`aih_schema_migrations`) shared across SQLite and PostgreSQL adapters. Migration choice is documented in ADR-0004. Every schema change shall ship with a migration script, a rollback note, and a migration test that runs on both backends in CI.

### 12.21 Local Developer CLI (`aih`)
The project shall ship a first-class command-line tool, `aih`, as the canonical local invocation surface for the Rust scoring core. It is the single tool a contributor uses to atomically test the analyzer without standing up the ingestion service or any SDK. Decision and rationale are recorded in ADR-0010.

Required MVP subcommands:
- `aih schema validate` — validate a JSON event against the canonical schema in strict or compatibility mode.
- `aih score` — run the deterministic scoring engine and emit `ScoreResult` (HEE, FEC, confidence, explainability trace) as JSON or human-readable table.
- `aih fixtures list|show` — enumerate and emit shipped golden fixtures for use in downstream SDK and adapter test suites.
- `aih diff` — diff two `ScoreResult`s for parity verification.
- `aih migrate up|status|verify` — apply storage migrations against a configured SQLite or PostgreSQL URL (subsumes the previously planned `heeczerctl` binary).
- `aih version` — print CLI, `scoring_version`, `spec_version`, and core crate versions.

Phase 2 subcommands (per ADR-0010 amendment, 2026-04-23):
- `aih score detail` — same scoring path as `aih score`, formatted explainability trace.
- `aih validate profile|tier` — validate scoring-profile / tier-set JSONs against their schemas.
- `aih replay <DB_URL> <event_id>` — read-only re-score of a persisted event; diffs against the latest persisted score row (does not insert a new row — that is reserved for the dashboard test-orchestration view per §21).
- `aih bench [--iter N] [--fixture PATH] [--budget-ms M]` — p50/p95/p99 measurement of `score()`; non-zero exit on budget breach.

The CLI's JSON output is part of the public contract (§12.15) and changes are versioned alongside `scoring_version` and `spec_version`. The CLI is published to crates.io and as signed prebuilt binaries on each GitHub Release per §27.4 and §27.5.

---

## 13. Canonical Event Schema (v1)

```json
{
  "spec_version": "1.0",
  "event_id": "uuid-v4",
  "correlation_id": "optional-parent-task-id",
  "timestamp": "2026-04-22T09:46:00Z",
  "framework_source": "langgraph",
  "workspace_id": "ws_001",
  "project_id": "proj_001",
  "identity": {
    "user_id": "usr_123",
    "team_id": "tm_456",
    "business_unit_id": "bu_007",
    "tier_id": "tier_mid_eng"
  },
  "task": {
    "name": "generate_api_spec",
    "category": "code_generation",
    "sub_category": "api_design",
    "outcome": "success"
  },
  "metrics": {
    "duration_ms": 14500,
    "tokens_prompt": 1200,
    "tokens_completion": 4000,
    "tool_call_count": 3,
    "workflow_steps": 5,
    "retries": 1,
    "artifact_count": 4,
    "output_size_proxy": 2.5
  },
  "context": {
    "human_in_loop": false,
    "review_required": true,
    "temperature": 0.2,
    "risk_class": "medium",
    "tags": ["frontend", "jira-sync"]
  },
  "meta": {
    "sdk_language": "python",
    "sdk_version": "0.1.0",
      "scoring_profile": "default",
      "extensions": {}
  }
}
```

### Schema Rules
- Prompt text and model output content are excluded by default.
- Unknown fields are accepted only under `meta.extensions` or a future versioned extension container explicitly defined by the schema.
- Unknown fields outside approved extension containers must fail validation in strict mode and may be dropped only in an explicitly configured compatibility mode.
- Extension metadata must be ignored by normalization, scoring, and confidence calculation unless promoted into the core schema by a versioned contract change.
- All schema revisions must be versioned.
- Schema fixtures must be shared across all supported languages.
- Schema validation behavior must be contract-tested in CI.

---

## 14. Scoring Model

The scoring model must be:
- deterministic
- explainable
- configurable
- bounded
- versioned
- robust to partial telemetry

### 14.1 Evaluated Factors

#### Required Factors
- task name
- task category, or the normalized fallback category `uncategorized` when omitted
- duration
- outcome
- token usage or equivalent complexity proxy where available

#### Optional but Strongly Recommended Factors
- tool call count
- workflow step count
- retries
- review requirement
- human-in-loop status
- artifact count
- output size proxy
- risk class
- temperature / creativity proxy

### 14.2 Base Scoring Formula

```text
BCU =
  token_component +
  duration_component +
  step_component +
  tool_component +
  artifact_component +
  output_component +
  review_component
```

Suggested default components:

```text
token_component    = total_tokens / 500
duration_component = duration_seconds / 2
step_component     = workflow_steps * 2
tool_component     = tool_call_count * 3
artifact_component = min(artifact_count, 20) * 1.5
output_component   = output_size_proxy * category_output_weight
review_component   = review_required ? category_review_weight : 0
```

Interpretation:  
**1 BCU is approximately 1 baseline human minute** before role and context adjustments.

### 14.2.1 Deterministic Arithmetic and Fallback Contract

- All scoring math shall use fixed-point decimal arithmetic with at least 4 fractional digits for intermediate steps. Binary floating-point implementations must not change persisted outputs.
- Normalization shall coerce absent optional numeric metrics to `0`, absent optional booleans to `false`, and absent optional multipliers to the neutral value `1.0`.
- `total_tokens` shall equal `tokens_prompt + tokens_completion`; missing token fields contribute `0`.
- Missing `task.category` shall normalize to `uncategorized`, use a default category multiplier of `1.0`, and incur the configured confidence penalty.
- Missing required non-derivable fields including `event_id`, `timestamp`, `task.name`, `task.outcome`, and `metrics.duration_ms` shall fail validation rather than infer values.
- Final persisted outputs shall use one documented rounding rule across all languages. The default rule is round half away from zero to 2 decimal places for minutes, hours, days, and FEC, and 4 decimal places for `confidence_score`.

### 14.3 Category Multiplier

Each task category has a configurable baseline multiplier, for example:

- summarization: 0.9
- drafting: 1.0
- code_generation: 1.2
- root_cause_analysis: 1.4
- planning_architecture: 1.5
- regulated_decision_support: 1.6

### 14.4 Contextual Multipliers

```text
context_multiplier =
  retry_multiplier *
  ambiguity_multiplier *
  risk_multiplier *
  human_in_loop_multiplier *
  outcome_multiplier
```

Suggested defaults:

- retry_multiplier = min(1.0 + retries * 0.25, 2.0)
- ambiguity_multiplier = 1.1 if temperature > 0.7 else 1.0
- risk_multiplier = 1.2 for high-risk tasks, else 1.0
- human_in_loop_multiplier = 0.7 when substantial human review is required
- outcome multiplier:
  - success = 1.0
  - partial_success = 0.6
  - failure = 0.25
  - timeout = 0.2

### 14.5 Human Tier Adjustment

```text
estimated_minutes_for_tier = baseline_human_minutes / tier_productivity_multiplier
```

### 14.6 Financial Equivalent Cost

```text
FEC = (estimated_minutes_for_tier / 60) * hourly_rate_for_tier
```

### 14.7 Scoring Contract Requirements
- Given the same normalized input, all supported languages must return identical scoring outputs.
- Identical scoring outputs includes identical arithmetic precision, rounding order, fallback behavior, and confidence band derivation.
- A scoring formula change is a versioned change and must not be shipped without fixture and contract updates.
- Explainability output is part of the scoring contract.
- Golden fixtures must assert exact persisted decimal outputs rather than approximate ranges.
- Extension metadata must not influence scoring unless the change is introduced as a versioned contract update.
- Confidence score semantics must be tested and documented.

---

## 15. Confidence Model

Each score must include a `confidence_score` and `confidence_band`.

### 15.1 Confidence Contract

- `confidence_score` shall be produced by a deterministic, versioned completeness and calibration matrix.
- Telemetry completeness penalties, retry penalties, calibration bonuses, and risk-based caps are part of the scoring contract and must be fixture-tested.
- `confidence_band` shall be derived from the unrounded `confidence_score` before display formatting.

### Suggested Confidence Logic
- Full telemetry + calibrated category: 0.90–0.95
- Tokens + duration + steps only: 0.70–0.85
- Duration only + generic category: 0.40–0.60
- Missing category or repeated retries: apply penalties
- High-risk tasks with limited telemetry must have capped confidence

### Confidence Bands
- **High:** 0.85–1.00
- **Medium:** 0.60–0.84
- **Low:** 0.40–0.59
- **Very Low:** below 0.40

---

## 16. Explainability Requirements

Every scored event must include a machine-readable explainability trace and a human-readable summary.

### Example Explainability Trace
```json
{
  "scoring_version": "1.0.0",
  "bcu_breakdown": {
    "tokens": 8.0,
    "duration": 7.0,
    "steps": 10.0,
    "tools": 9.0,
    "artifacts": 6.0,
    "review": 4.0
  },
  "category_multiplier": 1.2,
  "context_multiplier": {
    "retry": 1.25,
    "risk": 1.0,
    "human_in_loop": 0.7,
    "outcome": 1.0
  },
  "baseline_human_minutes": 45.36,
  "tier": {
    "id": "tier_mid_eng",
    "multiplier": 1.0,
    "hourly_rate": 75
  },
  "final_estimated_minutes": 45.36,
  "financial_equivalent_cost": 56.70,
   "confidence_score": 0.91,
   "confidence_band": "High"
}
```

---

## 17. Human Tier Model

ai-heeczer ships with default configurable role profiles.

### Default Engineering Tiers
- Principal Engineer — multiplier 3.0
- Senior Engineer — multiplier 2.0
- Mid-Level Engineer — multiplier 1.0
- Junior Engineer — multiplier 0.5

### Additional Default Business Tiers
- Business Analyst
- Operations Analyst
- Support Agent
- Technical Writer
- QA Analyst

Each tier must include:
- tier ID
- display name
- productivity multiplier
- hourly rate
- daily rate
- working hours per day
- version
- effective date
- scope level

---

## 18. Financial Model

The financial model shall support:
- blended hourly rate
- role-specific hourly rates
- workspace-level overrides
- multi-currency display
- historical rate versioning

### Default Formula
```text
financial_equivalent_cost = estimated_hours_for_tier × hourly_rate_for_tier
```

### Required UI Disclaimer
All financial outputs must be clearly labeled as **labor-equivalent estimates** and not payroll or audited accounting truth.

---

## 19. Architecture

### 19.1 Deployment Modes

#### Native Mode
- SDK performs local ingestion and scoring asynchronously inside the host application process
- writes to SQLite or configured local adapter
- suitable for local development, edge cases, offline workloads, and lightweight deployments

#### Image Mode (Phase 2 Default)
- SDK emits canonical events to the containerized heeczer ingestion service through API, hook, or queue-driven delivery
- the ingestion image manages incoming queue state, processing, persistence, retry handling, and client responses in isolation
- suitable for production, multi-tenant, enterprise, and microservice workloads

### 19.2 Architecture Requirement
The application using ai-heeczer shall be able to switch between native mode and image mode through configuration without changing instrumentation contracts.

### 19.3 Recommended Rollout
- MVP GA path: Native mode + SQLite
- Production-readiness target architecture: Image mode + PostgreSQL
- Image mode may ship in MVP as preview, but it does not become the default production recommendation until the queue backend, benchmark profile, and PostgreSQL worker path are release-ready.

### 19.4 Reliability
- at-least-once processing with deduplication by `event_id`
- duplicate submission with the same normalized payload returns the existing event and score records without mutation
- duplicate submission with a different normalized payload for the same `event_id` is rejected as a conflict and must use a new `event_id` or explicit replay API
- raw normalized events are immutable once accepted
- re-scoring creates a new score version linked to the same raw event and must not overwrite prior score versions
- replay and backfill operate from stored normalized raw events and preserve prior score history
- retry-safe processing
- queue visibility and status tracking in image mode
- explicit backpressure, queue overflow, and max queue age policies
- dead-letter handling in image mode

### 19.5 Monorepo and Build Architecture
The repository shall be structured as a monorepo with a single release control plane and language-specific publish targets. The build system shall support:
- Rust core as scoring source of truth
- language bindings generated or maintained against the same core contract
- isolated package manifests per ecosystem
- reproducible builds in CI
- publish stages gated by test and packaging success

---

## 20. Storage and Data Model

### Core Tables
- `aih_events`
- `aih_scores`
- `aih_jobs`
- `aih_tiers`
- `aih_rates`
- `aih_scoring_profiles`
- `aih_audit_log`
- `aih_daily_aggregates`
- `aih_workspaces`
- `aih_api_keys`
- `aih_tombstones`
- `aih_schema_migrations`

### Multi-Tenancy
All tenant-scoped tables shall include a non-null `workspace_id` column with appropriate indexes. The query layer shall enforce workspace scoping for all read and write paths. Cross-workspace queries are an admin-only operation guarded by RBAC.

### Storage Backends
- SQLite for local and development
- PostgreSQL for production

### Design Requirement
Raw normalized events and computed scores must be stored separately to enable safe re-scoring when formulas or configurations change.

Raw event records shall be immutable and append-only once accepted.

`aih_scores` or an equivalent score store shall be append-only and versioned by scoring profile and scoring version so that re-scoring preserves historical outputs.

---

## 21. Dashboard and Admin UX

### User Dashboard
Must display:
- total tasks processed
- HEE minutes / hours / days
- FEC totals
- time series trends
- leaderboards by user, team, project, framework, and category
- confidence distribution
- drill-down into event-level scoring traces
- incoming queue depth and health
- queue item and processing status views
- failure, retry, and dead-letter summaries for ingestion service operations

### Admin Console
Must support:
- tier management
- scoring profile management
- labor rate configuration
- audit log review
- calibration workflows
- workspace/project override management
- queue and worker operational visibility

### UX Guardrail
The UI must emphasize that scores are estimates with assumptions and confidence levels, not precise truth.

### UI Test Requirement
If the dashboard or admin console is in scope for a delivery, the implementation must include UI test coverage for critical flows, including:
- summary metric rendering
- filter and drill-down behavior
- explainability view loading
- settings persistence
- role-restricted actions where RBAC exists
- test-orchestration view: fixture run, golden diff, suite runner, replay (see §21 Test Orchestration View)

### Test Orchestration View
The dashboard must ship a `/test-orchestration` view that is the GUI counterpart to `aih` (§12.21) and provides back-to-back coverage of the scoring pipeline against shipped or user-supplied fixtures. Decision and scope are recorded in ADR-0012.

Required capabilities:
- fixture browser over `core/schema/fixtures/` with category and validity filters
- pipeline runner that scores a selected event + profile + tier-set via the ingestion service's RBAC-gated `POST /v1/test/score-pipeline` endpoint and renders the `ScoreResult` and explainability trace
- golden diff against any matching `*.score_result.json` fixture, with mismatched JSON paths surfaced inline
- one-click suite runner over the full golden set with a pass/fail matrix; progress streams via SSE
- benchmark stub that charts p50/p95 of `score()` over N iterations against a reference fixture
- replay of any persisted `event_id` against the currently selected profile, inserting new append-only score rows per §20

Constraints:
- view is RBAC-gated (`Tester`, `Admin`); test-orchestration endpoints sit behind a `features.test_orchestration` feature flag so production deployments can disable them
- never mutates `aih_events` or `aih_scores` outside the append-only contract (§20)
- every test-orchestration call emits a structured audit-log entry (§22)

---

## 22. Privacy, Security, and Compliance

### Privacy
By default, ai-heeczer must not store:
- prompt content
- model output content
- file attachments
- secrets
- access tokens

### Security
The product must support:
- authenticated ingestion endpoints (API key + optional mTLS)
- encrypted transport (TLS 1.2+; HSTS for the dashboard)
- RBAC for admin features
- audit logs for sensitive changes
- dependency and image scanning (CodeQL, Trivy, cargo-audit, pip-audit, npm audit, govulncheck)
- configurable retention and deletion policies
- rate limiting and quota enforcement (Section 12.18)
- secret scanning enabled at the repository and CI level
- signed container images (cosign keyless OIDC) and SLSA provenance attestations for release artifacts
- SBOM (CycloneDX) generation for every release

### Compliance Readiness
The architecture should support privacy-sensitive deployments, regional residency controls, and data minimization.

---

## 23. Cross-Language SDK Strategy

### Recommended Architecture
- Core scoring engine in Rust
- Node bindings via N-API / `napi-rs`
- Python bindings via PyO3 with `maturin`-based packaging and `abi3` wheels where compatible with the required feature set
- Go bindings via `cgo` over a stable C ABI exported by the Rust core and released as a Go module
- Java bindings via the Foreign Function & Memory API on JDK 22+; JNI is a compatibility fallback only when the supported runtime matrix requires older JDKs
- Native Rust crate for Rust consumers

### Rationale
This ensures:
- scoring parity across languages
- single source of truth for formulas and schema
- lower risk of implementation drift
- easier golden-fixture validation
- lower long-term packaging and native interop brittleness

### SDK Design Requirements
All SDKs must expose:
- async/non-blocking `track()`
- batch support
- schema validation
- identical semantic behavior
- identical arithmetic precision and rounding behavior
- documented timeout, acknowledgement, and cancellation semantics for native and image modes
- version reporting

### SDK Contract Requirement
Each SDK must satisfy a shared contract suite proving:
- consistent input validation behavior
- identical normalized representations
- identical scoring outputs
- identical explainability structure
- identical arithmetic precision, rounding order, and fallback behavior
- identical treatment of extension metadata during normalization and scoring
- consistent error-class mapping where language semantics allow

---

## 24. Framework Integrations

### MVP Integrations
- LangGraph
- Google ADK

### Phase 2 Integrations
- PydanticAI

### Future Integrations
- Langfuse webhook adapter
- OpenTelemetry bridge
- generic webhook adapter
- custom middleware templates

Each adapter must translate native telemetry into the canonical ai-heeczer schema with minimal manual effort.

---

## 25. Calibration and Benchmarking

ai-heeczer must support benchmark packs and scoring profile calibration.

### Calibration Goals
- align outputs with real-world human effort expectations
- improve confidence in enterprise reporting
- adapt the model to domain-specific workflows

### Example Benchmark Categories
- summarize a long document
- generate an API specification
- draft release notes
- triage a failed CI pipeline
- perform root cause analysis across logs and traces

### Calibration Requirements
- benchmark task definitions must be versioned
- profiles must be scoped and auditable
- recalculation after calibration changes must be supported

---

## 26. Developer Experience Requirements

### 26.1 Comprehensive README
The root README must be detailed enough that a new contributor can understand the product, setup the repo, run tests, and understand release behavior without reading source code first.

### 26.2 Makefile
The Makefile shall be treated as the primary human-facing command interface for common repository actions. Other scripts may exist, but README examples and CI examples should prefer Make targets.

### 26.3 Example Projects
The repo shall include runnable examples demonstrating at least:
- embedded mode usage from JS/TS
- embedded mode usage from Python
- embedded mode usage from Go
- sample canonical event ingestion
- basic dashboard run instructions

### 26.4 Development Environment
The repo shall document supported toolchain versions and bootstrap steps for:
- Rust
- Node.js
- Python
- Go
- Java
- Docker where required

---

## 27. CI/CD and Release Requirements

### 27.1 GitHub Actions Pipeline
The project shall use GitHub Actions for CI/CD with `release-please` manifest mode as the default release control plane. The pipeline shall include, at minimum:

- lint and formatting validation
- unit tests
- integration tests
- contract tests
- cross-language parity tests
- UI tests when UI scope is affected
- packaging checks
- security scanning
- release dry-run validation on PRs
- semantic version computation and publish on eligible merges/tags

### 27.2 Release Gate
A release must occur automatically only when:
- all required CI jobs pass
- repository branch protection rules are satisfied
- required approvals are present
- package packaging/verification steps pass
- release notes/changelog generation succeeds

### 27.3 Semantic Versioning
The project shall use semantic versioning and automate version determination from governed inputs such as conventional commits or an equivalent semantic release control mechanism.

Rules:
- breaking changes increment MAJOR
- backward-compatible features increment MINOR
- backward-compatible fixes increment PATCH
- release artifacts across all ecosystems must share the same product version

### 27.4 Publish Targets
The release workflow shall publish:
- npm package(s)
- PyPI package(s)
- Go module semantic version tags on the canonical module path
- crates.io crate(s)
- Maven Central artifact(s)
- GitHub Release assets
- optionally OCI container images for server/dashboard distributions

### 27.5 Release Integrity
The pipeline shall support:
- a versioned release manifest capturing the intended version, publish targets, artifact digests, and publish status
- immutable tagged releases
- changelog generation
- checksums (SHA-256) for binary/source artifacts where applicable
- SLSA Build Level 3 provenance attestations for all release artifacts
- CycloneDX SBOMs attached to each GitHub Release
- cosign keyless signatures for container images and release artifacts
- resumable publish procedures and operator intervention for partial failures
- a partial-publish state when one or more registries have published successfully but the release manifest is not yet complete
- release completion only after all required publish targets report success for the same version

### 27.6 Central Repository Credentials
Secrets for npm, PyPI, crates.io, Maven Central, container registries, and GitHub Release publication must be managed using GitHub Actions secrets or trusted publishing where supported. Trusted publishing is preferred whenever a target ecosystem supports it. Go module distribution relies primarily on repository reachability and semantic version tags rather than a separate central registry credential; private Go distribution may additionally require source control or private proxy credentials.

---

## 28. Test Strategy Requirements

### 28.1 TDD Policy
Development must follow test-driven development as a default engineering policy.

For every substantive feature or behavior change:
1. define scope and contract
2. write or update failing tests
3. implement behavior
4. make tests pass
5. refactor safely
6. update docs and fixtures

### 28.2 Required Test Layers
- **Unit tests** for deterministic logic and edge cases
- **Integration tests** for multi-component behavior
- **Contract tests** for schemas, APIs, SDK parity, and adapter invariants
- **Golden fixture tests** for stable scoring outputs
- **Benchmark smoke tests** for critical performance claims
- **UI tests** for dashboard/admin critical user flows
- **Migration tests** for schema/data changes
- **Release pipeline tests** for versioning and packaging behavior

### 28.3 Test Precision Requirement
Requirements must be expressed in a way that can be directly translated into executable tests. Ambiguous scope is unacceptable for core contracts.

### 28.4 UI Test Framework
Where UI behavior is in scope, the project shall use a suitable UI test framework capable of end-to-end or browser-level validation. Framework choice is implementation-specific, but the requirement for UI regression coverage is mandatory.

---

## 29. Non-Functional Requirements

### Performance
- in native mode, `track()` shall return control to the caller in under 2 ms p95 under the documented benchmark profile for validation plus local enqueue or persistence
- in image mode asynchronous acknowledgement mode, the client acknowledgement path shall complete in under 50 ms p95 under the documented same-region benchmark profile
- scoring throughput shall scale predictably under batch and worker execution

### Scalability
- ingestion API shall support at least 10,000 accepted asynchronous enqueue requests/sec/node in production mode under the documented benchmark profile
- benchmark claims shall publish payload size, authentication mode, durability mode, queue backend, storage backend, and reference hardware assumptions
- aggregate views shall remain performant at high event volumes

### Availability
- worker mode shall support retries, DLQ handling, and replay

### Portability
- heeczer ingestion service image shall be containerized and cloud-neutral
- dashboard shall be separately containerized and cloud-neutral

### Observability
- the platform shall expose Prometheus-compatible metrics
- structured logs shall be emitted for ingestion, queueing, scoring, and persistence flows
- queue depth, age, throughput, status counts, retry counts, and dead-letter counts shall be observable

### Maintainability
- all repository-critical actions must be available via Makefile
- all public contracts must be versioned
- release automation must be reproducible from CI
- README instructions must remain current with CI-verified commands

---

## 30. Risks and Mitigations

### False Precision
Risk: users interpret outputs as exact truth.  
Mitigation: confidence bands, explicit disclaimers, explainability, and calibration.

### Gaming the Model
Risk: loops or retries inflate value claims.  
Mitigation: caps, anomaly detection, bounded multipliers, and audit review.

### Incomplete Telemetry
Risk: poor instrumentation creates misleading results.  
Mitigation: schema validation, confidence penalties, and telemetry completeness reporting.

### Cross-Language Complexity
Risk: FFI and multi-language release pipelines increase engineering burden.  
Mitigation: Rust-first architecture, shared fixtures, strong CI/CD, and release contracts.

### Misuse for Individual Surveillance
Risk: organizations over-index on individual productivity scoring.  
Mitigation: policy guidance, default aggregation at team/project level, and role-based visibility.

### Release Drift
Risk: package versions diverge between ecosystems.  
Mitigation: one semantic release source of truth, one version per repo release, and publish verification.

### Documentation Drift
Risk: README and commands become stale.  
Mitigation: README examples anchored to Makefile targets, CI docs validation where feasible, and periodic release checklist enforcement.

---

## 31. MVP Definition

### Included in MVP
- Rust core engine
- JS/TS SDK
- Python SDK
- Go SDK
- canonical schema v1
- embedded mode
- preview image-mode ingest API and service contract
- SQLite adapter
- read-only dashboard
- explainability trace
- confidence score
- default tiers
- default financial model
- LangGraph integration
- Google ADK integration
- comprehensive README
- Makefile with standard developer targets
- GitHub Actions CI for lint, unit, integration, contract, and parity tests
- automated semantic release pipeline for supported MVP ecosystems

### Excluded from MVP
- image mode as the default production deployment
- Java SDK runtime parity at launch if not ready
- full PostgreSQL worker mode
- RBAC
- advanced calibration UI
- anomaly detection
- full Langfuse integration
- enterprise reporting packs

---

## 32. Roadmap

### Phase 1 — Foundation
Core engine, JS/TS + Python + Go SDKs, SQLite, local dashboard, baseline integrations, README, Makefile, and CI quality gates.

### Phase 2 — Production Readiness
Worker mode, PostgreSQL, admin console, RBAC, audit logs, Java support, mature publish flows to all target ecosystems.

### Phase 3 — Enterprise Analytics
Calibration workflows, anomaly detection, export integrations, benchmark packs, advanced reporting.

### Phase 4 — Ecosystem Expansion
OpenTelemetry bridge, Langfuse integration, profile marketplace, optional AI-assisted calibration.

---

## 33. Acceptance Criteria

1. Given identical JSON input, Rust, Go, Python, and JS/TS produce identical HEE, FEC, confidence, and explainability outputs.
2. In native mode, `track()` blocks the host thread for less than 2 ms p95 under the documented benchmark profile.
3. In image mode asynchronous acknowledgement mode, accepted requests receive acknowledgement in under 50 ms p95 under the documented same-region benchmark profile.
4. SQLite and PostgreSQL adapters can be swapped without application code changes where the adapter contract applies.
5. Every persisted score includes score version, tier version, rate version, and explainability trace.
6. Re-scoring preserves prior score versions and creates a new score record linked to the same raw event.
7. Dashboard renders aggregated data at target scale with acceptable latency.
8. Root README documents setup, test, build, release, publish, and common troubleshooting clearly.
9. Makefile targets exist and work for the documented local developer lifecycle.
10. CI blocks merge or release if required unit, integration, contract, parity, packaging, or UI tests fail.
11. A successful release pipeline computes the next semantic version, publishes changelog and tag, and updates all supported package repositories and module distribution channels consistently for the same version using the release manifest.
12. No feature is marked complete unless corresponding tests and contract updates are present.

---

## 34. Open Questions

1. Should the product distinguish between **replacement value** and **augmentation value** in a first-class way?
2. Should task category inference be partially automated when omitted by the client?
3. Should non-engineering role packs ship in MVP or Phase 2?
4. Should review burden be modeled separately from human-in-loop?
5. Should calibration profiles be exportable and shareable across organizations?
6. Which queue backend should be the default reference implementation for image mode?

---

## 35. Recommended Repository Structure

```text
/docs
  /prd
    ai-heeczer-prd.md
  /adr
    0001-rust-core-engine.md
    0002-canonical-event-schema.md
    0003-scoring-versioning.md
    0004-database-migration-tooling.md
    0005-ingestion-service-language.md
    0006-queue-backend.md
    0007-monorepo-tooling.md
    0008-dashboard-ui-framework.md
    0009-release-control-plane.md
    0010-local-developer-cli.md
  /architecture
    system-overview.md
    data-model.md
    deployment-modes.md
  /plan
    0000-overview.md
    0001-schema-and-contracts.md
    0002-scoring-core.md
    0003-storage-and-migrations.md
    0004-ingestion-service.md
    0005-sdk-jsts.md
    0006-sdk-python.md
    0007-sdk-go.md
    0008-sdk-rust.md
    0009-sdk-java.md
    0010-dashboard.md
    0011-framework-adapters.md
    0012-cicd-release.md
    0013-developer-experience.md
    0014-security-and-privacy.md
    0015-calibration-benchmarks.md
  /agents
    AGENT_HARNESS.md
  /implementation
    milestone-plan.md
    backlog.md
    test-strategy.md
/examples
/core
/bindings
/server
/dashboard
/scripts
/.github
  copilot-instructions.md
  chatmodes/
  workflows/
LICENSE
CONTRIBUTING.md
CODE_OF_CONDUCT.md
SECURITY.md
Makefile
README.md
```

---

## 36. GitHub / Copilot Execution Guidance

This PRD is intended to be used directly by GitHub Copilot and implementation agents.

### Required Working Style for Copilot
When implementing against this PRD, Copilot or any coding agent must:

1. Treat this document as product source of truth.
2. Preserve deterministic scoring behavior.
3. Avoid introducing prompt or output content storage unless explicitly approved.
4. Keep formulas, schema, and configuration versioned.
5. Maintain parity across language bindings.
6. Prefer incremental, test-backed implementation.
7. Do not silently change scoring semantics without updating score version and docs.
8. Write golden-fixture tests for all scoring changes.
9. Separate raw event persistence from computed score persistence.
10. Keep native mode lightweight and non-blocking.
11. Use Makefile targets where possible when adding examples to docs.
12. Implement tests before or alongside code; do not defer them.

### Required Deliverables for Each Major Feature
For each feature implementation, generate:
- code
- tests
- docs updates
- migration scripts if applicable
- example usage
- acceptance criteria mapping
- release impact note if the change affects package contracts or public APIs

### Implementation Order for Copilot
1. canonical schema
2. Rust scoring core
3. golden-fixture tests
4. JS/TS bindings
5. Python bindings
6. Go bindings
7. SQLite adapter
8. read-only dashboard
9. framework adapters
10. CI/release automation
11. production server mode

---

## 37. Engineering Milestones

### Milestone 1 — Schema, Contracts, and Scoring Core
- define schema v1
- define scoring contracts
- implement normalization
- implement deterministic scoring engine
- add explainability trace
- add confidence score
- create golden fixture suite

### Milestone 2 — SDK Foundation
- JS/TS SDK
- Python SDK
- Go SDK
- embedded async track flow
- local validation
- benchmark tests
- contract parity tests

### Milestone 3 — Persistence and Aggregation
- SQLite adapter
- raw event tables
- score tables
- aggregate tables
- local replay support

### Milestone 4 — Dashboard MVP
- summary metrics
- trend views
- event drill-down
- explainability view
- UI tests for critical flows

### Milestone 5 — CI/CD, README, and Makefile
- GitHub Actions CI matrix
- semantic version automation
- central repository publishing
- comprehensive README
- Makefile commands
- release dry-run pipeline

### Milestone 6 — Integrations and Production Mode
- LangGraph adapter
- Google ADK adapter
- worker mode foundation
- PostgreSQL path
- production deployment notes

---

## 38. Test Strategy Summary

### Required Test Types
- unit tests for scoring formula and normalization
- golden-fixture parity tests across languages
- schema validation tests
- persistence tests
- migration tests
- benchmark tests for `track()` latency
- integration tests for framework adapters
- UI tests for dashboard/admin workflows when in scope
- release pipeline verification tests

### Mandatory Quality Bar
No scoring or public contract change may be merged without:
- updated fixtures
- deterministic test coverage
- score version review
- explainability output validation
- release impact assessment where relevant

---

## 39. Product Positioning Statement

**ai-heeczer converts agentic AI telemetry into human-equivalent effort and cost analytics.**  
It helps organizations measure AI work in the language leadership understands: time, labor, and value.

---

## 40. Suggested Taglines

- Measure AI work in human terms
- From tokens to labor-equivalent value
- The effort and cost analytics layer for agentic AI
- Translate AI execution into human-equivalent business metrics
