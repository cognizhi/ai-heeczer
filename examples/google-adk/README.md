# Google ADK adapter example

This example shows how to use the `heeczer.adapters.google_adk` decorator to
track Google ADK agent invocations.

## Prerequisites

```bash
pip install google-adk
cd bindings/heeczer-py && pip install -e .
```

## Usage

```python
from heeczer import HeeczerClient
from heeczer.adapters.google_adk import heeczer_adk_wrapper

client = HeeczerClient(base_url="http://localhost:8080")

@heeczer_adk_wrapper(client=client, workspace_id="ws_myteam", task_name="my_agent")
async def my_agent(inputs):
    # Your ADK agent logic here
    ...
```

See [docs/architecture/integrations.md](../../docs/architecture/integrations.md) for the full mapping reference.
