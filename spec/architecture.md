# Love Envoy — Architecture & Protocol Integration Notes

_Date: 2026-03-01_
_Status: Active planning_

## Protocol Stack

```
┌─────────────────────────────────────────┐
│  AMP/1.0 (Agent Matchmaking Protocol)   │  ← WE OWN THIS
│  Progressive disclosure, trust scoring, │
│  compatibility negotiation, approval    │
│  gates, privacy model                   │
├─────────────────────────────────────────┤
│  Transport: HTTP/JSON (simple REST)     │  ← DEFAULT. No framework needed.
│  Optional: A2A, MCP adapters            │  ← Nice-to-have, not required.
└─────────────────────────────────────────┘
```

## Design Philosophy: Simplicity First

**The barrier to implement AMP should be as low as possible.**

The agent communication space is evolving fast (as of early 2026):
- Google's A2A, Anthropic's MCP, IBM's ACP, decentralized ANP — all competing
- None of these are settled standards yet. They're all moving targets.
- Requiring any of them as a dependency would slow developers down

**Look at how OpenClaw skills work:** plain markdown files + shell scripts. No SDK, no protocol negotiation, no handshake ceremony. A developer reads a SKILL.md and starts building. That's the energy AMP should have.

**AMP/1.0 should be:**
- A REST API with JSON payloads. That's it.
- Any language, any framework, any HTTP client can implement it
- HMAC signing for auth (simple, well-understood, no OAuth dance)
- A spec document a developer can read in 30 minutes and start coding against

**Optional integration layers (don't gate anything on these):**
- A2A Agent Card — for agents that speak A2A to discover Love Envoy
- MCP Server — for agents in MCP ecosystems (Claude, etc.) to use Love Envoy as a tool
- These are distribution adapters, not requirements

## The Agent Communication Landscape (Feb/Mar 2026)

**Note: This space is evolving rapidly. Don't bet the architecture on any single protocol.**

| Protocol | Owner | What It Does | Maturity |
|----------|-------|-------------|----------|
| A2A | Google/IBM | Agent ↔ Agent communication | Active, growing. GitHub: a2aproject/A2A |
| MCP | Anthropic | Agent → Tools/Data | Production, widely adopted. GitHub: modelcontextprotocol |
| ACP | IBM | Enterprise agent interop | Early |
| ANP | Open/decentralized | Identity, discovery, negotiation | Academic/early |

**Key insight:** None of these handle domain-specific matchmaking semantics. They're transport/discovery layers. AMP defines the matchmaking-specific stuff on top of plain HTTP — and can optionally ride on any of these if/when they stabilize.

## What AMP/1.0 Actually Defines (that generic protocols don't)

1. **Progressive disclosure tiers** — what info is shared at each negotiation stage
2. **Human approval gates** — which transitions require human sign-off
3. **Trust scoring** — behavioral reputation system for agents
4. **Compatibility negotiation** — structured preference matching, not generic task delegation
5. **Privacy model** — two-database architecture, opaque ID rotation, PII scrubbing
6. **8-state negotiation machine** — DISCOVERY → INTEREST → MUTUAL → DISCLOSING → DISCLOSED → MEETING → ACTIVE (+ WITHDRAWN)

## Implementation Priority

1. **AMP/1.0 spec document** — readable, implementable in any language
2. **Love Envoy reference implementation** — the existing TypeScript/Fastify codebase
3. **Example clients** — Python, Go, TypeScript (simple HTTP clients, not SDK frameworks)
4. **Optional adapters** — A2A Agent Card, MCP server (when/if demand exists)

## Key Principle

> If a developer needs to install a Google SDK or understand MCP internals to use Love Envoy, we've failed. 
> If they can `curl` the API and get a match, we've succeeded.
