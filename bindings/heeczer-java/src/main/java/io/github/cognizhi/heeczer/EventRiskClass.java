package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonValue;

/**
 * Closed risk-classification enum (mirrors the Rust {@code RiskClass} enum in
 * {@code heeczer_core::event}). See {@code core/schema/event.v1.json}
 * §context.risk_class.
 */
public enum EventRiskClass {
    low("low"),
    medium("medium"),
    high("high");

    private final String value;

    EventRiskClass(String value) {
        this.value = value;
    }

    @JsonValue
    public String getValue() {
        return value;
    }
}
