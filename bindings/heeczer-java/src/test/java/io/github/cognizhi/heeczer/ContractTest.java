package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.databind.exc.UnrecognizedPropertyException;
import com.fasterxml.jackson.annotation.JsonInclude;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.MethodSource;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.stream.Stream;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Contract tests for plan 0001 / ADR-0002 — Java binding.
 *
 * <p>
 * Verifies:
 * <ol>
 * <li>Every valid fixture round-trips through the typed {@link Event} POJO
 * without data loss (parse → serialize → re-parse == original).</li>
 * <li>Extension fields under {@code meta.extensions} survive a round-trip.</li>
 * <li>{@code FAIL_ON_UNKNOWN_PROPERTIES = true} rejects unknown top-level
 * fields.</li>
 * </ol>
 */
class ContractTest {

    /**
     * Lenient mapper matching HeeczerClient: NON_NULL, accept unknown (for open
     * parsing).
     */
    private static final ObjectMapper LENIENT = new ObjectMapper()
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
            .setSerializationInclusion(JsonInclude.Include.NON_NULL);

    /**
     * Strict mapper: rejects unknown properties; used for structural enforcement
     * tests.
     */
    private static final ObjectMapper STRICT = new ObjectMapper()
            .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, true)
            .setSerializationInclusion(JsonInclude.Include.NON_NULL);

    private static Path fixtureDir() {
        // Maven runs tests from the module root (bindings/heeczer-java).
        // Walk up two directories to reach the repo root, then down to core/schema.
        return Paths.get("../../core/schema/fixtures/events/valid").toAbsolutePath();
    }

    static Stream<Path> validFixtures() throws IOException {
        Path dir = fixtureDir();
        return Files.list(dir)
                .filter(p -> p.toString().endsWith(".json"))
                .sorted();
    }

    // ── Round-trip tests ──────────────────────────────────────────────────────

    @Test
    void atLeastOneValidFixtureExists() throws IOException {
        assertTrue(validFixtures().findAny().isPresent(),
                "No valid fixtures found in " + fixtureDir());
    }

    @ParameterizedTest(name = "{0}")
    @MethodSource("validFixtures")
    void validFixtureRoundTripsLosslessly(Path fixture) throws IOException {
        String body = Files.readString(fixture);

        // Parse with lenient mapper so optional fields are tolerated.
        Event event = LENIENT.readValue(body, Event.class);

        // Re-serialize with NON_NULL (matches Rust skip_serializing_if behavior).
        String reserialised = LENIENT.writeValueAsString(event);

        // Semantic comparison as JsonNode tree.
        JsonNode original = LENIENT.readTree(body);
        JsonNode roundtripped = LENIENT.readTree(reserialised);

        assertEquals(original, roundtripped,
                "Round-trip mismatch for " + fixture.getFileName());
    }

    // ── Extensions round-trip ─────────────────────────────────────────────────

    @Test
    void metaExtensionsSurviveRoundTrip() throws JsonProcessingException {
        Event event = new Event();
        event.specVersion = "1.0";
        event.eventId = "00000000-0000-4000-8000-aabbccddeeff";
        event.timestamp = "2026-04-22T10:00:00Z";
        event.frameworkSource = "test";
        event.workspaceId = "ws_ext";
        event.task = new EventTask("ext_test", Outcome.success);
        event.metrics = new EventMetrics(100);

        EventMeta meta = new EventMeta("java", "0.1.0");
        meta.extensions = LENIENT.readTree("{\"custom_key\":42,\"nested\":{\"x\":true}}");
        event.meta = meta;

        String serialised = LENIENT.writeValueAsString(event);
        Event back = LENIENT.readValue(serialised, Event.class);

        assertNotNull(back.meta.extensions, "meta.extensions must survive round-trip");
        assertEquals(42, back.meta.extensions.get("custom_key").asInt());
        assertTrue(back.meta.extensions.get("nested").get("x").asBoolean());
    }

    @Test
    void absentOptionalFieldsRemainAbsentAfterRoundTrip() throws JsonProcessingException {
        Event event = new Event();
        event.specVersion = "1.0";
        event.eventId = "00000000-0000-4000-8000-000000000001";
        event.timestamp = "2026-04-22T10:00:00Z";
        event.frameworkSource = "test";
        event.workspaceId = "ws_min";
        event.task = new EventTask("min_task", Outcome.success);
        event.metrics = new EventMetrics(50);
        event.meta = new EventMeta("java", "0.1.0");

        String serialised = LENIENT.writeValueAsString(event);
        JsonNode node = LENIENT.readTree(serialised);

        assertFalse(node.has("correlation_id"), "correlation_id must be absent");
        assertFalse(node.has("identity"), "identity must be absent");
        assertFalse(node.has("context"), "context must be absent");
        assertFalse(node.get("meta").has("extensions"), "meta.extensions must be absent");
    }

    // ── Strict unknown-field rejection ────────────────────────────────────────

    @Test
    void unknownTopLevelFieldRejectedByStrictMapper() {
        String bad = """
                {
                  "spec_version": "1.0",
                  "event_id": "00000000-0000-4000-8000-aabbccddeeff",
                  "timestamp": "2026-04-22T10:00:00Z",
                  "framework_source": "test",
                  "workspace_id": "ws_strict",
                  "task": {"name": "t", "outcome": "success"},
                  "metrics": {"duration_ms": 100},
                  "meta": {"sdk_language": "java", "sdk_version": "0.1.0"},
                  "forbidden_extra_field": "value"
                }
                """;

        assertThrows(UnrecognizedPropertyException.class,
                () -> STRICT.readValue(bad, Event.class),
                "Strict mapper must reject unknown top-level field");
    }

    @Test
    void unknownMetaFieldRejectedByStrictMapper() {
        String bad = """
                {
                  "spec_version": "1.0",
                  "event_id": "00000000-0000-4000-8000-aabbccddeeff",
                  "timestamp": "2026-04-22T10:00:00Z",
                  "framework_source": "test",
                  "workspace_id": "ws_strict",
                  "task": {"name": "t", "outcome": "success"},
                  "metrics": {"duration_ms": 100},
                  "meta": {
                    "sdk_language": "java",
                    "sdk_version": "0.1.0",
                    "unknown_meta_key": "oops"
                  }
                }
                """;

        assertThrows(UnrecognizedPropertyException.class,
                () -> STRICT.readValue(bad, Event.class),
                "Strict mapper must reject unknown field inside meta (use meta.extensions instead)");
    }
}
