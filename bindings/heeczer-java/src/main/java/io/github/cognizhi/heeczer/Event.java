package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Canonical ai-heeczer telemetry event (v1).
 *
 * <p>
 * Mirrors {@code heeczer_core::Event} (Rust) and the JSON Schema at
 * {@code core/schema/event.v1.json}. Construct this type and pass it to
 * {@link HeeczerClient#ingestEvent(String, Object)} as the event argument.
 *
 * <p>
 * Null fields are omitted from serialized JSON (matching the Rust
 * {@code skip_serializing_if = "Option::is_none"} behavior). Use
 * {@link com.fasterxml.jackson.databind.ObjectMapper} with
 * {@code FAIL_ON_UNKNOWN_PROPERTIES = true} for strict deserialization.
 *
 * <p>
 * Example:
 *
 * <pre>{@code
 * Event event = new Event();
 * event.specVersion = "1.0";
 * event.eventId = UUID.randomUUID().toString();
 * event.timestamp = Instant.now().toString();
 * event.frameworkSource = "langgraph";
 * event.workspaceId = "ws_default";
 * event.task = new EventTask("summarise_pr", Outcome.success);
 * event.metrics = new EventMetrics(3200);
 * event.meta = new EventMeta("java", HeeczerClient.SDK_VERSION);
 *
 * IngestEventResponse resp = client.ingestEvent("ws_default", event);
 * }</pre>
 */
@JsonIgnoreProperties(ignoreUnknown = false)
@JsonInclude(JsonInclude.Include.NON_NULL)
public class Event {

    /** Must be {@code "1.0"} for v1 events. Required. */
    @JsonProperty("spec_version")
    public String specVersion;

    /** RFC 4122 UUID; primary idempotency key (PRD §12.19). Required. */
    @JsonProperty("event_id")
    public String eventId;

    @JsonProperty("correlation_id")
    public String correlationId;

    /** RFC 3339 / ISO 8601 timestamp in UTC. Required. */
    @JsonProperty("timestamp")
    public String timestamp;

    /**
     * Originating framework slug ({@code "langgraph"}, {@code "google_adk"}, …).
     * Required.
     */
    @JsonProperty("framework_source")
    public String frameworkSource;

    /** Tenant workspace id. Required. */
    @JsonProperty("workspace_id")
    public String workspaceId;

    @JsonProperty("project_id")
    public String projectId;

    @JsonProperty("identity")
    public EventIdentity identity;

    /** Task descriptor. Required. */
    @JsonProperty("task")
    public EventTask task;

    /** Required telemetry metrics. Required. */
    @JsonProperty("metrics")
    public EventMetrics metrics;

    @JsonProperty("context")
    public EventContext context;

    /** SDK metadata. Required. */
    @JsonProperty("meta")
    public EventMeta meta;

    public Event() {
    }
}
