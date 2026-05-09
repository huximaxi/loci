# Cipher · Roadmap HTML Audit
*Date: 2026-05-09 · Status: CLEARED WITH NOTES*

---

## Audit Summary

The roadmap.html is **cleared to publish** with the following notes actioned or flagged for follow-up. No sovereignty contradictions found. No false claims. Security gate language is accurate.

---

## Checks Performed

### 1. Sovereignty claims accuracy
- ✓ "Your private key never touches Loci's code" — accurate for NIP-07 path (1C). Accurate when implemented correctly.
- ✓ "If Ollama goes offline, Loci fails closed" — accurate per 1A spec. Verify implementation enforces this.
- ✓ "Fail-closed: if Nym routing fails, sync pauses" — accurate per 2D spec. Non-negotiable invariant is documented.
- ✓ "Loci is Loci's first external API — built sovereign-first" — accurate; no prior API surface exists.
- ✓ "No observer, including Nym's relays, can see what you're syncing" — accurate description of mixnet cover traffic. Acceptable.

### 2. Partner claims accuracy
- ✓ Kagi: "subscription, no ads, no tracking, user is customer" — accurate per known Kagi model. Not open source — this is not claimed in the page.
- ✓ IPFS: "accessible without Loci's servers" — accurate for content-addressed data.
- ✓ Nym: "Loci protects the content... Nym protects the fact" — accurate framing of the complementary layers.
- ✓ AT Protocol / Bluesky: page says "AT Protocol" not "Bluesky" — correct protocol-layer framing.

### 3. Auth principle
- ✓ Banner text: "Auth is a feature, not a gate. Sovereignty is never paywalled." — accurate per confirmed decision. Correct.

### 4. THREAT-01 gate
- ✓ Feature 1B correctly shows `security` tag with "THREAT-01 gate" label.
- ⚠ **NOTE FOR SHIPPING:** The roadmap page does not explain *what* THREAT-01 gates (conversation context vs Locus context). This is intentional for public-facing copy — but internal docs (JUMP-IN brief) must remain accurate. Current JUMP-IN is accurate.

### 5. Horizon tier
- ✓ Bittensor/Commune listed as "Watching" — correct. Not endorsed, not dismissed.
- ✓ No claims about decentralized inference sovereignty — just watching signal.

### 6. No overstatements
- ✓ 1D (Nym Partnership): "Loci commits to metadata-private sync (Q4)" — commitment, not claim of completion. Accurate.
- ✓ 2H (Tailscale): "Foundation for autonomous palace instances" — aspirational framing with no false technical claims. Acceptable.

---

## Issues to Fix Before Publishing

**None blocking.** One recommended edit:

**Recommended:** In the manifesto pull-quote section, the attribution reads "Cognitive Sovereignty Manifesto · Loci × The Sovereignty Stack · 2026." Once Rune's full manifesto is finalised, this should be replaced with the exact opening line from that document for consistency. Flag: update before site goes live.

---

## Cipher's Sign-Off

> Architecture is defensible. Sovereignty claims are accurate. Security gate (THREAT-01) is correctly labelled. Fail-closed invariants are documented. Kagi trust tier is correctly described as "managed privacy" — not equivalent to self-hosted, and this distinction is preserved in the copy.
>
> **CLEARED.** Publish after Rune's manifesto is integrated.

*— Cipher, 2026-05-09*
