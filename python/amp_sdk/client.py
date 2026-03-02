"""Baseline AMP/1.0 Python client.

Implements core operations from ACTION_PLAN P0:
- register agent
- create profile
- discover candidates
- initiate negotiation
- approval status + approve/reject
"""

from __future__ import annotations

import json
from dataclasses import asdict, is_dataclass
from typing import Any, Mapping, MutableMapping, Optional
from urllib.error import HTTPError, URLError
from urllib.parse import urlencode, urljoin
from urllib.request import Request, urlopen

from .auth import signed_headers
from .errors import CredentialsError, HttpStatusError, SerializationError
from .models import (
    ApprovalDecisionResponse,
    ApprovalStatus,
    CreateNegotiationRequest,
    CreateProfileRequest,
    CreateProfileResponse,
    DiscoveryResponse,
    Negotiation,
    RegisterAgentRequest,
    RegisterAgentResponse,
)


class AmpClient:
    """Synchronous HTTP client for AMP/1.0 servers."""

    def __init__(
        self,
        base_url: str,
        *,
        api_key: Optional[str] = None,
        hmac_secret: Optional[str] = None,
        timeout_seconds: float = 30.0,
    ) -> None:
        self.base_url = base_url.rstrip("/") + "/"
        self.api_key = api_key
        self.hmac_secret = hmac_secret
        self.timeout_seconds = timeout_seconds

    def set_credentials(self, *, api_key: str, hmac_secret: str) -> None:
        self.api_key = api_key
        self.hmac_secret = hmac_secret

    def health(self) -> dict[str, Any]:
        return self._request("GET", "/health", auth_required=False)

    def register_agent(self, request: RegisterAgentRequest | Mapping[str, Any]) -> RegisterAgentResponse:
        payload = self._normalize_payload(request)
        data = self._request("POST", "/api/v1/agents/register", payload, auth_required=False)
        return RegisterAgentResponse.from_dict(data)

    def create_profile(self, request: CreateProfileRequest | Mapping[str, Any]) -> CreateProfileResponse:
        payload = self._normalize_payload(request)
        data = self._request("POST", "/api/v1/profiles", payload, auth_required=True)
        return CreateProfileResponse.from_dict(data)

    def discover(self, *, page: Optional[int] = None, limit: Optional[int] = None) -> DiscoveryResponse:
        query: dict[str, int] = {}
        if page is not None:
            query["page"] = page
        if limit is not None:
            query["limit"] = limit
        data = self._request("GET", "/api/v1/discover", auth_required=True, query=query)
        return DiscoveryResponse.from_dict(data)

    def create_negotiation(
        self,
        request: CreateNegotiationRequest | Mapping[str, Any],
    ) -> Negotiation:
        payload = self._normalize_payload(request)
        data = self._request("POST", "/api/v1/negotiations", payload, auth_required=True)
        return Negotiation.from_dict(data.get("negotiation", {}))

    def approval_status(self, negotiation_id: str) -> ApprovalStatus:
        data = self._request(
            "GET",
            f"/api/v1/approvals/{negotiation_id}",
            auth_required=True,
        )
        return ApprovalStatus.from_dict(data)

    def approve_negotiation(
        self,
        negotiation_id: str,
        *,
        notes: Optional[str] = None,
        human_token: Optional[str] = None,
    ) -> ApprovalDecisionResponse:
        payload: MutableMapping[str, str] = {}
        if notes is not None:
            payload["notes"] = notes
        if human_token is not None:
            payload["human_token"] = human_token
        data = self._request(
            "POST",
            f"/api/v1/approvals/{negotiation_id}/approve",
            payload,
            auth_required=True,
        )
        return ApprovalDecisionResponse.from_dict(data)

    def reject_negotiation(self, negotiation_id: str, *, notes: Optional[str] = None) -> ApprovalDecisionResponse:
        payload: MutableMapping[str, str] = {}
        if notes is not None:
            payload["notes"] = notes
        data = self._request(
            "POST",
            f"/api/v1/approvals/{negotiation_id}/reject",
            payload,
            auth_required=True,
        )
        return ApprovalDecisionResponse.from_dict(data)

    def _normalize_payload(self, payload: Any) -> dict[str, Any]:
        if isinstance(payload, Mapping):
            return dict(payload)
        if hasattr(payload, "to_dict"):
            value = payload.to_dict()
            if not isinstance(value, dict):
                raise SerializationError("to_dict() must return a dict")
            return value
        if is_dataclass(payload):
            return asdict(payload)
        raise SerializationError(f"Unsupported payload type: {type(payload)!r}")

    def _request(
        self,
        method: str,
        path: str,
        payload: Optional[Mapping[str, Any]] = None,
        *,
        auth_required: bool,
        query: Optional[Mapping[str, Any]] = None,
    ) -> dict[str, Any]:
        path_with_query = path if path.startswith("/") else f"/{path}"
        if query:
            path_with_query = f"{path_with_query}?{urlencode(query)}"

        body = ""
        data_bytes = None
        if payload is not None:
            body = json.dumps(payload, separators=(",", ":"), ensure_ascii=False)
            data_bytes = body.encode("utf-8")

        headers: dict[str, str] = {"Accept": "application/json"}
        if data_bytes is not None:
            headers["Content-Type"] = "application/json"

        if auth_required:
            if not self.api_key or not self.hmac_secret:
                raise CredentialsError(
                    "Authenticated endpoint requested without api_key and hmac_secret",
                )
            headers.update(
                signed_headers(
                    api_key=self.api_key,
                    hmac_secret=self.hmac_secret,
                    method=method,
                    path_with_query=path_with_query,
                    body=body,
                )
            )

        request = Request(
            url=urljoin(self.base_url, path_with_query.lstrip("/")),
            data=data_bytes,
            headers=headers,
            method=method.upper(),
        )

        try:
            with urlopen(request, timeout=self.timeout_seconds) as response:
                raw = response.read().decode("utf-8")
                if not raw.strip():
                    return {}
                return json.loads(raw)
        except HTTPError as exc:
            body_text = exc.read().decode("utf-8", errors="replace") if exc.fp else ""
            message = body_text.strip().splitlines()[0] if body_text.strip() else exc.reason
            raise HttpStatusError(exc.code, str(message), body=body_text) from exc
        except URLError as exc:
            raise HttpStatusError(0, f"Network error: {exc.reason}") from exc
        except json.JSONDecodeError as exc:
            raise SerializationError(f"Failed to decode JSON response: {exc}") from exc
