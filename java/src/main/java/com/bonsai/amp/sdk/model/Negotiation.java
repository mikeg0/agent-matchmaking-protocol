package com.bonsai.amp.sdk.model;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;

@JsonIgnoreProperties(ignoreUnknown = true)
public record Negotiation(
    @JsonProperty("id") String id,
    @JsonProperty("initiator_opaque_id") String initiatorOpaqueId,
    @JsonProperty("target_opaque_id") String targetOpaqueId,
    @JsonProperty("state") String state,
    @JsonProperty("state_history") List<StateTransition> stateHistory,
    @JsonProperty("disclosure_level") String disclosureLevel,
    @JsonProperty("pending_human_approval") Boolean pendingHumanApproval,
    @JsonProperty("human_approval_required_for") String humanApprovalRequiredFor,
    @JsonProperty("human_approval_expires_at") String humanApprovalExpiresAt,
    @JsonProperty("meeting_proposed_by") String meetingProposedBy,
    @JsonProperty("activated_by") List<String> activatedBy,
    @JsonProperty("created_at") String createdAt,
    @JsonProperty("updated_at") String updatedAt) {}
