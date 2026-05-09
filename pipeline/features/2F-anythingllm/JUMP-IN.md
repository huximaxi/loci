# 2F · AnythingLLM Bridge
## Jump-In Brief

**Persona:** Nyx — integration UX, closest competitor positioning
**Tier:** 2 · **Target:** Q4 2026 (free after 1B)
**Status:** `⏸ blocked on 1B`
**Branch:** `feat/2F-anythingllm`
**Last updated:** 2026-05-09

---

## Context
AnythingLLM is the closest existing product to what Loci builds on the AI/RAG side — self-hosted, MIT licensed, local-first document chat. The communities overlap substantially. The integration play is: Loci as AnythingLLM's primary knowledge source. Users of both tools get structured garden knowledge inside their RAG workflows.

This is also a deliberate positioning moment. AnythingLLM is document-centric (upload PDFs, query them). Loci is garden-centric — structured ontology, temporal awareness, `Locus` crystallisation, Room namespacing, identity binding. That's the differentiation story. The integration says: "they do different things well; use both."

Since AnythingLLM supports MCP tools, this is nearly free once 1B ships — an integration guide + one verified MCP connection.

---

## What Needs to Be Built

1. **Verify MCP tool support** in AnythingLLM (current version). Confirm connection method (MCP server URL config in AnythingLLM settings).
2. **AnythingLLM integration guide** — `pipeline/features/2F-anythingllm/INTEGRATION-GUIDE.md`. Step-by-step: install AnythingLLM → start Loci desktop → connect MCP server → query garden from AnythingLLM workspace.
3. **Positioning note in docs** — a single paragraph explaining the difference: "AnythingLLM handles your documents. Loci holds your thinking. Connect them."
4. **Test matrix** — AnythingLLM versions tested, Loci MCP server version pinned.
5. **Community post** — announce the integration in AnythingLLM Discord + GitHub. Frame: "Loci is a knowledge garden source for your AnythingLLM workspace."

---

## Technical Entry Points

```
pipeline/features/2F-anythingllm/INTEGRATION-GUIDE.md  ← write this
1B MCP server at localhost:3456 (prerequisite)
AnythingLLM MCP config location: Settings → Tools → MCP Servers
```

---

## Dependencies
- **1B (Goose MCP Plugin)** — same MCP server, different client. No new code needed.

---

## Cipher's Gate
- AnythingLLM self-hosted path is clean. Cloud version (Mintplex Labs) routes data through Mintplex — integration guide must clearly recommend self-hosted.
- The integration guide's first line: "This guide assumes you're running AnythingLLM locally. The cloud version routes data through Mintplex servers."
- No new Loci code required — Cipher review is minimal.

---

## Acceptance Criteria
- [ ] AnythingLLM connects to Loci MCP server
- [ ] Loci Loci and Rooms queryable from AnythingLLM workspace
- [ ] Integration guide published with self-hosted emphasis
- [ ] Community post in AnythingLLM Discord
- [ ] Differentiation note in Loci docs

---

## Changelog
- 2026-05-09: Brief created. Blocked on 1B.

---

## First Move
> After 1B ships: install AnythingLLM locally → connect to Loci MCP server → verify tool calls → write integration guide
