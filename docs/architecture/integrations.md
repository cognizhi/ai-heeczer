# Integration mapping reference

> Status: active reference
> Last reviewed: 2026-04-27
> Owner: SDK Engineer

This document describes how each supported framework's native telemetry maps to
the canonical ai-heeczer event schema.

## LangGraph

| LangGraph event        | ai-heeczer field            | Notes                                                           |
| ---------------------- | --------------------------- | --------------------------------------------------------------- |
| Node start time        | `metrics.duration_ms`       | Measured wall-clock inside callback                             |
| `on_llm_start` prompts | `metrics.tokens_prompt`     | Approximated at 4 chars/token; use tiktoken for accuracy        |
| `on_llm_end` usage     | `metrics.tokens_completion` | From `generation_info.completion_tokens`                        |
| `on_tool_start`        | `metrics.tool_call_count`   | Incremented per tool invocation                                 |
| `on_retry`             | `metrics.retries`           | Incremented per retry event                                     |
| `on_chain_end`         | `task.outcome = "success"`  |                                                                 |
| `on_chain_error`       | `task.outcome = "failure"`  | Error message in `meta.extensions.error_summary`                |
| Serialized node name   | `task.name`                 | From `serialized["name"]` or last element of `serialized["id"]` |

## Google ADK

| ADK concept                      | ai-heeczer field            | Notes                       |
| -------------------------------- | --------------------------- | --------------------------- |
| Agent coroutine duration         | `metrics.duration_ms`       | Measured wall-clock         |
| `result.usage.prompt_tokens`     | `metrics.tokens_prompt`     | If result exposes usage     |
| `result.usage.completion_tokens` | `metrics.tokens_completion` |                             |
| `result.tool_calls`              | `metrics.tool_call_count`   | `len(result.tool_calls)`    |
| Success                          | `task.outcome = "success"`  |                             |
| Exception                        | `task.outcome = "failure"`  |                             |
| `task_name` parameter            | `task.name`                 | Provided at decoration time |

## PydanticAI

| PydanticAI concept             | ai-heeczer field            | Notes                                               |
| ------------------------------ | --------------------------- | --------------------------------------------------- |
| `Agent.run()` / `run_sync()`   | event boundary              | One canonical event emitted per agent invocation    |
| `result.usage().input_tokens`  | `metrics.tokens_prompt`     | Falls back to `prompt_tokens` if present            |
| `result.usage().output_tokens` | `metrics.tokens_completion` | Falls back to `completion_tokens` if present        |
| `result.usage().tool_calls`    | `metrics.tool_call_count`   | Uses usage count or list length when exposed        |
| `result.usage().retries`       | `metrics.retries`           | Included only when surfaced by the result/usage API |
| agent `name` / `task_name`     | `task.name`                 | Explicit `task_name` overrides the agent name       |
| Raised exception               | `task.outcome = "failure"` | Error summary stored under `meta.extensions`        |

## Future adapters

- **Langfuse**: webhook adapter consuming Langfuse `trace.create` events.
- **OpenTelemetry bridge**: map OTel spans with `gen_ai.*` semantic conventions.
- **Generic webhook**: POST adapter for custom telemetry pipelines.
