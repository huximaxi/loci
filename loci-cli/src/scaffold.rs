//! Initial-palace scaffold for `loci new`.
//!
//! The starter content is embedded in the binary (not read from `templates/` at
//! runtime), so a `cargo install`ed `loci` can lay down a palace with no network
//! and no external files. This is a *minimal* working palace plus the four
//! lifecycle skills; the full templates kit (personas, more skills) is the
//! richer door at loci.garden.

use std::path::{Path, PathBuf};

/// Files written into a fresh palace, as (relative path, contents). The layout
/// is the `rooms/<room>/CLAUDE.md` shape that `palace::validate` accepts, with a
/// `PALACE.md` marker at the root.
pub const STARTER_FILES: &[(&str, &str)] = &[
    ("PALACE.md", PALACE_MD),
    ("CLAUDE.md", CLAUDE_MD),
    ("soul/SOUL.md", SOUL_MD),
    ("soul/handovers/.gitkeep", ""),
    ("rooms/general/CLAUDE.md", ROOM_MD),
    ("skills/session-open.md", SKILL_SESSION_OPEN),
    ("skills/session-close.md", SKILL_SESSION_CLOSE),
    ("skills/palace-maintenance.md", SKILL_PALACE_MAINTENANCE),
    ("skills/skill-eval.md", SKILL_SKILL_EVAL),
    ("tracker.json", TRACKER_JSON),
];

/// Write the starter palace under `root`. Creates parent directories as needed.
/// Returns the paths written, in declaration order. Callers are responsible for
/// the overwrite policy (see `main::cmd_new`); this function writes unconditionally.
pub fn write_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut written = Vec::with_capacity(STARTER_FILES.len());
    for (rel, contents) in STARTER_FILES {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, contents)?;
        written.push(path);
    }
    Ok(written)
}

const PALACE_MD: &str = r#"# [Your name]'s Palace
> Palace holder: [your name]
> Companion: (to be named)
> Gate: [name your gate], the review checkpoint for anything that ships, sends, or deletes outside this folder.

This is the palace root and the primary operating file: whose palace this is, how
it works, and where to look first. It belongs to no particular tool, so it is not
named for one. Change your AI and the palace stays; the `CLAUDE.md` beside it is
just a thin pointer for one specific agent.

## Read order (every session)
At the start of a session, read this file, then `soul/SOUL.md` if it exists, then
the newest file in `soul/handovers/`, then the room you are working in. Identity
always, current state always, the room when you enter it, deep history only when
asked.

## The gate
Nothing that ships, sends, publishes, or deletes outside this folder happens
without your yes. That is the whole trust rule: about to leave the folder, stop
and ask first. Give it a name on the `Gate:` line above, so you have one word for
it.

## What is here
- `soul/SOUL.md`: your AI's character, grown slowly over sessions.
- `soul/handovers/`: session memory, what happened and what is next.
- `rooms/<room>/CLAUDE.md`: context for one area of work.
- `skills/`: repeatable rituals (session open/close, maintenance, eval).
- `tracker.json`: what is active, what is blocked.
- `CLAUDE.md`: a thin pointer that sends Claude here first.

## How we work
- Read the palace first; act from context, not from scratch.
- Be direct and concrete: say the tradeoff, then the recommendation.
- Keep the palace current: prune what is stale, name what recurred.
- At session close, run the `session-close` skill.

## More
This is a starter palace. The full templates kit (personas, more skills) is the
richer door: loci.garden.
"#;

const CLAUDE_MD: &str = r#"# CLAUDE.md: pointer to PALACE.md

> A thin pointer for Claude. The operating instructions live in `PALACE.md`, the
> primary file, so a person who switches tools carries the palace, not this.

Read PALACE.md first. Then soul/SOUL.md if it exists. Then the newest file in
soul/handovers/. Then the room we are in.
"#;

const SOUL_MD: &str = r#"# SOUL.md — [your AI's name]

> Character, not a log. This file holds who your collaborator is and how they
> think. It grows slowly, updated only when a working principle shifts — never as
> a diary. The session narrative belongs in handovers.

## Name
[Name your collaborator here.]

## Character
[A few lines: temperament, voice, what they care about. Start small; let it grow
from how you actually work together.]

## Working principles
- Read the palace first; act from context, not from scratch.
- Precision over eloquence. Name the move before making it.
- [Add principles as they prove themselves.]
"#;

const ROOM_MD: &str = r#"# General — room context

> One room = one area of work. This is your starter room; rename it or add more
> (each room is a folder under `rooms/` with its own `CLAUDE.md`).

What should your AI know when working in this room? Fill in:
- What this area is about.
- Where things live.
- Any standing rules or preferences for this work.
"#;

const SKILL_SESSION_OPEN: &str = r#"# Skill: session-open

**When:** at the start of every session.

**Do:**
1. Read `CLAUDE.md` (who we are, how we work).
2. Read `soul/SOUL.md` (your character) and the most recent file in
   `soul/handovers/` (where we left off).
3. Skim `tracker.json` for what's active and blocked.
4. State, in a line or two, what this session is picking up.

**Output:** a short orientation — current focus + the first concrete step.

**Kill criteria:** on a fresh palace (empty handovers and tracker), skip straight
to the work and write the first handover at close.
"#;

const SKILL_SESSION_CLOSE: &str = r#"# Skill: session-close

**When:** at the end of a session, or when you sense one ending.

**Do:**
1. **Reflect** — what shifted? Decisions, new facts, patterns worth keeping.
2. **Persist** — update `soul/SOUL.md` only if a working principle shifted;
   update the relevant `rooms/<room>/CLAUDE.md` if context changed.
3. **Record** — write a handover to `soul/handovers/<date>.md`: what happened,
   decisions, open loops, what's next.
4. **Tracker** — update `tracker.json`: advance, block, or close tracks.

**Output:** a handover file, any memory updates, and a one-line summary.

**Kill criteria:** nothing changed worth carrying forward — note that and skip
the writes. Don't manufacture a handover for a no-op session.
"#;

const SKILL_PALACE_MAINTENANCE: &str = r#"# Skill: palace-maintenance

**When:** periodically, or when the palace feels stale or cluttered.

**Do:**
1. **Memory** — fold long logs into durable facts; prune what's wrong or dead.
   Keep what surprised you; let the rest decay.
2. **Rooms** — confirm each `rooms/<room>/CLAUDE.md` still matches the work;
   retire rooms you no longer use.
3. **Handovers** — the latest should reflect reality; archive old ones.
4. **Tracker** — close finished tracks, unblock what's unblocked, drop noise.
5. **Audit** — confirm what you assert against what's on disk before reporting.

**Output:** a shorter, truer palace and a note of what changed.

**Kill criteria:** the palace is small and current — a quick read is enough; do
not restructure for its own sake.
"#;

const SKILL_SKILL_EVAL: &str = r#"# Skill: skill-eval

**When:** every so often, to check how well we're actually working together and
where to grow.

**Do:**
1. Look back over the recent sessions (handovers).
2. Rate, honestly and concretely: did the palace help you start faster? Did we
   avoid re-deriving what we already knew? Where did friction repeat?
3. Name one thing working well (keep it) and one thing to change (a new rule, a
   pruned skill, a clearer room).
4. Fold the change into `CLAUDE.md` or a skill — don't just note it.

**Output:** a short, specific read + one concrete adjustment, applied.

**Kill criteria:** too few sessions to judge yet — wait until there's a pattern.
"#;

const TRACKER_JSON: &str = r#"{
  "tracks": [],
  "note": "Active work items. Each track: {id, title, status, next}. Your AI updates this; you review."
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palace;

    fn temp_dir() -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("loci-new-test-{unique}"))
    }

    /// The invariant that matters: the scaffold must be a palace `loci` can read,
    /// in the rooms-dir layout the kit builds.
    #[test]
    fn scaffold_produces_a_valid_palace() {
        let dir = temp_dir();
        write_files(&dir).unwrap();

        let p = palace::validate(&dir).expect("scaffold should be a valid palace");
        assert!(matches!(p.layout, palace::Layout::RoomsDir));
        assert!(dir.join("PALACE.md").exists());
        assert!(dir.join("CLAUDE.md").exists());
        assert!(dir.join("soul/SOUL.md").exists());
        assert!(dir.join("skills/session-close.md").exists());
        assert!(dir.join("skills/skill-eval.md").exists());

        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// Every starter file carries real content (the empty `.gitkeep` aside), so
    /// the scaffold never writes a blank placeholder.
    #[test]
    fn starter_files_are_nonempty_except_gitkeep() {
        for (rel, contents) in STARTER_FILES {
            if *rel == "soul/handovers/.gitkeep" {
                continue;
            }
            assert!(!contents.trim().is_empty(), "{rel} is empty");
        }
    }

    /// tracker.json must be valid JSON so the palace ships parseable.
    #[test]
    fn tracker_json_is_valid_json() {
        let v: serde_json::Value = serde_json::from_str(TRACKER_JSON).unwrap();
        assert!(v.get("tracks").is_some());
    }

    /// F2: `PALACE.md` is the primary, AI-agnostic operating file. It carries the
    /// operating instructions (holder, companion, read-order, gate line) that used
    /// to live in `CLAUDE.md`.
    #[test]
    fn palace_md_holds_the_operating_instructions() {
        assert!(PALACE_MD.contains("Palace holder:"), "PALACE.md needs a holder line");
        assert!(PALACE_MD.contains("Companion:"), "PALACE.md needs a companion line");
        assert!(PALACE_MD.contains("Gate:"), "PALACE.md needs a gate line");
        assert!(
            PALACE_MD.contains("read this file") && PALACE_MD.contains("soul/handovers/"),
            "PALACE.md needs the session read-order"
        );
    }

    /// F2: `CLAUDE.md` is a thin pointer, not the master prompt. It sends the reader
    /// to `PALACE.md` first and no longer carries the "who you are / who I am"
    /// substance.
    #[test]
    fn claude_md_is_a_thin_pointer_to_palace() {
        assert!(
            CLAUDE_MD.contains("Read PALACE.md first"),
            "CLAUDE.md must point at PALACE.md first"
        );
        assert!(
            !CLAUDE_MD.contains("master prompt"),
            "CLAUDE.md is no longer the master prompt"
        );
        assert!(
            !CLAUDE_MD.contains("## Who I am"),
            "the operating substance belongs in PALACE.md, not CLAUDE.md"
        );
        // A pointer, not a document: keep it short.
        assert!(
            CLAUDE_MD.lines().count() < 12,
            "CLAUDE.md should stay a thin pointer"
        );
    }

    /// Voice law: no em-dashes in the two operating templates (use a colon, a comma,
    /// or a new sentence). Guards the F2 fix to the old `CLAUDE.md — master prompt`
    /// header against regression.
    #[test]
    fn operating_templates_have_no_em_dash() {
        assert!(!PALACE_MD.contains('\u{2014}'), "PALACE.md has an em-dash");
        assert!(!CLAUDE_MD.contains('\u{2014}'), "CLAUDE.md has an em-dash");
    }
}
