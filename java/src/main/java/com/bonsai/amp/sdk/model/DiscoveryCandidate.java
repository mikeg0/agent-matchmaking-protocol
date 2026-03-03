package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.Map;

@JsonIgnoreProperties(ignoreUnknown = true)
public record DiscoveryCandidate(
    @JsonProperty("profile_id") String profileId,
    @JsonProperty("tier") String tier,
    @JsonProperty("compatibility_score") Double compatibilityScore,
    @JsonProperty("score_breakdown") Map<String, Object> scoreBreakdown,
    @JsonProperty("visible_attributes") Map<String, Object> visibleAttributes) {}
