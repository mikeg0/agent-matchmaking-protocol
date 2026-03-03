package com.bonsai.amp.sdk;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.bonsai.amp.sdk.model.ApprovalStatus;
import com.bonsai.amp.sdk.model.CreateNegotiationRequest;
import com.bonsai.amp.sdk.model.CreateProfileRequest;
import com.bonsai.amp.sdk.model.DiscoveryResponse;
import com.bonsai.amp.sdk.model.RegisterAgentRequest;
import com.bonsai.amp.sdk.model.RegisterAgentResponse;
import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import java.io.IOException;
import java.nio.file.Path;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;

class ConformanceFixturesTest {
  private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper().registerModule(new JavaTimeModule());
  private static final List<String> PROGRESSION =
      List.of("DISCOVERY", "INTEREST", "MUTUAL", "DISCLOSING", "DISCLOSED", "MEETING", "ACTIVE");
  private static final Set<String> TERMINAL =
      Set.of("WITHDRAWN", "EXPIRED", "REJECTED", "BLOCKED", "SAFETY_HOLD");

  @Test
  void requestFixturesRoundTrip() throws Exception {
    assertRoundTrip("http/register_agent_request.json", RegisterAgentRequest.class);
    assertRoundTrip("http/create_profile_request.json", CreateProfileRequest.class);
    assertRoundTrip("http/create_negotiation_request.json", CreateNegotiationRequest.class);
  }

  @Test
  void responseFixturesDeserialize() throws Exception {
    RegisterAgentResponse registerAgent =
        readFixtureValue("http/register_agent_response.json", RegisterAgentResponse.class);
    assertEquals("11111111-1111-1111-1111-111111111111", registerAgent.agentId());
    assertEquals("pending_human_verify", registerAgent.status());

    DiscoveryResponse discovery = readFixtureValue("http/discovery_response.json", DiscoveryResponse.class);
    assertEquals(1, discovery.candidates().size());
    assertEquals("trusted", discovery.candidates().get(0).tier());

    ApprovalStatus approval = readFixtureValue("http/approval_status_response.json", ApprovalStatus.class);
    assertTrue(approval.pending());
    assertEquals("DISCLOSED", approval.currentState());
  }

  @Test
  void transitionFixturesMatchCanonicalRules() throws Exception {
    TransitionFixture fixture = readFixtureValue("state/transition_cases.json", TransitionFixture.class);
    for (TransitionCase transitionCase : fixture.cases()) {
      boolean actual =
          validateTransition(
              transitionCase.from(),
              transitionCase.to(),
              transitionCase.pendingHumanApproval(),
              transitionCase.actorIsParticipant());
      assertEquals(actual, transitionCase.valid(), transitionCase.name());
    }
  }

  private static boolean validateTransition(
      String current, String target, boolean pendingHumanApproval, boolean actorIsParticipant) {
    if (TERMINAL.contains(current)) {
      return false;
    }

    if ("WITHDRAWN".equals(target)) {
      return actorIsParticipant;
    }

    if (Set.of("EXPIRED", "REJECTED", "BLOCKED", "SAFETY_HOLD").contains(target)) {
      return false;
    }

    String expected = nextState(current);
    if (!target.equals(expected)) {
      return false;
    }

    if ((("MUTUAL".equals(current) && "DISCLOSING".equals(target))
            || ("DISCLOSED".equals(current) && "MEETING".equals(target)))
        && !pendingHumanApproval) {
      return false;
    }

    return true;
  }

  private static String nextState(String current) {
    int idx = PROGRESSION.indexOf(current);
    if (idx < 0 || idx == PROGRESSION.size() - 1) {
      return null;
    }
    return PROGRESSION.get(idx + 1);
  }

  private static <T> void assertRoundTrip(String fixture, Class<T> modelType) throws Exception {
    JsonNode expected = readFixtureTree(fixture);
    T model = OBJECT_MAPPER.treeToValue(expected, modelType);
    JsonNode actual = OBJECT_MAPPER.valueToTree(model);
    assertEquals(expected, actual);
  }

  private static JsonNode readFixtureTree(String fixture) throws IOException {
    return OBJECT_MAPPER.readTree(fixturePath(fixture).toFile());
  }

  private static <T> T readFixtureValue(String fixture, Class<T> modelType) throws IOException {
    return OBJECT_MAPPER.readValue(fixturePath(fixture).toFile(), modelType);
  }

  private static Path fixturePath(String fixture) {
    return Path.of("..", "spec", "fixtures", fixture);
  }

  private record TransitionFixture(@JsonProperty("cases") List<TransitionCase> cases) {}

  private record TransitionCase(
      @JsonProperty("name") String name,
      @JsonProperty("from") String from,
      @JsonProperty("to") String to,
      @JsonProperty("pending_human_approval") boolean pendingHumanApproval,
      @JsonProperty("actor_is_participant") boolean actorIsParticipant,
      @JsonProperty("valid") boolean valid) {}
}
