#!/usr/bin/env python3
"""Lightweight drift checks between OpenAPI schemas and SDK model surfaces.

This is intentionally dependency-free so it can run in CI without extra tooling.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Iterable, Set

ROOT = Path(__file__).resolve().parents[1]
SPEC_PATH = ROOT / "spec" / "openapi.yaml"

# Core model names expected to stay aligned across all SDKs.
SHARED_MODEL_NAMES = {
    "RegisterAgentRequest",
    "RegisterAgentResponse",
    "CreateProfileRequest",
    "DiscoveryCandidate",
    "Negotiation",
    "StateTransition",
    "ApprovalStatus",
}


def extract_openapi_version(spec_text: str) -> str:
    info_block = re.search(r"(?ms)^info:\n(.*?)(?:^\S|\Z)", spec_text)
    if not info_block:
        raise ValueError("Could not find OpenAPI info block")
    match = re.search(r"(?m)^  version:\s*([0-9]+\.[0-9]+\.[0-9]+)\s*$", info_block.group(1))
    if not match:
        raise ValueError("Could not find semantic version in info.version")
    return match.group(1)


def extract_component_schema_names(spec_text: str) -> Set[str]:
    lines = spec_text.splitlines()
    in_components = False
    in_schemas = False
    names: set[str] = set()

    for raw_line in lines:
        line = raw_line.rstrip("\n")

        if line == "components:":
            in_components = True
            continue

        if in_components and not line.startswith("  ") and line:
            in_components = False
            in_schemas = False

        if in_components and line == "  schemas:":
            in_schemas = True
            continue

        if in_schemas:
            if line.startswith("  ") and not line.startswith("    ") and line.strip():
                # End of components.schemas block (e.g. responses)
                in_schemas = False
                continue

            match = re.match(r"^    ([A-Za-z_][A-Za-z0-9_]*)\s*:\s*$", line)
            if match:
                names.add(match.group(1))

    if not names:
        raise ValueError("No component schema names discovered from spec/openapi.yaml")

    return names


def extract_python_models(path: Path) -> Set[str]:
    text = path.read_text(encoding="utf-8")
    return set(re.findall(r"(?m)^class\s+([A-Za-z_][A-Za-z0-9_]*)\b", text))


def extract_go_models(path: Path) -> Set[str]:
    text = path.read_text(encoding="utf-8")
    return set(re.findall(r"(?m)^type\s+([A-Za-z_][A-Za-z0-9_]*)\s+(?:struct|map|\[)", text))


def extract_rust_models(path: Path) -> Set[str]:
    text = path.read_text(encoding="utf-8")
    return set(re.findall(r"(?m)^pub\s+(?:struct|enum)\s+([A-Za-z_][A-Za-z0-9_]*)\b", text))


def extract_java_models(path: Path) -> Set[str]:
    return {file.stem for file in path.glob("*.java")}


def missing(expected: Iterable[str], actual: Set[str]) -> list[str]:
    return sorted(name for name in expected if name not in actual)


def main() -> int:
    spec_text = SPEC_PATH.read_text(encoding="utf-8")
    spec_version = extract_openapi_version(spec_text)
    schema_names = extract_component_schema_names(spec_text)

    missing_in_spec = missing(SHARED_MODEL_NAMES, schema_names)

    python_models = extract_python_models(ROOT / "python" / "amp_sdk" / "models.py")
    go_models = extract_go_models(ROOT / "go" / "models.go")
    rust_models = extract_rust_models(ROOT / "rust" / "amp-sdk" / "src" / "models.rs")
    java_models = extract_java_models(ROOT / "java" / "src" / "main" / "java" / "com" / "bonsai" / "amp" / "sdk" / "model")

    failures: list[str] = []
    if missing_in_spec:
        failures.append(f"spec/openapi.yaml missing schemas: {', '.join(missing_in_spec)}")

    for label, models in (
        ("python", python_models),
        ("go", go_models),
        ("rust", rust_models),
        ("java", java_models),
    ):
        missing_models = missing(SHARED_MODEL_NAMES, models)
        if missing_models:
            failures.append(f"{label} SDK missing models: {', '.join(missing_models)}")

    if failures:
        print("❌ OpenAPI drift check failed")
        for failure in failures:
            print(f"  - {failure}")
        return 1

    print(f"✅ OpenAPI drift check passed (spec version {spec_version})")
    print(f"   Verified shared models: {', '.join(sorted(SHARED_MODEL_NAMES))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
