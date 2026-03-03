package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;

@JsonInclude(JsonInclude.Include.NON_NULL)
public record RegisterAgentRequest(
    @JsonProperty("name") String name,
    @JsonProperty("agent_platform") String agentPlatform,
    @JsonProperty("agent_version") String agentVersion,
    @JsonProperty("capabilities") List<String> capabilities,
    @JsonProperty("webhook_url") String webhookUrl,
    @JsonProperty("webhook_secret") String webhookSecret) {}
