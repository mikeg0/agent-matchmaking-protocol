import json
import unittest
from pathlib import Path

from amp_sdk import (
    ApprovalStatus,
    CreateNegotiationRequest,
    CreateProfileRequest,
    CreateProfileResponse,
    DiscoveryResponse,
    RegisterAgentRequest,
    RegisterAgentResponse,
)

FIXTURES_DIR = Path(__file__).resolve().parents[2] / "spec" / "fixtures"

PROGRESSION = [
    "DISCOVERY",
    "INTEREST",
    "MUTUAL",
    "DISCLOSING",
    "DISCLOSED",
    "MEETING",
    "ACTIVE",
]
TERMINAL = {"WITHDRAWN", "EXPIRED", "REJECTED", "BLOCKED", "SAFETY_HOLD"}
HUMAN_GATED = {("MUTUAL", "DISCLOSING"), ("DISCLOSED", "MEETING")}


def load_fixture(*parts: str) -> dict:
    path = FIXTURES_DIR.joinpath(*parts)
    return json.loads(path.read_text(encoding="utf-8"))


def validate_transition(case: dict) -> bool:
    current = case["from"]
    target = case["to"]
    pending_human_approval = bool(case["pending_human_approval"])
    actor_is_participant = bool(case["actor_is_participant"])

    if current in TERMINAL:
        return False

    if target == "WITHDRAWN":
        return actor_is_participant

    if target in {"EXPIRED", "REJECTED", "BLOCKED", "SAFETY_HOLD"}:
        return False

    expected = None
    if current in PROGRESSION:
        idx = PROGRESSION.index(current)
        if idx < len(PROGRESSION) - 1:
            expected = PROGRESSION[idx + 1]

    if target != expected:
        return False

    if (current, target) in HUMAN_GATED and not pending_human_approval:
        return False

    return True


class ConformanceFixtureTests(unittest.TestCase):
    def test_request_model_round_trip_matches_http_fixtures(self):
        register_agent = load_fixture("http", "register_agent_request.json")
        self.assertEqual(RegisterAgentRequest(**register_agent).to_dict(), register_agent)

        create_profile = load_fixture("http", "create_profile_request.json")
        self.assertEqual(CreateProfileRequest(**create_profile).to_dict(), create_profile)

        create_negotiation = load_fixture("http", "create_negotiation_request.json")
        self.assertEqual(CreateNegotiationRequest(**create_negotiation).to_dict(), create_negotiation)

    def test_response_model_deserialization_matches_http_fixtures(self):
        register_agent = RegisterAgentResponse.from_dict(load_fixture("http", "register_agent_response.json"))
        self.assertEqual(register_agent.agent_id, "11111111-1111-1111-1111-111111111111")
        self.assertEqual(register_agent.status, "pending_human_verify")

        create_profile = CreateProfileResponse.from_dict(load_fixture("http", "create_profile_response.json"))
        self.assertEqual(create_profile.profile_id, "22222222-2222-2222-2222-222222222222")

        discovery = DiscoveryResponse.from_dict(load_fixture("http", "discovery_response.json"))
        self.assertEqual(len(discovery.candidates), 1)
        self.assertEqual(discovery.candidates[0].tier, "trusted")

        approval_status = ApprovalStatus.from_dict(load_fixture("http", "approval_status_response.json"))
        self.assertTrue(approval_status.pending)
        self.assertEqual(approval_status.current_state, "DISCLOSED")

    def test_state_transition_cases_match_reference_rules(self):
        fixtures = load_fixture("state", "transition_cases.json")
        for case in fixtures["cases"]:
            with self.subTest(name=case["name"]):
                self.assertEqual(validate_transition(case), case["valid"])


if __name__ == "__main__":
    unittest.main()
