# AMP/1.0 SDK — ACTION_PLAN

_Last updated: 2026-03-02 (UTC)_

## Review Scope (this run)
- Repo structure and implementation completeness for Python, Go, Rust, Java
- Spec readiness (`spec/openapi.yaml`, state machine reference, whitepaper)
- Build/test/release hygiene across language targets
- Security and protocol parity risks

## Current Snapshot
- ✅ Canonical protocol materials exist in `spec/` (OpenAPI + architecture + whitepaper)
- ⚠️ SDK implementations are mostly scaffolding:
  - `python/` has no package/client code yet
  - `go/` has only `go.mod`
  - `java/` has only `pom.xml`
  - `rust/amp-sdk` has `lib.rs` exports but referenced modules are not present
- ⚠️ No cross-language contract tests
- ⚠️ No CI matrix for multi-language validation
- ⚠️ Host currently lacks toolchains (`go`, `cargo`, `mvn`, `pytest`) for local verification

## Prioritized TODOs

### P0 — Make SDKs Real (minimum usable clients)
- [ ] **Bootstrap baseline client + models in all 4 languages**
  - Implement core operations: register agent, create profile, discover, negotiate, approvals
  - Keep transport simple: HTTP/JSON + pluggable base URL
  - Ensure generated/handwritten models align with OpenAPI 0.4.0

- [ ] **Complete Rust crate so it compiles/tests**
  - Add missing modules (`auth`, `client`, `error`, `models`, `state_machine`)
  - Provide typed error handling + serialization round-trip tests

- [ ] **Define a shared conformance test fixture set**
  - JSON request/response fixtures in `spec/fixtures/`
  - State transition validity/invalidity cases from canonical state machine
  - Run these fixtures against all SDKs

### P1 — Security + Reliability
- [ ] **Implement consistent auth helpers across SDKs**
  - HMAC signer with canonical payload builder
  - Replay-safe timestamp helpers and clock-skew handling
  - Optional OAuth2 token provider interface (where applicable)

- [ ] **Add retries, timeouts, and idempotency hooks**
  - Sensible defaults (exponential backoff, 429/5xx retry policy)
  - Per-request override knobs
  - Structured error taxonomy shared semantically across languages

### P1 — Build/Release Infrastructure
- [ ] **Set up multi-language CI matrix**
  - Python (pytest + type check), Go (test/vet), Rust (fmt/clippy/test), Java (mvn test)
  - OpenAPI drift check in CI (SDK models vs spec)

- [ ] **Add packaging/publishing pipelines**
  - Python: PyPI package skeleton + versioning
  - Go: tagged module release policy
  - Rust: crates.io metadata + release checklist
  - Java: Maven Central/Sonatype publishing plan

### P2 — Developer Adoption
- [ ] **Create copy-paste quickstarts per language**
  - 5-minute “register → profile → discover → negotiate” example
  - Env var and signing examples with secure defaults

- [ ] **Publish compatibility matrix and semantic-versioning policy**
  - Map SDK versions to AMP spec versions
  - Declare deprecation and migration guarantees

## Execution Order (recommended)
1. Rust compile-complete baseline + tests (smallest path to first working SDK)
2. Python and Go baseline clients
3. Java baseline client
4. Shared fixture-based conformance tests
5. CI matrix + release pipelines
6. Docs/quickstarts + compatibility policy
