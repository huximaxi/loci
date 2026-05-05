# loci Desktop App

Tauri v2 desktop application for detecting and migrating memory palace setups to loci format.

## What it does

One job: **Detect existing memory palaces and migrate them to `~/.loci/` format.**

Detects:
- **loci** — Already migrated to `~/.loci/`
- **mempalace** — Vesper × Hux `_palace/` pattern
- **mila-mempalace** — Mila's palace structure
- **karpathy** — Karpathy-style `LLM/` folder

## Requirements

- Node.js 18+
- Rust 1.70+
- Tauri CLI 2.0+

## Development

```bash
# Install dependencies
npm install

# Run in dev mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

## Build script

```bash
./build.sh
```

## Architecture

- **Frontend**: Vite + TypeScript + Scholar theme (green/cream)
- **Backend**: Rust + Tauri v2
- **Plugins**: `tauri-plugin-dialog`, `tauri-plugin-opener`

## Files

```
desktop/
├── package.json           # Node dependencies
├── vite.config.ts         # Vite config
├── tsconfig.json          # TypeScript config
├── index.html             # UI (Scholar theme)
├── src/
│   └── main.ts            # Frontend logic
├── src-tauri/
│   ├── Cargo.toml         # Rust dependencies
│   ├── build.rs           # Build script
│   ├── tauri.conf.json    # App config
│   └── src/
│       └── main.rs        # Rust backend (detection + migration)
└── build.sh               # Build helper
```

## Detection logic

1. Check `~/.loci/` → already migrated
2. Check `<search_path>/_palace/` → Vesper × Hux palace
3. Check `<search_path>/mila-mempalace/` → Mila palace
4. Check `<search_path>/LLM/` → Karpathy pattern
5. None found → offer to create new

## Migration

Copies entire palace structure to `~/.loci/` and creates `loci.json` manifest:

```json
{
  "version": "1.0",
  "migrated_from": "/Users/you/Dev/_palace",
  "migrated_at": "2026-05-05T17:30:00Z"
}
```

## Window

- **Size**: 680×520px (not resizable)
- **Theme**: Scholar (green `#4a6b54` / cream `#faf9f6`)
- **Logo**: 4-triangle loci diamond

## Next

- Icon assets (`icons/` folder)
- Palace creation wizard
- Multi-palace support
- Garden integration
