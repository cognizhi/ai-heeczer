//! Quickstart: native (in-process) scoring with the `heeczer` Rust SDK.
//!
//! Run with:
//!
//! ```bash
//! cargo run -p heeczer --example quickstart
//! ```
//!
//! Loads the canonical demo event from `examples/event.json` (relative to
//! the workspace root) and prints the resulting `ScoreResult` summary plus
//! the JSON for downstream tooling.

use std::path::PathBuf;

use heeczer::{Client, IngestInput};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The example file lives at <workspace_root>/examples/event.json.
    // CARGO_MANIFEST_DIR points at bindings/heeczer-rs/, so jump two up.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let event_path = manifest.join("../../examples/event.json");
    let body = std::fs::read_to_string(&event_path)?;
    let event: heeczer::Event = serde_json::from_str(&body)?;

    let client = Client::native();
    let result = client.score_event(IngestInput {
        workspace_id: "ws_default".into(),
        event,
        profile: None,  // use embedded default profile
        tier_set: None, // use embedded default tier set
        tier_override: None,
    })?;

    println!("=== summary ===");
    println!("{}", result.human_summary);
    println!();
    println!("=== full ScoreResult JSON ===");
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
