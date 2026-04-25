package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonValue;

/**
 * Closed task-outcome enum (mirrors the Rust {@code Outcome} enum in
 * {@code heeczer_core::event}). See {@code core/schema/event.v1.json}
 * §task.outcome.
 */
public enum Outcome {
    success("success"),
    partial_success("partial_success"),
    failure("failure"),
    timeout("timeout");

    private final String value;

    Outcome(String value) {
        this.value = value;
    }

    @JsonValue
    public String getValue() {
        return value;
    }
}
