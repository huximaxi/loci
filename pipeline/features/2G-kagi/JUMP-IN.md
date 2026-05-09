# 2G · Kagi Web Enrichment
## Jump-In Brief

**Persona:** Rune — managed privacy framing, trust-model positioning
**Tier:** 2 · **Target:** Q4 2026
**Status:** `🔴 not-started`
**Branch:** `feat/2G-kagi`
**Last updated:** 2026-05-09

---

## Context
Kagi is a subscription-based, privacy-first search engine — no ads, no tracking, user is customer not product. Their AI agents (Kagi Assistant) provide AI-grounded search with access to KIMI, Qwen, and other models. It's in active use by sovereignty-aligned technical users (confirmed: Nym engineering team, April 2026).

Kagi occupies a different trust tier than SearXNG — it's "managed privacy" (you trust Kagi, a private company) rather than "self-hosted sovereignty" (you trust your own server). Both tiers are valid for different users. Loci offers both, and is explicit about the difference.

The integration surface is the Kagi Universal Summarizer and Search API. User supplies their own Kagi API token — Loci never proxies queries server-side. The token stays on device.

---

## What Needs to Be Built

1. **Kagi search adapter** — `search_kagi(query: string, api_token: string): SearchResult[]`. Calls Kagi Search API. Token passed per-request from OS keychain.
2. **Settings UI** — "Web enrichment" section in Wizard settings. Two options:
   - "Kagi (managed privacy)" — enter API token, stored in OS keychain
   - "SearXNG (self-hosted)" — enter instance URL
   - Explanatory text for each (see below)
3. **Agent enrichment action** — "Enrich this Locus with current information" action calls the selected search backend, appends results as a citations section.
4. **Trust model explainer** — in settings and docs:
   > **Kagi:** Your queries go to Kagi's servers. Kagi is subscription-only — they have no financial incentive to profile or sell your data. Good for users who want strong search quality without running their own infrastructure.
   >
   > **SearXNG:** Your queries go to your own server, which aggregates from multiple sources. No third party sees your queries. Requires self-hosting.

5. **No server-side proxying** — Loci desktop calls Kagi API directly from the Tauri process. User's API token never touches any Loci backend (there isn't one).

---

## Technical Entry Points

```
desktop/src-tauri/src/search.rs     ← create: Kagi + SearXNG adapters behind SearchBackend trait
desktop/src-tauri/src/keychain.rs   ← Kagi API token stored here
packages/core/src/types.ts          ← extend LociConfig with SearchConfig
```

**SearchConfig:**
```typescript
interface SearchConfig {
  backend: 'kagi' | 'searxng' | 'none';
  kagi_token?: string;      // resolved from OS keychain at runtime
  searxng_url?: string;     // e.g. "http://localhost:8888"
}
```

**Rust trait:**
```rust
trait SearchBackend: Send + Sync {
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchError>;
}

struct KagiSearch { api_token: String }
struct SearXNGSearch { base_url: String }
```

---

## Dependencies
None. Fully independent.

---

## Cipher's Gate
- **Kagi API token stored in OS keychain only.** Never in `~/.loci/config.json` or `chrome.storage.local`.
- **No server-side proxy.** The Tauri process calls Kagi directly. This is the sovereignty invariant — a future Loci backend must not become a query proxy.
- **SearXNG URL validation.** Accept only `http://localhost:*` or explicit user-entered URLs. No redirect-following to arbitrary URLs.
- **Trust model explanation is mandatory in UI.** Users must understand what "managed privacy" means before entering a Kagi token.

---

## Acceptance Criteria
- [ ] Kagi search adapter returns results via API
- [ ] SearXNG adapter works against self-hosted instance
- [ ] Settings UI shows both options with trust model explanation
- [ ] Kagi API token stored in OS keychain
- [ ] "Enrich Locus" agent action uses selected backend
- [ ] No Loci server-side proxying of queries
- [ ] Works offline gracefully (enrichment disabled, no crash)

---

## Changelog
- 2026-05-09: Brief created.

---

## First Move
> `git checkout -b feat/2G-kagi` → define `SearchBackend` Rust trait → implement `KagiSearch` → test against Kagi API with a personal token → add settings UI option
