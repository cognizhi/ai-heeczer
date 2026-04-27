package tools

// ToolTraceEntry is displayed by the local chatbot UI and mirrored into event metadata.
type ToolTraceEntry struct {
	ToolName    string  `json:"tool_name"`
	InvokedAtMs int     `json:"invoked_at_ms"`
	OutputSize  float64 `json:"output_size"`
}

var contributions = map[string]float64{
	"web_search":      0.0,
	"code_executor":   0.5,
	"document_reader": 0.0,
	"data_analyst":    1.0,
	"plan_reviewer":   0.0,
	"risk_checker":    0.0,
	"summarizer":      0.8,
	"diff_generator":  0.3,
}

// TraceForTools turns a mock script's tool list into a stable trace.
func TraceForTools(activeTools []string) []ToolTraceEntry {
	trace := make([]ToolTraceEntry, 0, len(activeTools))
	for index, toolName := range activeTools {
		trace = append(trace, ToolTraceEntry{
			ToolName:    toolName,
			InvokedAtMs: index * 25,
			OutputSize:  contributions[toolName],
		})
	}
	return trace
}

// FunctionSchemas returns OpenAI-compatible function declarations.
func FunctionSchemas(activeTools []string) []map[string]any {
	schemas := make([]map[string]any, 0, len(activeTools))
	for _, toolName := range activeTools {
		schemas = append(schemas, map[string]any{
			"type": "function",
			"function": map[string]any{
				"name":        toolName,
				"description": "Synthetic " + toolName + " tool used by the ai-heeczer local stack.",
				"parameters": map[string]any{
					"type":       "object",
					"properties": map[string]any{"input": map[string]any{"type": "string"}},
					"required":   []string{"input"},
				},
			},
		})
	}
	return schemas
}
