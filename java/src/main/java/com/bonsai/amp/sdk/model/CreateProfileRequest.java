package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.Map;

@JsonInclude(JsonInclude.Include.NON_NULL)
public record CreateProfileRequest(
    @JsonProperty("basics") Map<String, Object> basics,
    @JsonProperty("lifestyle") Map<String, Object> lifestyle,
    @JsonProperty("personality") Map<String, Object> personality,
    @JsonProperty("interests") Map<String, Object> interests,
    @JsonProperty("dealbreakers") Map<String, Object> dealbreakers,
    @JsonProperty("dealmakers") Map<String, Object> dealmakers,
    @JsonProperty("preferences") Map<String, Object> preferences,
    @JsonProperty("negotiation_prefs") Map<String, Object> negotiationPrefs) {}
