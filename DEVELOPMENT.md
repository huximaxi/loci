# loci desktop — Development Guide

This is the technical guide for building and contributing to the loci desktop app (the Tauri palace viewer). For governance, values, and the contributor charter, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Prerequisites

- **Rust** (stable, via [rustup](https://rustup.rs)). Minimum 1.75.
- **Node.js** 18+
- **Tauri CLI**: bundled as a dev dependency — no global install needed.

## Build

```bash
cd desktop
npm install              # installs Tauri CLI and JS deps
npm run tauri:dev        # dev mode: live-reload, opens app window
npm run tauri:build      # release build: bundles into src-tauri/target/release/bundle/
```

The app window opens at 680×520px by default.

## Structure

```
desktop/
  src-leptos/      Leptos (Rust/WASM) frontend: views, models, Tauri bindings
  src-tauri/       Tauri backend: Rust commands, palace bridge, managed state
  index.html       Entry point served by Tauri
  package.json     Tauri CLI wrapper + JS dev dependencies
```

The frontend is written in Rust using [Leptos](https://leptos.dev), compiled to WASM. There is no React, Vue, or TypeScript in the rendering layer.

## Palace bridge

The app communicates with the palace (your local knowledge base) via Tauri commands defined in `src-tauri/src/`. Key commands:

- `validate_palace_path` — checks if a directory is a valid palace (PALACE.md or _palace/ present)
- `read_palace_summary` — room and crystal counts
- `build_vesper_grounding` — assembles L0/L1/L3 context for inference
- `find_latest_handover` — locates the most recent handover file

Palace detection follows the `GardenKind` spec: it reads `PALACE.md` first (loci-native), then falls back to legacy markers.

## Testing with a local palace

The test palace is at `~/Dev/Moci-MOC/`. It has a CLAUDE.md and a `_palace/` directory, and it passes `validate_palace_path`.

To test the palace bridge manually:
1. `npm run tauri:dev`
2. In the app, select `~/Dev/Moci-MOC` as your palace path
3. Both local and remote inference backends should surface in the brain-toggle

## TCC (macOS privacy permissions)

`is_dir()` and `exists()` do not reliably detect inaccessible directories on macOS — they return `false` rather than an error when TCC blocks access. Use `lstat` + `ErrorKind::PermissionDenied` checks and propagate `PermissionDenied` to the caller; never silently collapse to empty.

This is field-validated: `stat` passes the TCC wall, `readdir` does not. Any new palace traversal code must follow the same pattern as existing palace detection.

## Submitting a PR

- Branch from `main`. `feat/<slug>`, `fix/<slug>`, `doc/<slug>`.
- One concern per PR. No mixed refactor + feature commits.
- Test manually with the Moci-MOC palace and with `~/Garden of Hux` (the full palace, if accessible).
- `src-tauri` changes that touch the palace bridge need a corresponding test in `loci-core` if the logic is extractable.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full charter: provenance disclosure, agent contributions, what the zoku will not do.

---

*loci desktop is tended by the Rainbow Zoku. See [loci.garden](https://loci.garden) for the garden.*
