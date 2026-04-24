# Integration mapping reference

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

## Future adapters

- **PydanticAI**: instrument `Agent.run()` / `Agent.run_sync()` hooks.
- **Langfuse**: webhook adapter consuming Langfuse `trace.create` events.
- **OpenTelemetry bridge**: map OTel spans with `gen_ai.*` semantic conventions.
- **Generic webhook**: POST adapter for custom telemetry pipelines.
