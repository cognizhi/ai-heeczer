package io.github.cognizhi.heeczer;

/**
 * Closed enum of error kinds the ingestion service can return (envelope v1).
 * Mirrors {@code services/heeczer-ingest/src/error.rs}.
 *
 * <p>
 * Constants intentionally use lowercase to match the wire-format strings
 * returned by the API. Use {@link #fromWireValue(String)} instead of
 * {@link #valueOf(String)} to safely parse server-side kind strings.
 */
public enum ApiErrorKind {
    schema,
    bad_request,
    scoring,
    storage,
    not_found,
    unauthorized,
    forbidden,
    conflict,
    payload_too_large,
    rate_limit_exceeded,
    feature_disabled,
    unsupported_spec_version,
    unavailable,
    unknown;

    /**
     * Parse a wire-format kind string. Returns {@link #unknown} for any
     * unrecognised value, rather than throwing {@link IllegalArgumentException}.
     *
     * @param value the kind string from the API error envelope
     * @return the matching enum constant, or {@link #unknown}
     */
    public static ApiErrorKind fromWireValue(String value) {
        if (value == null)
            return unknown;
        return switch (value) {
            case "schema" -> schema;
            case "bad_request" -> bad_request;
            case "scoring" -> scoring;
            case "storage" -> storage;
            case "not_found" -> not_found;
            case "unauthorized" -> unauthorized;
            case "forbidden" -> forbidden;
            case "conflict" -> conflict;
            case "payload_too_large" -> payload_too_large;
            case "rate_limit_exceeded" -> rate_limit_exceeded;
            case "feature_disabled" -> feature_disabled;
            case "unsupported_spec_version" -> unsupported_spec_version;
            case "unavailable" -> unavailable;
            default -> unknown;
        };
    }
}
