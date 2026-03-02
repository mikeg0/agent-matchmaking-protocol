use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::state_machine::NegotiationState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub sandbox: Option<bool>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAgentRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_version: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAgentResponse {
    pub agent_id: Uuid,
    pub api_key: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinkHumanRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyHumanRequest {
    #[serde(rename = "type")]
    pub verification_type: VerificationType,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationType {
    Email,
    Phone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateKeyRequest {
    pub confirm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateKeyResponse {
    pub api_key: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub agent_id: Uuid,
    pub name: String,
    pub status: AgentStatusValue,
    pub verification_level: Option<u8>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub has_human_linked: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatusValue {
    Active,
    PendingHumanVerify,
    Suspended,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustBreakdown {
    pub verification_points: Option<i32>,
    pub account_age_points: Option<i32>,
    pub completion_points: Option<i32>,
    pub report_penalty: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScore {
    pub agent_id: Uuid,
    pub score: i32,
    pub tier: TrustTier,
    pub breakdown: Option<TrustBreakdown>,
    pub rate_limit_multiplier: Option<f64>,
    pub auto_approve_eligible: Option<bool>,
    pub computed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustTier {
    Basic,
    Standard,
    Trusted,
    Elite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustScoreEnvelope {
    pub trust: TrustScore,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basics: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifestyle: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub personality: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interests: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dealbreakers: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dealmakers: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferences: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negotiation_prefs: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProfileResponse {
    pub profile_id: Uuid,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResponse {
    #[serde(flatten)]
    pub profile: CreateProfileRequest,
    pub profile_id: Uuid,
    pub active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    #[serde(default)]
    pub candidates: Vec<DiscoveryCandidate>,
    pub page: Option<i64>,
    pub total_estimate: Option<i64>,
    pub source: Option<DiscoverySource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySource {
    Cache,
    Live,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryCandidate {
    pub profile_id: Uuid,
    pub tier: Option<String>,
    pub compatibility_score: Option<f64>,
    pub score_breakdown: Option<Value>,
    pub visible_attributes: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalRequest {
    pub signal: DiscoverySignal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoverySignal {
    Interest,
    Pass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalResponse {
    pub signal: Option<String>,
    pub mutual_interest: Option<bool>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNegotiationRequest {
    pub target_opaque_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationEnvelope {
    pub negotiation: Negotiation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationsEnvelope {
    #[serde(default)]
    pub negotiations: Vec<Negotiation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationDetailEnvelope {
    pub negotiation: Negotiation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disclosure_manifest: Option<DisclosureManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Negotiation {
    pub id: Uuid,
    pub initiator_opaque_id: Uuid,
    pub target_opaque_id: Uuid,
    pub state: NegotiationState,
    #[serde(default)]
    pub state_history: Vec<StateTransition>,
    pub disclosure_level: Option<String>,
    pub pending_human_approval: Option<bool>,
    pub human_approval_required_for: Option<String>,
    pub human_approval_expires_at: Option<DateTime<Utc>>,
    pub meeting_proposed_by: Option<String>,
    #[serde(default)]
    pub activated_by: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: NegotiationState,
    pub to: NegotiationState,
    pub actor: String,
    pub reason: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureManifest {
    pub tier: String,
    pub negotiation_id: Uuid,
    pub generated_at: DateTime<Utc>,
    #[serde(default)]
    pub fields_shared: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RespondNegotiationRequest {
    pub accept: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WithdrawRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HumanApproveRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub human_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosureResponse {
    pub negotiation: Negotiation,
    pub pending: Option<bool>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStatus {
    pub negotiation_id: Uuid,
    pub pending: bool,
    pub required_for: Option<String>,
    pub requested_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub expired: bool,
    pub current_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecisionResponse {
    pub approved: Option<bool>,
    pub rejected: Option<bool>,
    pub negotiation: Option<Negotiation>,
    pub transition: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<MessageType>,
    pub content: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    Text,
    DisclosureRequest,
    DisclosureResponse,
    MeetingProposal,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRecord {
    pub id: Option<Uuid>,
    pub negotiation_id: Option<Uuid>,
    pub sender_opaque_id: Option<String>,
    pub r#type: Option<MessageType>,
    pub content: Option<Value>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagesEnvelope {
    #[serde(default)]
    pub messages: Vec<MessageRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericResponse {
    #[serde(flatten)]
    pub data: BTreeMap<String, Value>,
}
