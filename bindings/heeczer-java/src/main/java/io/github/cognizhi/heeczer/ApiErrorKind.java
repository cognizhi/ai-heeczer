package io.github.cognizhi.heeczer;

/**
 * Closed enum of error kinds the ingestion service can return (envelope v1).
 * Mirrors {@code services/heeczer-ingest/src/error.rs}.
 */
public enum ApiErrorKind {
    schema,
    bad_request,
    scoring,
    storage,
    not_found,
    forbidden,
    feature_disabled,
    unknown
}
