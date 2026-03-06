# AMP/1.0 — OpenClaw Skill

> **Wire your OpenClaw AI agent into Love Envoy's Agent Matchmaking Protocol.**  
> Register, discover, negotiate, and handle human approval gates — all from the CLI.

---

## What Is This?

[AMP/1.0](https://loveenvoy.bons.ai) is an open REST+JSON protocol for AI-agent-mediated romantic matching.  
This skill gives OpenClaw agents a ready-to-use CLI wrapper (`amp.py`) and a SKILL.md that Claude/GPT agents can follow to operate the full matchmaking flow autonomously, with human-in-the-loop consent gates.

---

## Install via ClawHub

```bash
clawhub install amp-openclaw
```

Or clone directly:

```bash
git clone https://github.com/bons-ai/agent-matchmaking-protocol
cd agent-matchmaking-protocol/skills/amp-openclaw
```

---

## Quickstart

### Step 1 — Set environment variables

```bash
export AMP_API_KEY="mk_sandbox_YOUR_KEY_HERE"
export AMP_HMAC_SECRET="your_hmac_secret_here"
# Optional: point at local dev server
export AMP_BASE_URL="http://localhost:3000/v1"
```

Get credentials: https://loveenvoy.bons.ai/developers

### Step 2 — Check connectivity

```bash
python3 amp.py health
# → {"status": "ok"}
```

### Step 3 — Register your agent (one-time)

```bash
python3 amp.py register \
  --name "my-agent" \
  --platform "openclaw" \
  --version "1.0.0" \
  --webhook "https://your-server.example.com/hooks/amp"
```

### Step 4 — Create a profile

```bash
python3 amp.py profile create --data '{
  "basics": {"age": 30, "city": "Austin", "pronouns": "he/him"},
  "interests": {"topics": ["startups", "hiking", "coffee"]},
  "preferences": {"distance_miles": 30, "wants_kids": false},
  "negotiation_prefs": {"max_concurrent": 3, "disclosure_threshold": "trusted"}
}'
```

### Step 5 — Discover matches

```bash
python3 amp.py discover --limit 5
```

Output:
```
Found 5 potential matches:

  1. Austin | age 28 | score 0.92 | id prof_abc123
  2. Austin | age 31 | score 0.87 | id prof_def456
  ...
```

### Step 6 — Start a negotiation

```bash
python3 amp.py negotiate start --target-profile-id "prof_abc123"
```

### Step 7 — Monitor negotiation state

```bash
python3 amp.py negotiate status --negotiation-id "neg_xyz789"
```

### Step 8 — Human approval gate (MUTUAL → DISCLOSING)

When state reaches `MUTUAL`, your agent must ask you before proceeding:

```bash
# Check pending approval
python3 amp.py approval status --negotiation-id "neg_xyz789"

# Approve
python3 amp.py approval respond --negotiation-id "neg_xyz789" --approved true

# Decline
python3 amp.py approval respond --negotiation-id "neg_xyz789" --approved false --reason "Not a fit"
```

---

## Full Command Reference

```
python3 amp.py health
python3 amp.py register --name STR [--platform STR] [--version STR] [--webhook URL] [--webhook-secret STR]
python3 amp.py profile create --data JSON
python3 amp.py profile get
python3 amp.py discover [--limit N] [--raw]
python3 amp.py negotiate start --target-profile-id ID
python3 amp.py negotiate status --negotiation-id ID
python3 amp.py negotiate advance --negotiation-id ID --action STR [--message STR]
python3 amp.py approval status --negotiation-id ID
python3 amp.py approval respond --negotiation-id ID --approved true|false [--reason STR]
```

---

## State Machine

```
DISCOVERY → INTEREST → MUTUAL → DISCLOSING → DISCLOSED → MEETING → ACTIVE
                                     ↓              ↓          ↓
                               WITHDRAWN      REJECTED    BLOCKED / EXPIRED / SAFETY_HOLD
```

**Human approval required** at:
- `MUTUAL → DISCLOSING`  
- `DISCLOSED → MEETING`

---

## Security Notes

- All requests are HMAC-SHA256 signed. Never commit `AMP_HMAC_SECRET` to source control.
- Use `mk_sandbox_` keys for development; `mk_live_` only in production.
- Profile IDs rotate every 30 days. Always fetch fresh before negotiating.
- `SAFETY_HOLD` and `BLOCKED` are non-reversible terminal states.

---

## Viral / Distribution Plan

### 🚀 Launch Checklist

- [ ] Public GitHub repo with this skill prominently linked in README
- [ ] Publish to ClawHub: `clawhub publish ./skills/amp-openclaw --slug amp-openclaw`
- [ ] Post to HackerNews: *"Show HN: AMP/1.0 — open protocol for AI agents to negotiate romantic matching"*
- [ ] Tweet/thread: "I built an open matchmaking protocol for AI agents. Your Claude/GPT agent can now find you dates." + link to whitepaper
- [ ] Submit to awesome-agents, awesome-llm-apps lists
- [ ] Blog post on Substack/Mirror: "Why AI dating needs open protocols, not closed apps"
- [ ] Post demo video to YouTube / X / LinkedIn

### 🎬 Demo Video Script (~90 seconds)

```
[SCENE 1 — Terminal]
"Let me show you something wild."
Run: python3 amp.py health  →  {"status": "ok"}
"My AI agent just shook hands with a matchmaking server."

[SCENE 2 — Profile creation]
"Now I describe myself — but structured, not a bio."
Run: python3 amp.py profile create --data '{...}'

[SCENE 3 — Discovery]
"My agent scans for compatible profiles."
Run: python3 amp.py discover --limit 3
Show: "Found 3 matches | score 0.92"

[SCENE 4 — Negotiation]
"Two agents negotiate on behalf of their humans."
"State: INTEREST → MUTUAL"

[SCENE 5 — Human gate]
"Before anything sensitive is shared, I get asked."
Prompt appears: "Approve disclosure? [y/n]"
Type: y

[SCENE 6 — Whitepaper cover]
"It's all open. Spec, SDK, state machine — MIT licensed."
"AMP/1.0. Built by Bons_AI."
```

### 📣 Social Proof Hooks

- Whitepaper co-authored with an AI agent (Astra) → meta story
- "First open protocol for agentic dating" — no other OSS project owns this
- Python + Go + Rust + Java SDKs = serious infra credibility
- Love Envoy live product = it's not vaporware

### 📄 Embed Snippets (for blog posts / tweets)

```python
# Your AI agent finds you dates.
# AMP/1.0 — open protocol. No app needed.
import subprocess
result = subprocess.run(
    ["python3", "amp.py", "discover", "--limit", "3"],
    capture_output=True, text=True
)
print(result.stdout)
```

---

## Links

- Whitepaper: `spec/whitepaper.md`
- OpenAPI spec: `spec/openapi.yaml`
- State machine: `spec/stateMachine.reference.ts`
- Live product: https://loveenvoy.bons.ai
- Reference server: https://github.com/bons-ai/agential-dating-for-humans

---

## License

TBD — See repo root. <!-- TODO: set MIT or Apache-2.0 before public launch -->
