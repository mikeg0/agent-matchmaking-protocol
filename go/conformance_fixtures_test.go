package ampsdk

import (
	"encoding/json"
	"os"
	"path/filepath"
	"reflect"
	"testing"
)

type transitionCase struct {
	Name                 string `json:"name"`
	From                 string `json:"from"`
	To                   string `json:"to"`
	PendingHumanApproval bool   `json:"pending_human_approval"`
	ActorIsParticipant   bool   `json:"actor_is_participant"`
	Valid                bool   `json:"valid"`
}

type transitionFixture struct {
	Cases []transitionCase `json:"cases"`
}

func fixturePath(parts ...string) string {
	base := []string{"..", "spec", "fixtures"}
	base = append(base, parts...)
	return filepath.Join(base...)
}

func readJSONFixture(t *testing.T, parts ...string) []byte {
	t.Helper()
	payload, err := os.ReadFile(fixturePath(parts...))
	if err != nil {
		t.Fatalf("read fixture %v: %v", parts, err)
	}
	return payload
}

func assertJSONRoundTripEqual(t *testing.T, fixture []byte, model any) {
	t.Helper()
	actualBytes, err := json.Marshal(model)
	if err != nil {
		t.Fatalf("marshal model: %v", err)
	}

	var expectedJSON any
	if err := json.Unmarshal(fixture, &expectedJSON); err != nil {
		t.Fatalf("unmarshal expected json: %v", err)
	}

	var actualJSON any
	if err := json.Unmarshal(actualBytes, &actualJSON); err != nil {
		t.Fatalf("unmarshal actual json: %v", err)
	}

	if !reflect.DeepEqual(expectedJSON, actualJSON) {
		t.Fatalf("json mismatch\nexpected: %#v\nactual:   %#v", expectedJSON, actualJSON)
	}
}

func TestConformanceHTTPFixturesRoundTrip(t *testing.T) {
	t.Parallel()

	registerReqBytes := readJSONFixture(t, "http", "register_agent_request.json")
	var registerReq RegisterAgentRequest
	if err := json.Unmarshal(registerReqBytes, &registerReq); err != nil {
		t.Fatalf("unmarshal register request fixture: %v", err)
	}
	assertJSONRoundTripEqual(t, registerReqBytes, registerReq)

	createProfileReqBytes := readJSONFixture(t, "http", "create_profile_request.json")
	var createProfileReq CreateProfileRequest
	if err := json.Unmarshal(createProfileReqBytes, &createProfileReq); err != nil {
		t.Fatalf("unmarshal create profile request fixture: %v", err)
	}
	assertJSONRoundTripEqual(t, createProfileReqBytes, createProfileReq)

	createNegotiationReqBytes := readJSONFixture(t, "http", "create_negotiation_request.json")
	var createNegotiationReq CreateNegotiationRequest
	if err := json.Unmarshal(createNegotiationReqBytes, &createNegotiationReq); err != nil {
		t.Fatalf("unmarshal create negotiation request fixture: %v", err)
	}
	assertJSONRoundTripEqual(t, createNegotiationReqBytes, createNegotiationReq)
}

func TestConformanceHTTPFixturesResponseDeserialization(t *testing.T) {
	t.Parallel()

	registerRespBytes := readJSONFixture(t, "http", "register_agent_response.json")
	var registerResp RegisterAgentResponse
	if err := json.Unmarshal(registerRespBytes, &registerResp); err != nil {
		t.Fatalf("unmarshal register response fixture: %v", err)
	}
	if registerResp.AgentID != "11111111-1111-1111-1111-111111111111" {
		t.Fatalf("unexpected agent id: %s", registerResp.AgentID)
	}

	discoveryRespBytes := readJSONFixture(t, "http", "discovery_response.json")
	var discoveryResp DiscoveryResponse
	if err := json.Unmarshal(discoveryRespBytes, &discoveryResp); err != nil {
		t.Fatalf("unmarshal discovery response fixture: %v", err)
	}
	if len(discoveryResp.Candidates) != 1 || discoveryResp.Candidates[0].Tier != "trusted" {
		t.Fatalf("unexpected discovery fixture parse: %+v", discoveryResp)
	}

	approvalStatusBytes := readJSONFixture(t, "http", "approval_status_response.json")
	var approvalStatus ApprovalStatus
	if err := json.Unmarshal(approvalStatusBytes, &approvalStatus); err != nil {
		t.Fatalf("unmarshal approval status fixture: %v", err)
	}
	if !approvalStatus.Pending || approvalStatus.CurrentState != "DISCLOSED" {
		t.Fatalf("unexpected approval status fixture parse: %+v", approvalStatus)
	}
}

func TestConformanceStateTransitionFixtures(t *testing.T) {
	t.Parallel()

	payload := readJSONFixture(t, "state", "transition_cases.json")
	var fixture transitionFixture
	if err := json.Unmarshal(payload, &fixture); err != nil {
		t.Fatalf("unmarshal transition fixture: %v", err)
	}

	for _, c := range fixture.Cases {
		c := c
		t.Run(c.Name, func(t *testing.T) {
			t.Parallel()
			if got := validateTransition(c); got != c.Valid {
				t.Fatalf("expected %v, got %v", c.Valid, got)
			}
		})
	}
}

func validateTransition(c transitionCase) bool {
	terminal := map[string]bool{
		"WITHDRAWN":   true,
		"EXPIRED":     true,
		"REJECTED":    true,
		"BLOCKED":     true,
		"SAFETY_HOLD": true,
	}
	progression := []string{"DISCOVERY", "INTEREST", "MUTUAL", "DISCLOSING", "DISCLOSED", "MEETING", "ACTIVE"}
	humanGated := map[string]bool{
		"MUTUAL->DISCLOSING":  true,
		"DISCLOSED->MEETING": true,
	}

	if terminal[c.From] {
		return false
	}

	if c.To == "WITHDRAWN" {
		return c.ActorIsParticipant
	}

	if terminal[c.To] {
		return false
	}

	expected := ""
	for idx, state := range progression {
		if state == c.From {
			if idx+1 < len(progression) {
				expected = progression[idx+1]
			}
			break
		}
	}
	if c.To != expected {
		return false
	}

	if humanGated[c.From+"->"+c.To] && !c.PendingHumanApproval {
		return false
	}

	return true
}
