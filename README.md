# AMP/1.0 — Agent Matchmaking Protocol

**Multi-language SDK implementations for the Agent Matchmaking Protocol**

AMP/1.0 is a domain-specific protocol for AI-agent-driven romantic matchmaking, built on plain REST + JSON. It defines progressive disclosure, consent-gated negotiation states, trust scoring, identity protection, and auditable transitions.

**Product:** [Love Envoy](https://loveenvoy.ai) by Bons_AI, LLC

## Implementations

| Language | Directory | Status |
|----------|-----------|--------|
| Python | `python/` | ✅ Baseline client + models complete |
| Go | `go/` | 🚧 In Progress |
| Rust | `rust/` | 🚧 In Progress |
| Java | `java/` | 🚧 In Progress |

## Protocol Spec

The canonical AMP/1.0 specification lives in `spec/`.

## Reference Implementation

The Love Envoy TypeScript reference server lives at: https://github.com/bons-ai/agential-dating-for-humans

## Architecture

- **Transport:** Plain REST + JSON (no MCP/A2A dependency)
- **Auth:** HMAC for service-to-service + OAuth2/OIDC for delegated user consent
- **State Machine:** 12 states (DISCOVERY → INTEREST → MUTUAL → DISCLOSING → DISCLOSED → MEETING → ACTIVE + WITHDRAWN, EXPIRED, REJECTED, BLOCKED, SAFETY_HOLD)
- **Privacy:** Two-database model, opaque ID rotation, PII scrubbing

## License

TBD
