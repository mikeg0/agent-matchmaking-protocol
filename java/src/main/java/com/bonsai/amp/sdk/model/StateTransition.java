package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonIgnoreProperties(ignoreUnknown = true)
public record StateTransition(
    @JsonProperty("from") String fromState,
    @JsonProperty("to") String toState,
    @JsonProperty("actor") String actor,
    @JsonProperty("reason") String reason,
    @JsonProperty("timestamp") String timestamp) {}
