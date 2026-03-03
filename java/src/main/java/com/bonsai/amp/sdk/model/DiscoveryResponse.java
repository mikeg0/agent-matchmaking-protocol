package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;

@JsonIgnoreProperties(ignoreUnknown = true)
public record DiscoveryResponse(
    @JsonProperty("candidates") List<DiscoveryCandidate> candidates,
    @JsonProperty("page") Integer page,
    @JsonProperty("total_estimate") Integer totalEstimate,
    @JsonProperty("source") String source) {}
