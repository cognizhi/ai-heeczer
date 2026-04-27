use criterion::{criterion_group, criterion_main, Criterion};
use heeczer_core::{score, Event, ScoringProfile, TierSet};
use std::hint::black_box;

fn load_canonical() -> Event {
    let body = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../schema/fixtures/events/valid/01-prd-canonical.json"),
    )
    .unwrap();
    serde_json::from_str(&body).unwrap()
}

fn score_canonical(c: &mut Criterion) {
    let event = load_canonical();
    let profile = ScoringProfile::default_v1();
    let tiers = TierSet::default_v1();

    c.bench_function("score_canonical", |b| {
        b.iter(|| {
            score(
                black_box(&event),
                black_box(&profile),
                black_box(&tiers),
                None,
            )
            .unwrap();
        });
    });
}

criterion_group!(benches, score_canonical);
criterion_main!(benches);
