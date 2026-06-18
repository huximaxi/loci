# Build a room

A room is a domain of work. Rooms keep a large palace legible: when you enter one, exactly
its context loads, and nothing else.

Uses: `templates/room-template.md`, `templates/retrieval-hierarchy.md`,
`templates/local-map-template.md`, `templates/_PALACE_CONTEXT.md`, `templates/tracker.json`

## Steps

1. Decide your rooms (work, writing, a side project). Copy `room-template.md` into
   `rooms/<name>/CLAUDE.md` for each.
2. In each room file, set a `session_strategy`: `always-load`, `on-demand`, or
   `per-session`. This is what tells your AI when to pull the room in.
3. Add the room to the room table in your main `CLAUDE.md` so your AI can state the room
   at session open.
4. Keep room files room-scoped only (the two-layer rule): behavioural constants live in the
   global layer, living state in the project layer, room specifics here.
5. Optional: draw a `local-map` of how the rooms connect, and track open work in
   `tracker.json`.

## You are done when

You open a session, name a room, and only that room's context loads (L0-L3). The palace
stops dumping everything at once.

## Next

`your-first-crystal.md` (add room-scoped facts) · `close-and-reopen.md` (hand a room off
to the next session).

---

*loci · [loci.garden](https://loci.garden)*
