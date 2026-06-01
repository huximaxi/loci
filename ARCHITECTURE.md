# Architecture Decision Record: loci-garden Workspace Split

> Status: DECIDED. Local workfolder only until Phase 4b is committed.
> Author: Hux × Vesper. Date: 2026-05-29.

---

## Workspace Structure

Three crates under a `loci-garden/` Cargo workspace:

```
loci-garden/
  Cargo.toml            # workspace manifest
  loci-core/            # lib crate, no Tauri deps, private
  loci-tauri/           # bin crate, Tauri app shell, private
  loci-cli/             # bin crate, LLMAGE ratatui TUI, public
```

All three share `loci-core` as a dependency. `loci-tauri` and `loci-cli` are independent binaries; neither depends on the other.

---

## What Moves to loci-core

All logic with zero Tauri imports:

- `inference/mod.rs` in full (already extraction-ready: trait-based `InferenceBackend`, reqwest/tokio/url/serde only)
- `mcp/server.rs`, `mcp/resources.rs`, `mcp/tools.rs` (no Tauri imports; `McpServerHandle` stays in loci-tauri)
- Palace and filesystem helpers: `read_palace_field`, `extract_md_section`, `extract_md_section_prefix`, `count_md_files`, `count_crystals`, `detect_rooms`, `copy_dir_recursive`, `make_slug`, `first_words`, `is_vague_heuristic`, `resolve_manifest_path`
- Grounding builders: `build_vesper_grounding`, `find_latest_handover_delta`, `build_companion_grounding`
- Task helpers: `extract_pending_tasks`, `extract_task_title`, `parse_frontmatter`
- `resolve_claude()` (zero Tauri imports, called from commands but trivially separable)
- All data structs: `Manifest`, `ManifestNode`, `ManifestRelation`, `PalaceState`, `PalaceManifest`, `RoomInfo`, `CronJobSnapshot`, `HandoverEntry`, `QuestlogItem`, `InferenceStatus`, `NameOption`, `DetectionResult`, and config structs `LociRustConfig`, `OllamaRustConfig`, `McpRustConfig`
- `notify` watcher logic, abstracted to a channel (removes the Tauri emission coupling so the watcher lives in loci-core; loci-tauri subscribes to the channel and calls `app.emit`)

---

## What Stays in loci-tauri

Only what is structurally coupled to Tauri:

- All `#[tauri::command]` wrappers (thin adapters calling loci-core functions)
- `main()` with `tauri::Builder`, plugin registration, and managed state wiring
- `OllamaState`, `McpServerHandle`, `WatcherState` managed-state structs (use `AppHandle`, `tauri::State`, `app.emit`)
- `pick_palace_dir` (requires `AppHandle` + `DialogExt`)
- `start_state_watcher` emission callback (subscribes to the channel from loci-core, calls `app.emit('state_changed', ...)`)

---

## Phase 4b Sequencing

We do not start the workspace refactor until Phase 4b is committed.

1. Commit Phase 4b (two-brain provider, Vesper grounding) on `fix/phase35-harden-viewer` in the existing `huximaxi/loci` repo. That branch must land cleanly first.
2. Open a new branch `feat/workspace-split` from the post-merge main.
3. Initialise the `loci-garden/` workspace structure on that branch. No code is deleted from the current repo until the workspace build is green.

Rationale: the uncommitted Phase 4b work is the higher-priority ship. A workspace restructure mid-flight on the same branch is a merge-conflict trap.

---

## LOCI-CORE.md Naming Resolution

`LOCI-CORE.md` in the repo root is a palace methodology changelog (structural and intellectual concepts, versioned from v0.1 to v1.2 candidate). It is not a code file.

We rename it to `PALACE-METHODOLOGY.md` before the workspace split begins. The Rust crate keeps the name `loci-core`. The conceptual mapping is consistent (the crate is the code form of the palace firmware), but distinct filenames prevent reader confusion between the methodology changelog and the Cargo crate manifest.

---

## GitHub Migration Plan

Three steps, deferred until local workspace build is stable:

1. Create `loci-garden` GitHub org. Transfer `huximaxi/loci` into it as `loci-garden/loci-tauri` (private). No history rewrite.
2. Create `loci-garden/loci-core` (private) and `loci-garden/loci-cli` (public) as new repos. Wire as git submodules or keep as a monorepo workspace (decide at transfer time based on release cadence).
3. Update CI, `DEPLOY.md`, and `loci-github-watch-daily` REPOS list to reflect the new org paths.
