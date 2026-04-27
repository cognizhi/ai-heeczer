use heeczer::{Client, Event, IngestInput};
use std::path::{Path, PathBuf};

fn valid_fixture_paths() -> Vec<PathBuf> {
    let dir = std::env::var("HEECZER_PARITY_FIXTURE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../core/schema/fixtures/events/valid")
        });
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("valid fixture dir should be readable")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    paths.sort();
    paths
}

fn reference_path(reference_dir: &Path, fixture_path: &Path) -> PathBuf {
    reference_dir.join(format!(
        "{}.json",
        fixture_path.file_stem().unwrap().to_string_lossy()
    ))
}

#[test]
fn native_score_results_match_rust_reference_json() {
    let Ok(reference_dir) = std::env::var("HEECZER_PARITY_REFERENCE_DIR") else {
        eprintln!("skipping SDK parity byte comparison; HEECZER_PARITY_REFERENCE_DIR is unset");
        return;
    };
    let reference_dir = PathBuf::from(reference_dir);
    let fixtures = valid_fixture_paths();
    assert!(!fixtures.is_empty(), "expected at least one valid fixture");

    let client = Client::native();
    for fixture_path in fixtures {
        let raw = std::fs::read_to_string(&fixture_path).expect("fixture should be readable");
        let event: Event = serde_json::from_str(&raw).expect("fixture should parse as Event");
        let result = client
            .score_event(IngestInput {
                workspace_id: event.workspace_id.clone(),
                event,
                profile: None,
                tier_set: None,
                tier_override: None,
            })
            .unwrap_or_else(|err| panic!("{} should score: {err}", fixture_path.display()));
        let actual = serde_json::to_string(&result).expect("score result should serialise");
        let expected = std::fs::read_to_string(reference_path(&reference_dir, &fixture_path))
            .expect("reference output should exist");
        assert_eq!(
            actual,
            expected.trim_end(),
            "{} should match the Rust reference scorer byte-for-byte",
            fixture_path.display()
        );
    }
}
