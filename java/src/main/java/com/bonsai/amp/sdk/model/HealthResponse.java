package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonIgnoreProperties(ignoreUnknown = true)
public record HealthResponse(
    @JsonProperty("status") String status,
    @JsonProperty("service") String service,
    @JsonProperty("version") String version,
    @JsonProperty("sandbox") Boolean sandbox,
    @JsonProperty("timestamp") String timestamp) {}
