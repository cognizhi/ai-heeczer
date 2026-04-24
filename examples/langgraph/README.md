# LangGraph adapter example

This example shows how to use the `heeczer.adapters.langgraph` callback handler
to automatically track LangGraph node executions.

## Prerequisites

```bash
pip install langgraph langchain-openai
cd bindings/heeczer-py && pip install -e .
```

## Usage

```python
from heeczer import HeeczerClient
from heeczer.adapters.langgraph import HeeczerLangGraphCallback

client = HeeczerClient(base_url="http://localhost:8080")
callback = HeeczerLangGraphCallback(
    client=client,
    workspace_id="ws_myteam",
)

# Pass to your LangGraph graph invocation
result = graph.invoke(
    {"messages": [...]},
    config={"callbacks": [callback]},
)
```

See [docs/architecture/integrations.md](../../docs/architecture/integrations.md) for the full mapping reference.
