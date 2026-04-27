package main

import (
	"bytes"
	"context"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strings"
	"time"

	heeczer "github.com/cognizhi/ai-heeczer/bindings/heeczer-go"
	"heeczer-test-go-chatbot/skills"
	"heeczer-test-go-chatbot/tools"
)

type chatRequest struct {
	Skill    string `json:"skill"`
	Prompt   string `json:"prompt"`
	Provider string `json:"provider"`
}

type providerUsage struct {
	PromptTokens     any    `json:"prompt_tokens"`
	CompletionTokens any    `json:"completion_tokens"`
	Text             string `json:"text"`
}

var workspaceID = envDefault("CHATBOT_WORKSPACE_ID", "local-test-go")
var scoringProfile = envDefault("CHATBOT_SCORING_PROFILE", "default")
var heeczerClient = mustClient()

func main() {
	mux := http.NewServeMux()
	mux.HandleFunc("/healthz", healthz)
	mux.HandleFunc("/", root)
	mux.HandleFunc("/chat", chat)
	port := envDefault("CHATBOT_PORT", "8000")
	log.Printf("heeczer Go chatbot listening on %s", port)
	log.Fatal(http.ListenAndServe("0.0.0.0:"+port, mux))
}

func mustClient() *heeczer.Client {
	client, clientErr := heeczer.New(envDefault("HEECZER_BASE_URL", "http://heeczer-ingest:8080"))
	if clientErr != nil {
		panic(clientErr)
	}
	return client
}

func healthz(responseWriter http.ResponseWriter, _ *http.Request) {
	writeJSON(responseWriter, http.StatusOK, map[string]bool{"ok": true})
}

func root(responseWriter http.ResponseWriter, _ *http.Request) {
	responseWriter.Header().Set("content-type", "text/html; charset=utf-8")
	_, _ = responseWriter.Write([]byte("<!doctype html><html lang='en'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>ai-heeczer Go stack</title></head><body><main><h1>ai-heeczer Go stack</h1><form id='chat'><select name='skill'><option value='code_gen'>code_gen</option><option value='rca'>rca</option><option value='doc_summary'>doc_summary</option><option value='compliance'>compliance</option><option value='ci_triage'>ci_triage</option><option value='architecture'>architecture</option></select><input name='prompt' value='Summarize this local SDK stack'><button>Send</button></form><pre id='out'></pre></main><script>document.querySelector('#chat').addEventListener('submit',async(event)=>{event.preventDefault();const form=new FormData(event.target);const response=await fetch('/chat',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify(Object.fromEntries(form))});document.querySelector('#out').textContent=JSON.stringify(await response.json(),null,2);});</script></body></html>"))
}

func chat(responseWriter http.ResponseWriter, request *http.Request) {
	if request.Method != http.MethodPost {
		writeJSON(responseWriter, http.StatusMethodNotAllowed, map[string]any{"ok": false})
		return
	}
	var chatBody chatRequest
	if decodeErr := json.NewDecoder(request.Body).Decode(&chatBody); decodeErr != nil && decodeErr != io.EOF {
		writeJSON(responseWriter, http.StatusBadRequest, map[string]any{"ok": false, "error": decodeErr.Error()})
		return
	}
	fixture, fixtureErr := skills.Load(chatBody.Skill)
	if fixtureErr != nil {
		writeJSON(responseWriter, http.StatusBadRequest, map[string]any{"ok": false, "error": fixtureErr.Error()})
		return
	}
	provider := chatBody.Provider
	if provider == "" {
		provider = envDefault("LLM_PROVIDER", "mock")
	}
	prompt := chatBody.Prompt
	if prompt == "" {
		prompt = "Summarize this local SDK stack."
	}
	startedAt := time.Now()
	usage, providerErr := callProvider(fixture, prompt, provider)
	if providerErr != nil {
		writeJSON(responseWriter, http.StatusBadGateway, map[string]any{"ok": false, "error": providerErr.Error()})
		return
	}
	toolTrace := tools.TraceForTools(skills.ActiveTools(fixture))
	event := buildEvent(fixture, usage, toolTrace, startedAt)
	submission, submitErr := heeczerClient.IngestEvent(context.Background(), workspaceID, event)
	if submitErr != nil {
		writeJSON(responseWriter, http.StatusBadGateway, map[string]any{"ok": false, "error": submitErr.Error()})
		return
	}
	var scoreResult map[string]any
	_ = json.Unmarshal([]byte(submission.Score.JSON()), &scoreResult)
	writeJSON(responseWriter, http.StatusOK, map[string]any{
		"ok":           true,
		"skill":        fixture.Skill,
		"event_id":     submission.EventID,
		"reply":        usage.Text,
		"tool_trace":   toolTrace,
		"event":        event,
		"score_result": scoreResult,
	})
}

func callProvider(fixture *skills.Fixture, prompt string, provider string) (*providerUsage, error) {
	if provider == "mock" {
		return &providerUsage{
			PromptTokens:     int(fixture.ExpectedEvent.Metrics["tokens_prompt_min"]),
			CompletionTokens: int(fixture.ExpectedEvent.Metrics["tokens_completion_min"]),
			Text:             "Mock " + fixture.Skill + " turn completed.",
		}, nil
	}
	if provider == "openrouter" || provider == "gemini" {
		isGemini := provider == "gemini"
		apiKeyName := "OPENROUTER_API_KEY"
		modelName := "OPENROUTER_MODEL"
		endpoint := "https://openrouter.ai/api/v1/chat/completions"
		if isGemini {
			apiKeyName = "GEMINI_API_KEY"
			modelName = "GEMINI_MODEL"
			endpoint = "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
		}
		apiKey := os.Getenv(apiKeyName)
		model := os.Getenv(modelName)
		if apiKey == "" || strings.Contains(apiKey, "changeme") || model == "" {
			return nil, fmt.Errorf("%s requires an API key and model", provider)
		}
		requestBody := map[string]any{
			"model": model,
			"messages": []map[string]string{
				{"role": "system", "content": "Run the " + fixture.Skill + " local-stack scenario without revealing raw prompts."},
				{"role": "user", "content": prompt},
			},
			"tools":       tools.FunctionSchemas(skills.ActiveTools(fixture)),
			"tool_choice": "auto",
		}
		payload, _ := json.Marshal(requestBody)
		httpRequest, requestErr := http.NewRequest(http.MethodPost, endpoint, bytes.NewReader(payload))
		if requestErr != nil {
			return nil, requestErr
		}
		httpRequest.Header.Set("authorization", "Bearer "+apiKey)
		httpRequest.Header.Set("content-type", "application/json")
		response, responseErr := http.DefaultClient.Do(httpRequest)
		if responseErr != nil {
			return nil, responseErr
		}
		defer response.Body.Close()
		if response.StatusCode >= 400 {
			return nil, fmt.Errorf("%s returned HTTP %d", provider, response.StatusCode)
		}
		var completion map[string]any
		if decodeErr := json.NewDecoder(response.Body).Decode(&completion); decodeErr != nil {
			return nil, decodeErr
		}
		usage, _ := completion["usage"].(map[string]any)
		return &providerUsage{PromptTokens: usage["prompt_tokens"], CompletionTokens: usage["completion_tokens"], Text: provider + " completed " + fixture.Skill + "."}, nil
	}
	if provider == "local" {
		baseURL := strings.TrimRight(envDefault("LOCAL_MODEL_BASE_URL", "http://ollama:11434"), "/")
		model := envDefault("LOCAL_MODEL", "llama3.2:1b")
		requestBody := map[string]any{
			"model":  model,
			"stream": false,
			"messages": []map[string]string{
				{"role": "user", "content": prompt},
			},
		}
		payload, _ := json.Marshal(requestBody)
		response, responseErr := http.Post(baseURL+"/api/chat", "application/json", bytes.NewReader(payload))
		if responseErr != nil {
			return nil, responseErr
		}
		defer response.Body.Close()
		if response.StatusCode >= 400 {
			return nil, fmt.Errorf("local model returned HTTP %d", response.StatusCode)
		}
		var completion map[string]any
		if decodeErr := json.NewDecoder(response.Body).Decode(&completion); decodeErr != nil {
			return nil, decodeErr
		}
		message, _ := completion["message"].(map[string]any)
		text, _ := message["content"].(string)
		if text == "" {
			text = "Local model completed."
		}
		return &providerUsage{PromptTokens: nil, CompletionTokens: nil, Text: text}, nil
	}
	return nil, fmt.Errorf("unsupported provider %s", provider)
}

func buildEvent(fixture *skills.Fixture, usage *providerUsage, toolTrace []tools.ToolTraceEntry, startedAt time.Time) map[string]any {
	contextBlock := map[string]any{}
	for key, value := range fixture.ExpectedEvent.Context {
		contextBlock[key] = value
	}
	contextBlock["tags"] = []string{"local-stack", "go", fixture.Skill}
	metrics := fixture.ExpectedEvent.Metrics
	traceNames := make([]string, 0, len(toolTrace))
	for _, traceEntry := range toolTrace {
		traceNames = append(traceNames, traceEntry.ToolName)
	}
	event := map[string]any{
		"spec_version":     "1.0",
		"event_id":         uuidV4(),
		"correlation_id":   fmt.Sprintf("go-session:%d", time.Now().UnixMilli()),
		"timestamp":        time.Now().UTC().Format(time.RFC3339),
		"framework_source": "chatbot-go",
		"workspace_id":     workspaceID,
		"task": map[string]any{
			"name":         fixture.Skill + ": local stack turn",
			"category":     fixture.ExpectedEvent.Task["category"],
			"sub_category": fixture.ExpectedEvent.Task["sub_category"],
			"outcome":      fixture.ExpectedEvent.Task["outcome"],
		},
		"metrics": map[string]any{
			"duration_ms":       max(1, int(time.Since(startedAt).Milliseconds())),
			"tokens_prompt":     usage.PromptTokens,
			"tokens_completion": usage.CompletionTokens,
			"tool_call_count":   int(metrics["tool_call_count"]),
			"workflow_steps":    int(metrics["workflow_steps"]),
			"retries":           int(metrics["retries"]),
			"artifact_count":    int(metrics["artifact_count"]),
			"output_size_proxy": metrics["output_size_proxy"],
		},
		"context": contextBlock,
		"meta": map[string]any{
			"sdk_language":    "go",
			"sdk_version":     heeczer.Version,
			"scoring_profile": scoringProfile,
			"extensions": map[string]any{
				"chatbot.skill":      fixture.Skill,
				"chatbot.turn":       1,
				"chatbot.tool_trace": traceNames,
			},
		},
	}
	if projectID := os.Getenv("CHATBOT_PROJECT_ID"); projectID != "" {
		event["project_id"] = projectID
	}
	return event
}

func writeJSON(responseWriter http.ResponseWriter, statusCode int, body any) {
	payload, _ := json.Marshal(body)
	responseWriter.Header().Set("content-type", "application/json; charset=utf-8")
	responseWriter.WriteHeader(statusCode)
	_, _ = responseWriter.Write(payload)
}

func envDefault(name string, fallback string) string {
	value := os.Getenv(name)
	if value == "" {
		return fallback
	}
	return value
}

func uuidV4() string {
	randomBytes := make([]byte, 16)
	_, _ = rand.Read(randomBytes)
	randomBytes[6] = (randomBytes[6] & 0x0f) | 0x40
	randomBytes[8] = (randomBytes[8] & 0x3f) | 0x80
	encoded := hex.EncodeToString(randomBytes)
	return encoded[0:8] + "-" + encoded[8:12] + "-" + encoded[12:16] + "-" + encoded[16:20] + "-" + encoded[20:32]
}

func max(left int, right int) int {
	if left > right {
		return left
	}
	return right
}
