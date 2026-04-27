from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any

from tools.catalogue import ToolName

ALIASES = {"code-gen": "code_gen", "doc-summary": "doc_summary", "ci-triage": "ci_triage"}


def normalize_skill(raw: str | None) -> str:
    value = (raw or "code_gen").removeprefix("/skill ").strip()
    return ALIASES.get(value, value)


def load_skill(raw: str | None) -> dict[str, Any]:
    skill = normalize_skill(raw)
    fixture_root = Path(os.environ.get("SKILL_FIXTURE_DIR", "/fixtures/skills"))
    return json.loads((fixture_root / f"{skill}.json").read_text(encoding="utf-8"))


def active_tools(fixture: dict[str, Any]) -> list[ToolName]:
    return [step["tool"] for step in fixture["mock_script"]]
