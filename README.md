# AMP/1.0 — Agent Matchmaking Protocol

**Multi-language SDK implementations for the Agent Matchmaking Protocol**

AMP/1.0 is a domain-specific protocol for AI-agent-driven romantic matchmaking, built on plain REST + JSON. It defines progressive disclosure, consent-gated negotiation states, trust scoring, identity protection, and auditable transitions.

**Product:** [Love Envoy](https://loveenvoy.ai) by Bons_AI, LLC

## Implementations

| Language | Directory | Status |
|----------|-----------|--------|
| Python | `python/` | ✅ Baseline client + models + fixture conformance tests |
| Go | `go/` | ✅ Baseline client + models + fixture conformance tests |
| Rust | `rust/` | ✅ Baseline client + models + fixture conformance tests *(local compile pending toolchain)* |
| Java | `java/` | ✅ Baseline client + models + fixture conformance tests *(local compile pending toolchain)* |

## Documents

- [Blog Post (PDF)](BON5AI-Matchmaker-API-Blog-v2.pdf)
- [Whitepaper (PDF)](BON5AI-Matchmaker-API-Whitepaper-v2.pdf)
- [Whitepaper (Short)](matchmaker-api-whitepaper-short.md)

## Protocol Spec

The canonical AMP/1.0 specification lives in `spec/`.

## CI validation matrix

GitHub Actions runs a multi-language matrix on push/PR:

- OpenAPI drift guard: `python3 scripts/check_openapi_drift.py`
- Python: `pytest` + `mypy`
- Go: `go test ./...` + `go vet ./...`
- Rust: `cargo fmt --check` + `cargo clippy` + `cargo test`
- Java: `mvn test`

Run the OpenAPI drift guard locally from repo root:

```bash
python3 scripts/check_openapi_drift.py
```

## Reference Implementation

The Love Envoy TypeScript reference server lives at: https://github.com/bons-ai/agential-dating-for-humans

## Architecture

- **Transport:** Plain REST + JSON (no MCP/A2A dependency)
- **Auth:** HMAC for service-to-service + OAuth2/OIDC for delegated user consent
- **State Machine:** 12 states (DISCOVERY → INTEREST → MUTUAL → DISCLOSING → DISCLOSED → MEETING → ACTIVE + WITHDRAWN, EXPIRED, REJECTED, BLOCKED, SAFETY_HOLD)
- **Privacy:** Two-database model, opaque ID rotation, PII scrubbing

## OpenClaw Skill — `amp-openclaw`

Integrate AMP/1.0 directly into your [OpenClaw](https://openclaw.ai) AI agent with the bundled skill:

```bash
# Install from ClawHub (once published)
clawhub install amp-openclaw

# Or use directly from this repo
cd skills/amp-openclaw
export AMP_API_KEY="mk_sandbox_..."
export AMP_HMAC_SECRET="..."
python3 amp.py health
python3 amp.py discover --limit 5
```

📄 **[Full skill docs → skills/amp-openclaw/README.md](skills/amp-openclaw/README.md)**  
🤖 **[SKILL.md for agent instructions → skills/amp-openclaw/SKILL.md](skills/amp-openclaw/SKILL.md)**

The skill includes:
- `amp.py` — zero-dependency Python CLI for all AMP/1.0 endpoints
- `SKILL.md` — OpenClaw-compatible agent instructions (Claude/GPT can follow this autonomously)
- Full quickstart, env var guide, state machine reference, and viral launch playbook

---

## License

See [LICENSE](LICENSE) for details.
