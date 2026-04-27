use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ToolTraceEntry {
    pub tool_name: String,
    pub invoked_at_ms: usize,
    pub output_size: f64,
}

pub fn trace_for_tools(active_tools: &[String]) -> Vec<ToolTraceEntry> {
    active_tools
        .iter()
        .enumerate()
        .map(|(index, tool_name)| ToolTraceEntry {
            tool_name: tool_name.clone(),
            invoked_at_ms: index * 25,
            output_size: output_size(tool_name),
        })
        .collect()
}

pub fn function_schemas(active_tools: &[String]) -> Vec<serde_json::Value> {
    active_tools
        .iter()
        .map(|tool_name| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": tool_name,
                    "description": format!("Synthetic {tool_name} tool used by the ai-heeczer local stack."),
                    "parameters": {
                        "type": "object",
                        "properties": { "input": { "type": "string" } },
                        "required": ["input"]
                    }
                }
            })
        })
        .collect()
}

fn output_size(tool_name: &str) -> f64 {
    match tool_name {
        "code_executor" => 0.5,
        "data_analyst" => 1.0,
        "summarizer" => 0.8,
        "diff_generator" => 0.3,
        _ => 0.0,
    }
}
