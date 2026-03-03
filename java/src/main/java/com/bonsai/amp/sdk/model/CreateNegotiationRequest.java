package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonInclude(JsonInclude.Include.NON_NULL)
public record CreateNegotiationRequest(
    @JsonProperty("target_opaque_id") String targetOpaqueId,
    @JsonProperty("initial_message") String initialMessage) {}
