# Keep your memory yours

loci gives you a memory you own: plain text you can read, move, and delete. The risk is the
tool you point at it keeping its own memory of you, server-side, learned across chats, that
you cannot see or erase. This flow closes that gap so the substrate stays the only memory of
record. Two memories of you is one too many, and only one is yours.

Uses: `templates/CLAUDE-master.md` (the gate and memory rules), `templates/.lociignore`

## Steps

1. **Make the substrate the only memory.** If your tool offers its own memory that learns
   you across sessions, turn it off. The substrate already remembers, on your terms, and
   nothing about you should accumulate in a place you cannot open.
2. **Tier what you expose.** Everything in an always-loaded file is sent every session. Keep
   the sensitive facts out of the always-on map and into rooms that load only when that work
   is open. Load-on-demand is a privacy setting, not only a token one.
3. **Keep the outward gate** (see `set-a-gate.md`). The gate you set for shipping code is the
   same gate for what your companion transmits about you: no names, no secrets, no private
   shape of your thinking leaves without your yes. Scrub before anything reaches a vendor
   endpoint or a public surface.
4. **Own the deletion.** Because the substrate is your backup, the tool's copy is disposable:
   wipe it whenever you like and lose nothing that matters. One caution: some vendor resets
   are irreversible, so glance at the store first and port anything that lived only there.

### If your companion is Claude

loci federates to no one and works across every tool. When the tool is Claude, four concrete
settings hold the line:

- **Memory** (claude.ai → Settings → Memory): turn off *Generate memory from chats* and
  *Search and reference chats*. The substrate is your memory now.
- **Training and retention** (claude.ai → data controls): turn off *Improve Claude for
  everyone*. This keeps you on the short retention window and out of the training set.
- **Quiet the client** (Claude Code, `~/.claude/settings.json` `env`): `DISABLE_TELEMETRY`,
  `DISABLE_ERROR_REPORTING`, `DISABLE_FEEDBACK_COMMAND`. These are presence-based, so `"0"`
  still enables the disable: unset to reverse. Prefer the named flags over the umbrella
  `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`, which also switches off auto security updates.
- **Sensitive one-offs**: use a temporary chat, which is not saved and not learned from.

The floor, if a tier of your thinking must never leave at all, is a local model pointed at
the same plain-text substrate. Same blueprint, no wire.

## You are done when

The tool you work through holds no memory of you that you cannot see, move, or delete, and
the only durable record of your mind is the plain text you own. The vendor forgets; the
substrate remembers.

## Next

`set-a-gate.md` (the outward gate this leans on) · `go-cross-tool.md` (the same ownership when
memory travels between tools).

---

*loci · [loci.garden](https://loci.garden)*
