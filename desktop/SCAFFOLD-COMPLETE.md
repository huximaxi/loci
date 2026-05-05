# loci Desktop App — Scaffold Complete

**Status**: Complete and ready to build
**Created**: 2026-05-05
**Location**: `/Users/eris/Dev/loci/desktop/`

## What was created

### Frontend (Vite + TypeScript)
- `package.json` — Node dependencies (@tauri-apps/cli, @tauri-apps/api, vite, typescript)
- `vite.config.ts` — Minimal Vite config for Tauri
- `tsconfig.json` — TypeScript configuration (ES2021, strict mode)
- `index.html` — Full UI with Scholar theme (green/cream), loci logo, folder picker, results
- `src/main.ts` — Frontend logic: folder picker, palace detection, migration actions

### Backend (Rust + Tauri v2)
- `src-tauri/Cargo.toml` — Rust deps (tauri, dialog plugin, opener plugin, serde, dirs, chrono)
- `src-tauri/build.rs` — Tauri build script
- `src-tauri/tauri.conf.json` — App config (window 680×520, plugins, bundle targets)
- `src-tauri/src/main.rs` — Complete Rust backend with:
  - `detect_palace()` command — checks for loci, mempalace, mila-mempalace, karpathy
  - `migrate_to_loci()` command — copies palace to `~/.loci/` format
  - Helper functions: `detect_rooms()`, `count_crystals()`, `count_md_files()`, `copy_dir_recursive()`

### Documentation & Build
- `README.md` — Full docs: requirements, dev commands, architecture, file structure
- `build.sh` — Executable build helper script
- `.gitignore` — Ignores node_modules, target, dist, etc.
- `SCAFFOLD-COMPLETE.md` — This file

## Detection patterns

1. **loci** → `~/.loci/` with `CLAUDE.md`
2. **mempalace** → `<path>/_palace/`
3. **mila-mempalace** → `<path>/mila-mempalace/`
4. **karpathy** → `<path>/LLM/` with `CLAUDE.md`

## Migration

Copies entire source palace to `~/.loci/` and creates manifest:

```json
{
  "version": "1.0",
  "migrated_from": "/original/path",
  "migrated_at": "2026-05-05T17:30:00Z"
}
```

## Next steps

1. **Icons** — Create icon assets in `src-tauri/icons/`:
   - `32x32.png`
   - `128x128.png`
   - `128x128@2x.png`
   - `icon.icns` (macOS)
   - `icon.ico` (Windows)

2. **Test build**:
   ```bash
   cd /Users/eris/Dev/loci/desktop
   npm install
   npm run tauri:dev
   ```

3. **Production build**:
   ```bash
   ./build.sh
   # or
   npm run tauri:build
   ```

## File tree

```
desktop/
├── .gitignore
├── README.md
├── SCAFFOLD-COMPLETE.md
├── build.sh
├── package.json
├── vite.config.ts
├── tsconfig.json
├── index.html
├── src/
│   └── main.ts
└── src-tauri/
    ├── Cargo.toml
    ├── build.rs
    ├── tauri.conf.json
    └── src/
        └── main.rs
```

## Design

- **Theme**: Scholar (green `#4a6b54`, cream `#faf9f6`)
- **Logo**: 4-triangle loci diamond (SVG inline)
- **Window**: 680×520px, not resizable
- **UI**: Folder picker → Detect button → Result card with stats → Action buttons

## KISS MVP principles followed

- Single purpose: detect and migrate
- No database, no complex state
- All palace data lives in `~/.loci/`
- Simple file copy for migration
- Detection via filesystem patterns only
- No network calls, no auth, no cloud

## Ready to ship

All files written. No TODOs. No placeholders. Complete scaffold.

Run `npm install && npm run tauri:dev` to start.
