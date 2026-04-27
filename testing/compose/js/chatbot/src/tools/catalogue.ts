export type ToolName =
  | "web_search"
  | "code_executor"
  | "document_reader"
  | "data_analyst"
  | "plan_reviewer"
  | "risk_checker"
  | "summarizer"
  | "diff_generator";

export interface ToolContribution {
  tool_call_count: number;
  workflow_steps: number;
  tokens_prompt: number;
  tokens_completion: number;
  artifact_count: number;
  output_size_proxy: number;
}

export interface ToolTraceEntry {
  tool_name: ToolName;
  invoked_at_ms: number;
  output_size: number;
}

export const TOOL_CONTRIBUTIONS: Record<ToolName, ToolContribution> = {
  web_search: {
    tool_call_count: 1,
    workflow_steps: 1,
    tokens_prompt: 500,
    tokens_completion: 0,
    artifact_count: 0,
    output_size_proxy: 0,
  },
  code_executor: {
    tool_call_count: 1,
    workflow_steps: 1,
    tokens_prompt: 0,
    tokens_completion: 0,
    artifact_count: 1,
    output_size_proxy: 0.5,
  },
  document_reader: {
    tool_call_count: 1,
    workflow_steps: 1,
    tokens_prompt: 2000,
    tokens_completion: 0,
    artifact_count: 0,
    output_size_proxy: 0,
  },
  data_analyst: {
    tool_call_count: 1,
    workflow_steps: 0,
    tokens_prompt: 0,
    tokens_completion: 800,
    artifact_count: 1,
    output_size_proxy: 1,
  },
  plan_reviewer: {
    tool_call_count: 1,
    workflow_steps: 1,
    tokens_prompt: 0,
    tokens_completion: 0,
    artifact_count: 0,
    output_size_proxy: 0,
  },
  risk_checker: {
    tool_call_count: 1,
    workflow_steps: 0,
    tokens_prompt: 0,
    tokens_completion: 0,
    artifact_count: 0,
    output_size_proxy: 0,
  },
  summarizer: {
    tool_call_count: 1,
    workflow_steps: 0,
    tokens_prompt: 0,
    tokens_completion: 0,
    artifact_count: 1,
    output_size_proxy: 0.8,
  },
  diff_generator: {
    tool_call_count: 1,
    workflow_steps: 0,
    tokens_prompt: 0,
    tokens_completion: 0,
    artifact_count: 1,
    output_size_proxy: 0.3,
  },
};

export function functionSchemas(activeTools: ToolName[]): object[] {
  return activeTools.map((name) => ({
    type: "function",
    function: {
      name,
      description: `Synthetic ${name} tool used by the ai-heeczer local stack.`,
      parameters: {
        type: "object",
        properties: {
          input: { type: "string" },
        },
        required: ["input"],
      },
    },
  }));
}

export function traceForTools(tools: ToolName[]): ToolTraceEntry[] {
  return tools.map((toolName, index) => ({
    tool_name: toolName,
    invoked_at_ms: index * 25,
    output_size: TOOL_CONTRIBUTIONS[toolName].output_size_proxy,
  }));
}
