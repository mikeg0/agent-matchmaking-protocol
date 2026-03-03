package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonIgnoreProperties(ignoreUnknown = true)
public record ApprovalDecisionResponse(
    @JsonProperty("approved") Boolean approved,
    @JsonProperty("rejected") Boolean rejected,
    @JsonProperty("negotiation") Negotiation negotiation,
    @JsonProperty("transition") String transition,
    @JsonProperty("message") String message) {}
