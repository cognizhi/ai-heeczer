package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonAnyGetter;
import com.fasterxml.jackson.annotation.JsonAnySetter;
import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonDeserializer;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.annotation.JsonDeserialize;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.io.IOException;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Set;

/**
 * Typed surface of the engine's scoring result.
 *
 * <p>Extra fields that the engine may add in future versions are accumulated in
 * {@link #extras()} so the typed surface never breaks on additive changes
 * (ADR-0003).
 */
@JsonDeserialize(using = ScoreResult.Deserializer.class)
public final class ScoreResult {

    private static final ObjectMapper MAPPER = new ObjectMapper();
    private static final Set<String> KNOWN_FIELDS = Set.of(
            "scoring_version",
            "spec_version",
            "scoring_profile",
            "category",
            "final_estimated_minutes",
            "estimated_hours",
            "estimated_days",
            "financial_equivalent_cost",
            "confidence_score",
            "confidence_band",
            "human_summary");

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

    @JsonIgnore
    private JsonNode raw;

    private final Map<String, Object> extras = new LinkedHashMap<>();

    @JsonAnyGetter
    public Map<String, Object> extras() { return extras; }

    @JsonAnySetter
    public void setExtra(String key, Object value) { extras.put(key, value); }

    /**
     * Return compact JSON for the score result, preserving additive engine
     * fields and their original order when this object came from the wire.
     */
    @JsonIgnore
    public String toJson() throws JsonProcessingException {
        if (raw != null) {
            return MAPPER.writeValueAsString(raw);
        }
        ObjectNode node = MAPPER.createObjectNode();
        put(node, "scoring_version", scoringVersion);
        put(node, "spec_version", specVersion);
        put(node, "scoring_profile", scoringProfile);
        put(node, "category", category);
        put(node, "final_estimated_minutes", finalEstimatedMinutes);
        put(node, "estimated_hours", estimatedHours);
        put(node, "estimated_days", estimatedDays);
        put(node, "financial_equivalent_cost", financialEquivalentCost);
        put(node, "confidence_score", confidenceScore);
        if (confidenceBand != null) {
            node.put("confidence_band", confidenceBand.name());
        }
        put(node, "human_summary", humanSummary);
        for (Map.Entry<String, Object> entry : extras.entrySet()) {
            node.set(entry.getKey(), MAPPER.valueToTree(entry.getValue()));
        }
        return MAPPER.writeValueAsString(node);
    }

    private static void put(ObjectNode node, String key, String value) {
        if (value != null) {
            node.put(key, value);
        }
    }

    static final class Deserializer extends JsonDeserializer<ScoreResult> {
        @Override
        public ScoreResult deserialize(JsonParser parser, DeserializationContext context)
                throws IOException {
            JsonNode node = parser.getCodec().readTree(parser);
            ScoreResult result = new ScoreResult();
            result.raw = node.deepCopy();
            result.scoringVersion = text(node, "scoring_version");
            result.specVersion = text(node, "spec_version");
            result.scoringProfile = text(node, "scoring_profile");
            result.category = text(node, "category");
            result.finalEstimatedMinutes = text(node, "final_estimated_minutes");
            result.estimatedHours = text(node, "estimated_hours");
            result.estimatedDays = text(node, "estimated_days");
            result.financialEquivalentCost = text(node, "financial_equivalent_cost");
            result.confidenceScore = text(node, "confidence_score");
            String band = text(node, "confidence_band");
            if (band != null) {
                result.confidenceBand = ConfidenceBand.valueOf(band);
            }
            result.humanSummary = text(node, "human_summary");

            Iterator<Map.Entry<String, JsonNode>> fields = node.fields();
            while (fields.hasNext()) {
                Map.Entry<String, JsonNode> field = fields.next();
                if (!KNOWN_FIELDS.contains(field.getKey())) {
                    result.extras.put(field.getKey(), MAPPER.convertValue(field.getValue(), Object.class));
                }
            }
            return result;
        }

        private static String text(JsonNode node, String key) {
            JsonNode value = node.get(key);
            return value == null || value.isNull() ? null : value.asText();
        }
    }
}
