package io.github.cognizhi.heeczer;

import com.github.tomakehurst.wiremock.WireMockServer;
import com.github.tomakehurst.wiremock.core.WireMockConfiguration;
import org.junit.jupiter.api.*;

import static com.github.tomakehurst.wiremock.client.WireMock.*;
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
            """.strip();

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
        assertEquals("1.0",   v.specVersion);
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
        assertEquals("evt-1",  resp.eventId);
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
        assertEquals(504,               ex.getStatus());
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
                        """.strip())));
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
}
