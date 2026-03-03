package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonInclude(JsonInclude.Include.NON_NULL)
public record ApproveNegotiationRequest(
    @JsonProperty("notes") String notes,
    @JsonProperty("human_token") String humanToken) {}
