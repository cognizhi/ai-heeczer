package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.tomakehurst.wiremock.WireMockServer;
import com.github.tomakehurst.wiremock.core.WireMockConfiguration;
import org.junit.jupiter.api.*;

import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;
import java.util.Locale;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;

import static com.github.tomakehurst.wiremock.client.WireMock.*;
import static com.github.tomakehurst.wiremock.stubbing.Scenario.STARTED;
import static org.junit.jupiter.api.Assertions.*;

/**
 * Unit tests for {@link HeeczerClient} using WireMock as an in-process stub
 * (per the TDD-with-emulation guideline — no mocking frameworks).
 */
class HeeczerClientTest {

        private static WireMockServer wm;
        private HeeczerClient client;

        private static final String VERSION_BODY = """
                        {"ok":true,"envelope_version":"1","scoring_version":"1.0.0","spec_version":"1.0","service":"0.1.0"}
                        """
                        .strip();

        private static final String INGEST_BODY = """
                        {"ok":true,"envelope_version":"1","event_id":"evt-1","score":{
                          "scoring_version":"1.0.0","spec_version":"1.0","scoring_profile":"default",
                          "category":"uncategorized","final_estimated_minutes":"1","estimated_hours":"0.02",
                          "estimated_days":"0.0025","financial_equivalent_cost":"1","confidence_score":"0.5",
                          "confidence_band":"Medium","human_summary":"ok"}}
                        """.strip();

        private static final String ERROR_BODY = """
                        {"ok":false,"envelope_version":"1","error":{"kind":"schema","message":"missing field event_id"}}
                        """.strip();

        @BeforeAll
        static void startWireMock() {
                wm = new WireMockServer(WireMockConfiguration.wireMockConfig().dynamicPort());
                wm.start();
        }

        @AfterAll
        static void stopWireMock() {
                wm.stop();
        }

        @BeforeEach
        void setUp() {
                wm.resetAll();
                client = new HeeczerClient.Builder("http://localhost:" + wm.port())
                                .apiKey("k_secret")
                                .build();
        }

        @Test
        void constructorRejectsBlankBaseUrl() {
                assertThrows(IllegalArgumentException.class,
                                () -> new HeeczerClient.Builder(""));
        }

        @Test
        void nativeModeAdaptsVersionEnvelope() throws Exception {
                NativeBridge bridge = new NativeBridge() {
                        @Override
                        public NativeVersion version() {
                                return new NativeVersion("1.0.0", "1.0");
                        }

                        @Override
                        public ScoreResult score(Object event, Object profile, Object tiers,
                                        String tierOverride) {
                                fail("score should not be called by version()");
                                return null;
                        }
                };

                var nativeClient = new HeeczerClient.Builder("http://localhost:" + wm.port())
                                .mode(HeeczerMode.native_mode)
                                .nativeBridgeForTests(bridge)
                                .build();

                VersionResponse version = nativeClient.version();
                assertTrue(version.ok);
                assertEquals("1", version.envelopeVersion);
                assertEquals("1.0.0", version.scoringVersion);
                assertEquals("1.0", version.specVersion);
                assertEquals(HeeczerClient.SDK_VERSION, version.service);
        }

        @Test
        void nativeModeInjectsWorkspaceIdForLocalScoring() throws Exception {
                AtomicReference<Object> capturedEvent = new AtomicReference<>();
                NativeBridge bridge = new NativeBridge() {
                        @Override
                        public NativeVersion version() {
                                return new NativeVersion("1.0.0", "1.0");
                        }

                        @Override
                        public ScoreResult score(Object event, Object profile, Object tiers,
                                        String tierOverride) {
                                capturedEvent.set(event);
                                ScoreResult result = new ScoreResult();
                                result.scoringVersion = "1.0.0";
                                result.specVersion = "1.0";
                                result.confidenceBand = ConfidenceBand.Medium;
                                result.humanSummary = "native ok";
                                return result;
                        }
                };

                var nativeClient = new HeeczerClient.Builder("http://localhost:" + wm.port())
                                .mode(HeeczerMode.native_mode)
                                .nativeBridgeForTests(bridge)
                                .build();

                IngestEventResponse response = nativeClient.ingestEvent("ws_native",
                                Map.of("event_id", "evt-1", "workspace_id", "stale"));

                assertTrue(response.ok);
                assertEquals("1", response.envelopeVersion);
                assertEquals("evt-1", response.eventId);
                assertEquals(ConfidenceBand.Medium, response.score.confidenceBand);

                JsonNode normalizedEvent = (JsonNode) capturedEvent.get();
                assertNotNull(normalizedEvent);
                assertEquals("ws_native", normalizedEvent.get("workspace_id").asText());
                assertEquals("evt-1", normalizedEvent.get("event_id").asText());
        }

        @Test
        void nativeModeHealthzUsesVersionProbe() throws Exception {
                AtomicInteger versionCalls = new AtomicInteger();
                NativeBridge bridge = new NativeBridge() {
                        @Override
                        public NativeVersion version() {
                                versionCalls.incrementAndGet();
                                return new NativeVersion("1.0.0", "1.0");
                        }

                        @Override
                        public ScoreResult score(Object event, Object profile, Object tiers,
                                        String tierOverride) {
                                fail("score should not be called by healthz()");
                                return null;
                        }
                };

                var nativeClient = new HeeczerClient.Builder("http://localhost:" + wm.port())
                                .mode(HeeczerMode.native_mode)
                                .nativeBridgeForTests(bridge)
                                .build();

                assertTrue(nativeClient.healthz());
                assertEquals(1, versionCalls.get());
        }

        @Test
        void nativeModeTestScorePipelinePassesOverrides() throws Exception {
                AtomicReference<Object> capturedEvent = new AtomicReference<>();
                AtomicReference<Object> capturedProfile = new AtomicReference<>();
                AtomicReference<Object> capturedTiers = new AtomicReference<>();
                AtomicReference<String> capturedTierOverride = new AtomicReference<>();
                NativeBridge bridge = new NativeBridge() {
                        @Override
                        public NativeVersion version() {
                                return new NativeVersion("1.0.0", "1.0");
                        }

                        @Override
                        public ScoreResult score(Object event, Object profile, Object tiers,
                                        String tierOverride) {
                                capturedEvent.set(event);
                                capturedProfile.set(profile);
                                capturedTiers.set(tiers);
                                capturedTierOverride.set(tierOverride);
                                ScoreResult result = new ScoreResult();
                                result.scoringVersion = "1.0.0";
                                result.specVersion = "1.0";
                                result.confidenceBand = ConfidenceBand.High;
                                result.humanSummary = "pipeline ok";
                                return result;
                        }
                };

                var nativeClient = new HeeczerClient.Builder("http://localhost:" + wm.port())
                                .mode(HeeczerMode.native_mode)
                                .nativeBridgeForTests(bridge)
                                .build();
                var request = new HeeczerClient.TestPipelineRequest(Map.of("event_id", "evt-native"));
                request.profile = Map.of("version", "1");
                request.tierSet = Map.of("tiers", java.util.List.of(Map.of("id", "t1")));
                request.tierOverride = "t1";

                TestPipelineResponse response = nativeClient.testScorePipeline(request);

                assertTrue(response.ok);
                assertEquals("1", response.envelopeVersion);
                assertEquals(ConfidenceBand.High, response.score.confidenceBand);
                assertEquals(Map.of("event_id", "evt-native"), capturedEvent.get());
                assertEquals(Map.of("version", "1"), capturedProfile.get());
                assertEquals(Map.of("tiers", java.util.List.of(Map.of("id", "t1"))), capturedTiers.get());
                assertEquals("t1", capturedTierOverride.get());
        }

        @Test
        void nativeModeFfmBridgeWorksWhenJdk22AndCoreLibraryAreAvailable() throws Exception {
                Assumptions.assumeTrue(Runtime.version().feature() >= 22,
                                "requires JDK 22+ to exercise java.lang.foreign");

                Path repoRoot = repositoryRoot();
                Path library = buildNativeLibrary(repoRoot);
                String previous = System.getProperty("heeczer.native.library");
                System.setProperty("heeczer.native.library", library.toString());
                try (var nativeClient = new HeeczerClient.Builder("http://localhost:" + wm.port())
                                .mode(HeeczerMode.native_mode)
                                .build()) {
                        JsonNode event = new ObjectMapper().readTree(
                                        Files.readString(repoRoot.resolve("examples/event.json")));

                        VersionResponse version = nativeClient.version();
                        IngestEventResponse response = nativeClient.ingestEvent("ws_default", event);

                        assertTrue(version.ok);
                        assertFalse(version.scoringVersion.isBlank());
                        assertTrue(response.ok);
                        assertNotNull(response.eventId);
                        assertNotNull(response.score);
                        assertNotNull(response.score.humanSummary);
                } finally {
                        if (previous == null) {
                                System.clearProperty("heeczer.native.library");
                        } else {
                                System.setProperty("heeczer.native.library", previous);
                        }
                }
        }

        @Test
        void nativeLegacyPanicEnvelopeMapsToUnavailable() {
                assertEquals(ApiErrorKind.unavailable,
                                NativeFfmBridge.classifyLegacyError("panic in heeczer_score_json"));
        }

        @Test
        void healthzReturnsTrueOn200() throws Exception {
                wm.stubFor(get("/healthz").willReturn(ok()));
                assertTrue(client.healthz());
        }

        @Test
        void versionReturnsEnvelope() throws Exception {
                wm.stubFor(get("/v1/version")
                                .willReturn(okJson(VERSION_BODY)));
                VersionResponse v = client.version();
                assertEquals("1.0.0", v.scoringVersion);
                assertEquals("1.0", v.specVersion);
        }

        @Test
        void ingestEventPostsCanonicalBody() throws Exception {
                wm.stubFor(post("/v1/events")
                                .withHeader("x-heeczer-api-key", equalTo("k_secret"))
                                .withRequestBody(matchingJsonPath("$.workspace_id", equalTo("ws_test")))
                                .withRequestBody(matchingJsonPath("$.event.event_id", equalTo("evt-1")))
                                .willReturn(okJson(INGEST_BODY)));
                var resp = client.ingestEvent("ws_test",
                                java.util.Map.of("event_id", "evt-1"));
                assertEquals("evt-1", resp.eventId);
                assertEquals(ConfidenceBand.Medium, resp.score.confidenceBand);
        }

        @Test
        void errorEnvelopeMapsToTypedException() {
                wm.stubFor(post("/v1/events")
                                .willReturn(badRequest().withBody(ERROR_BODY)
                                                .withHeader("content-type", "application/json")));
                var ex = assertThrows(HeeczerApiException.class,
                                () -> client.ingestEvent("ws", java.util.Map.of()));
                assertEquals(400, ex.getStatus());
                assertEquals(ApiErrorKind.schema, ex.getKind());
                assertTrue(ex.getApiMessage().contains("missing field event_id"));
        }

        @Test
        void nonJsonErrorFallsBackToUnknown() {
                wm.stubFor(get("/v1/version")
                                .willReturn(aResponse().withStatus(504).withBody("upstream timeout")));
                var ex = assertThrows(HeeczerApiException.class, () -> client.version());
                assertEquals(504, ex.getStatus());
                assertEquals(ApiErrorKind.unknown, ex.getKind());
        }

        @Test
        void testScorePipelineAlwaysSendsTesterHeader() throws Exception {
                wm.stubFor(post("/v1/test/score-pipeline")
                                .withHeader("x-heeczer-tester", equalTo("1"))
                                .willReturn(okJson(
                                                """
                                                                {"ok":true,"envelope_version":"1","score":{
                                                                  "scoring_version":"1.0.0","spec_version":"1.0","scoring_profile":"default",
                                                                  "category":"uncategorized","final_estimated_minutes":"1","estimated_hours":"0.02",
                                                                  "estimated_days":"0.0025","financial_equivalent_cost":"1","confidence_score":"0.5",
                                                                  "confidence_band":"Medium","human_summary":"ok"}}
                                                                """
                                                                .strip())));
                var req = new HeeczerClient.TestPipelineRequest(java.util.Map.of("event_id", "evt"));
                var resp = client.testScorePipeline(req);
                assertEquals(ConfidenceBand.Medium, resp.score.confidenceBand);
                // Verify the stub was actually hit (tester header required).
                wm.verify(postRequestedFor(urlEqualTo("/v1/test/score-pipeline"))
                                .withHeader("x-heeczer-tester", equalTo("1")));
        }

        @Test
        void baseUrlTrailingSlashIsNormalised() throws Exception {
                var c = new HeeczerClient.Builder("http://localhost:" + wm.port() + "/").build();
                wm.stubFor(get("/healthz").willReturn(ok()));
                assertTrue(c.healthz());
                // Verify exactly one GET /healthz (no double slash).
                wm.verify(1, getRequestedFor(urlEqualTo("/healthz")));
        }

        @Test
        void apiKeyForwardedOnEveryRequest() throws Exception {
                wm.stubFor(get("/v1/version")
                                .withHeader("x-heeczer-api-key", equalTo("k_secret"))
                                .willReturn(okJson(VERSION_BODY)));
                client.version();
                wm.verify(1, getRequestedFor(urlEqualTo("/v1/version"))
                                .withHeader("x-heeczer-api-key", equalTo("k_secret")));
        }

        @Test
        void retriesTransientStatuses() throws Exception {
                var c = new HeeczerClient.Builder("http://localhost:" + wm.port())
                                .retry(2, java.time.Duration.ZERO, 503)
                                .build();
                wm.stubFor(get("/healthz")
                                .inScenario("retry")
                                .whenScenarioStateIs(STARTED)
                                .willReturn(aResponse().withStatus(503))
                                .willSetStateTo("ready"));
                wm.stubFor(get("/healthz")
                                .inScenario("retry")
                                .whenScenarioStateIs("ready")
                                .willReturn(ok()));

                assertTrue(c.healthz());
                wm.verify(2, getRequestedFor(urlEqualTo("/healthz")));
        }

        private static Path repositoryRoot() {
                Path current = Path.of("").toAbsolutePath();
                while (current != null && !Files.exists(current.resolve("Cargo.toml"))) {
                        current = current.getParent();
                }
                assertNotNull(current, "repository root with Cargo.toml must exist");
                return current;
        }

        private static Path buildNativeLibrary(Path repoRoot) throws Exception {
                Process process = new ProcessBuilder("cargo", "build", "-p", "heeczer-core-c")
                                .directory(repoRoot.toFile())
                                .redirectErrorStream(true)
                                .start();
                ByteArrayOutputStream output = new ByteArrayOutputStream();
                process.getInputStream().transferTo(output);
                int exitCode = process.waitFor();
                String text = output.toString(StandardCharsets.UTF_8);
                assertEquals(0, exitCode, () -> "cargo build -p heeczer-core-c failed:\n" + text);

                Path library = repoRoot.resolve("target/debug").resolve(nativeLibraryFileName());
                assertTrue(Files.exists(library), () -> "expected native library at " + library);
                return library;
        }

        private static String nativeLibraryFileName() {
                String os = System.getProperty("os.name", "").toLowerCase(Locale.ROOT);
                if (os.contains("win")) {
                        return "heeczer_core_c.dll";
                }
                if (os.contains("mac")) {
                        return "libheeczer_core_c.dylib";
                }
                return "libheeczer_core_c.so";
        }
}
