package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;

/**
 * Optional execution context block of a canonical event.
 * All fields are optional; null values are omitted from serialized JSON.
 */
@JsonIgnoreProperties(ignoreUnknown = false)
@JsonInclude(JsonInclude.Include.NON_NULL)
public class EventContext {

    @JsonProperty("human_in_loop")
    public Boolean humanInLoop;

    @JsonProperty("review_required")
    public Boolean reviewRequired;

    @JsonProperty("temperature")
    public Double temperature;

    @JsonProperty("risk_class")
    public EventRiskClass riskClass;

    @JsonProperty("tags")
    public List<String> tags;

    public EventContext() {
    }
}
