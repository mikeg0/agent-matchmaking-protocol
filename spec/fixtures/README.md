# AMP Shared Conformance Fixtures

These fixtures provide canonical request/response payloads and state-transition cases
that every AMP SDK implementation should consume.

## Layout

- `http/*.json` — canonical JSON payloads for model serialization/deserialization checks.
- `state/transition_cases.json` — validity/invalidity transition cases derived from
  `spec/stateMachine.reference.ts`.

## Intended usage

Each SDK should:

1. Deserialize request fixtures into typed request models and ensure a serialize round-trip
   preserves payload shape.
2. Deserialize response fixtures into typed response models and verify critical fields.
3. Execute `state/transition_cases.json` against the SDK's transition validator (or
   equivalent conformance test helper) and assert `valid` matches expected behavior.

This fixture set is intentionally small and deterministic so it can run quickly in CI
across all language targets.
