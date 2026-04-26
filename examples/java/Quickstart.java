// Quickstart: submit an event to the ingestion service via the Java SDK.
//
// Prereq: ingestion service running locally in unauthenticated smoke-test mode
// (HEECZER_AUTH__ENABLED=false cargo run -p heeczer-ingest).
//
// Easiest path (JDK 17+ single-file source mode, JEP 458):
//   1. mvn -f bindings/heeczer-java install -DskipTests -Dgpg.skip=true
//   2. JAR=~/.m2/repository/com/cognizhi/heeczer-sdk/0.5.1-SNAPSHOT/heeczer-sdk-0.5.1-SNAPSHOT.jar
//      JACKSON=$(mvn -f bindings/heeczer-java -q exec:exec \
//        -Dexec.executable=echo -Dexec.args='%classpath')
//      java --class-path "$JAR:$JACKSON" examples/java/Quickstart.java
//
// Once the SDK is on Maven Central (plan 0012), Maven/Gradle coordinates will
// resolve Jackson transitively. The raw java --class-path invocation shown here
// still needs Jackson on the class-path explicitly.
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import io.github.cognizhi.heeczer.HeeczerApiException;
import io.github.cognizhi.heeczer.HeeczerClient;
import io.github.cognizhi.heeczer.IngestEventResponse;
import io.github.cognizhi.heeczer.VersionResponse;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;

public final class Quickstart {

    public static void main(String[] args) throws IOException, InterruptedException {
        Path eventPath = Paths.get(System.getProperty("user.dir"), "examples", "event.json");
        ObjectMapper mapper = new ObjectMapper();
        JsonNode event = mapper.readTree(Files.readAllBytes(eventPath));

        String baseUrl = System.getenv().getOrDefault("HEECZER_BASE_URL", "http://127.0.0.1:8080");
        try (HeeczerClient client = new HeeczerClient.Builder(baseUrl)
                .apiKey(System.getenv("HEECZER_API_KEY"))
                .build()) {

            VersionResponse v = client.version();
            System.out.println("» service version: " + v.scoringVersion + "/" + v.specVersion);

            try {
                IngestEventResponse resp = client.ingestEvent("ws_default", event);
                System.out.println("» event " + resp.eventId + " ingested");
                System.out.println("» summary: " + resp.score.humanSummary);
                System.out.println("» minutes=" + resp.score.finalEstimatedMinutes
                        + " band=" + resp.score.confidenceBand);
            } catch (HeeczerApiException e) {
                System.err.println("SDK error: kind=" + e.getKind()
                        + " status=" + e.getStatus()
                        + " message=" + e.getMessage());
                System.exit(1);
            }
        }
    }
}
