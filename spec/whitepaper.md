# Love Envoy: A Domain-Specific Protocol and Trust Infrastructure for Agent-Mediated Romantic Matching

**Michael Gorham\(^1\), Astra\(^1\)**  
\(^1\) **Bons_AI, LLC**, Salt Lake City, Utah, USA  
Contact: research@bons-ai.com  

**Tagline:** *Love Envoy — AI agents find love so you don't have to*

---

## Abstract

Consumer dating software has reached global scale but faces persistent user dissatisfaction, including documented burnout, low trust, and weak alignment between platform incentives and user outcomes [1,2,3]. At the same time, personal AI agents have transitioned from experimental assistants to practical systems that increasingly execute digital work on behalf of individuals [4,5,6]. This creates a structural gap: no open, domain-specific protocol currently exists for AI agents to negotiate romantic compatibility with privacy, consent, and safety guarantees. Existing agent standards are mostly transport- and tooling-level primitives (e.g., A2A, MCP, ACP), not matchmaking semantics [7,8,9].

We introduce **Love Envoy**, an agent-first infrastructure layer for romantic matching, and **AMP/1.0 (Agent Matchmaking Protocol)**, a domain-specific protocol implemented over plain REST+JSON with optional adapters for A2A and MCP. AMP defines progressive disclosure, consent-gated negotiation states, trust scoring, identity protection, and auditable transitions. Relative to prior drafts of this work, AMP/1.0 incorporates critical design revisions from independent review: expanded terminal states (`EXPIRED`, `REJECTED`, `BLOCKED`, `SAFETY_HOLD`), event-triggered opaque ID rotation, hybrid authentication (HMAC service-to-service plus OAuth2/OIDC for delegated consent), replay protection, explainable trust scoring with appeal/recovery pathways, anti-sybil weighting, additional human approval gates for high-risk transitions, and machine-readable per-field disclosure policies.

We situate Love Envoy in the 2025–2026 protocol landscape (MCP and A2A as center of gravity; ACP converging toward A2A; ANP in earlier formation), and in the emerging AI dating market where funded startups are focused on consumer applications rather than infrastructure. We argue the near-term go-to-market opportunity is trust-and-safety-first B2B2C deployment with matchmakers, coaching platforms, and niche communities. The paper contributes a protocol model, security architecture, and commercialization thesis for a category currently lacking open standards.

---

## 1. Introduction

Online dating is economically successful and experientially strained. Major platforms continue to generate substantial revenue, yet users report fatigue, frustration, and distrust of engagement-driven mechanics [1,2,3]. The dominant interaction loop—profile browsing, rapid binary selection, text initiation, and high abandonment—has persisted for more than a decade with incremental UX variation but little structural change.

In parallel, agentic AI has moved from concept to product. Leading model providers now offer tool-using systems capable of multistep action, memory, and delegated execution in common digital workflows [4,5,6]. A practical implication follows: as users increasingly delegate email, calendaring, travel planning, and research, they will expect delegation in social and relational domains as well.

Romantic matching is an especially important test case because it combines high personal stakes with difficult information asymmetry. Human users typically express preferences in compressed, noisy forms (short bios, sparse fields, reactive swipes). Agents can, in principle, negotiate compatibility from richer structured representations and longitudinal preference signals, but only if protocol infrastructure exists.

Today, that infrastructure is missing. There is no widely adopted open protocol specifying how two agents exchange compatibility disclosures, verify consent thresholds, enforce safety gates, and transition toward human introduction. General agent protocols partially solve discovery and communication transport but do not specify matchmaking semantics [7,8,9].

This paper presents Love Envoy and AMP/1.0 as a domain-specific response. The contribution is not another consumer dating application; it is protocol and trust infrastructure intended to be implemented by multiple products.

### 1.1 Problem Statement

Current systems expose three structural failures:

1. **Interaction inefficiency:** users must manually evaluate large candidate volumes under limited context.
2. **Privacy leakage:** personal data is often broadly visible early, before bilateral trust exists.
3. **Incentive misalignment:** engagement metrics can conflict with user goals of finding durable relationships.

### 1.2 Research Questions

This work investigates:

- What minimum protocol primitives are required for agent-mediated romantic negotiation?
- Can progressive disclosure and stateful consent reduce risk while preserving matching quality?
- How should domain-specific matchmaking protocols relate to general-purpose agent standards?
- What commercialization model is most viable in a market currently dominated by consumer app entrants?

### 1.3 Contributions

This paper contributes:

- A formalized domain-specific protocol model (**AMP/1.0**) atop REST+JSON.
- A revised negotiation state machine including explicit safety terminal states.
- A two-plane security architecture (service auth + delegated user consent).
- A privacy model combining opaque IDs, event-triggered rotation, and policy descriptors.
- A trust framework with explainability, anti-sybil weighting, and appeal pathways.
- A 2026 market analysis positioning infrastructure as a white space.

---

## 2. Related Work

### 2.1 Agent Communication and Tooling Protocols

The current protocol ecosystem is converging, but still heterogeneous:

- **MCP (Model Context Protocol)** is increasingly adopted for model-to-tool and model-to-data integration [8].
- **A2A (Agent2Agent)** has become a focal standard for agent-to-agent interoperability and capability exchange, with broad ecosystem support [7].
- **ACP** efforts in enterprise settings appear to be converging toward A2A-compatible patterns rather than maintaining fully separate long-term stacks [9].
- **ANP** and related decentralized agent network efforts remain promising but comparatively early-stage in deployment maturity [10].

Related initiatives are complementary rather than substitutive:

- **AG-UI** addresses agent↔frontend interaction contracts.
- **AGNTCY** explores discovery, identity, and messaging infrastructure across agent networks.

None of the above defines romantic-specific negotiation semantics such as mutual disclosure tiers, high-risk human gating, or compatibility treaty logic. AMP/1.0 is therefore positioned as a domain protocol that can run natively on REST+JSON and optionally bridge to A2A/MCP.

### 2.2 Online Dating Technology and AI Augmentation

Incumbent dating platforms increasingly deploy AI for profile assistance, ranking, and messaging support; however, public interfaces remain predominantly human-facing consumer products. Independent entrants (e.g., Keeper, Amata, Fate, Overtone, Ditto, Known) validate demand for AI-assisted dating but are concentrated on applications, not open infrastructure [11–16].

This creates a technical asymmetry: many products consume AI, while few publish interoperable negotiation protocols.

### 2.3 Trust, Safety, and Sensitive Data Regulation

Romantic matching involves sensitive inference and personal data handling. Regulatory precedent around dating data, deceptive design, and consent management suggests strong obligations for explicit consent, minimization, transparency, and incident readiness [17–20]. Existing protocol proposals in neighboring domains under-specify romantic-context safety gates, especially for transitions involving off-platform contact, travel, and financial requests.

---

## 3. Protocol Design (AMP/1.0)

### 3.1 Design Goals

AMP/1.0 is designed to:

1. Enable agent-to-agent compatibility negotiation without requiring any single vendor runtime.
2. Preserve user autonomy via explicit consent checkpoints and reversible transitions.
3. Minimize early disclosure by default and escalate only after bilateral signal.
4. Provide auditable, machine-verifiable state transitions.
5. Support implementation in plain HTTP environments (REST+JSON), with optional adapters.

### 3.2 Positioning Relative to A2A and MCP

AMP/1.0 is **not** a replacement for A2A or MCP.

- A2A/MCP: transport, capability exposure, tool interoperability.
- AMP/1.0: romantic negotiation semantics, disclosure policy, trust and safety rules.

Therefore:

- **Dependency model:** none required.
- **Adapter model:** available where ecosystems prefer A2A/MCP integration.

This avoids lock-in and lowers integration friction for teams with existing HTTP stacks.

### 3.3 Core Entities

- **Principal:** human represented by an agent.
- **Agent:** software acting with scoped authority.
- **Profile:** structured compatibility representation.
- **Negotiation:** stateful bilateral thread.
- **Disclosure policy:** machine-readable per-field release constraints.
- **Trust ledger:** explainable scoring artifacts and moderation outcomes.

### 3.4 Revised Negotiation State Machine

AMP/1.0 defines progression and terminal states:

**Primary progression**  
`DISCOVERY → MUTUAL_INTEREST → NEGOTIATING → DEEP_PROFILE → HUMAN_REVIEW → HUMAN_APPROVED → MEETING_PROPOSED → ACTIVE`

**Terminal / stop states**  
`WITHDRAWN`, `EXPIRED`, `REJECTED`, `BLOCKED`, `SAFETY_HOLD`

#### State semantics

- **DISCOVERY:** anonymized compatibility signals only.
- **MUTUAL_INTEREST:** bilateral interest confirmation; no direct identity.
- **NEGOTIATING:** structured compatibility inquiry/response.
- **DEEP_PROFILE:** broader non-PII disclosure.
- **HUMAN_REVIEW:** each principal reviews generated rationale.
- **HUMAN_APPROVED:** bilateral human consent to advance.
- **MEETING_PROPOSED:** introduction coordination with safeguards.
- **ACTIVE:** handoff to direct human communication.

Terminal states provide explicit closure and abuse controls:

- **EXPIRED:** timeout due to inactivity/SLA breach.
- **REJECTED:** human or policy rejection.
- **BLOCKED:** one side blocks future interaction.
- **SAFETY_HOLD:** automatic or manual pause pending risk review.

### 3.5 High-Risk Approval Gates

Beyond baseline human review, AMP/1.0 introduces mandatory additional approvals for:

1. **Off-platform contact exchange** (outside platform channels).
2. **Travel-related proposals** (distance above configured threshold).
3. **Financial asks** (any transfer, gift, loan, or purchase request).

These actions are disabled unless explicitly approved by both principals and policy engine.

### 3.6 Machine-Readable Disclosure Policies

Every profile field includes a descriptor:

```json
{
  "field": "phone_number",
  "classification": "PII_HIGH",
  "default_tier": "MEETING_PROPOSED",
  "requires_human_approval": true,
  "risk_tags": ["off_platform_contact"],
  "retention_days": 30,
  "cross_border_transfer": "disallowed"
}
```

This improves deterministic enforcement and auditability across implementations.

### 3.7 Message Types

AMP/1.0 messages are structured and signed:

- `compatibility_inquiry`
- `compatibility_response`
- `tier_advance_request`
- `tier_advance_decision`
- `human_gate_request`
- `human_gate_decision`
- `withdrawal_notice`
- `safety_signal`

Free-form narrative content can be attached, but state changes require typed payloads.

---

## 4. Architecture

### 4.1 Reference Architecture Overview

Love Envoy reference implementation separates concerns into:

1. **Negotiation Plane:** profile matching, state transitions, non-PII computation.
2. **Identity/Consent Plane:** account identity, authorization grants, approval events.
3. **Safety Plane:** scoring, holds, moderation workflows, policy runtime.
4. **Audit Plane:** immutable event logs and verification artifacts.

### 4.2 Data Segmentation

A strict two-store model:

- **PII Store:** identity and direct contact data, encrypted and heavily restricted.
- **Matching Store:** compatibility vectors, preferences, and stateful negotiations.

Cross-store linkage uses opaque handles, not natural identifiers.

### 4.3 Opaque ID Rotation (Time + Event)

Earlier designs rotated identifiers on schedule only. AMP/1.0 adds event triggers.

**Time trigger:** periodic rotation (e.g., 30 days).  
**Event triggers:**

- state transition into/out of sensitive stages,
- block actions,
- safety incidents,
- suspected enumeration activity,
- credential compromise or key rollover.

This reduces longitudinal linkage risk.

### 4.4 Compatibility Computation

The scoring pipeline remains multi-pass:

1. **Hard constraints:** non-negotiable incompatibilities.
2. **Soft alignment:** weighted values, goals, lifestyle, and interest semantics.
3. **Negotiation confidence:** quality and consistency of responses over time.

A key rule: hard incompatibility can zero soft score.

### 4.5 Interop Adapters

- **REST-native mode:** default implementation path.
- **A2A adapter:** maps AMP negotiation intents to A2A messages.
- **MCP adapter:** exposes Love Envoy functions as MCP tools.

Adapters are optional deployment modules, not protocol prerequisites.

---

## 5. Security and Privacy

### 5.1 Authentication and Authorization Model

AMP/1.0 adopts layered auth:

1. **HMAC signatures** for service-to-service authenticity and integrity.
2. **OAuth2/OIDC** for delegated user consent and revocable grants.
3. **Replay protection** via nonce + timestamp windows + idempotency keys.

This separates machine trust from human permission.

### 5.2 Replay and Tampering Controls

Each write operation includes:

- timestamp,
- nonce,
- signed digest,
- idempotency key.

Servers reject stale or duplicate envelopes; logs retain signature verification results.

### 5.3 Disclosure Minimization

Default policy: release the minimum required field subset for current state. All fields are labeled by risk class (`PUBLIC`, `SENSITIVE`, `PII_LOW`, `PII_HIGH`, `SAFETY_CRITICAL`) and enforced by policy runtime.

### 5.4 Trust Scoring with Explainability

Trust systems are often opaque and brittle; AMP/1.0 requires explainable outputs.

Example artifact:

```json
{
  "trust_score": 74,
  "contributors": [
    {"factor": "verification_level", "impact": +12},
    {"factor": "response_consistency", "impact": +7},
    {"factor": "report_density", "impact": -10},
    {"factor": "network_sybil_risk", "impact": -8}
  ],
  "actions": ["increased_human_gating"],
  "appeal_available": true
}
```

### 5.5 Appeal and Recovery

A scored party can:

- request explanation,
- submit correction evidence,
- trigger manual review,
- progress through staged recovery after confirmed remediation.

This prevents permanent silent exclusion from false positives.

### 5.6 Anti-Sybil Weighting

The safety plane adds graph-aware and behavior-aware defenses:

- account provenance signals,
- velocity anomalies,
- repeated coordination signatures,
- trust propagation caps.

These weights influence ranking and gate strictness without exposing private model internals.

### 5.7 Safety Holds

`SAFETY_HOLD` is invoked by deterministic policy or moderation operator when risk indicators cross thresholds (e.g., coercion language, financial solicitation pattern, abnormal contact escalation). During hold:

- no forward state transitions,
- no additional sensitive disclosure,
- human principals notified with context,
- review SLA initiated.

### 5.8 Regulatory Alignment (Design-Level)

The architecture is designed for alignment with core privacy and consumer-protection principles:

- explicit consent and revocation,
- data minimization and segregation,
- explainability and contestability,
- logging for accountability,
- differentiated protection for high-risk actions.

This does not eliminate jurisdiction-specific legal work, but reduces retrofitting burden.

---

## 6. Market Analysis

### 6.1 Demand Signals

The dating market is large and stable, while user trust remains fragile [1–3]. AI adoption in personal productivity is accelerating [4–6], and consumer familiarity with AI-mediated recommendation behavior is growing.

### 6.2 Competitive Update (2026)

Notable AI dating entrants include:

- **Fate (London):** publicly framed as an agentic AI dating app, including major press visibility (Feb 2026).
- **Overtone:** launched Dec 2025 with voice+AI positioning and high-profile leadership linkage.
- **Ditto:** iMessage-native strategy and reported $9.2M seed backing.
- **Known:** voice AI dating workflow and reported $9.7M seed; strong date-conversion claims.
- **Grindr EDGE experiment:** reported $499/month AI tier testing, indicating premium willingness in some segments.

These examples validate market interest and monetization experimentation. However, they are predominantly consumer endpoints, not open infrastructure.

### 6.3 Infrastructure White Space

As of this writing, no leading startup has established an open protocol and API layer dedicated to agent-to-agent romantic matchmaking. The landscape suggests a clear gap: applications are proliferating faster than interoperability standards.

### 6.4 Strategic Implication

Novelty in “agentic dating app UX” is likely to commoditize quickly. Durable differentiation is more likely in trust/safety infrastructure, policy tooling, and protocol adoption.

### 6.5 Go-to-Market Recommendation

A staged B2B2C path is favored:

1. **B2B partners first:** matchmakers, coaching companies, niche communities.
2. **Trust and compliance as primary value proposition:** not “interop novelty.”
3. **Adapter compatibility:** optional A2A/MCP connectors for distribution.
4. **Consumer surfaces later:** after network quality and safety operations mature.

### 6.6 Pricing and Business Model Observations

Early evidence from AI dating products indicates heterogeneous willingness to pay, from low monthly subscriptions to premium assisted tiers. For infrastructure providers, usage-based API pricing plus safety/compliance modules is a plausible foundation, with enterprise contracts for regulated or brand-sensitive partners.

---

## 7. Discussion

### 7.1 Why Domain-Specific Protocols Matter

General protocols optimize interoperability breadth. Domains like romantic matching require narrower but deeper semantics: sensitive disclosure choreography, abuse-resistant escalation, and bilateral autonomy preservation.

AMP/1.0 proposes that these should be first-class protocol constructs, not ad hoc app logic.

### 7.2 Human Agency and Delegation Boundaries

Agent mediation should reduce low-value cognitive load, not remove human intent. The multi-gate model is a practical compromise:

- agents handle filtering and structured negotiation,
- humans approve consequential transitions,
- high-risk contexts demand explicit bilateral acknowledgement.

### 7.3 Fairness and Interpretability Tradeoffs

Trust weighting and anti-sybil defenses can unintentionally penalize legitimate users with sparse history or nonstandard behavior. Explainability and appeal are not optional features—they are guardrails against automated exclusion.

### 7.4 Open Standard vs. Proprietary Product Tension

A domain protocol can be open while implementations compete on reliability, safety operations, tooling quality, and ecosystem support. The strategic challenge is balancing interoperability with sustainable business incentives.

### 7.5 Limitations

This paper is a design and strategy preprint, not a randomized efficacy trial. Current limitations include:

- limited public benchmark datasets for agent-mediated compatibility,
- evolving legal interpretations in sensitive-data AI contexts,
- potential cultural variability in consent and disclosure expectations,
- uncertain standardization pace across agent protocol ecosystems.

### 7.6 Future Work

1. Empirical studies on match quality outcomes vs. swipe-first baselines.
2. Cross-cultural calibration of disclosure policy defaults.
3. Formal verification of state-transition safety invariants.
4. Interoperability test suites for A2A/MCP adapters.
5. Auditable fairness metrics for trust scoring and moderation.

---

## 8. Conclusion

Agentic software is now capable of negotiating many categories of digital work, but romantic matching infrastructure remains under-standardized. Love Envoy and AMP/1.0 propose a domain-specific protocol architecture that prioritizes privacy-aware disclosure, explicit human control, explainable trust decisions, and operational safety.

The 2026 ecosystem suggests two simultaneous truths: (1) AI dating interest is real and rapidly commercializing; (2) open infrastructure for agent-to-agent matchmaking remains largely unbuilt. In this context, the strongest near-term strategy is trust-first B2B2C adoption with optional bridges to prevailing protocol ecosystems, rather than dependence on any single transport standard.

The central claim of this paper is pragmatic: for agent-mediated romantic matching to scale responsibly, protocol semantics must include consent, safety, and explainability as core primitives—not post hoc product features.

---

## References

[1] Pew Research Center, “The Virtues and Downsides of Online Dating,” 2020.  
[2] Pew Research Center, “Key findings about online dating in the U.S.,” 2023 update.  
[3] Forbes Health / OnePoll, U.S. online dating sentiment survey, 2024.  
[4] OpenAI, “Operator” product release materials, 2025.  
[5] Anthropic, model tool-use and MCP documentation, 2024–2026.  
[6] Apple, Apple Intelligence platform announcements, 2024–2026.  
[7] A2A Project (Linux Foundation ecosystem), Agent2Agent protocol materials, 2025–2026.  
[8] Model Context Protocol (MCP) specification and ecosystem docs, 2024–2026.  
[9] IBM Agent Communication Protocol (ACP) public materials and interoperability commentary, 2025–2026.  
[10] Agent Network Protocol (ANP) community/early-stage materials, 2025–2026.  
[11] Public reporting on Fate (London) as an agentic AI dating application, Feb 2026.  
[12] Public reporting on Overtone launch (voice+AI), Dec 2025 onward.  
[13] Public reporting on Ditto seed funding and iMessage-native strategy, 2025–2026.  
[14] Public reporting on Known seed funding and conversion metrics claims, 2025–2026.  
[15] Public reporting on Grindr EDGE premium AI-tier testing, 2026.  
[16] Public statements from matchmaking startup leaders regarding automation timelines (e.g., Q3 2026 forecasts).  
[17] Norwegian Data Protection Authority (Datatilsynet), Grindr enforcement decision, 2021.  
[18] EU GDPR (Regulation (EU) 2016/679), especially Articles 6, 7, 9, and 17.  
[19] EU AI Act timeline and compliance materials, 2024–2026.  
[20] U.S. FTC actions and guidance relevant to online dating, dark patterns, and deceptive AI representations, 2024–2026.

---

### Appendix A: AMP/1.0 Minimal Transition Rules (Normative Summary)

1. No state skipping is allowed.  
2. Any principal may withdraw at any non-terminal state.  
3. `HUMAN_REVIEW` and `HUMAN_APPROVED` require explicit bilateral human decisions.  
4. Off-platform contact, travel, and financial asks require additional high-risk gates.  
5. `SAFETY_HOLD` supersedes advancement permissions.  
6. All transitions must be signed, timestamped, and auditable.

### Appendix B: Sample Event-Triggered ID Rotation Policy

- Rotate on schedule every 30 days.  
- Rotate immediately on `BLOCKED`, credential compromise, suspicious enumeration, and safety incidents.  
- Rotate on transition into or out of `MEETING_PROPOSED` for non-linked observers.  
- Maintain bounded mapping windows for active negotiations only.

### Appendix C: Formal State Transition Constraints and Invariants

This appendix provides a more explicit, implementation-oriented treatment of AMP/1.0 state behavior. It is intended for protocol implementers who need deterministic validation at runtime.

#### C.1 Canonical state transition function

Let `S` be the set of states and `A` the set of actions. Define a deterministic transition function:

`T: S × A × P -> S`

where `P` is policy context (configuration, trust thresholds, risk flags, human approvals).

A transition request is valid iff all of the following hold:

1. The current state is non-terminal.
2. The action is allowed from the current state by protocol grammar.
3. Required signatures verify.
4. Required human approvals exist and are unexpired.
5. No blocking policy flag is active (e.g., `SAFETY_HOLD`).
6. Disclosure policy permits required field release.

#### C.2 Safety invariants

Implementers should enforce at least these invariants:

- **Invariant I1 (No skip):** A request to transition from `DISCOVERY` directly to `DEEP_PROFILE` is invalid.
- **Invariant I2 (Consent before identity):** No direct contact field can be emitted unless state `>= MEETING_PROPOSED` and high-risk gate approvals are present.
- **Invariant I3 (Terminal closure):** Once in `BLOCKED` or `REJECTED`, no forward transition is valid.
- **Invariant I4 (Hold supremacy):** If `SAFETY_HOLD` is active, all non-admin advancement actions fail.
- **Invariant I5 (Audit completeness):** Every state transition emits a signed audit event before response completion.

#### C.3 Time-bound logic

Negotiations are not perpetual; to reduce ghost threads and stale risk assumptions:

- inactivity thresholds trigger warning windows,
- warning expiration triggers `EXPIRED`,
- recovery from `EXPIRED` requires explicit re-initiation and policy check,
- expired threads do not auto-resume disclosure level.

#### C.4 Human decision windows

High-risk decisions require bounded windows (e.g., 24–72 hours configurable). If the window lapses:

- gate request status becomes `TIMED_OUT`,
- the negotiation can remain in-place or move to `EXPIRED` based on policy,
- no implicit approval is allowed.

#### C.5 Block semantics

`BLOCKED` is not equivalent to `WITHDRAWN`.

- `WITHDRAWN`: polite closure; optional future rematch allowed by policy.
- `BLOCKED`: hard preference and safety signal; future rediscovery requires explicit override policy, usually disallowed.

A block action should also trigger event-based ID rotation and visibility suppression across discovery caches.

### Appendix D: Disclosure Policy Descriptor Schema (Extended)

To support machine-readable, cross-implementation disclosure enforcement, AMP/1.0 uses per-field descriptors with tier, risk, and policy metadata.

#### D.1 Core descriptor

```json
{
  "field": "first_name",
  "version": "1.0",
  "classification": "PII_LOW",
  "default_tier": "HUMAN_APPROVED",
  "allowed_before_tier": false,
  "requires_dual_human_approval": true,
  "requires_safety_clearance": true,
  "retention_days": 30,
  "purpose": ["identity_confirmation"],
  "cross_border_transfer": "restricted",
  "audit_level": "strict"
}
```

#### D.2 Recommended classifications

- `PUBLIC`: non-sensitive compatibility metadata.
- `SENSITIVE`: preference or lifestyle information that may imply protected attributes.
- `PII_LOW`: low-risk identity hints (e.g., first name).
- `PII_HIGH`: direct identity/contact (phone, email, precise location).
- `SAFETY_CRITICAL`: fields whose misuse directly increases coercion/fraud risk.

#### D.3 Policy engine behavior

At read time, response builders evaluate descriptors as policy contracts:

1. Resolve current negotiation state.
2. Check descriptor minimum tier.
3. Check required approvals and expiry.
4. Check active risk tags.
5. Emit field or redact with reason code.

Reason-coded redactions improve debugging and compliance audits:

```json
{
  "field": "phone_number",
  "status": "redacted",
  "reason": "HIGH_RISK_GATE_PENDING"
}
```

#### D.4 Benefits

- deterministic behavior across teams and adapters,
- lower chance of accidental over-disclosure,
- auditable policy drift control through descriptor versioning,
- easier legal/compliance review.

### Appendix E: Threat Model and Attack Surface Analysis

This appendix sketches a pragmatic threat model for an internet-facing implementation.

#### E.1 Adversary classes

1. **Enumeration adversary:** attempts broad profile harvesting.
2. **Sybil adversary:** creates many synthetic accounts to manipulate trust/ranking.
3. **Romance scam adversary:** seeks contact/financial extraction.
4. **Credential adversary:** attempts token replay or key theft.
5. **Insider misuse adversary:** attempts unauthorized data access via internal privileges.

#### E.2 Key attack paths and mitigations

**Path A: ID harvesting via discovery loops**  
Mitigations: rotating opaque IDs, query budgets, anomaly detection, and event-triggered rotation.

**Path B: Replay of signed mutation requests**  
Mitigations: nonce tracking, strict timestamp windows, idempotency keys, and signature scope binding.

**Path C: Social engineering via premature contact requests**  
Mitigations: strict high-risk gates, automatic `SAFETY_HOLD` triggers for policy-violating language patterns.

**Path D: Trust gaming via collusive account clusters**  
Mitigations: anti-sybil weighting, graph concentration penalties, verification-level weighting.

**Path E: Excessive operator privilege**  
Mitigations: role separation, just-in-time privileged access, immutable audit logs, periodic access review.

#### E.3 Security logging requirements

Every sensitive action should emit an event with:

- actor identity (service, agent, moderator),
- cryptographic validation outcome,
- policy decision path,
- before/after state snapshots,
- correlation IDs for incident reconstruction.

#### E.4 Incident response posture

A mature deployment should define:

- severity matrix,
- containment steps,
- key revocation procedures,
- user notification flows,
- post-incident policy hardening loop.

### Appendix F: Trust Scoring Governance and Appeal Workflow

Trust scoring can improve safety but creates governance risk if treated as an opaque black box. AMP/1.0 recommends a governance model with minimum due-process guarantees.

#### F.1 Score component taxonomy

A score should separate dimensions rather than collapse all risk into one scalar:

- `identity_assurance_score`
- `behavioral_reliability_score`
- `community_risk_score`
- `policy_violation_score`

A composite score can be useful for ranking, but operational actions should reference underlying contributors.

#### F.2 Explainability requirements

When trust-driven gating changes user experience (e.g., additional approvals, reduced discovery visibility), the affected party should receive a concise explanation category:

- verification-related,
- behavior-related,
- network-integrity-related,
- moderation-related.

Fine-grained fraud signatures may remain confidential, but the reason class should be exposed.

#### F.3 Appeal path

Recommended workflow:

1. User requests review within defined window.
2. System returns non-sensitive rationale summary.
3. User submits correction/evidence.
4. Human or hybrid review renders decision with outcome code.
5. If sustained, staged recovery plan (e.g., temporary enhanced monitoring) applies.

#### F.4 Recovery states

Rather than permanent penalties, recovery ladders improve fairness and reduce lockout effects:

- `RECOVERY_STAGE_1`: enhanced verification required.
- `RECOVERY_STAGE_2`: monitored negotiation limits.
- `RECOVERY_COMPLETE`: normal policy profile restored.

#### F.5 Anti-sybil weighting guardrails

Sybil defenses should avoid amplifying socioeconomic or regional bias. Recommended controls:

- cap penalties from any single signal class,
- evaluate false-positive rates by cohort,
- publish periodic fairness diagnostics.

### Appendix G: Interoperability Notes for A2A and MCP Adapters

AMP/1.0 is intentionally transport-agnostic. This appendix describes adapter patterns for teams integrating into wider agent ecosystems.

#### G.1 A2A mapping model

A2A can carry AMP intents as typed payloads:

- discovery and capability exchange via A2A mechanisms,
- AMP negotiation payloads as domain messages,
- state decisions returned as AMP-conformant objects.

Important: A2A transport success is not equivalent to AMP policy success. Adapter layers must preserve AMP state and gate semantics.

#### G.2 MCP integration model

MCP adapters expose Love Envoy actions as tools:

- `search_candidates`
- `signal_interest`
- `request_tier_advance`
- `submit_human_gate_decision`

The MCP server should enforce scope constraints from OAuth grants and avoid exposing unauthorized mutation actions.

#### G.3 Adapter conformance tests

A conformance suite should test:

- no state skipping across adapters,
- descriptor-based redaction fidelity,
- replay rejection behavior,
- high-risk gate enforcement,
- consistent terminal-state behavior.

#### G.4 Versioning strategy

Recommend semantic versioning with compatibility profiles:

- `AMP/1.x`: backward-compatible schema additions.
- `AMP/2.0`: breaking state or policy semantics.
- adapter-specific compatibility matrix published with each release.

### Appendix H: Market and Deployment Scenarios (B2B2C Focus)

#### H.1 Why B2B2C first

Direct-to-consumer launches in dating are expensive due to network effects, trust bootstrapping, and safety operations overhead. A B2B2C strategy allows Love Envoy to:

- leverage existing high-intent communities,
- distribute integration cost,
- validate protocol semantics with controlled partner cohorts,
- harden trust/safety operations before broad public rollout.

#### H.2 Priority partner segments

1. **Professional matchmakers:** already operate high-friction, high-intent funnels.
2. **Coaching platforms:** can pair guidance products with structured matching tools.
3. **Niche communities:** concentrated values/goals improve early match quality.
4. **Selective consumer apps:** can integrate protocol infrastructure without full rebuild.

#### H.3 Deployment maturity phases

- **Phase 0 (Sandbox):** synthetic data and conformance tooling for developers.
- **Phase 1 (Pilot):** limited production with vetted partners and strict moderation.
- **Phase 2 (Scaled B2B2C):** expanded partner network, adapter ecosystem, advanced analytics.
- **Phase 3 (Broader ecosystem):** optional federation and wider consumer touchpoints.

#### H.4 Success metrics beyond engagement

Traditional dating metrics overvalue time-on-app. Infrastructure metrics should prioritize outcomes and safety:

- negotiation-to-human-review conversion,
- human-approval precision,
- safety-incident rate per 1,000 negotiations,
- appeal resolution time,
- policy-consistent disclosure rate,
- partner retention and SLA adherence.

#### H.5 Economic model hypotheses

Infrastructure monetization can combine:

- usage-based API pricing,
- verification/safety service modules,
- enterprise governance and compliance tooling,
- premium support and integration services.

This aligns revenue with dependable operations rather than user doom-scroll behavior.

### Appendix I: Research Ethics and Human-Centered Safeguards

Because this domain concerns intimacy and vulnerable populations, protocol design should include explicit ethical commitments.

#### I.1 Human primacy

Agents can recommend and negotiate, but humans retain authority over identity disclosure, contact exchange, and meeting commitments.

#### I.2 Transparency-by-default

Users should always know when actions are agent-originated, which gates are pending, and why a negotiation is paused or terminated.

#### I.3 Avoiding manipulative optimization

Objective functions should avoid maximizing interaction time for its own sake. Priority should be placed on safe, consent-consistent progress toward meaningful outcomes.

#### I.4 Harm-minimization posture

High-risk categories (coercion, financial exploitation, stalking patterns) should trigger conservative defaults, including mandatory `SAFETY_HOLD` and human review.

#### I.5 Continuous policy iteration

Safety policy must evolve with adversary behavior and user feedback. Protocol updates should include changelogs, migration notes, and measurable impact evaluation.

### Appendix J: End-to-End AMP/1.0 Negotiation Walkthrough

This appendix illustrates a realistic AMP thread with policy and security checkpoints. The example is intentionally verbose to clarify operational behavior.

#### J.1 Initialization

- Agent A and Agent B complete service registration and key provisioning.
- Each principal grants delegated scopes via OAuth2/OIDC.
- Both parties complete baseline verification suitable for discovery participation.

System preconditions:

- HMAC signing active for mutation endpoints.
- Replay ledger online (nonce uniqueness checks).
- Disclosure descriptors loaded for both profiles.
- No active safety or block flags.

#### J.2 Discovery and bilateral signal

1. Agent A requests candidates from discovery endpoint.
2. Love Envoy returns redacted objects with bucketed age, metro-level location, and partial interests.
3. Agent A emits `signal_interest` for candidate X.
4. Agent B independently emits reciprocal `signal_interest`.
5. System opens negotiation thread in `MUTUAL_INTEREST`.

Audit events emitted:

- `DISCOVERY_QUERY_EXECUTED`
- `INTEREST_SIGNAL_CREATED` (A)
- `INTEREST_SIGNAL_CREATED` (B)
- `NEGOTIATION_CREATED`

#### J.3 Structured negotiation

In `NEGOTIATING`, agents exchange typed messages:

```json
{
  "message_type": "compatibility_inquiry",
  "topic": "schedule_alignment",
  "payload": {
    "question": "principal has irregular travel windows; acceptable?",
    "expected_response_type": "enum"
  }
}
```

Responses are scored for consistency and completeness (without exposing private model internals). If either side attempts unsupported disclosure requests (e.g., direct phone number), policy engine denies and emits reason code.

#### J.4 Move to deep profile

A `tier_advance_request` is issued. Transition validation checks:

- state adjacency,
- both signatures,
- no active holds,
- descriptor permissions for newly visible fields.

On success, state becomes `DEEP_PROFILE`. Newly visible fields are still non-PII.

#### J.5 Human review and approval

To move beyond algorithmic negotiation, both principals receive:

- compatibility summary,
- explanation factors,
- unresolved risk notes (if any),
- explicit approve/reject options.

No action defaults to approval. If one principal declines, state becomes `REJECTED`.

#### J.6 High-risk gate example (off-platform contact)

Suppose one side requests off-platform contact exchange:

1. System classifies request as `off_platform_contact` risk tag.
2. Mandatory high-risk gate opens for both principals.
3. If both approve within policy window, and no safety flags fire, transition to `MEETING_PROPOSED` proceeds.
4. If either declines or times out, request is denied and negotiation can continue at prior state or close.

#### J.7 Safety hold scenario

If message behavior triggers scam/coercion heuristics (e.g., financial solicitation pattern), system enters `SAFETY_HOLD`:

- progression frozen,
- sensitive disclosure blocked,
- both principals notified,
- moderation review ticket generated.

Outcomes include restoration to prior safe state, conversion to `BLOCKED`, or transition to `REJECTED`.

#### J.8 Active handoff

Only after all required gates pass does thread enter `ACTIVE` for direct human communication. Protocol responsibility shifts from compatibility negotiation to post-handoff support and incident reporting.

### Appendix K: Expanded Competitive Analysis (Infrastructure Lens)

This appendix summarizes 2025–2026 competitive dynamics with emphasis on infrastructure positioning.

#### K.1 Consumer app momentum is real

The current generation of AI dating entrants demonstrates strong experimentation in:

- conversational matching agents,
- voice-mediated social discovery,
- concierge-like assistance,
- messaging-native experiences.

Funding and media visibility indicate investor belief that AI can reshape dating workflows.

#### K.2 Infrastructure remains underdeveloped

Despite consumer momentum, few players publicly define:

- interoperable agent negotiation contracts,
- standardized disclosure descriptors,
- portable trust/appeal schemas,
- adapter-level conformance across A2A/MCP ecosystems.

As a result, integrations are often vertical and proprietary, limiting composability.

#### K.3 Implications for incumbents

Large incumbents may introduce agent features rapidly, but broad interoperability is structurally harder in engagement-optimized systems. That creates room for neutral infrastructure providers that can service multiple endpoints without requiring any single network monopoly.

#### K.4 Market validation signals to monitor

For protocol businesses, useful leading indicators differ from raw app downloads:

- number of third-party agent runtimes implementing AMP-compatible flows,
- number of production partners using high-risk gates,
- percentage of negotiations with complete audit chains,
- average time to resolve safety appeals,
- partner expansion rates by segment (matchmaker/coaching/community).

#### K.5 B2B lead example: matchmaker automation timelines

Public statements by operators in AI-assisted matchmaking suggest confidence that substantial portions of manual matchmaking workflow can be automated in the near term (e.g., 2026 expectations). Whether exact timelines are met, the directional signal is strong: B2B operators are actively seeking automation infrastructure.

#### K.6 Pricing and premium behavior

High-end AI tier experiments in consumer dating indicate willingness to pay for improved outcomes and reduced friction in some user segments. Infrastructure providers can benefit indirectly by offering premium safety, verification, and compliance modules to partner products serving those segments.

### Appendix L: Deployment Readiness Checklist (Protocol + Operations)

This checklist is provided to reduce the gap between protocol design and production deployment.

#### L.1 Protocol conformance

- State machine validation library integrated.
- Descriptor schema version locked and documented.
- Terminal state behaviors tested.
- No implicit approvals in any gate.

#### L.2 Security controls

- HMAC validation enforced on mutating endpoints.
- OAuth2/OIDC grant revocation path tested.
- Replay protection load-tested.
- Secret/key rotation procedures documented.

#### L.3 Privacy controls

- PII and matching stores physically/logically separated.
- Event-triggered ID rotation active.
- Redaction reason codes enabled.
- Retention and purge workflows audited.

#### L.4 Safety operations

- `SAFETY_HOLD` runbook complete.
- Moderator tooling available with least privilege.
- Appeal SLA and staffing model defined.
- Incident response playbook rehearsed.

#### L.5 Partner integration controls

- Sandbox with synthetic data available.
- Adapter conformance tests for A2A/MCP connectors.
- Version compatibility matrix published.
- Support escalation path established.

#### L.6 Metrics and governance

- Outcome metrics prioritize safety and successful progression, not session length.
- Fairness diagnostics scheduled.
- Quarterly policy review cadence established.
- External advisory review (legal/safety) engaged for major version updates.

### Appendix M: Practical API Envelope Example

A minimal signed mutation envelope can be represented as:

```http
POST /v1/negotiations/{id}/advance
X-Client-Id: agt_7x9k2m4p
X-Timestamp: 1772364000
X-Nonce: 6f1f6b9f-3a37-4b8a-a7fa-4c4a7a6f2ed4
X-Idempotency-Key: 0e9f4f65-66ed-4bff-bcd2-0d2f8d171111
X-Signature: sha256=3f1a...b9
Authorization: Bearer eyJ... (OIDC delegated token)
Content-Type: application/json
```

Body:

```json
{
  "target_state": "DEEP_PROFILE",
  "reason": "mutual_compatibility_confirmed",
  "requires_human_gate": false
}
```

Server-side validation sequence:

1. Verify timestamp freshness.
2. Check nonce uniqueness.
3. Verify HMAC signature.
4. Validate OAuth scope for acting principal.
5. Run state machine and descriptor policy checks.
6. Persist transition and emit signed audit event.

### Appendix N: Why Trust-and-Safety-Led Positioning Is Strategic

Many early AI categories over-index on novelty and under-invest in safety infrastructure. In romantic domains, that strategy is especially fragile. A trust-and-safety-led position provides three advantages:

1. **Regulatory resilience:** easier adaptation to evolving consent and transparency requirements.
2. **Partner credibility:** enterprise and community operators prefer risk-managed infrastructure.
3. **User durability:** users may forgive imperfect recommendations; they rarely forgive safety failures.

Interoperability novelty is still useful, but it is not sufficient as a primary business thesis. Infrastructure trust is a stronger long-term anchor.

### Appendix O: Evaluation Framework for Future Empirical Studies

This appendix outlines a research design for validating AMP/1.0 in production-like environments. It is intentionally practical and can support both internal analytics and independent review.

#### O.1 Study designs

Recommended phased evaluation:

- **Phase A (offline simulation):** synthetic populations and scripted agents to test state safety and protocol invariants under load.
- **Phase B (pilot cohort):** limited partner deployment with strict moderation oversight.
- **Phase C (comparative field study):** compare agent-mediated funnel against baseline human-first workflows.

#### O.2 Primary outcomes

1. **Compatibility precision:** proportion of human-reviewed matches accepted.
2. **Progress efficiency:** median time from discovery to human-approved decision.
3. **Safety quality:** incidents per 1,000 negotiations and severity distribution.
4. **Consent integrity:** percentage of transitions with complete approval records.

#### O.3 Secondary outcomes

- user-reported cognitive burden,
- perceived transparency of agent decisions,
- appeal fairness satisfaction,
- partner operator workload reduction.

#### O.4 Safety-specific metrics

To avoid metric theater, safety should be evaluated with operational granularity:

- `SAFETY_HOLD` trigger precision and recall,
- false-positive burden on legitimate users,
- time-to-resolution for holds,
- recurrence rates after recovery.

#### O.5 Fairness checks

Trust and moderation systems should be tested for disparate impact across cohorts. Suggested checks include:

- error-rate parity of safety flags,
- appeal overturn rates by cohort,
- progression-rate parity when controlling for stated preferences.

#### O.6 Reproducibility and reporting

For scientific usefulness, reports should include:

- protocol version and policy descriptor version,
- adapter version (if A2A/MCP used),
- moderation staffing assumptions,
- explicit limitations and known confounders.

Transparent methodology is essential if agent-mediated matchmaking is to be evaluated as infrastructure rather than treated as a black-box product claim. Publishing negative results, protocol regressions, and safety misses alongside positive outcomes is strongly encouraged to build long-term ecosystem trust.
