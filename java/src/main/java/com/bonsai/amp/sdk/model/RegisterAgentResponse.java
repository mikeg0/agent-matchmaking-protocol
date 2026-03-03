package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonIgnoreProperties(ignoreUnknown = true)
public record RegisterAgentResponse(
    @JsonProperty("agent_id") String agentId,
    @JsonProperty("api_key") String apiKey,
    @JsonProperty("status") String status,
    @JsonProperty("message") String message) {}
