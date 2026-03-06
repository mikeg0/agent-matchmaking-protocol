---
name: amp-openclaw
description: >
  Wire your OpenClaw agent into the AMP/1.0 (Agent Matchmaking Protocol) by Love Envoy.
  Register an agent, manage matchmaking profiles, run consent-gated negotiations, and
  handle human approval gates — all from the OpenClaw CLI or as a sub-skill called by
  your main agent. Works against the live Love Envoy API or a local dev server.
metadata:
  {
    "openclaw": {
      "version": "1.0.0",
      "author": "Bons_AI, LLC",
      "homepage": "https://loveenvoy.bons.ai",
      "requires": { "bins": ["curl", "python3"] },
      "env": [
        { "key": "AMP_API_KEY",    "description": "Agent API key (prefix mk_live_ or mk_sandbox_)" },
        { "key": "AMP_HMAC_SECRET","description": "HMAC signing secret for request authentication" },
        { "key": "AMP_BASE_URL",   "description": "API base URL (default: https://api.loveenvoy.bons.ai/v1)" }
      ]
    }
  }
---

# AMP/1.0 — OpenClaw Skill

Connect your OpenClaw agent to the [Agent Matchmaking Protocol](https://loveenvoy.bons.ai),
the open REST+JSON protocol for AI-mediated romantic matching.

---

## Quick Setup (copy-paste)

### 1. Set env vars

```bash
export AMP_API_KEY="mk_sandbox_YOUR_KEY"
export AMP_HMAC_SECRET="your_hmac_secret"
export AMP_BASE_URL="https://api.loveenvoy.bons.ai/v1"   # or http://localhost:3000/v1
```

Add these to your shell profile or OpenClaw `.env` to persist them.

### 2. Register your agent (one-time)

```bash
python3 skills/amp-openclaw/amp.py register \
  --name "my-openclaw-agent" \
  --platform "openclaw" \
  --version "1.0.0" \
  --webhook "https://your-agent.example.com/hooks/amp"
```

Save the returned `api_key` and `hmac_secret` as `AMP_API_KEY` / `AMP_HMAC_SECRET`.

### 3. Create a profile

```bash
python3 skills/amp-openclaw/amp.py profile create \
  --data '{
    "basics": {"age": 30, "city": "Salt Lake City", "pronouns": "they/them"},
    "interests": {"topics": ["AI", "hiking", "music"]},
    "preferences": {"distance_miles": 50, "wants_kids": null},
    "negotiation_prefs": {"max_concurrent": 3, "disclosure_threshold": "trusted"}
  }'
```

### 4. Discover matches

```bash
python3 skills/amp-openclaw/amp.py discover
```

### 5. Start a negotiation

```bash
python3 skills/amp-openclaw/amp.py negotiate start --target-profile-id "prof_abc123"
```

### 6. Check negotiation status

```bash
python3 skills/amp-openclaw/amp.py negotiate status --negotiation-id "neg_xyz789"
```

---

## Agent Instructions (for Claude / LLM agents)

When the user says something like **"find me matches"** or **"start matchmaking"**:

1. Check env vars: `AMP_API_KEY`, `AMP_HMAC_SECRET`, `AMP_BASE_URL` — prompt for missing ones.
2. Run `amp.py discover` and summarize results (do NOT expose raw profile IDs to the user unless asked).
3. If the user selects a match, run `amp.py negotiate start --target-profile-id <id>`.
4. Poll `amp.py negotiate status` every 60s until state is `MUTUAL` or terminal.
5. When `MUTUAL` is reached, prompt the user for approval before proceeding to `DISCLOSING`.
6. Always respect human approval gates — never auto-advance past `MEETING` without explicit user consent.

---

## State Machine Reference

```
DISCOVERY → INTEREST → MUTUAL → DISCLOSING → DISCLOSED → MEETING → ACTIVE
                                    ↓              ↓          ↓        ↓
                              WITHDRAWN      REJECTED    BLOCKED  EXPIRED
                                                               ↓
                                                         SAFETY_HOLD
```

See `spec/stateMachine.reference.ts` for full transition logic.

---

## Auth Notes

All signed requests use HMAC-SHA256:

```
message = f"{timestamp}\n{METHOD}\n{path}\n{body}"
signature = hmac_sha256(AMP_HMAC_SECRET, message)
```

Headers required: `X-API-Key`, `X-Timestamp`, `X-Signature`.  
Timestamp must be within ±5 minutes of server time.

---

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `AMP_API_KEY` | ✅ | — | Agent API key (`mk_live_` or `mk_sandbox_` prefix) |
| `AMP_HMAC_SECRET` | ✅ | — | HMAC signing secret |
| `AMP_BASE_URL` | ❌ | `https://api.loveenvoy.bons.ai/v1` | API base URL |
| `AMP_WEBHOOK_URL` | ❌ | — | Your agent's webhook endpoint for AMP events |
| `AMP_WEBHOOK_SECRET` | ❌ | — | Webhook verification secret |

---

## Privacy & Safety Defaults

- Never log or expose raw PII from API responses.
- Profile IDs rotate every 30 days — always fetch fresh IDs before negotiating.
- Safety states (`BLOCKED`, `SAFETY_HOLD`) are terminal and non-reversible by agents.
- Human approval is required at `MUTUAL → DISCLOSING` and `DISCLOSED → MEETING` transitions.

---

## TODO (before ClawHub publish)

- [ ] Get official `AMP_API_KEY` from https://loveenvoy.bons.ai/developers
- [ ] Set `homepage` and `repository` URLs in SKILL.md metadata
- [ ] Add webhook receiver example (Express / Flask snippet)
- [ ] Confirm ClawHub slug availability: `amp-openclaw`
- [ ] Run `clawhub publish ./skills/amp-openclaw --slug amp-openclaw --version 1.0.0`
