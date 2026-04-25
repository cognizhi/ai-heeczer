package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Telemetry metrics block of a canonical event.
 * {@code duration_ms} is required; all other fields are optional.
 */
@JsonIgnoreProperties(ignoreUnknown = false)
@JsonInclude(JsonInclude.Include.NON_NULL)
public class EventMetrics {

    /** Wall-clock task duration in milliseconds. Required. */
    @JsonProperty("duration_ms")
    public long durationMs;

    @JsonProperty("tokens_prompt")
    public Long tokensPrompt;

    @JsonProperty("tokens_completion")
    public Long tokensCompletion;

    @JsonProperty("tool_call_count")
    public Integer toolCallCount;

    @JsonProperty("workflow_steps")
    public Integer workflowSteps;

    @JsonProperty("retries")
    public Integer retries;

    @JsonProperty("artifact_count")
    public Integer artifactCount;

    @JsonProperty("output_size_proxy")
    public Double outputSizeProxy;

    public EventMetrics() {
    }

    public EventMetrics(long durationMs) {
        this.durationMs = durationMs;
    }
}
