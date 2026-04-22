# heeczer-sdk (Java)

Java HTTP client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

Requires Java 17+. Uses `java.net.http.HttpClient` (no third-party HTTP dep).

## Install

```xml
<dependency>
  <groupId>io.github.cognizhi</groupId>
  <artifactId>heeczer-sdk</artifactId>
  <version>0.1.0</version>
</dependency>
```

## Usage

```java
import io.github.cognizhi.heeczer.*;

try (var client = new HeeczerClient.Builder("https://ingest.example.com")
        .apiKey(System.getenv("HEECZER_API_KEY"))
        .build()) {

    IngestEventResponse resp = client.ingestEvent("ws_default", canonicalEvent);
    System.out.println(resp.score.finalEstimatedMinutes + " " + resp.score.confidenceBand);
}
```

## Error handling

All methods throw `HeeczerApiException` on non-2xx responses:

```java
try {
    client.ingestEvent("ws", badEvent);
} catch (HeeczerApiException e) {
    if (e.getKind() == ApiErrorKind.schema) { /* … */ }
}
```

## License

MIT.
