#!/usr/bin/env python3
"""
amp.py — AMP/1.0 CLI helper for OpenClaw agents
Usage: python3 amp.py <command> [options]

Commands:
  health                          Check API health
  register                        Register a new agent
  profile create --data <json>    Create a profile
  profile get                     Get current profile
  discover                        Discover compatible profiles
  negotiate start --target-profile-id <id>
  negotiate status --negotiation-id <id>
  negotiate advance --negotiation-id <id> --action <action>
  approval status --negotiation-id <id>
  approval respond --negotiation-id <id> --approved <true|false>
"""

import argparse
import hashlib
import hmac
import json
import os
import sys
import time
import urllib.request
import urllib.error
from typing import Any, Dict, Optional

# ── Config ─────────────────────────────────────────────────────────────────────

BASE_URL = os.environ.get("AMP_BASE_URL", "https://api.loveenvoy.bons.ai/v1").rstrip("/")
API_KEY = os.environ.get("AMP_API_KEY", "")
HMAC_SECRET = os.environ.get("AMP_HMAC_SECRET", "")


# ── Auth ────────────────────────────────────────────────────────────────────────

def _sign(method: str, path: str, body: str = "") -> Dict[str, str]:
    ts = str(int(time.time()))
    msg = f"{ts}\n{method.upper()}\n{path}\n{body}"
    sig = hmac.new(HMAC_SECRET.encode(), msg.encode(), hashlib.sha256).hexdigest()
    return {
        "X-API-Key": API_KEY,
        "X-Timestamp": ts,
        "X-Signature": sig,
        "Content-Type": "application/json",
    }


def _request(method: str, path: str, payload: Optional[Dict] = None) -> Any:
    body = json.dumps(payload) if payload else ""
    headers = _sign(method, path, body)
    url = f"{BASE_URL}{path}"
    data = body.encode() if body else None
    req = urllib.request.Request(url, data=data, headers=headers, method=method.upper())
    try:
        with urllib.request.urlopen(req) as resp:
            return json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        err = e.read().decode()
        print(f"HTTP {e.code}: {err}", file=sys.stderr)
        sys.exit(1)


# ── Commands ────────────────────────────────────────────────────────────────────

def cmd_health(_args):
    # Health endpoint needs no auth
    req = urllib.request.Request(f"{BASE_URL}/health")
    with urllib.request.urlopen(req) as resp:
        print(json.dumps(json.loads(resp.read().decode()), indent=2))


def cmd_register(args):
    payload = {
        "name": args.name,
        "agent_platform": args.platform,
        "agent_version": args.version,
        "capabilities": ["discover", "negotiate", "safety"],
    }
    if args.webhook:
        payload["webhook_url"] = args.webhook
    if args.webhook_secret:
        payload["webhook_secret"] = args.webhook_secret

    # Register doesn't require auth headers per spec
    body = json.dumps(payload)
    url = f"{BASE_URL}/agents/register"
    req = urllib.request.Request(
        url,
        data=body.encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req) as resp:
            result = json.loads(resp.read().decode())
            print(json.dumps(result, indent=2))
            print("\n⚡ Save these credentials:", file=sys.stderr)
            print(f"  AMP_API_KEY={result.get('api_key','<see above>')}", file=sys.stderr)
            print(f"  AMP_HMAC_SECRET={result.get('hmac_secret','<see above>')}", file=sys.stderr)
    except urllib.error.HTTPError as e:
        print(f"HTTP {e.code}: {e.read().decode()}", file=sys.stderr)
        sys.exit(1)


def cmd_profile_create(args):
    payload = json.loads(args.data)
    result = _request("POST", "/profiles", payload)
    print(json.dumps(result, indent=2))


def cmd_profile_get(_args):
    result = _request("GET", "/profiles/me")
    print(json.dumps(result, indent=2))


def cmd_discover(args):
    path = "/discovery"
    if args.limit:
        path += f"?limit={args.limit}"
    result = _request("GET", path)
    # Pretty-print without exposing raw IDs unless --raw
    if args.raw:
        print(json.dumps(result, indent=2))
    else:
        profiles = result.get("profiles", result)
        if isinstance(profiles, list):
            print(f"Found {len(profiles)} potential matches:\n")
            for i, p in enumerate(profiles, 1):
                basics = p.get("basics", {})
                score = p.get("compatibility_score", "?")
                pid = p.get("profile_id", "?")
                print(f"  {i}. {basics.get('city','?')} | age {basics.get('age','?')} | score {score} | id {pid}")
        else:
            print(json.dumps(result, indent=2))


def cmd_negotiate_start(args):
    payload = {"target_profile_id": args.target_profile_id}
    result = _request("POST", "/negotiations", payload)
    print(json.dumps(result, indent=2))


def cmd_negotiate_status(args):
    result = _request("GET", f"/negotiations/{args.negotiation_id}")
    print(json.dumps(result, indent=2))


def cmd_negotiate_advance(args):
    payload = {"action": args.action}
    if args.message:
        payload["message"] = args.message
    result = _request("POST", f"/negotiations/{args.negotiation_id}/advance", payload)
    print(json.dumps(result, indent=2))


def cmd_approval_status(args):
    result = _request("GET", f"/negotiations/{args.negotiation_id}/approval")
    print(json.dumps(result, indent=2))


def cmd_approval_respond(args):
    payload = {"approved": args.approved.lower() == "true"}
    if args.reason:
        payload["reason"] = args.reason
    result = _request("POST", f"/negotiations/{args.negotiation_id}/approval", payload)
    print(json.dumps(result, indent=2))


# ── CLI ─────────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="AMP/1.0 CLI for OpenClaw agents")
    sub = parser.add_subparsers(dest="cmd", required=True)

    sub.add_parser("health")

    reg = sub.add_parser("register")
    reg.add_argument("--name", required=True)
    reg.add_argument("--platform", default="openclaw")
    reg.add_argument("--version", default="1.0.0")
    reg.add_argument("--webhook")
    reg.add_argument("--webhook-secret")

    prof = sub.add_parser("profile")
    prof_sub = prof.add_subparsers(dest="prof_cmd", required=True)
    pc = prof_sub.add_parser("create")
    pc.add_argument("--data", required=True, help="JSON string")
    prof_sub.add_parser("get")

    disc = sub.add_parser("discover")
    disc.add_argument("--limit", type=int, default=10)
    disc.add_argument("--raw", action="store_true")

    neg = sub.add_parser("negotiate")
    neg_sub = neg.add_subparsers(dest="neg_cmd", required=True)
    ns = neg_sub.add_parser("start")
    ns.add_argument("--target-profile-id", required=True)
    nst = neg_sub.add_parser("status")
    nst.add_argument("--negotiation-id", required=True)
    na = neg_sub.add_parser("advance")
    na.add_argument("--negotiation-id", required=True)
    na.add_argument("--action", required=True)
    na.add_argument("--message")

    appr = sub.add_parser("approval")
    appr_sub = appr.add_subparsers(dest="appr_cmd", required=True)
    ast = appr_sub.add_parser("status")
    ast.add_argument("--negotiation-id", required=True)
    ar = appr_sub.add_parser("respond")
    ar.add_argument("--negotiation-id", required=True)
    ar.add_argument("--approved", required=True, choices=["true", "false"])
    ar.add_argument("--reason")

    args = parser.parse_args()

    # Validate auth for signed commands
    unsigned = {"health", "register"}
    if args.cmd not in unsigned:
        if not API_KEY or not HMAC_SECRET:
            print("Error: AMP_API_KEY and AMP_HMAC_SECRET must be set.", file=sys.stderr)
            print("  export AMP_API_KEY=mk_sandbox_...", file=sys.stderr)
            print("  export AMP_HMAC_SECRET=...", file=sys.stderr)
            sys.exit(1)

    dispatch = {
        "health": cmd_health,
        "register": cmd_register,
        "discover": cmd_discover,
    }

    if args.cmd in dispatch:
        dispatch[args.cmd](args)
    elif args.cmd == "profile":
        if args.prof_cmd == "create":
            cmd_profile_create(args)
        elif args.prof_cmd == "get":
            cmd_profile_get(args)
    elif args.cmd == "negotiate":
        if args.neg_cmd == "start":
            cmd_negotiate_start(args)
        elif args.neg_cmd == "status":
            cmd_negotiate_status(args)
        elif args.neg_cmd == "advance":
            cmd_negotiate_advance(args)
    elif args.cmd == "approval":
        if args.appr_cmd == "status":
            cmd_approval_status(args)
        elif args.appr_cmd == "respond":
            cmd_approval_respond(args)


if __name__ == "__main__":
    main()
