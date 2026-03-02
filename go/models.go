package ampsdk

// JSONMap is an unstructured JSON object.
type JSONMap map[string]any

type RegisterAgentRequest struct {
	Name          string   `json:"name"`
	AgentPlatform string   `json:"agent_platform,omitempty"`
	AgentVersion  string   `json:"agent_version,omitempty"`
	Capabilities  []string `json:"capabilities,omitempty"`
	WebhookURL    string   `json:"webhook_url,omitempty"`
	WebhookSecret string   `json:"webhook_secret,omitempty"`
}

type RegisterAgentResponse struct {
	AgentID string `json:"agent_id"`
	APIKey  string `json:"api_key"`
	Status  string `json:"status"`
	Message string `json:"message,omitempty"`
}

type CreateProfileRequest struct {
	Basics           JSONMap `json:"basics,omitempty"`
	Lifestyle        JSONMap `json:"lifestyle,omitempty"`
	Personality      JSONMap `json:"personality,omitempty"`
	Interests        JSONMap `json:"interests,omitempty"`
	Dealbreakers     JSONMap `json:"dealbreakers,omitempty"`
	Dealmakers       JSONMap `json:"dealmakers,omitempty"`
	Preferences      JSONMap `json:"preferences,omitempty"`
	NegotiationPrefs JSONMap `json:"negotiation_prefs,omitempty"`
}

type CreateProfileResponse struct {
	ProfileID string `json:"profile_id"`
	Status    string `json:"status,omitempty"`
}

type DiscoveryCandidate struct {
	ProfileID          string  `json:"profile_id"`
	Tier               string  `json:"tier,omitempty"`
	CompatibilityScore float64 `json:"compatibility_score,omitempty"`
	ScoreBreakdown     JSONMap `json:"score_breakdown,omitempty"`
	VisibleAttributes  JSONMap `json:"visible_attributes,omitempty"`
}

type DiscoveryResponse struct {
	Candidates    []DiscoveryCandidate `json:"candidates"`
	Page          int                  `json:"page,omitempty"`
	TotalEstimate int                  `json:"total_estimate,omitempty"`
	Source        string               `json:"source,omitempty"`
}

type StateTransition struct {
	FromState string `json:"from"`
	ToState   string `json:"to"`
	Actor     string `json:"actor"`
	Reason    string `json:"reason,omitempty"`
	Timestamp string `json:"timestamp,omitempty"`
}

type Negotiation struct {
	ID                     string            `json:"id"`
	InitiatorOpaqueID      string            `json:"initiator_opaque_id"`
	TargetOpaqueID         string            `json:"target_opaque_id"`
	State                  string            `json:"state"`
	StateHistory           []StateTransition `json:"state_history,omitempty"`
	DisclosureLevel        string            `json:"disclosure_level,omitempty"`
	PendingHumanApproval   bool              `json:"pending_human_approval,omitempty"`
	HumanApprovalRequired  string            `json:"human_approval_required_for,omitempty"`
	HumanApprovalExpiresAt string            `json:"human_approval_expires_at,omitempty"`
	MeetingProposedBy      string            `json:"meeting_proposed_by,omitempty"`
	ActivatedBy            []string          `json:"activated_by,omitempty"`
	CreatedAt              string            `json:"created_at,omitempty"`
	UpdatedAt              string            `json:"updated_at,omitempty"`
}

type CreateNegotiationRequest struct {
	TargetOpaqueID string `json:"target_opaque_id"`
	InitialMessage string `json:"initial_message,omitempty"`
}

type ApprovalStatus struct {
	NegotiationID string `json:"negotiation_id"`
	Pending       bool   `json:"pending"`
	RequiredFor   string `json:"required_for,omitempty"`
	RequestedAt   string `json:"requested_at,omitempty"`
	ExpiresAt     string `json:"expires_at,omitempty"`
	Expired       bool   `json:"expired,omitempty"`
	CurrentState  string `json:"current_state,omitempty"`
}

type ApprovalDecisionResponse struct {
	Approved    *bool       `json:"approved,omitempty"`
	Rejected    *bool       `json:"rejected,omitempty"`
	Negotiation *Negotiation `json:"negotiation,omitempty"`
	Transition  string      `json:"transition,omitempty"`
	Message     string      `json:"message,omitempty"`
}

type ApproveNegotiationRequest struct {
	Notes      string `json:"notes,omitempty"`
	HumanToken string `json:"human_token,omitempty"`
}

type RejectNegotiationRequest struct {
	Notes string `json:"notes,omitempty"`
}

type HealthResponse struct {
	Status    string `json:"status"`
	Service   string `json:"service,omitempty"`
	Version   string `json:"version,omitempty"`
	Sandbox   bool   `json:"sandbox,omitempty"`
	Timestamp string `json:"timestamp,omitempty"`
}
