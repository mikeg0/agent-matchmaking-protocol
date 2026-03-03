package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonInclude(JsonInclude.Include.NON_NULL)
public record RejectNegotiationRequest(@JsonProperty("notes") String notes) {}
