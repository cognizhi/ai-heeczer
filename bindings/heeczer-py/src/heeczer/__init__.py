"""Python client for the ai-heeczer ingestion service (plan 0006)."""

from .client import (
    ApiErrorKind,
    ConfidenceBand,
    Event,
    EventContext,
    EventIdentity,
    EventMeta,
    EventMetrics,
    EventTask,
    HeeczerApiError,
    HeeczerClient,
    IngestEventResponse,
    Outcome,
    RiskClass,
    ScoreResult,
    SyncHeeczerClient,
    VersionResponse,
)

__all__ = [
    "ApiErrorKind",
    "ConfidenceBand",
    "Event",
    "EventContext",
    "EventIdentity",
    "EventMeta",
    "EventMetrics",
    "EventTask",
    "HeeczerApiError",
    "HeeczerClient",
    "IngestEventResponse",
    "Outcome",
    "RiskClass",
    "ScoreResult",
    "SyncHeeczerClient",
    "VersionResponse",
]

__version__ = "0.1.0"
