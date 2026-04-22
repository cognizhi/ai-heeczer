package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.HashMap;
import java.util.Map;
import java.util.Objects;

/**
 * Synchronous HTTP client for the ai-heeczer ingestion service (plan 0009).
 *
 * <p>Speaks the {@code envelope_version=1} contract documented in ADR-0011.
 * Uses {@link java.net.http.HttpClient} (JDK 17+; no third-party HTTP dep).
 *
 * <p>Instances are thread-safe and reusable. Create one per JVM and share it.
 *
 * <p>All methods throw {@link HeeczerApiException} on non-2xx responses and
 * {@link java.io.IOException} / {@link InterruptedException} on transport
 * failures.
 */
public final class HeeczerClient implements AutoCloseable {

    /** SDK version constant. */
    public static final String SDK_VERSION = "0.1.0";

    private static final ObjectMapper MAPPER = new ObjectMapper()
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);

    private final String baseUrl;
    private final String apiKey;
    private final HttpClient http;

    private HeeczerClient(Builder builder) {
        this.baseUrl = builder.baseUrl;
        this.apiKey  = builder.apiKey;
        this.http    = builder.httpClient != null ? builder.httpClient
                : HttpClient.newBuilder()
                    .connectTimeout(Duration.ofSeconds(10))
                    .build();
    }

    /** Builder for {@link HeeczerClient}. */
    public static final class Builder {
        private final String baseUrl;
        private String apiKey;
        private HttpClient httpClient;

        /**
         * @param baseUrl Base URL of the ingestion service, e.g.
         *                {@code https://ingest.example.com}. Must not be null or empty.
         */
        public Builder(String baseUrl) {
            if (baseUrl == null || baseUrl.isBlank()) {
                throw new IllegalArgumentException("baseUrl is required");
            }
            // Strip trailing slash.
            this.baseUrl = baseUrl.endsWith("/")
                    ? baseUrl.substring(0, baseUrl.length() - 1)
                    : baseUrl;
        }

        /** Optional API key sent as {@code x-heeczer-api-key}. */
        public Builder apiKey(String key) {
            this.apiKey = key;
            return this;
        }

        /**
         * Inject a custom {@link HttpClient}. Useful for tests and for
         * environments that need a proxy or mutual TLS.
         */
        public Builder httpClient(HttpClient client) {
            this.httpClient = client;
            return this;
        }

        public HeeczerClient build() {
            return new HeeczerClient(this);
        }
    }

    // -------------------------------------------------------------------------
    // Public API
    // -------------------------------------------------------------------------

    /**
     * Liveness probe; returns {@code true} if the service responds 2xx to
     * {@code GET /healthz}.
     */
    public boolean healthz() throws IOException, InterruptedException {
        HttpRequest req = baseRequest("/healthz").GET().build();
        HttpResponse<Void> resp = http.send(req, HttpResponse.BodyHandlers.discarding());
        return resp.statusCode() >= 200 && resp.statusCode() < 300;
    }

    /**
     * Returns the engine + spec versions advertised by the service.
     */
    public VersionResponse version() throws IOException, InterruptedException {
        return getJson("/v1/version", VersionResponse.class);
    }

    /**
     * Validate, score, and persist a single canonical event.
     *
     * @param workspaceId tenant workspace identifier
     * @param event       the canonical event as a Jackson-serialisable object
     *                    (e.g. {@link com.fasterxml.jackson.databind.JsonNode},
     *                    a Map, or a POJO)
     */
    public IngestEventResponse ingestEvent(String workspaceId, Object event)
            throws IOException, InterruptedException {
        Map<String, Object> body = new HashMap<>();
        body.put("workspace_id", workspaceId);
        body.put("event", event);
        return postJson("/v1/events", body, Map.of(), IngestEventResponse.class);
    }

    /**
     * Run the scoring pipeline back-to-back without persisting. Always sends
     * the {@code x-heeczer-tester: 1} header so deployments without the role
     * return a structured {@code forbidden} envelope and those without the
     * feature flag return {@code feature_disabled}.
     */
    public TestPipelineResponse testScorePipeline(TestPipelineRequest request)
            throws IOException, InterruptedException {
        Objects.requireNonNull(request, "request");
        Map<String, Object> body = new HashMap<>();
        body.put("event", request.event);
        if (request.profile != null)      body.put("profile",        request.profile);
        if (request.tierSet != null)      body.put("tier_set",       request.tierSet);
        if (request.tierOverride != null) body.put("tier_override",  request.tierOverride);
        return postJson("/v1/test/score-pipeline", body,
                Map.of("x-heeczer-tester", "1"), TestPipelineResponse.class);
    }

    @Override
    public void close() {
        // HttpClient does not require explicit closing in JDK 17,
        // but AutoCloseable allows try-with-resources usage.
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    private <T> T getJson(String path, Class<T> type)
            throws IOException, InterruptedException {
        HttpRequest req = baseRequest(path).GET().build();
        return send(req, type);
    }

    private <T> T postJson(String path, Object body, Map<String, String> extraHeaders,
                           Class<T> type) throws IOException, InterruptedException {
        byte[] bytes = MAPPER.writeValueAsBytes(body);
        HttpRequest.Builder b = baseRequest(path)
                .header("content-type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofByteArray(bytes));
        extraHeaders.forEach(b::header);
        return send(b.build(), type);
    }

    private <T> T send(HttpRequest req, Class<T> type)
            throws IOException, InterruptedException {
        HttpResponse<byte[]> resp = http.send(req, HttpResponse.BodyHandlers.ofByteArray());
        if (resp.statusCode() >= 200 && resp.statusCode() < 300) {
            return MAPPER.readValue(resp.body(), type);
        }
        // Parse the error envelope.
        ApiErrorKind kind = ApiErrorKind.unknown;
        String message = new String(resp.body(), StandardCharsets.UTF_8);
        try {
            JsonNode root = MAPPER.readTree(resp.body());
            if (root.has("error")) {
                JsonNode err = root.get("error");
                if (err.has("kind")) {
                    kind = ApiErrorKind.fromWireValue(err.get("kind").asText());
                }
                if (err.has("message")) message = err.get("message").asText();
            }
        } catch (Exception ignored) {
            // Non-JSON body → keep raw text as message.
        }
        throw new HeeczerApiException(resp.statusCode(), kind, message);
    }

    private HttpRequest.Builder baseRequest(String path) {
        HttpRequest.Builder b = HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + path))
                .timeout(Duration.ofSeconds(30));
        if (apiKey != null && !apiKey.isBlank()) {
            b.header("x-heeczer-api-key", apiKey);
        }
        return b;
    }

    // -------------------------------------------------------------------------
    // Nested request DTO
    // -------------------------------------------------------------------------

    /** Input for {@link #testScorePipeline}. */
    public static final class TestPipelineRequest {
        /** The canonical event to score. */
        public Object event;
        /** Optional scoring profile override. */
        public Object profile;
        /** Optional tier set override. */
        public Object tierSet;
        /** Optional tier identifier override. */
        public String tierOverride;

        public TestPipelineRequest(Object event) {
            this.event = Objects.requireNonNull(event, "event");
        }
    }
}
