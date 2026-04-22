package io.github.cognizhi.heeczer;

/**
 * Confidence band mirroring the Rust {@code ConfidenceBand} enum and the
 * ingestion service's closed-kind contract (ADR-0011).
 */
public enum ConfidenceBand {
    Low, Medium, High
}
