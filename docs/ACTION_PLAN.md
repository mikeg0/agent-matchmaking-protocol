# AMP/1.0 SDK — ACTION_PLAN

_Last updated: 2026-03-03 (UTC, run 4)_

## Review Scope (this run)
- Repo structure and implementation completeness for Python, Go, Rust, Java
- Spec readiness (`spec/openapi.yaml`, state machine reference, whitepaper)
- Build/test/release hygiene across language targets
- Security and protocol parity risks

## Current Snapshot
- ✅ Canonical protocol materials exist in `spec/` (OpenAPI + architecture + whitepaper)
- ✅ Baseline SDK clients/models now exist in all 4 languages:
  - `python/amp_sdk` baseline client + models + unit tests
  - `go/` baseline client + models + auth + unit tests
  - `rust/` baseline modules (`auth`, `client`, `error`, `models`, `state_machine`) + unit-test scaffolding
  - `java/` baseline client + models + auth + unit tests
- ⚠️ No cross-language contract tests yet
- ⚠️ No CI matrix for multi-language validation
- ⚠️ Host currently lacks toolchains (`go`, `cargo`, `mvn`, `pytest`, `javac`) for local verification

## Prioritized TODOs

### P0 — Make SDKs Real (minimum usable clients)
- [x] **Bootstrap baseline client + models in all 4 languages**
  - Implement core operations: register agent, create profile, discover, negotiate, approvals
  - Keep transport simple: HTTP/JSON + pluggable base URL
  - Ensure generated/handwritten models align with OpenAPI 0.4.0
  - Progress:
    - [x] Python baseline SDK (`python/amp_sdk`) with core client ops + models + unit tests
    - [x] Go baseline SDK (`go/`) with core client ops + models + unit tests
    - [x] Rust baseline SDK modules + typed error/state-machine scaffolding
    - [x] Java baseline SDK (`java/`) with core client ops + models + unit tests

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
1. Shared fixture-based conformance tests
2. CI matrix + release pipelines
3. Auth/retry consistency passes
4. Docs/quickstarts + compatibility policy
