"""HMAC signing helpers for AMP requests."""

from __future__ import annotations

import hashlib
import hmac
import time


def build_signature_payload(timestamp: str, method: str, path_with_query: str, body: str) -> str:
    """Build the canonical payload used by Love Envoy HMAC verification.

    Payload format: "{timestamp}.{METHOD}.{path}.{sha256(body)}"
    """

    body_hash = hashlib.sha256(body.encode("utf-8")).hexdigest()
    return f"{timestamp}.{method.upper()}.{path_with_query}.{body_hash}"


def sign(payload: str, secret: str) -> str:
    """Return hex-encoded HMAC-SHA256 digest."""

    return hmac.new(secret.encode("utf-8"), payload.encode("utf-8"), hashlib.sha256).hexdigest()


def signed_headers(
    *,
    api_key: str,
    hmac_secret: str,
    method: str,
    path_with_query: str,
    body: str,
    timestamp: str | None = None,
) -> dict[str, str]:
    """Generate AMP auth headers for a request."""

    ts = timestamp or str(int(time.time()))
    payload = build_signature_payload(ts, method, path_with_query, body)
    signature = sign(payload, hmac_secret)
    return {
        "X-API-Key": api_key,
        "X-Timestamp": ts,
        "X-Signature": signature,
    }
