package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonAnyGetter;
import com.fasterxml.jackson.annotation.JsonAnySetter;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.HashMap;
import java.util.Map;

/**
 * Typed surface of the engine's scoring result.
 *
 * <p>Extra fields that the engine may add in future versions are accumulated in
 * {@link #extras()} so the typed surface never breaks on additive changes
 * (ADR-0003).
 */
public final class ScoreResult {

    @JsonProperty("scoring_version")   public String scoringVersion;
    @JsonProperty("spec_version")      public String specVersion;
    @JsonProperty("scoring_profile")   public String scoringProfile;
    @JsonProperty("category")          public String category;
    @JsonProperty("final_estimated_minutes") public String finalEstimatedMinutes;
    @JsonProperty("estimated_hours")   public String estimatedHours;
    @JsonProperty("estimated_days")    public String estimatedDays;
    @JsonProperty("financial_equivalent_cost") public String financialEquivalentCost;
    @JsonProperty("confidence_score")  public String confidenceScore;
    @JsonProperty("confidence_band")   public ConfidenceBand confidenceBand;
    @JsonProperty("human_summary")     public String humanSummary;

    private final Map<String, Object> extras = new HashMap<>();

    @JsonAnyGetter
    public Map<String, Object> extras() { return extras; }

    @JsonAnySetter
    public void setExtra(String key, Object value) { extras.put(key, value); }
}
