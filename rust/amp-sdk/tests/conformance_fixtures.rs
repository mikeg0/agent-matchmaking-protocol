use std::fs;
use std::path::PathBuf;

use amp_sdk::models::{
    ApprovalStatus, CreateNegotiationRequest, CreateProfileRequest, DiscoveryResponse,
    RegisterAgentRequest, RegisterAgentResponse,
};
use amp_sdk::{NegotiationMachine, NegotiationState};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct TransitionFixtures {
    cases: Vec<TransitionCase>,
}

#[derive(Debug, Deserialize)]
struct TransitionCase {
    name: String,
    from: String,
    to: String,
    pending_human_approval: bool,
    actor_is_participant: bool,
    valid: bool,
}

fn fixture_path(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..");
    path.push("..");
    path.push("spec");
    path.push("fixtures");
    for part in parts {
        path.push(part);
    }
    path
}

fn read_fixture(parts: &[&str]) -> String {
    fs::read_to_string(fixture_path(parts)).expect("fixture should be readable")
}

#[test]
fn conformance_http_request_fixtures_round_trip() {
    let register_json = read_fixture(&["http", "register_agent_request.json"]);
    let register_model: RegisterAgentRequest =
        serde_json::from_str(&register_json).expect("register fixture should deserialize");
    let expected: Value = serde_json::from_str(&register_json).expect("expected json to parse");
    let actual = serde_json::to_value(register_model).expect("register request should serialize");
    assert_eq!(actual, expected);

    let create_profile_json = read_fixture(&["http", "create_profile_request.json"]);
    let create_profile_model: CreateProfileRequest =
        serde_json::from_str(&create_profile_json).expect("create profile fixture should deserialize");
    let expected: Value = serde_json::from_str(&create_profile_json).expect("expected json to parse");
    let actual =
        serde_json::to_value(create_profile_model).expect("create profile request should serialize");
    assert_eq!(actual, expected);

    let create_negotiation_json = read_fixture(&["http", "create_negotiation_request.json"]);
    let create_negotiation_model: CreateNegotiationRequest = serde_json::from_str(&create_negotiation_json)
        .expect("create negotiation fixture should deserialize");
    let expected: Value = serde_json::from_str(&create_negotiation_json).expect("expected json to parse");
    let actual = serde_json::to_value(create_negotiation_model)
        .expect("create negotiation request should serialize");
    assert_eq!(actual, expected);
}

#[test]
fn conformance_http_response_fixtures_deserialize() {
    let register_json = read_fixture(&["http", "register_agent_response.json"]);
    let register_model: RegisterAgentResponse =
        serde_json::from_str(&register_json).expect("register response should deserialize");
    assert_eq!(register_model.agent_id.to_string(), "11111111-1111-1111-1111-111111111111");
    assert_eq!(register_model.status, "pending_human_verify");

    let discovery_json = read_fixture(&["http", "discovery_response.json"]);
    let discovery_model: DiscoveryResponse =
        serde_json::from_str(&discovery_json).expect("discovery response should deserialize");
    assert_eq!(discovery_model.candidates.len(), 1);
    assert_eq!(
        discovery_model.candidates[0].tier.as_deref(),
        Some("trusted")
    );

    let approval_json = read_fixture(&["http", "approval_status_response.json"]);
    let approval_model: ApprovalStatus =
        serde_json::from_str(&approval_json).expect("approval status response should deserialize");
    assert!(approval_model.pending);
    assert_eq!(approval_model.current_state, "DISCLOSED");
}

#[test]
fn conformance_state_transition_fixtures_match_state_machine() {
    let cases_json = read_fixture(&["state", "transition_cases.json"]);
    let fixture: TransitionFixtures =
        serde_json::from_str(&cases_json).expect("transition fixture should deserialize");

    for case in fixture.cases {
        let from = parse_state(&case.from);
        let to = parse_state(&case.to);
        let result = NegotiationMachine::validate_transition(
            from,
            to,
            case.pending_human_approval,
            case.actor_is_participant,
        )
        .is_ok();
        assert_eq!(result, case.valid, "case failed: {}", case.name);
    }
}

fn parse_state(state: &str) -> NegotiationState {
    match state {
        "DISCOVERY" => NegotiationState::Discovery,
        "INTEREST" => NegotiationState::Interest,
        "MUTUAL" => NegotiationState::Mutual,
        "DISCLOSING" => NegotiationState::Disclosing,
        "DISCLOSED" => NegotiationState::Disclosed,
        "MEETING" => NegotiationState::Meeting,
        "ACTIVE" => NegotiationState::Active,
        "WITHDRAWN" => NegotiationState::Withdrawn,
        "EXPIRED" => NegotiationState::Expired,
        "REJECTED" => NegotiationState::Rejected,
        "BLOCKED" => NegotiationState::Blocked,
        "SAFETY_HOLD" => NegotiationState::SafetyHold,
        other => panic!("unsupported state in fixture: {other}"),
    }
}
