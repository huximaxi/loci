# 1C · Nostr Keypair Identity
## Jump-In Brief

**Persona:** Cipher — key management, cryptographic identity
**Tier:** 1 · **Target:** Q3 2026
**Status:** `🔴 not-started`
**Branch:** `feat/1C-nostr-identity`
**Last updated:** 2026-05-09

---

## Context
There is no existing auth model in Loci. That's the opportunity: Nostr keypair identity becomes Loci's *first* auth model — sovereign by design, not retrofitted. A Nostr keypair (secp256k1) is just a private key and a public key. NIP-07 browser extensions (Alby, nos2x, Nostr-Tools) expose `window.nostr` which lets a page request signing without ever seeing the private key.

Auth is a feature users choose — not a tier gate. A user without a keypair can still use Loci. Auth increases internal trust, enables keypair-bound garden encryption, and is the prerequisite for everything in the Nostr publishing track (2B).

---

## Current State

No auth. The extension uses `chrome.storage.local` scoped to the extension — implicitly isolated. The Tauri desktop app uses `~/.loci/` with `config.json`. No user identity anywhere.

**Relevant files:**
- `packages/core/src/types.ts` → add `Identity` type
- `extension/src/background/service-worker.ts` → add NIP-07 auth flow
- `extension/src/sidepanel/sidepanel.ts` → "Sign in with Nostr" UI
- `desktop/src-tauri/src/main.rs` → OS keychain for nsec storage (already specced)

---

## What Needs to Be Built

1. **NIP-07 sign-in (extension)** — detect `window.nostr` on load. If present, offer "Sign in with your Nostr key" as primary onboarding option. Call `window.nostr.getPublicKey()` — user approves in their extension. Store pubkey in `chrome.storage.local`. Never ask for or store the private key.
2. **Manual nsec import (desktop)** — for users without a NIP-07 extension. Accept nsec in a one-time form, derive pubkey, store nsec in OS keychain via Tauri command (already specced as `store_api_key` pattern). Clear form immediately after import.
3. **Identity type** in core:
   ```typescript
   interface Identity {
     pubkey: string;        // hex npub
     npub: string;          // bech32 display form
     method: 'nip07' | 'imported';
     createdAt: number;
   }
   ```
4. **Garden binding** — when identity is set, garden data is logically "owned" by the pubkey. Not enforced cryptographically in v1 (that's a follow-on), but the association is stored in config.
5. **"Bring your own key" onboarding moment** (Nyx spec) — primary option in Scholar onboarding step 1, before email/password options. Quiet, not pushy.
6. **No-auth path preserved** — if user skips keypair, Loci works identically. Auth state is surfaced in a small indicator, not in the main UI.

---

## Technical Entry Points

```
extension/src/sidepanel/sidepanel.ts    ← NIP-07 sign-in button + flow
extension/src/background/service-worker.ts  ← message handler for auth
packages/core/src/types.ts              ← Identity interface
desktop/src-tauri/src/keychain.rs       ← nsec storage via OS keychain
```

**NIP-07 flow:**
```typescript
// In sidepanel.ts
async function connectNostr() {
  if (!window.nostr) {
    showMessage("Install a Nostr extension (Alby, nos2x) to use keypair sign-in");
    return;
  }
  const pubkey = await window.nostr.getPublicKey(); // user approves in extension
  const npub = nip19.npubEncode(pubkey);
  await chrome.storage.local.set({ identity: { pubkey, npub, method: 'nip07' } });
  showIdentityBadge(npub);
}
```

---

## Dependencies
None. Ships independently. 2B (Nostr Zaps) depends on this.

---

## Cipher's Gate
- **Private key never touches Loci's code.** NIP-07 ensures this for browser extension users. For nsec import: write to OS keychain immediately, zero in-memory retention after write, clear UI form on submit.
- **No backend storage of pubkeys.** Identity lives in `chrome.storage.local` (extension) and `~/.loci/config.json` (desktop). Not on any server.
- **Session binding:** If pubkey changes (user switches Nostr accounts), garden state should surface this — don't silently mix identities.

---

## Acceptance Criteria
- [ ] NIP-07 sign-in works with Alby and nos2x extensions
- [ ] Private key never accessible to Loci's JavaScript context
- [ ] `Identity` type stored in `chrome.storage.local`
- [ ] "Bring your own key" is the first option in Scholar onboarding
- [ ] No-auth path works identically — auth is optional
- [ ] nsec import (desktop) stores via OS keychain, clears form immediately
- [ ] Identity badge shown in UI when authenticated
- [ ] Nostr session persists across browser restarts

---

## Changelog
- 2026-05-09: Brief created. Auth principle confirmed: feature, not gate.

---

## First Move
> `git checkout -b feat/1C-nostr-identity` → add `Identity` type to `packages/core/src/types.ts` → implement `connectNostr()` in sidepanel → test with Alby extension installed
