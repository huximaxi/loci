# 2C · AT Protocol DID Support
## Jump-In Brief

**Persona:** Cipher — identity, trust chains, DID resolution
**Tier:** 2 · **Target:** Q4 2026
**Status:** `🔴 not-started`
**Branch:** `feat/2C-at-protocol`
**Last updated:** 2026-05-09

---

## Context
AT Protocol (the open protocol underlying Bluesky) uses DIDs (Decentralized Identifiers) as portable identity. A `did:plc` or `did:web` is your persistent identity that survives platform changes. This is the social discovery layer for Loci: AT Protocol DID as an identity option (alongside Nostr keypair), plus "Share to Bluesky" as a publishing action.

**Critical Cipher note:** Integrate at the AT Protocol level, not the Bluesky company level. Bluesky the company is capturable; the AT Protocol spec is not. DID resolution, lexicon contributions, and the `com.atproto.*` namespace are the integration surface — not Bluesky's proprietary endpoints.

---

## What Needs to Be Built

1. **AT Protocol DID identity option** — resolve a user's AT handle or DID to a DID document. Store DID in Identity config alongside (or instead of) Nostr pubkey. Users can have both.
2. **DID resolution** — `resolveHandle(handle: string): Promise<DID>` via `https://bsky.social/xrpc/com.atproto.identity.resolveHandle` (or user-specified PDS).
3. **"Share to Bluesky" action** — create an AT Protocol record via `com.atproto.repo.createRecord`. Post a link + excerpt from a published Locus to the user's Bluesky feed. Auth via App Password (not OAuth in v1).
4. **Loci lexicon** — define a `garden.loci.note` lexicon for knowledge-garden objects. Contribution to AT Protocol community schemas. This makes Loci content natively understandable by AT Protocol clients.
5. **Garden feed** (stretch) — expose a Loci garden as an AT Protocol feed generator. Users can follow a Loci garden from any AT Protocol client.

---

## Technical Entry Points

```
packages/core/src/types.ts            ← extend Identity with atDid?, atHandle?
extension/src/shared/at-protocol.ts  ← create: DID resolution, record creation
pipeline/features/2C-at-protocol/lexicon/garden.loci.note.json  ← lexicon definition
```

**DID resolution:**
```typescript
async function resolveAtHandle(handle: string): Promise<string> {
  const resp = await fetch(
    `https://bsky.social/xrpc/com.atproto.identity.resolveHandle?handle=${handle}`
  );
  const { did } = await resp.json();
  return did;
}
```

**Lexicon draft:**
```json
{
  "$type": "com.atproto.lexicon.schema",
  "id": "garden.loci.note",
  "defs": {
    "main": {
      "type": "record",
      "description": "A crystallised insight node in a Loci knowledge garden",
      "key": "tid",
      "record": {
        "type": "object",
        "required": ["title", "content"],
        "properties": {
          "title": { "type": "string", "maxLength": 200 },
          "content": { "type": "string", "description": "Markdown" },
          "tags": { "type": "array", "items": { "type": "string" } },
          "gardenUri": { "type": "string", "format": "at-uri" }
        }
      }
    }
  }
}
```

---

## Dependencies
None — independent of Nostr identity track.

---

## Cipher's Gate
- **Protocol layer only.** DID resolution via `com.atproto.identity.*` endpoints. Avoid Bluesky-specific endpoints (`app.bsky.*`) in the core identity flow.
- **App Password handling** — if using App Password for auth, store in OS keychain (same pattern as nsec). Never in extension storage.
- **PDS agnosticism** — allow user to specify their own PDS URL, not hardcode `bsky.social`. A user on a self-hosted PDS must work identically.

---

## Acceptance Criteria
- [ ] AT handle resolves to DID
- [ ] DID stored in Identity config
- [ ] "Share to Bluesky" posts a link + excerpt to user's AT Protocol feed
- [ ] App Password stored in OS keychain
- [ ] Works against custom PDS (not just bsky.social)
- [ ] `garden.loci.note` lexicon drafted and submitted to AT Protocol community
- [ ] No dependency on Bluesky-proprietary endpoints in core identity flow

---

## Changelog
- 2026-05-09: Brief created.

---

## First Move
> `git checkout -b feat/2C-at-protocol` → create `extension/src/shared/at-protocol.ts` → implement `resolveAtHandle()` → test against bsky.social and a self-hosted PDS → extend Identity type
