import {
  createServer,
  type IncomingMessage,
  type ServerResponse,
} from "node:http";
import { randomUUID } from "node:crypto";

import { HeeczerClient } from "@cognizhi/heeczer-sdk";

import {
  activeTools,
  loadSkill,
  type SkillFixture,
} from "./skills/catalogue.js";
import {
  functionSchemas,
  traceForTools,
  type ToolTraceEntry,
} from "./tools/catalogue.js";

const port = Number(process.env["CHATBOT_PORT"] ?? "8000");
const sdkVersion = process.env["CHATBOT_SDK_VERSION"] ?? "0.5.1";
const workspaceId = process.env["CHATBOT_WORKSPACE_ID"] ?? "local-test-js";
const scoringProfile = process.env["CHATBOT_SCORING_PROFILE"] ?? "default";
const client = new HeeczerClient({
  baseUrl: process.env["HEECZER_BASE_URL"] ?? "http://heeczer-ingest:8080",
  validateEvents: true,
});

interface ChatRequest {
  skill?: string;
  prompt?: string;
  provider?: string;
}

interface ProviderUsage {
  prompt_tokens: number | null;
  completion_tokens: number | null;
  text: string;
}

function json(res: ServerResponse, status: number, body: unknown): void {
  const payload = JSON.stringify(body);
  res.writeHead(status, {
    "content-type": "application/json; charset=utf-8",
    "content-length": Buffer.byteLength(payload),
  });
  res.end(payload);
}

async function readBody(req: IncomingMessage): Promise<ChatRequest> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  if (chunks.length === 0) return {};
  return JSON.parse(Buffer.concat(chunks).toString("utf8")) as ChatRequest;
}

function html(res: ServerResponse): void {
  const body = `<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>ai-heeczer JS stack</title></head>
<body><main><h1>ai-heeczer JS stack</h1><form id="chat"><select name="skill"><option value="code_gen">code_gen</option><option value="rca">rca</option><option value="doc_summary">doc_summary</option><option value="compliance">compliance</option><option value="ci_triage">ci_triage</option><option value="architecture">architecture</option></select><input name="prompt" value="Summarize this local SDK stack"><button>Send</button></form><pre id="out"></pre></main><script>document.querySelector('#chat').addEventListener('submit',async(e)=>{e.preventDefault();const f=new FormData(e.target);const r=await fetch('/chat',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(Object.fromEntries(f))});document.querySelector('#out').textContent=JSON.stringify(await r.json(),null,2);});</script></body></html>`;
  res.writeHead(200, { "content-type": "text/html; charset=utf-8" });
  res.end(body);
}

async function callProvider(
  fixture: SkillFixture,
  prompt: string,
  provider: string,
): Promise<ProviderUsage> {
  if (provider === "mock") {
    return {
      prompt_tokens: fixture.expected_event.metrics["tokens_prompt_min"],
      completion_tokens:
        fixture.expected_event.metrics["tokens_completion_min"],
      text: `Mock ${fixture.skill} turn completed.`,
    };
  }

  if (provider === "openrouter" || provider === "gemini") {
    const isGemini = provider === "gemini";
    const apiKey =
      process.env[isGemini ? "GEMINI_API_KEY" : "OPENROUTER_API_KEY"];
    const model = process.env[isGemini ? "GEMINI_MODEL" : "OPENROUTER_MODEL"];
    if (!apiKey || !model || apiKey.includes("changeme")) {
      throw new Error(`${provider} requires an API key and model`);
    }
    const endpoint = isGemini
      ? "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
      : "https://openrouter.ai/api/v1/chat/completions";
    const response = await fetch(endpoint, {
      method: "POST",
      headers: {
        authorization: `Bearer ${apiKey}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        model,
        messages: [
          {
            role: "system",
            content: `Run the ${fixture.skill} local-stack scenario without revealing raw prompts.`,
          },
          { role: "user", content: prompt },
        ],
        tools: functionSchemas(activeTools(fixture)),
        tool_choice: "auto",
      }),
    });
    if (!response.ok)
      throw new Error(`${provider} returned HTTP ${response.status}`);
    const body = (await response.json()) as any;
    return {
      prompt_tokens: body.usage?.prompt_tokens ?? null,
      completion_tokens: body.usage?.completion_tokens ?? null,
      text:
        body.choices?.[0]?.message?.content ??
        `${provider} completed ${fixture.skill}.`,
    };
  }

  if (provider === "local") {
    const baseUrl =
      process.env["LOCAL_MODEL_BASE_URL"] ?? "http://ollama:11434";
    const model = process.env["LOCAL_MODEL"] ?? "llama3.2:1b";
    const response = await fetch(`${baseUrl.replace(/\/$/, "")}/api/chat`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        model,
        stream: false,
        messages: [{ role: "user", content: prompt }],
      }),
    });
    if (!response.ok)
      throw new Error(`local model returned HTTP ${response.status}`);
    const body = (await response.json()) as any;
    return {
      prompt_tokens: null,
      completion_tokens: null,
      text: body.message?.content ?? "Local model completed.",
    };
  }

  throw new Error(`unsupported provider ${provider}`);
}

function buildEvent(
  fixture: SkillFixture,
  usage: ProviderUsage,
  toolTrace: ToolTraceEntry[],
) {
  const metrics = fixture.expected_event.metrics;
  const context = { ...fixture.expected_event.context } as Record<
    string,
    unknown
  >;
  context["tags"] = ["local-stack", "js", fixture.skill];
  const projectId = process.env["CHATBOT_PROJECT_ID"];
  return {
    spec_version: "1.0",
    event_id: randomUUID(),
    correlation_id: `js-session:${Date.now()}`,
    timestamp: new Date().toISOString(),
    framework_source: "chatbot-js",
    workspace_id: workspaceId,
    ...(projectId ? { project_id: projectId } : {}),
    task: {
      name: `${fixture.skill}: local stack turn`,
      ...fixture.expected_event.task,
    },
    metrics: {
      duration_ms: 1,
      tokens_prompt: usage.prompt_tokens,
      tokens_completion: usage.completion_tokens,
      tool_call_count: metrics["tool_call_count"],
      workflow_steps: metrics["workflow_steps"],
      retries: metrics["retries"],
      artifact_count: metrics["artifact_count"],
      output_size_proxy: metrics["output_size_proxy"],
    },
    context,
    meta: {
      sdk_language: "node",
      sdk_version: sdkVersion,
      scoring_profile: scoringProfile,
      extensions: {
        "chatbot.skill": fixture.skill,
        "chatbot.turn": 1,
        "chatbot.tool_trace": toolTrace.map((entry) => entry.tool_name),
      },
    },
  };
}

async function handleChat(
  req: IncomingMessage,
  res: ServerResponse,
): Promise<void> {
  const body = await readBody(req);
  const fixture = loadSkill(body.skill);
  const provider = body.provider ?? process.env["LLM_PROVIDER"] ?? "mock";
  const prompt = body.prompt ?? "Summarize this local SDK stack.";
  const usage = await callProvider(fixture, prompt, provider);
  const toolTrace = traceForTools(activeTools(fixture));
  const event = buildEvent(fixture, usage, toolTrace);
  const submission = await client.ingestEvent({ workspaceId, event });
  json(res, 200, {
    ok: true,
    skill: fixture.skill,
    event_id: submission.event_id,
    reply: usage.text,
    tool_trace: toolTrace,
    event,
    score_result: submission.score,
  });
}

const server = createServer((req, res) => {
  void (async () => {
    try {
      if (req.method === "GET" && req.url === "/healthz")
        return json(res, 200, { ok: true });
      if (req.method === "GET" && req.url === "/") return html(res);
      if (req.method === "POST" && req.url === "/chat")
        return await handleChat(req, res);
      return json(res, 404, { ok: false, error: "not_found" });
    } catch (error) {
      return json(res, 500, {
        ok: false,
        error: error instanceof Error ? error.message : String(error),
      });
    }
  })();
});

server.listen(port, "0.0.0.0", () => {
  console.log(`heeczer JS chatbot listening on ${port}`);
});
