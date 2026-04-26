package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import java.io.IOException;
import java.io.InputStream;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.Arrays;
import java.util.HashMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Objects;
import java.util.Properties;
import java.util.Set;

/**
 * Synchronous HTTP client for the ai-heeczer ingestion service (plan 0009).
 *
 * <p>
 * Speaks the {@code envelope_version=1} contract documented in ADR-0011.
 * Uses {@link java.net.http.HttpClient} (JDK 17+; no third-party HTTP dep).
 *
 * <p>
 * Instances are thread-safe and reusable. Create one per JVM and share it.
 *
 * <p>
 * All methods throw {@link HeeczerApiException} on non-2xx responses and
 * {@link java.io.IOException} / {@link InterruptedException} on transport
 * failures.
 */
public final class HeeczerClient implements AutoCloseable {

    /** SDK version constant. */
    public static final String SDK_VERSION = loadSdkVersion();

    private static final ObjectMapper MAPPER = new ObjectMapper()
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);

        private final HeeczerMode mode;
    private final String baseUrl;
    private final String apiKey;
    private final HttpClient http;
        private final NativeBridge nativeBridge;
    private final Duration requestTimeout;
    private final int retryAttempts;
    private final Duration retryBackoff;
    private final Set<Integer> retryStatuses;

    private HeeczerClient(Builder builder) {
        this.mode = builder.mode;
        this.baseUrl = builder.baseUrl;
        this.apiKey = builder.apiKey;
        this.nativeBridge = builder.mode == HeeczerMode.native_mode
            ? (builder.nativeBridge != null ? builder.nativeBridge : NativeBridge.create())
            : null;
        this.http = builder.httpClient != null ? builder.httpClient
                : HttpClient.newBuilder()
                        .connectTimeout(Duration.ofSeconds(10))
                        .build();
        this.requestTimeout = builder.requestTimeout;
        this.retryAttempts = Math.max(1, builder.retryAttempts);
        this.retryBackoff = builder.retryBackoff;
        this.retryStatuses = Set.copyOf(builder.retryStatuses);
    }

    /** Builder for {@link HeeczerClient}. */
    public static final class Builder {
        private final String baseUrl;
        private String apiKey;
        private HttpClient httpClient;
        private HeeczerMode mode = HeeczerMode.image;
        private NativeBridge nativeBridge;
        private Duration requestTimeout = Duration.ofSeconds(30);
        private int retryAttempts = 2;
        private Duration retryBackoff = Duration.ofMillis(100);
        private Set<Integer> retryStatuses = new HashSet<>(Arrays.asList(408, 429, 500, 502, 503, 504));

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

        /**
         * Select image or native mode.
         */
        public Builder mode(HeeczerMode mode) {
            this.mode = Objects.requireNonNull(mode, "mode");
            return this;
        }

        Builder nativeBridgeForTests(NativeBridge bridge) {
            this.nativeBridge = Objects.requireNonNull(bridge, "bridge");
            return this;
        }

        /** Per-request timeout applied to every HTTP request. */
        public Builder requestTimeout(Duration timeout) {
            this.requestTimeout = Objects.requireNonNull(timeout, "timeout");
            return this;
        }

        /** Configure retries for transient status codes and IO failures. */
        public Builder retry(int attempts, Duration backoff, Integer... statuses) {
            this.retryAttempts = Math.max(1, attempts);
            this.retryBackoff = Objects.requireNonNull(backoff, "backoff");
            if (statuses.length > 0) {
                this.retryStatuses = new HashSet<>(Arrays.asList(statuses));
            }
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
        if (mode == HeeczerMode.native_mode) {
            nativeBridge.version();
            return true;
        }
        HttpRequest req = baseRequest("/healthz").GET().build();
        int status = sendStatus(req);
        return status >= 200 && status < 300;
    }

    /**
     * Returns the engine + spec versions advertised by the service.
     */
    public VersionResponse version() throws IOException, InterruptedException {
        if (mode == HeeczerMode.native_mode) {
            return nativeVersion();
        }
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
        if (mode == HeeczerMode.native_mode) {
            return nativeIngestEvent(workspaceId, event);
        }
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
        if (mode == HeeczerMode.native_mode) {
            return nativeTestScorePipeline(request);
        }
        Map<String, Object> body = new HashMap<>();
        body.put("event", request.event);
        if (request.profile != null)
            body.put("profile", request.profile);
        if (request.tierSet != null)
            body.put("tier_set", request.tierSet);
        if (request.tierOverride != null)
            body.put("tier_override", request.tierOverride);
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

    private VersionResponse nativeVersion() {
        NativeVersion version = nativeBridge.version();
        VersionResponse response = new VersionResponse();
        response.ok = true;
        response.envelopeVersion = "1";
        response.scoringVersion = version.scoringVersion();
        response.specVersion = version.specVersion();
        response.service = SDK_VERSION;
        return response;
    }

    private IngestEventResponse nativeIngestEvent(String workspaceId, Object event)
            throws IOException {
        JsonNode normalizedEvent = normalizeNativeEvent(workspaceId, event);
        IngestEventResponse response = new IngestEventResponse();
        response.ok = true;
        response.envelopeVersion = "1";
        response.eventId = extractEventId(normalizedEvent);
        response.score = nativeBridge.score(normalizedEvent, null, null, null);
        return response;
    }

    private TestPipelineResponse nativeTestScorePipeline(TestPipelineRequest request)
            throws IOException {
        TestPipelineResponse response = new TestPipelineResponse();
        response.ok = true;
        response.envelopeVersion = "1";
        response.score = nativeBridge.score(
                request.event,
                request.profile,
                request.tierSet,
                request.tierOverride);
        return response;
    }

    private JsonNode normalizeNativeEvent(String workspaceId, Object event) {
        JsonNode tree = MAPPER.valueToTree(event);
        if (!tree.isObject()) {
            return tree;
        }
        ObjectNode normalized = ((ObjectNode) tree).deepCopy();
        normalized.put("workspace_id", workspaceId);
        return normalized;
    }

    private String extractEventId(JsonNode event) {
        JsonNode eventId = event.get("event_id");
        if (eventId == null || eventId.isNull()) {
            return null;
        }
        return eventId.asText();
    }

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
        IOException lastIo = null;
        for (int attempt = 0; attempt < retryAttempts; attempt++) {
            HttpResponse<byte[]> resp;
            try {
                resp = http.send(req, HttpResponse.BodyHandlers.ofByteArray());
            } catch (IOException err) {
                lastIo = err;
                if (attempt == retryAttempts - 1)
                    throw err;
                sleepBeforeRetry(attempt);
                continue;
            }
            if (!retryStatuses.contains(resp.statusCode()) || attempt == retryAttempts - 1) {
                return handleResponse(resp, type);
            }
            sleepBeforeRetry(attempt);
        }
        throw lastIo != null ? lastIo : new IOException("heeczer retry loop ended without response");
    }

    private int sendStatus(HttpRequest req) throws IOException, InterruptedException {
        IOException lastIo = null;
        for (int attempt = 0; attempt < retryAttempts; attempt++) {
            HttpResponse<Void> resp;
            try {
                resp = http.send(req, HttpResponse.BodyHandlers.discarding());
            } catch (IOException err) {
                lastIo = err;
                if (attempt == retryAttempts - 1)
                    throw err;
                sleepBeforeRetry(attempt);
                continue;
            }
            if (!retryStatuses.contains(resp.statusCode()) || attempt == retryAttempts - 1) {
                return resp.statusCode();
            }
            sleepBeforeRetry(attempt);
        }
        throw lastIo != null ? lastIo : new IOException("heeczer retry loop ended without response");
    }

    private <T> T handleResponse(HttpResponse<byte[]> resp, Class<T> type) throws IOException {
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
                if (err.has("message"))
                    message = err.get("message").asText();
            }
        } catch (Exception ignored) {
            // Non-JSON body → keep raw text as message.
        }
        throw new HeeczerApiException(resp.statusCode(), kind, message);
    }

    private void sleepBeforeRetry(int attempt) throws InterruptedException {
        if (retryBackoff.isZero() || retryBackoff.isNegative())
            return;
        long millis = retryBackoff.toMillis() * (1L << attempt);
        Thread.sleep(millis);
    }

    private HttpRequest.Builder baseRequest(String path) {
        HttpRequest.Builder b = HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + path))
                .timeout(requestTimeout);
        if (apiKey != null && !apiKey.isBlank()) {
            b.header("x-heeczer-api-key", apiKey);
        }
        return b;
    }

    private static String loadSdkVersion() {
        try (InputStream in = HeeczerClient.class.getResourceAsStream("/heeczer-sdk.properties")) {
            if (in != null) {
                Properties properties = new Properties();
                properties.load(in);
                String version = properties.getProperty("sdk.version");
                if (version != null && !version.isBlank()) {
                    return version;
                }
            }
        } catch (IOException ignored) {
            // Fall through to package metadata.
        }

        Package pkg = HeeczerClient.class.getPackage();
        if (pkg != null && pkg.getImplementationVersion() != null && !pkg.getImplementationVersion().isBlank()) {
            return pkg.getImplementationVersion();
        }
        return "dev";
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
