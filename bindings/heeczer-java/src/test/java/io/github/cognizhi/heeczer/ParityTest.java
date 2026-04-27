package io.github.cognizhi.heeczer;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Stream;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/** Byte-equal SDK parity against Rust reference ScoreResult JSON. */
class ParityTest {
    private static final ObjectMapper MAPPER = new ObjectMapper();

    @Test
    void testScorePipelineMatchesRustReferenceJson() throws Exception {
        String baseUrl = requiredEnvOrSkip("HEECZER_PARITY_BASE_URL");
        Path referenceDir = Path.of(requiredEnvOrSkip("HEECZER_PARITY_REFERENCE_DIR"));
        Path fixtureDir = Path.of(System.getenv().getOrDefault(
                "HEECZER_PARITY_FIXTURE_DIR",
                repositoryRoot().resolve("core/schema/fixtures/events/valid").toString()));

        List<Path> fixtures;
        try (Stream<Path> stream = Files.list(fixtureDir)) {
            fixtures = stream
                    .filter(path -> path.toString().endsWith(".json"))
                    .sorted()
                    .toList();
        }
        assertFalse(fixtures.isEmpty(), "expected at least one valid fixture");

        HeeczerClient client = new HeeczerClient.Builder(baseUrl)
                .retry(3, java.time.Duration.ofMillis(50))
                .build();
        List<String> failures = new ArrayList<>();
        for (Path fixture : fixtures) {
            JsonNode event = MAPPER.readTree(Files.readString(fixture));
            TestPipelineResponse response = client.testScorePipeline(
                    new HeeczerClient.TestPipelineRequest(event));
            String expected = Files.readString(referenceDir.resolve(stem(fixture) + ".json")).stripTrailing();
            String actual = response.score.toJson();
            if (!actual.equals(expected)) {
                failures.add(fixture.getFileName() + ": score JSON differed from Rust reference");
            }
        }

        assertEquals(List.of(), failures);
    }

    private static String requiredEnvOrSkip(String name) {
        String value = System.getenv(name);
        Assumptions.assumeTrue(value != null && !value.isBlank(), name + " is unset");
        return value;
    }

    private static Path repositoryRoot() {
        Path current = Path.of("").toAbsolutePath();
        while (current != null && !Files.exists(current.resolve("Cargo.toml"))) {
            current = current.getParent();
        }
        if (current == null) {
            throw new IllegalStateException("repository root with Cargo.toml not found");
        }
        return current;
    }

    private static String stem(Path path) {
        String name = path.getFileName().toString();
        return name.substring(0, name.length() - ".json".length());
    }
}
