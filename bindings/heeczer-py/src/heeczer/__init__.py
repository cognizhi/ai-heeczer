"""Python client for the ai-heeczer ingestion service (plan 0006)."""

from .client import (
    ApiErrorKind,
    ConfidenceBand,
    HeeczerApiError,
    HeeczerClient,
    IngestEventResponse,
    ScoreResult,
    VersionResponse,
)

__all__ = [
    "ApiErrorKind",
    "ConfidenceBand",
    "HeeczerApiError",
    "HeeczerClient",
    "IngestEventResponse",
    "ScoreResult",
    "VersionResponse",
]

__version__ = "0.1.0"
