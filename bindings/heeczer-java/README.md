# heeczer-sdk (Java)

Java client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service and in-process core bridge.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

Requires Java 17+ for HTTP mode. Native mode uses the JDK Foreign Function & Memory API at runtime and requires JDK 22+ plus a resolvable `heeczer_core_c` native library.

## Install

> **Pre-release.** `com.cognizhi:heeczer-sdk` is not on Maven
> Central yet (see plan 0012). For local development, build and install
> with `mvn -f bindings/heeczer-java install -DskipTests -Dgpg.skip=true`.

```xml
<dependency>
    <groupId>com.cognizhi</groupId>
  <artifactId>heeczer-sdk</artifactId>
    <version>0.5.1-SNAPSHOT</version>
</dependency>
```

## Usage

```java
import io.github.cognizhi.heeczer.*;

try (var client = new HeeczerClient.Builder("https://ingest.example.com")
        .apiKey(System.getenv("HEECZER_API_KEY"))
    .mode(HeeczerMode.image)
    .retry(2, java.time.Duration.ofMillis(100))
        .build()) {

    IngestEventResponse resp = client.ingestEvent("ws_default", canonicalEvent);
    System.out.println(resp.score.finalEstimatedMinutes + " " + resp.score.confidenceBand);
}
```

Native mode stays on the same public surface, but scores locally with no network hop:

```bash
export HEECZER_NATIVE_LIBRARY=/absolute/path/to/libheeczer_core_c.so
```

```java
try (var client = new HeeczerClient.Builder("http://unused-in-native-mode")
        .mode(HeeczerMode.native_mode)
        .build()) {

    IngestEventResponse resp = client.ingestEvent("ws_default", canonicalEvent);
    System.out.println(resp.score.humanSummary);
}
```

When `mode(HeeczerMode.native_mode)` is selected:

- `healthz()` verifies the local bridge by calling `heeczer_versions_json()`.
- `version()` reports the embedded scoring/spec versions from `heeczer-core-c`; the `service` field stays version-shaped and carries the Java SDK version because there is no remote service in native mode.
- `ingestEvent()` injects `workspace_id` into the canonical event and returns a synthetic envelope after local scoring. No persistence or network I/O occurs.
- `testScorePipeline()` runs the same local scoring path with optional `profile`, `tier_set`, and `tier_override` overrides.

## Error handling

All methods throw `HeeczerApiException` on non-2xx responses or native-envelope failures. `getKind()` returns a closed `ApiErrorKind` enum mirroring the ingestion service; native mode maps the C ABI's error kinds into the closest service-facing kind:

| Kind                       | When                                                       |
| -------------------------- | ---------------------------------------------------------- |
| `schema`                   | Event failed canonical schema validation.                  |
| `bad_request`              | Malformed JSON or missing top-level fields.                |
| `scoring`                  | Engine rejected a normalized event (e.g. unknown tier id). |
| `storage`                  | Persistence layer error.                                   |
| `not_found`                | Read endpoint did not find the resource.                   |
| `unauthorized`             | Missing, invalid, or revoked API key.                      |
| `forbidden`                | Auth or RBAC denied the request.                           |
| `conflict`                 | Duplicate idempotency key or conflicting event payload.    |
| `payload_too_large`        | Payload exceeded service limits.                           |
| `rate_limit_exceeded`      | Per-key or workspace quota was exceeded.                   |
| `feature_disabled`         | Endpoint exists but the feature flag is off.               |
| `unsupported_spec_version` | Event `spec_version` is not accepted.                      |
| `unavailable`              | Readiness or dependency check failed.                      |
| `unknown`                  | Non-JSON 5xx body; the raw text is in `getMessage()`.      |

```java
try {
    client.ingestEvent("ws", badEvent);
} catch (HeeczerApiException e) {
    if (e.getKind() == ApiErrorKind.schema) { /* … */ }
}
```

## Builder options

| Method                                               | Description                                                               |
| ---------------------------------------------------- | ------------------------------------------------------------------------- |
| `apiKey(String)`                                     | Sets the `x-heeczer-api-key` header.                                      |
| `httpClient(HttpClient)`                             | Inject a custom `HttpClient` (proxy, mTLS, fake in tests).                |
| `mode(HeeczerMode.image \| HeeczerMode.native_mode)` | Selects HTTP or in-process scoring mode. Native mode requires JDK 22+ and `heeczer_core_c`. |
| `requestTimeout(Duration)`                           | Per-request timeout.                                                      |
| `retry(int, Duration, Integer...)`                   | Retries transient transport/status failures.                              |

## Methods

| Method                            | Image mode                     | Native mode                                                                 |
| --------------------------------- | ------------------------------ | --------------------------------------------------------------------------- |
| `healthz()`                       | `GET /healthz`                 | local bridge/version check                                                   |
| `version()`                       | `GET /v1/version`              | embedded versions from `heeczer-core-c`                                      |
| `ingestEvent(workspaceId, event)` | `POST /v1/events`              | local scoring with injected `workspace_id`; returns a synthetic ingest envelope |
| `testScorePipeline(request)`      | `POST /v1/test/score-pipeline` | local scoring with optional overrides; no feature-flag dependency            |
| `close()`                         | —                              | implements `AutoCloseable` for try-with-resources.                           |

## Contract

The SDK speaks `envelope_version: "1"` to the ingestion service per
[ADR-0011](../../docs/adr/0011-c-abi-envelope.md). Additive fields land
without breaking the typed surface. `ScoreResult.toJson()` returns compact JSON
for the full score object as received from the API, including additive engine
fields such as the explainability trace.

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

**Batching note.** The ingestion service exposes `POST /v1/events:batch`;
the SDK batch helper follows the public method expansion tracked in
[plan 0009](../../docs/plan/0009-sdk-java.md). Until then, send events
concurrently via `CompletableFuture` or a virtual-thread executor.

## License

MIT.
