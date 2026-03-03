"""HMAC signing helpers for AMP requests."""

from __future__ import annotations

import hashlib
import hmac
import secrets
import time

DEFAULT_CLOCK_SKEW_SECONDS = 300
_NONCE_ALPHABET = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._:-"


def build_signature_payload(
    timestamp: str,
    method: str,
    path_with_query: str,
    body: str,
    nonce: str = "",
) -> str:
    """Build the canonical payload used by Love Envoy HMAC verification.

    Payload format: "{timestamp}.{METHOD}.{path}.{sha256(body)}.{nonce}"
    """

    body_hash = hashlib.sha256(body.encode("utf-8")).hexdigest()
    return f"{timestamp}.{method.upper()}.{path_with_query}.{body_hash}.{nonce}"


def sign(payload: str, secret: str) -> str:
    """Return hex-encoded HMAC-SHA256 digest."""

    return hmac.new(secret.encode("utf-8"), payload.encode("utf-8"), hashlib.sha256).hexdigest()


def current_unix_timestamp(now: int | float | None = None) -> str:
    """Return the current unix timestamp (seconds) as a string."""

    value = now if now is not None else time.time()
    return str(int(value))


def is_timestamp_fresh(
    timestamp: str,
    *,
    now: int | float | None = None,
    max_skew_seconds: int = DEFAULT_CLOCK_SKEW_SECONDS,
) -> bool:
    """Validate that a timestamp is within an acceptable clock-skew window."""

    try:
        ts = int(timestamp)
    except (TypeError, ValueError):
        return False

    reference = int(now if now is not None else time.time())
    return abs(reference - ts) <= max(0, max_skew_seconds)


def generate_nonce(length: int = 32) -> str:
    """Generate a request nonce that matches Love Envoy's allowed character set."""

    if length < 8:
        raise ValueError("nonce length must be >= 8")
    return "".join(secrets.choice(_NONCE_ALPHABET) for _ in range(length))


def signed_headers(
    *,
    api_key: str,
    hmac_secret: str,
    method: str,
    path_with_query: str,
    body: str,
    timestamp: str | None = None,
    nonce: str | None = None,
) -> dict[str, str]:
    """Generate AMP auth headers for a request."""

    ts = timestamp or current_unix_timestamp()
    request_nonce = generate_nonce() if nonce is None else nonce
    payload = build_signature_payload(ts, method, path_with_query, body, request_nonce)
    signature = sign(payload, hmac_secret)

    headers = {
        "X-API-Key": api_key,
        "X-Timestamp": ts,
        "X-Signature": signature,
    }

    if request_nonce:
        headers["X-Nonce"] = request_nonce

    return headers
