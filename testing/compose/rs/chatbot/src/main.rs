mod skills;
mod tools;

use axum::{
    extract::State,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use heeczer::http::Client as HeeczerHttpClient;
use heeczer_core::Event;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

use skills::catalogue::{active_tools, load_skill, SkillFixture};
use tools::catalogue::{function_schemas, trace_for_tools, ToolTraceEntry};

#[derive(Clone)]
struct AppState {
    client: HeeczerHttpClient,
    workspace_id: String,
    scoring_profile: String,
}

#[derive(Debug, Deserialize)]
struct ChatRequest {
    skill: Option<String>,
    prompt: Option<String>,
    provider: Option<String>,
}

#[derive(Debug)]
struct ProviderUsage {
    prompt_tokens: Value,
    completion_tokens: Value,
    text: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    ok: bool,
    error: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let port = std::env::var("CHATBOT_PORT").unwrap_or_else(|_| "8000".to_owned());
    let state = Arc::new(AppState {
        client: HeeczerHttpClient::new(
            std::env::var("HEECZER_BASE_URL")
                .unwrap_or_else(|_| "http://heeczer-ingest:8080".to_owned()),
            "",
        ),
        workspace_id: std::env::var("CHATBOT_WORKSPACE_ID")
            .unwrap_or_else(|_| "local-test-rs".to_owned()),
        scoring_profile: std::env::var("CHATBOT_SCORING_PROFILE")
            .unwrap_or_else(|_| "default".to_owned()),
    });
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/", get(root))
        .route("/chat", post(chat))
        .with_state(state);
    let address: SocketAddr = format!("0.0.0.0:{port}").parse()?;
    println!("heeczer Rust chatbot listening on {address}");
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

async fn root() -> Html<&'static str> {
    Html("<!doctype html><html lang='en'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>ai-heeczer Rust stack</title></head><body><main><h1>ai-heeczer Rust stack</h1><form id='chat'><select name='skill'><option value='code_gen'>code_gen</option><option value='rca'>rca</option><option value='doc_summary'>doc_summary</option><option value='compliance'>compliance</option><option value='ci_triage'>ci_triage</option><option value='architecture'>architecture</option></select><input name='prompt' value='Summarize this local SDK stack'><button>Send</button></form><pre id='out'></pre></main><script>document.querySelector('#chat').addEventListener('submit',async(event)=>{event.preventDefault();const form=new FormData(event.target);const response=await fetch('/chat',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(Object.fromEntries(form))});document.querySelector('#out').textContent=JSON.stringify(await response.json(),null,2);});</script></body></html>")
}

async fn chat(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<Value>, Json<ErrorResponse>> {
    let started_at = Instant::now();
    let fixture = load_skill(request.skill.as_deref()).map_err(internal_error)?;
    let provider = request
        .provider
        .as_deref()
        .map(str::to_owned)
        .or_else(|| std::env::var("LLM_PROVIDER").ok())
        .unwrap_or_else(|| "mock".to_owned());
    let prompt = request
        .prompt
        .as_deref()
        .unwrap_or("Summarize this local SDK stack.");
    let usage = call_provider(&fixture, prompt, &provider)
        .await
        .map_err(internal_error)?;
    let active_tools = active_tools(&fixture);
    let tool_trace = trace_for_tools(&active_tools);
    let event_value = build_event(&state, &fixture, &usage, &tool_trace, started_at);
    let event: Event = serde_json::from_value(event_value.clone()).map_err(internal_error)?;
    let score_result = state
        .client
        .score_event(&state.workspace_id, &event)
        .await
        .map_err(internal_error)?;
    Ok(Json(serde_json::json!({
        "ok": true,
        "skill": fixture.skill,
        "event_id": event.event_id,
        "reply": usage.text,
        "tool_trace": tool_trace,
        "event": event_value,
        "score_result": score_result,
    })))
}

async fn call_provider(
    fixture: &SkillFixture,
    prompt: &str,
    provider: &str,
) -> anyhow::Result<ProviderUsage> {
    if provider == "mock" {
        let metrics = &fixture.expected_event.metrics;
        return Ok(ProviderUsage {
            prompt_tokens: metrics["tokens_prompt_min"].clone(),
            completion_tokens: metrics["tokens_completion_min"].clone(),
            text: format!("Mock {} turn completed.", fixture.skill),
        });
    }
    if provider == "openrouter" || provider == "gemini" {
        let is_gemini = provider == "gemini";
        let api_key_name = if is_gemini {
            "GEMINI_API_KEY"
        } else {
            "OPENROUTER_API_KEY"
        };
        let model_name = if is_gemini {
            "GEMINI_MODEL"
        } else {
            "OPENROUTER_MODEL"
        };
        let api_key = std::env::var(api_key_name).unwrap_or_default();
        let model = std::env::var(model_name).unwrap_or_default();
        anyhow::ensure!(
            !api_key.is_empty() && !api_key.contains("changeme") && !model.is_empty(),
            "{provider} requires an API key and model"
        );
        let endpoint = if is_gemini {
            "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
        } else {
            "https://openrouter.ai/api/v1/chat/completions"
        };
        let active_tools = active_tools(fixture);
        let response: Value = reqwest::Client::new()
            .post(endpoint)
            .bearer_auth(api_key)
            .json(&serde_json::json!({
                "model": model,
                "messages": [
                    { "role": "system", "content": format!("Run the {} local-stack scenario without revealing raw prompts.", fixture.skill) },
                    { "role": "user", "content": prompt }
                ],
                "tools": function_schemas(&active_tools),
                "tool_choice": "auto"
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        return Ok(ProviderUsage {
            prompt_tokens: response["usage"]
                .get("prompt_tokens")
                .cloned()
                .unwrap_or(Value::Null),
            completion_tokens: response["usage"]
                .get("completion_tokens")
                .cloned()
                .unwrap_or(Value::Null),
            text: response["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("Provider completed.")
                .to_owned(),
        });
    }
    if provider == "local" {
        let base_url = std::env::var("LOCAL_MODEL_BASE_URL")
            .unwrap_or_else(|_| "http://ollama:11434".to_owned())
            .trim_end_matches('/')
            .to_owned();
        let model = std::env::var("LOCAL_MODEL").unwrap_or_else(|_| "llama3.2:1b".to_owned());
        let response: Value = reqwest::Client::new()
            .post(format!("{base_url}/api/chat"))
            .json(&serde_json::json!({
                "model": model,
                "stream": false,
                "messages": [{ "role": "user", "content": prompt }]
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        return Ok(ProviderUsage {
            prompt_tokens: Value::Null,
            completion_tokens: Value::Null,
            text: response["message"]["content"]
                .as_str()
                .unwrap_or("Local model completed.")
                .to_owned(),
        });
    }
    anyhow::bail!("unsupported provider {provider}")
}

fn build_event(
    state: &AppState,
    fixture: &SkillFixture,
    usage: &ProviderUsage,
    tool_trace: &[ToolTraceEntry],
    started_at: Instant,
) -> Value {
    let mut context = fixture.expected_event.context.clone();
    context["tags"] = serde_json::json!(["local-stack", "rs", fixture.skill]);
    let trace_names: Vec<_> = tool_trace
        .iter()
        .map(|trace_entry| trace_entry.tool_name.clone())
        .collect();
    let metrics = &fixture.expected_event.metrics;
    let mut event = serde_json::json!({
        "spec_version": "1.0",
        "event_id": Uuid::new_v4().to_string(),
        "correlation_id": format!("rs-session:{}", unix_millis()),
        "timestamp": rfc3339_now(),
        "framework_source": "chatbot-rs",
        "workspace_id": state.workspace_id,
        "task": {
            "name": format!("{}: local stack turn", fixture.skill),
            "category": fixture.expected_event.task["category"],
            "sub_category": fixture.expected_event.task["sub_category"],
            "outcome": fixture.expected_event.task["outcome"]
        },
        "metrics": {
            "duration_ms": started_at.elapsed().as_millis().max(1) as u64,
            "tokens_prompt": usage.prompt_tokens,
            "tokens_completion": usage.completion_tokens,
            "tool_call_count": metrics["tool_call_count"],
            "workflow_steps": metrics["workflow_steps"],
            "retries": metrics["retries"],
            "artifact_count": metrics["artifact_count"],
            "output_size_proxy": metrics["output_size_proxy"]
        },
        "context": context,
        "meta": {
            "sdk_language": "rust",
            "sdk_version": "0.5.1",
            "scoring_profile": state.scoring_profile,
            "extensions": {
                "chatbot.skill": fixture.skill,
                "chatbot.turn": 1,
                "chatbot.tool_trace": trace_names
            }
        }
    });
    if let Ok(project_id) = std::env::var("CHATBOT_PROJECT_ID") {
        event["project_id"] = Value::String(project_id);
    }
    event
}

fn internal_error(error: impl std::fmt::Display) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        ok: false,
        error: error.to_string(),
    })
}

fn unix_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis()
}

fn rfc3339_now() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs();
    chrono_like_timestamp(seconds)
}

fn chrono_like_timestamp(seconds: u64) -> String {
    let datetime = time::OffsetDateTime::from_unix_timestamp(seconds as i64)
        .unwrap_or(time::OffsetDateTime::UNIX_EPOCH);
    datetime
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "2026-04-27T00:00:00Z".to_owned())
}
