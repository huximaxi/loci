# Loci · Integration Roadmap Tracker
*Pipeline: feature-roadmap · Last synced: 2026-05-09 · Source of truth for roadmap.html*

> **Auth principle:** Auth (Nostr keypair, AT DID) is available at every tier. It is a feature users choose — not a gate. It increases internal trust and identity consistency but is never required.

---

## Status Key
| Symbol | Meaning |
|--------|---------|
| `🔴 not-started` | Brief written, no code yet |
| `🟡 in-progress` | Active development |
| `🟢 shipped` | Live in main |
| `⏸ blocked` | Waiting on dependency |
| `📋 specced` | ADR or spec complete, not started |

---

## Tier 1 — Q3 2026

| ID | Feature | Persona | Status | Dependency | Branch | Last Activity |
|----|---------|---------|--------|-----------|--------|---------------|
| 1A | Ollama Local Inference | Kata | 🔴 not-started | — | `feat/1A-ollama` | — |
| 1B | Goose MCP Plugin | Cipher | 🔴 not-started | — | `feat/1B-goose-mcp` | — |
| 1C | Nostr Keypair Identity | Cipher | 🔴 not-started | — | `feat/1C-nostr-identity` | — |
| 1D | Nym Partnership | Rune | 🔴 not-started | 1A + 1B live | `comms/1D-nym-announcement` | — |

## Tier 2 — Q4 2026

| ID | Feature | Persona | Status | Dependency | Branch | Last Activity |
|----|---------|---------|--------|-----------|--------|---------------|
| 2A | Continue.dev Context Provider | Kata | ⏸ blocked | 1B shipped | `feat/2A-continue` | — |
| 2B | Nostr Publishing + Zaps | Nyx | ⏸ blocked | 1C shipped | `feat/2B-nostr-zaps` | — |
| 2C | AT Protocol DID | Cipher | 🔴 not-started | — | `feat/2C-at-protocol` | — |
| 2D | Nym Private Sync | Kata | ⏸ blocked | 1D confirmed | `feat/2D-nym-sync` | — |
| 2E | IPFS Garden Export | Kata | 🔴 not-started | — | `feat/2E-ipfs` | — |
| 2F | AnythingLLM Bridge | Nyx | ⏸ blocked | 1B shipped | `feat/2F-anythingllm` | — |
| 2G | Kagi Web Enrichment | Rune | 🔴 not-started | — | `feat/2G-kagi` | — |
| 2H | Tailscale Local Brain | Cipher | 📋 specced | — | `feat/2H-tailscale` | — |

---

## Changelog

```
2026-05-09  All briefs initialised by Vesper convent session
2026-05-09  Auth principle confirmed: no gating, feature-level choice
2026-05-09  v1.3 alliance brief locked
```

*Watcher appends here automatically. Manual entries use format: `YYYY-MM-DD  [ID] [action] [detail]`*

---

## Dependency Graph

```
          1A (Ollama) ──────────────────────────────────┐
          1B (MCP) ──── 2A (Continue) ── (free after 1B)│
          1B (MCP) ──── 2F (AnythingLLM)                │
          1C (Nostr ID) ── 2B (Zaps)                    │
          1D (Nym) ─ requires 1A+1B live ── 2D (Sync)   │
          2C (AT Protocol) ─ independent               │
          2E (IPFS) ─ independent                      │
          2G (Kagi) ─ independent                      │
          2H (Tailscale) ─ independent (ADR first)     ┘
```

---

## Roadmap HTML Sync

To regenerate `pipeline/roadmap/roadmap.html` from this tracker, run:
```bash
cd ~/Dev/loci/pipeline
./watcher/roadmap-watcher.sh --html
```

The watcher reads feature branch git activity, updates "Last Activity" column, and flags any feature where status hasn't changed in >14 days as stale.

---

## Open Security Gates (Cipher)

| Gate | Feature | Condition to clear |
|------|---------|-------------------|
| THREAT-01 | 1B MCP (conversation context) | Sanitise-before-write implemented + untrusted-content disclaimer in MCP responses |
| THREAT-03 | All | Exact version pins, lock file committed, npm audit in CI |

---
*pipeline/ROADMAP-TRACKER.md · auto-synced by watcher/roadmap-watcher.sh*
