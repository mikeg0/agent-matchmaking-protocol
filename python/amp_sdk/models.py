"""Data models for the baseline AMP Python SDK."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional

JSONDict = Dict[str, Any]


@dataclass
class RegisterAgentRequest:
    name: str
    agent_platform: Optional[str] = None
    agent_version: Optional[str] = None
    capabilities: List[str] = field(default_factory=list)
    webhook_url: Optional[str] = None
    webhook_secret: Optional[str] = None

    def to_dict(self) -> JSONDict:
        payload: JSONDict = {"name": self.name}
        if self.agent_platform is not None:
            payload["agent_platform"] = self.agent_platform
        if self.agent_version is not None:
            payload["agent_version"] = self.agent_version
        if self.capabilities:
            payload["capabilities"] = self.capabilities
        if self.webhook_url is not None:
            payload["webhook_url"] = self.webhook_url
        if self.webhook_secret is not None:
            payload["webhook_secret"] = self.webhook_secret
        return payload


@dataclass
class RegisterAgentResponse:
    agent_id: str
    api_key: str
    status: str
    message: Optional[str] = None

    @classmethod
    def from_dict(cls, data: JSONDict) -> "RegisterAgentResponse":
        return cls(
            agent_id=str(data.get("agent_id", "")),
            api_key=str(data.get("api_key", "")),
            status=str(data.get("status", "")),
            message=data.get("message"),
        )


@dataclass
class CreateProfileRequest:
    basics: Optional[JSONDict] = None
    lifestyle: Optional[JSONDict] = None
    personality: Optional[JSONDict] = None
    interests: Optional[JSONDict] = None
    dealbreakers: Optional[JSONDict] = None
    dealmakers: Optional[JSONDict] = None
    preferences: Optional[JSONDict] = None
    negotiation_prefs: Optional[JSONDict] = None

    def to_dict(self) -> JSONDict:
        payload: JSONDict = {}
        for field_name in (
            "basics",
            "lifestyle",
            "personality",
            "interests",
            "dealbreakers",
            "dealmakers",
            "preferences",
            "negotiation_prefs",
        ):
            value = getattr(self, field_name)
            if value is not None:
                payload[field_name] = value
        return payload


@dataclass
class CreateProfileResponse:
    profile_id: str
    status: Optional[str] = None

    @classmethod
    def from_dict(cls, data: JSONDict) -> "CreateProfileResponse":
        return cls(profile_id=str(data.get("profile_id", "")), status=data.get("status"))


@dataclass
class DiscoveryCandidate:
    profile_id: str
    tier: Optional[str] = None
    compatibility_score: Optional[float] = None
    score_breakdown: Optional[JSONDict] = None
    visible_attributes: Optional[JSONDict] = None

    @classmethod
    def from_dict(cls, data: JSONDict) -> "DiscoveryCandidate":
        return cls(
            profile_id=str(data.get("profile_id", "")),
            tier=data.get("tier"),
            compatibility_score=data.get("compatibility_score"),
            score_breakdown=data.get("score_breakdown"),
            visible_attributes=data.get("visible_attributes"),
        )


@dataclass
class DiscoveryResponse:
    candidates: List[DiscoveryCandidate] = field(default_factory=list)
    page: Optional[int] = None
    total_estimate: Optional[int] = None
    source: Optional[str] = None

    @classmethod
    def from_dict(cls, data: JSONDict) -> "DiscoveryResponse":
        return cls(
            candidates=[DiscoveryCandidate.from_dict(item) for item in data.get("candidates", [])],
            page=data.get("page"),
            total_estimate=data.get("total_estimate"),
            source=data.get("source"),
        )


@dataclass
class StateTransition:
    from_state: str
    to_state: str
    actor: str
    reason: Optional[str] = None
    timestamp: Optional[str] = None

    @classmethod
    def from_dict(cls, data: JSONDict) -> "StateTransition":
        return cls(
            from_state=str(data.get("from", "")),
            to_state=str(data.get("to", "")),
            actor=str(data.get("actor", "")),
            reason=data.get("reason"),
            timestamp=data.get("timestamp"),
        )


@dataclass
class Negotiation:
    id: str
    initiator_opaque_id: str
    target_opaque_id: str
    state: str
    state_history: List[StateTransition] = field(default_factory=list)
    disclosure_level: Optional[str] = None
    pending_human_approval: Optional[bool] = None
    human_approval_required_for: Optional[str] = None
    human_approval_expires_at: Optional[str] = None
    meeting_proposed_by: Optional[str] = None
    activated_by: List[str] = field(default_factory=list)
    created_at: Optional[str] = None
    updated_at: Optional[str] = None

    @classmethod
    def from_dict(cls, data: JSONDict) -> "Negotiation":
        return cls(
            id=str(data.get("id", "")),
            initiator_opaque_id=str(data.get("initiator_opaque_id", "")),
            target_opaque_id=str(data.get("target_opaque_id", "")),
            state=str(data.get("state", "")),
            state_history=[StateTransition.from_dict(item) for item in data.get("state_history", [])],
            disclosure_level=data.get("disclosure_level"),
            pending_human_approval=data.get("pending_human_approval"),
            human_approval_required_for=data.get("human_approval_required_for"),
            human_approval_expires_at=data.get("human_approval_expires_at"),
            meeting_proposed_by=data.get("meeting_proposed_by"),
            activated_by=list(data.get("activated_by", [])),
            created_at=data.get("created_at"),
            updated_at=data.get("updated_at"),
        )


@dataclass
class CreateNegotiationRequest:
    target_opaque_id: str
    initial_message: Optional[str] = None

    def to_dict(self) -> JSONDict:
        payload: JSONDict = {"target_opaque_id": self.target_opaque_id}
        if self.initial_message is not None:
            payload["initial_message"] = self.initial_message
        return payload


@dataclass
class ApprovalStatus:
    negotiation_id: str
    pending: bool
    required_for: Optional[str] = None
    requested_at: Optional[str] = None
    expires_at: Optional[str] = None
    expired: bool = False
    current_state: str = ""

    @classmethod
    def from_dict(cls, data: JSONDict) -> "ApprovalStatus":
        return cls(
            negotiation_id=str(data.get("negotiation_id", "")),
            pending=bool(data.get("pending", False)),
            required_for=data.get("required_for"),
            requested_at=data.get("requested_at"),
            expires_at=data.get("expires_at"),
            expired=bool(data.get("expired", False)),
            current_state=str(data.get("current_state", "")),
        )


@dataclass
class ApprovalDecisionResponse:
    approved: Optional[bool] = None
    rejected: Optional[bool] = None
    negotiation: Optional[Negotiation] = None
    transition: Optional[str] = None
    message: Optional[str] = None

    @classmethod
    def from_dict(cls, data: JSONDict) -> "ApprovalDecisionResponse":
        negotiation = data.get("negotiation")
        return cls(
            approved=data.get("approved"),
            rejected=data.get("rejected"),
            negotiation=Negotiation.from_dict(negotiation) if isinstance(negotiation, dict) else None,
            transition=data.get("transition"),
            message=data.get("message"),
        )
