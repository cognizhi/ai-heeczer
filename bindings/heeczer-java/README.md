# heeczer-sdk (Java)

Java HTTP client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

Requires Java 17+. Uses `java.net.http.HttpClient` (no third-party HTTP dep).

## Install

> **Pre-release.** `io.github.cognizhi:heeczer-sdk` is not on Maven
> Central yet (see plan 0012). For local development, build and install
> with `mvn -f bindings/heeczer-java install -DskipTests`.

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

All methods throw `HeeczerApiException` on non-2xx responses. `getKind()`
returns a closed `ApiErrorKind` enum mirroring the ingestion service:

| Kind               | When                                                       |
| ------------------ | ---------------------------------------------------------- |
| `schema`           | Event failed canonical schema validation.                  |
| `bad_request`      | Malformed JSON or missing top-level fields.                |
| `scoring`          | Engine rejected a normalized event (e.g. unknown tier id). |
| `storage`          | Persistence layer error.                                   |
| `not_found`        | Read endpoint did not find the resource.                   |
| `forbidden`        | Auth or RBAC denied the request.                           |
| `feature_disabled` | Endpoint exists but the feature flag is off.               |
| `unknown`          | Non-JSON 5xx body; the raw text is in `getMessage()`.      |

```java
try {
    client.ingestEvent("ws", badEvent);
} catch (HeeczerApiException e) {
    if (e.getKind() == ApiErrorKind.schema) { /* … */ }
}
```

## Builder options

| Method                   | Description                                                |
| ------------------------ | ---------------------------------------------------------- |
| `apiKey(String)`         | Sets the `x-heeczer-api-key` header.                       |
| `httpClient(HttpClient)` | Inject a custom `HttpClient` (proxy, mTLS, fake in tests). |

## Methods

| Method                            | HTTP                           | Returns                                                               |
| --------------------------------- | ------------------------------ | --------------------------------------------------------------------- |
| `healthz()`                       | `GET /healthz`                 | `boolean`                                                             |
| `version()`                       | `GET /v1/version`              | `VersionResponse`                                                     |
| `ingestEvent(workspaceId, event)` | `POST /v1/events`              | `IngestEventResponse`                                                 |
| `testScorePipeline(request)`      | `POST /v1/test/score-pipeline` | `TestPipelineResponse` (gated by the test-orchestration feature flag) |
| `close()`                         | —                              | implements `AutoCloseable` for try-with-resources.                    |

## Contract

The SDK speaks `envelope_version: "1"` to the ingestion service per
[ADR-0011](../../docs/adr/0011-c-abi-envelope.md). Additive fields land
without breaking the typed surface.

## Runnable example

See [`examples/java/Quickstart.java`](../../examples/java/Quickstart.java)
and the cross-language index in [`examples/README.md`](../../examples/README.md).

## Common patterns

**Validate locally before sending** (avoids a network round-trip on bad
events). The schema is JSON Schema Draft 2020-12;
[Networknt json-schema-validator](https://github.com/networknt/json-schema-validator)
works well:

```java
import com.networknt.schema.*;
import com.fasterxml.jackson.databind.ObjectMapper;

var mapper = new ObjectMapper();
var factory = JsonSchemaFactory.getInstance(SpecVersion.VersionFlag.V202012);
try (var in = new FileInputStream("core/schema/event.v1.json")) {
    var schema = factory.getSchema(in);
    var errors = schema.validate(mapper.valueToTree(event));
    if (!errors.isEmpty()) throw new IllegalArgumentException(errors.toString());
}
```

**Surface schema field errors from the service:**

```java
try {
    client.ingestEvent("ws", badEvent);
} catch (HeeczerApiException e) {
    if (e.getKind() == ApiErrorKind.schema) {
        // e.getMessage() contains the field-level detail from the server envelope.
        System.err.println("schema rejection: " + e.getMessage());
    }
}
```

**Batching note.** `POST /v1/events:batch` (single-transaction,
partial-success semantics) is planned but not yet shipped — see
[plan 0004](../../docs/plan/0004-ingestion-service.md). Until then,
send events concurrently via `CompletableFuture` or a virtual-thread executor.

## License

MIT.
