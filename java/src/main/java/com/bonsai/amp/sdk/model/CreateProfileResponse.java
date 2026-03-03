package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonIgnoreProperties(ignoreUnknown = true)
public record CreateProfileResponse(
    @JsonProperty("profile_id") String profileId,
    @JsonProperty("status") String status) {}
