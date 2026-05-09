# 2B · Nostr Publishing + Zaps
## Jump-In Brief

**Persona:** Nyx — νύξ · UX of value flows
**Tier:** 2 · **Target:** Q4 2026
**Status:** `⏸ blocked on 1C`
**Branch:** `feat/2B-nostr-zaps`
**Last updated:** 2026-05-09

---

## Context
Once a user has a Nostr identity (1C), their knowledge can flow back to the protocol. Publishing a Locus as a signed Nostr event means it's censorship-resistant, permanent, and can receive Lightning zaps — micropayments from readers who find it valuable. This is the value-flow vision: knowledge that earns for the thinker.

The UX must feel effortless. "Publish to Nostr" is one option alongside "copy link" — not a setting, not a wizard, just a share action. The note is signed with the user's keypair (via NIP-07), exists on the protocol forever, and the user's audience on Nostr can discover it.

---

## What Needs to Be Built

1. **NIP-01 event publishing** — sign a Locus as a kind:1 (text note) or kind:30023 (long-form article) Nostr event using `window.nostr.signEvent()`. Broadcast to 2-3 default relays (wss://relay.damus.io, wss://nos.lol, user-configurable).
2. **"Publish to Nostr" share action** — appears in Locus context menu and share panel. Shows relay list, confirms before publishing.
3. **NIP-57 zap support** — add `zap` metadata to published events (LNURL or bolt11). User supplies their Lightning address in settings (e.g. `user@getalby.com`). Zap button appears on public garden view for published Loci.
4. **Published badge** — small indicator on Locus cards showing "Published to Nostr" + event ID link (opens in njump.me or user's preferred Nostr client).
5. **Zap receipt display** — when zaps are received, surface them in a lightweight "zap feed" in the garden. Not a notification flood — a quiet stream.

---

## Technical Entry Points

```
extension/src/sidepanel/sidepanel.ts    ← "Publish to Nostr" action in Locus menu
packages/core/src/types.ts             ← extend Locus with nostrEventId?, zapAddress?
extension/src/shared/nostr.ts          ← create: NIP-01 event building, relay pool
```

**Event building:**
```typescript
// extension/src/shared/nostr.ts
async function publishLocus(locus: Locus): Promise<string> {
  const event = {
    kind: 30023,  // NIP-23 long-form article
    content: locus.content,
    tags: [
      ['title', locus.title],
      ['d', locus.slug],
      ['t', ...locus.tags],
    ],
    created_at: Math.floor(Date.now() / 1000),
  };
  const signed = await window.nostr.signEvent(event);
  // broadcast to relay pool
  return signed.id;
}
```

---

## Dependencies
- **1C (Nostr Identity)** — must be authenticated to sign events. NIP-07 is the signing interface.

---

## Cipher's Gate
- Private key never touches Loci — NIP-07 handles signing.
- Relay connections are outbound only. No inbound relay connections in the extension.
- Zap LNURL is user-supplied. Validate format. Never auto-populate from untrusted sources.
- Published event IDs stored locally (in Locus metadata) — no server-side record needed.

---

## Acceptance Criteria
- [ ] "Publish to Nostr" action appears in Locus context menu
- [ ] Events signed via NIP-07, broadcast to configurable relays
- [ ] Published Locus shows event ID badge with njump.me link
- [ ] NIP-57 zap address configurable in settings
- [ ] Zap feed shown in garden (quiet, non-intrusive)
- [ ] Private key never in Loci's JavaScript context

---

## Changelog
- 2026-05-09: Brief created. Blocked on 1C.

---

## First Move
> After 1C ships: create `extension/src/shared/nostr.ts` → implement `publishLocus()` → test signing with Alby → test broadcast to relay.damus.io → add share action to Locus UI
