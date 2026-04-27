//! ai-heeczer scoring core.
//!
//! Deterministic, fixed-point scoring engine that converts a canonical
//! [`Event`] into a [`ScoreResult`] containing HEE, FEC, confidence, and an
//! explainability trace. This crate is the single source of truth used by every
//! language binding (see ADR-0001).
//!
//! # Quickstart
//! ```no_run
//! use heeczer_core::{Event, ScoringProfile, TierSet, score};
//!
//! let event_json = std::fs::read_to_string("event.json").unwrap();
//! let event: Event = serde_json::from_str(&event_json).unwrap();
//! let profile = ScoringProfile::default_v1();
//! let tiers = TierSet::default_v1();
//!
//! let result = score(&event, &profile, &tiers, None).unwrap();
//! println!("{}", serde_json::to_string_pretty(&result).unwrap());
//! ```

#![cfg_attr(not(test), warn(missing_docs))]

pub mod calibration;
pub mod confidence;
pub mod error;
pub mod event;
pub mod explain;
pub mod normalize;
pub mod profile;
pub mod schema;
pub mod scoring;
pub mod tier;
pub mod version;

pub use calibration::{
    build_suggested_profile, run_calibration, BenchmarkPack, CalibrationRunReport,
};
pub use error::{Error, Result};
pub use event::Event;
pub use explain::{BcuBreakdown, ContextMultiplierTrace, ScoreResult, TierTrace};
pub use profile::ScoringProfile;
pub use scoring::score;
pub use tier::{Tier, TierSet};
pub use version::{SCORING_VERSION, SPEC_VERSION};
