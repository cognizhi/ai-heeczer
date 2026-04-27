package com.cognizhi.heeczer.teststack;

import com.cognizhi.heeczer.teststack.skills.SkillCatalogue;
import com.cognizhi.heeczer.teststack.tools.Catalogue;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;
import io.github.cognizhi.heeczer.HeeczerClient;
import io.github.cognizhi.heeczer.IngestEventResponse;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.InetSocketAddress;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Instant;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

public final class App {
    private static final ObjectMapper MAPPER = new ObjectMapper();
    private static final String WORKSPACE_ID = env("CHATBOT_WORKSPACE_ID", "local-test-java");
    private static final String SCORING_PROFILE = env("CHATBOT_SCORING_PROFILE", "default");
    private static final HeeczerClient HEECZER = new HeeczerClient.Builder(
            env("HEECZER_BASE_URL", "http://heeczer-ingest:8080")).build();

    private App() {
    }

    public static void main(String[] args) throws IOException {
        int port = Integer.parseInt(env("CHATBOT_PORT", "8000"));
        HttpServer server = HttpServer.create(new InetSocketAddress("0.0.0.0", port), 0);
        server.createContext("/healthz", App::healthz);
        server.createContext("/chat", App::chat);
        server.createContext("/", App::root);
        server.start();
        System.out.println("heeczer Java chatbot listening on " + port);
    }

    private static void healthz(HttpExchange exchange) throws IOException {
        writeJson(exchange, 200, Map.of("ok", true));
    }

    private static void root(HttpExchange exchange) throws IOException {
        String html = "<!doctype html><html lang='en'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>ai-heeczer Java stack</title></head><body><main><h1>ai-heeczer Java stack</h1><form id='chat'><select name='skill'><option value='code_gen'>code_gen</option><option value='rca'>rca</option><option value='doc_summary'>doc_summary</option><option value='compliance'>compliance</option><option value='ci_triage'>ci_triage</option><option value='architecture'>architecture</option></select><input name='prompt' value='Summarize this local SDK stack'><button>Send</button></form><pre id='out'></pre></main><script>document.querySelector('#chat').addEventListener('submit',async(event)=>{event.preventDefault();const form=new FormData(event.target);const response=await fetch('/chat',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(Object.fromEntries(form))});document.querySelector('#out').textContent=JSON.stringify(await response.json(),null,2);});</script></body></html>";
        byte[] payload = html.getBytes(StandardCharsets.UTF_8);
        exchange.getResponseHeaders().set("content-type", "text/html; charset=utf-8");
        exchange.sendResponseHeaders(200, payload.length);
        try (OutputStream body = exchange.getResponseBody()) {
            body.write(payload);
        }
    }

    private static void chat(HttpExchange exchange) throws IOException {
        if (!"POST".equals(exchange.getRequestMethod())) {
            writeJson(exchange, 405, Map.of("ok", false));
            return;
        }
        try {
            JsonNode request = readRequest(exchange);
            JsonNode fixture = SkillCatalogue.load(text(request, "skill", null));
            String provider = text(request, "provider", env("LLM_PROVIDER", "mock"));
            String prompt = text(request, "prompt", "Summarize this local SDK stack.");
            long startedAt = System.currentTimeMillis();
            ProviderUsage usage = callProvider(fixture, prompt, provider);
            List<String> activeTools = SkillCatalogue.activeTools(fixture);
            List<Catalogue.ToolTraceEntry> toolTrace = Catalogue.traceForTools(activeTools);
            ObjectNode event = buildEvent(fixture, usage, toolTrace, startedAt);
            IngestEventResponse submission = HEECZER.ingestEvent(WORKSPACE_ID, event);
            JsonNode scoreResult = MAPPER.readTree(submission.score.toJson());
            Map<String, Object> response = new LinkedHashMap<>();
            response.put("ok", true);
            response.put("skill", fixture.path("skill").asText());
            response.put("event_id", submission.eventId);
            response.put("reply", usage.text);
            response.put("tool_trace", toolTrace);
            response.put("event", event);
            response.put("score_result", scoreResult);
            writeJson(exchange, 200, response);
        } catch (Exception error) {
            writeJson(exchange, 500, Map.of("ok", false, "error", error.getMessage()));
        }
    }

    private static ProviderUsage callProvider(JsonNode fixture, String prompt, String provider) throws Exception {
        if ("mock".equals(provider)) {
            JsonNode metrics = fixture.path("expected_event").path("metrics");
            return new ProviderUsage(metrics.path("tokens_prompt_min").asInt(),
                    metrics.path("tokens_completion_min").asInt(),
                    "Mock " + fixture.path("skill").asText() + " turn completed.");
        }
        if ("openrouter".equals(provider) || "gemini".equals(provider)) {
            boolean gemini = "gemini".equals(provider);
            String apiKey = env(gemini ? "GEMINI_API_KEY" : "OPENROUTER_API_KEY", "");
            String model = env(gemini ? "GEMINI_MODEL" : "OPENROUTER_MODEL", "");
            if (apiKey.isBlank() || apiKey.contains("changeme") || model.isBlank()) {
                throw new IllegalArgumentException(provider + " requires an API key and model");
            }
            String endpoint = gemini
                    ? "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
                    : "https://openrouter.ai/api/v1/chat/completions";
            ObjectNode body = MAPPER.createObjectNode();
            body.put("model", model);
            ArrayNode messages = body.putArray("messages");
            messages.addObject().put("role", "system").put("content", "Run the " + fixture.path("skill").asText()
                    + " local-stack scenario without revealing raw prompts.");
            messages.addObject().put("role", "user").put("content", prompt);
            body.set("tools", MAPPER.valueToTree(Catalogue.functionSchemas(SkillCatalogue.activeTools(fixture))));
            body.put("tool_choice", "auto");
            HttpRequest request = HttpRequest.newBuilder(URI.create(endpoint))
                    .header("authorization", "Bearer " + apiKey)
                    .header("content-type", "application/json")
                    .POST(HttpRequest.BodyPublishers.ofString(MAPPER.writeValueAsString(body)))
                    .build();
            HttpResponse<String> response = HttpClient.newHttpClient().send(request,
                    HttpResponse.BodyHandlers.ofString());
            if (response.statusCode() >= 400) {
                throw new IOException(provider + " returned HTTP " + response.statusCode());
            }
            JsonNode payload = MAPPER.readTree(response.body());
            JsonNode usage = payload.path("usage");
            JsonNode message = payload.path("choices").path(0).path("message");
            return new ProviderUsage(nullableNumber(usage, "prompt_tokens"), nullableNumber(usage, "completion_tokens"),
                    message.path("content").asText(provider + " completed " + fixture.path("skill").asText() + "."));
        }
        if ("local".equals(provider)) {
            String baseUrl = env("LOCAL_MODEL_BASE_URL", "http://ollama:11434").replaceFirst("/+$", "");
            String model = env("LOCAL_MODEL", "llama3.2:1b");
            ObjectNode body = MAPPER.createObjectNode();
            body.put("model", model);
            body.put("stream", false);
            ArrayNode messages = body.putArray("messages");
            messages.addObject().put("role", "user").put("content", prompt);
            HttpRequest request = HttpRequest.newBuilder(URI.create(baseUrl + "/api/chat"))
                    .header("content-type", "application/json")
                    .POST(HttpRequest.BodyPublishers.ofString(MAPPER.writeValueAsString(body)))
                    .build();
            HttpResponse<String> response = HttpClient.newHttpClient().send(request,
                    HttpResponse.BodyHandlers.ofString());
            if (response.statusCode() >= 400) {
                throw new IOException("local model returned HTTP " + response.statusCode());
            }
            JsonNode payload = MAPPER.readTree(response.body());
            String text = payload.path("message").path("content").asText("Local model completed.");
            return new ProviderUsage(null, null, text);
        }
        throw new IllegalArgumentException("unsupported provider " + provider);
    }

    private static ObjectNode buildEvent(JsonNode fixture, ProviderUsage usage,
            List<Catalogue.ToolTraceEntry> toolTrace, long startedAt) {
        JsonNode expected = fixture.path("expected_event");
        JsonNode metrics = expected.path("metrics");
        ObjectNode context = expected.path("context").deepCopy();
        ArrayNode tags = context.putArray("tags");
        tags.add("local-stack");
        tags.add("java");
        tags.add(fixture.path("skill").asText());
        ObjectNode task = MAPPER.createObjectNode();
        task.put("name", fixture.path("skill").asText() + ": local stack turn");
        task.set("category", expected.path("task").path("category"));
        task.set("sub_category", expected.path("task").path("sub_category"));
        task.set("outcome", expected.path("task").path("outcome"));
        ObjectNode metricNode = MAPPER.createObjectNode();
        metricNode.put("duration_ms", Math.max(1, (int) (System.currentTimeMillis() - startedAt)));
        metricNode.putPOJO("tokens_prompt", usage.promptTokens);
        metricNode.putPOJO("tokens_completion", usage.completionTokens);
        metricNode.put("tool_call_count", metrics.path("tool_call_count").asInt());
        metricNode.put("workflow_steps", metrics.path("workflow_steps").asInt());
        metricNode.put("retries", metrics.path("retries").asInt());
        metricNode.put("artifact_count", metrics.path("artifact_count").asInt());
        metricNode.put("output_size_proxy", metrics.path("output_size_proxy").asDouble());
        ArrayNode traceNames = MAPPER.createArrayNode();
        for (Catalogue.ToolTraceEntry traceEntry : toolTrace) {
            traceNames.add(traceEntry.tool_name());
        }
        ObjectNode extensions = MAPPER.createObjectNode();
        extensions.put("chatbot.skill", fixture.path("skill").asText());
        extensions.put("chatbot.turn", 1);
        extensions.set("chatbot.tool_trace", traceNames);
        ObjectNode meta = MAPPER.createObjectNode();
        meta.put("sdk_language", "java");
        meta.put("sdk_version", HeeczerClient.SDK_VERSION);
        meta.put("scoring_profile", SCORING_PROFILE);
        meta.set("extensions", extensions);
        ObjectNode event = MAPPER.createObjectNode();
        event.put("spec_version", "1.0");
        event.put("event_id", UUID.randomUUID().toString());
        event.put("correlation_id", "java-session:" + System.currentTimeMillis());
        event.put("timestamp", Instant.now().toString());
        event.put("framework_source", "chatbot-java");
        event.put("workspace_id", WORKSPACE_ID);
        if (!env("CHATBOT_PROJECT_ID", "").isBlank()) {
            event.put("project_id", env("CHATBOT_PROJECT_ID", ""));
        }
        event.set("task", task);
        event.set("metrics", metricNode);
        event.set("context", context);
        event.set("meta", meta);
        return event;
    }

    private static JsonNode readRequest(HttpExchange exchange) throws IOException {
        try (InputStream body = exchange.getRequestBody()) {
            byte[] payload = body.readAllBytes();
            if (payload.length == 0) {
                return MAPPER.createObjectNode();
            }
            return MAPPER.readTree(payload);
        }
    }

    private static String text(JsonNode node, String field, String fallback) {
        JsonNode value = node.path(field);
        return value.isTextual() ? value.asText() : fallback;
    }

    private static Object nullableNumber(JsonNode node, String field) {
        JsonNode value = node.path(field);
        return value.isNumber() ? value.numberValue() : null;
    }

    private static void writeJson(HttpExchange exchange, int status, Object body) throws IOException {
        byte[] payload = MAPPER.writeValueAsBytes(body);
        exchange.getResponseHeaders().set("content-type", "application/json; charset=utf-8");
        exchange.sendResponseHeaders(status, payload.length);
        try (OutputStream responseBody = exchange.getResponseBody()) {
            responseBody.write(payload);
        }
    }

    private static String env(String name, String fallback) {
        String value = System.getenv(name);
        return value == null || value.isBlank() ? fallback : value;
    }

    private record ProviderUsage(Object promptTokens, Object completionTokens, String text) {
    }
}
