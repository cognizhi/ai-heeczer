# PydanticAI adapter example

This example shows how to wrap a PydanticAI agent so ai-heeczer emits one
canonical event for each `run()` or `run_sync()` invocation.

## Prerequisites

```bash
pip install pydantic-ai
cd bindings/heeczer-py && pip install -e .
```

## Usage

```python
import asyncio

from pydantic_ai import Agent

from heeczer import HeeczerClient
from heeczer.adapters.pydantic_ai import instrument_pydanticai_agent

async def main() -> None:
    client = HeeczerClient(base_url="http://localhost:8080")
    agent = Agent("openai:gpt-5.2", name="support_agent")
    instrumented = instrument_pydanticai_agent(
        agent=agent,
        client=client,
        workspace_id="ws_myteam",
    )

    result = await instrumented.run("Summarize the ticket")
    print(result.output)


asyncio.run(main())
```

For synchronous PydanticAI calls, pair the adapter with `SyncHeeczerClient`:

```python
from pydantic_ai import Agent

from heeczer import SyncHeeczerClient
from heeczer.adapters.pydantic_ai import instrument_pydanticai_agent

client = SyncHeeczerClient(base_url="http://localhost:8080")
agent = Agent("openai:gpt-5.2", name="support_agent")
instrumented = instrument_pydanticai_agent(
    agent=agent,
    client=client,
    workspace_id="ws_myteam",
)

result = instrumented.run_sync("Summarize the ticket")
print(result.output)
```

See [docs/architecture/integrations.md](../../docs/architecture/integrations.md) for the full mapping reference.
