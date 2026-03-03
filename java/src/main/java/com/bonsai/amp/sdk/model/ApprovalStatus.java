package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonIgnoreProperties(ignoreUnknown = true)
public record ApprovalStatus(
    @JsonProperty("negotiation_id") String negotiationId,
    @JsonProperty("pending") boolean pending,
    @JsonProperty("required_for") String requiredFor,
    @JsonProperty("requested_at") String requestedAt,
    @JsonProperty("expires_at") String expiresAt,
    @JsonProperty("expired") Boolean expired,
    @JsonProperty("current_state") String currentState) {}
