"""Pydantic v2 models for the canonical ai-heeczer event contract."""

from __future__ import annotations

from typing import Any, Literal

from pydantic import BaseModel, ConfigDict, Field

Outcome = Literal["success", "partial_success", "failure", "timeout"]
RiskClass = Literal["low", "medium", "high"]


class EventIdentityModel(BaseModel):
    model_config = ConfigDict(extra="forbid")

    user_id: str | None = Field(default=None, max_length=128)
    team_id: str | None = Field(default=None, max_length=128)
    business_unit_id: str | None = Field(default=None, max_length=128)
    tier_id: str | None = Field(default=None, max_length=64)


class EventTaskModel(BaseModel):
    model_config = ConfigDict(extra="forbid")

    name: str = Field(min_length=1, max_length=256)
    outcome: Outcome
    category: str | None = Field(
        default=None,
        min_length=1,
        max_length=64,
        pattern=r"^[a-z0-9][a-z0-9_]*$",
    )
    sub_category: str | None = Field(
        default=None,
        min_length=1,
        max_length=64,
        pattern=r"^[a-z0-9][a-z0-9_]*$",
    )


class EventMetricsModel(BaseModel):
    model_config = ConfigDict(extra="forbid")

    duration_ms: int = Field(ge=0, le=86_400_000)
    tokens_prompt: int | None = Field(default=None, ge=0, le=10_000_000)
    tokens_completion: int | None = Field(default=None, ge=0, le=10_000_000)
    tool_call_count: int | None = Field(default=None, ge=0, le=10_000)
    workflow_steps: int | None = Field(default=None, ge=0, le=10_000)
    retries: int | None = Field(default=None, ge=0, le=1_000)
    artifact_count: int | None = Field(default=None, ge=0, le=10_000)
    output_size_proxy: float | None = Field(default=None, ge=0, le=1_000_000)


class EventContextModel(BaseModel):
    model_config = ConfigDict(extra="forbid")

    human_in_loop: bool | None = None
    review_required: bool | None = None
    temperature: float | None = Field(default=None, ge=0, le=2)
    risk_class: RiskClass | None = None
    tags: list[str] | None = Field(default=None, max_length=32)


class EventMetaModel(BaseModel):
    model_config = ConfigDict(extra="forbid")

    sdk_language: Literal["rust", "node", "python", "go", "java", "cli", "test"]
    sdk_version: str = Field(min_length=1, max_length=32)
    scoring_profile: str | None = Field(default=None, max_length=64)
    extensions: dict[str, Any] | None = None


class EventModel(BaseModel):
    """Strict runtime model for ``core/schema/event.v1.json``.

    Unknown fields are rejected everywhere except ``meta.extensions``, matching
    ADR-0002 and the server-side JSON Schema validator.
    """

    model_config = ConfigDict(extra="forbid")

    spec_version: Literal["1.0"]
    event_id: str = Field(min_length=1)
    correlation_id: str | None = Field(default=None, min_length=1, max_length=256)
    timestamp: str
    framework_source: str = Field(
        min_length=1,
        max_length=64,
        pattern=r"^[a-z0-9][a-z0-9_.-]*$",
    )
    workspace_id: str = Field(min_length=1, max_length=64, pattern=r"^[a-zA-Z0-9_.-]+$")
    project_id: str | None = Field(
        default=None,
        min_length=1,
        max_length=64,
        pattern=r"^[a-zA-Z0-9_.-]+$",
    )
    identity: EventIdentityModel | None = None
    task: EventTaskModel
    metrics: EventMetricsModel
    context: EventContextModel | None = None
    meta: EventMetaModel


def validate_event(event: dict[str, Any]) -> EventModel:
    """Validate and materialize an event with the strict Pydantic contract."""

    return EventModel.model_validate(event)
