from __future__ import annotations

from typing import Any, Literal, TypedDict

ToolName = Literal[
    "web_search",
    "code_executor",
    "document_reader",
    "data_analyst",
    "plan_reviewer",
    "risk_checker",
    "summarizer",
    "diff_generator",
]


class ToolContribution(TypedDict):
    tool_call_count: int
    workflow_steps: int
    tokens_prompt: int
    tokens_completion: int
    artifact_count: int
    output_size_proxy: float


TOOL_CONTRIBUTIONS: dict[ToolName, ToolContribution] = {
    "web_search": {"tool_call_count": 1, "workflow_steps": 1, "tokens_prompt": 500, "tokens_completion": 0, "artifact_count": 0, "output_size_proxy": 0.0},
    "code_executor": {"tool_call_count": 1, "workflow_steps": 1, "tokens_prompt": 0, "tokens_completion": 0, "artifact_count": 1, "output_size_proxy": 0.5},
    "document_reader": {"tool_call_count": 1, "workflow_steps": 1, "tokens_prompt": 2000, "tokens_completion": 0, "artifact_count": 0, "output_size_proxy": 0.0},
    "data_analyst": {"tool_call_count": 1, "workflow_steps": 0, "tokens_prompt": 0, "tokens_completion": 800, "artifact_count": 1, "output_size_proxy": 1.0},
    "plan_reviewer": {"tool_call_count": 1, "workflow_steps": 1, "tokens_prompt": 0, "tokens_completion": 0, "artifact_count": 0, "output_size_proxy": 0.0},
    "risk_checker": {"tool_call_count": 1, "workflow_steps": 0, "tokens_prompt": 0, "tokens_completion": 0, "artifact_count": 0, "output_size_proxy": 0.0},
    "summarizer": {"tool_call_count": 1, "workflow_steps": 0, "tokens_prompt": 0, "tokens_completion": 0, "artifact_count": 1, "output_size_proxy": 0.8},
    "diff_generator": {"tool_call_count": 1, "workflow_steps": 0, "tokens_prompt": 0, "tokens_completion": 0, "artifact_count": 1, "output_size_proxy": 0.3},
}


def function_schemas(active_tools: list[ToolName]) -> list[dict[str, object]]:
    return [
        {
            "type": "function",
            "function": {
                "name": tool,
                "description": f"Synthetic {tool} tool used by the ai-heeczer local stack.",
                "parameters": {"type": "object", "properties": {"input": {"type": "string"}}, "required": ["input"]},
            },
        }
        for tool in active_tools
    ]


def trace_for_tools(active_tools: list[ToolName]) -> list[dict[str, object]]:
    return [
        {
            "tool_name": tool,
            "invoked_at_ms": index * 25,
            "output_size": TOOL_CONTRIBUTIONS[tool]["output_size_proxy"],
        }
        for index, tool in enumerate(active_tools)
    ]


def pydantic_ai_tools(active_tools: list[ToolName]) -> list[Any]:
    from pydantic_ai import Tool

    tools: list[Any] = []
    for tool_name in active_tools:
        async def stub(input: str, selected_tool: str = tool_name) -> dict[str, str]:
            return {"tool": selected_tool, "summary": f"synthetic result for {input[:32]}"}

        tools.append(Tool(stub, name=tool_name, description=f"Synthetic {tool_name} local-stack tool"))
    return tools
