"""Error types for the AMP Python SDK."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional


class AmpError(Exception):
    """Base exception for SDK errors."""


@dataclass
class HttpStatusError(AmpError):
    """Raised when the server returns a non-2xx HTTP status."""

    status_code: int
    message: str
    body: Optional[str] = None

    def __str__(self) -> str:
        return f"HTTP {self.status_code}: {self.message}"


class CredentialsError(AmpError):
    """Raised when authenticated operations are attempted without credentials."""


class SerializationError(AmpError):
    """Raised when payload encoding/decoding fails."""
