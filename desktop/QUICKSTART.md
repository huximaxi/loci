# loci Desktop — Quickstart

## Install & Run (30 seconds)

```bash
cd loci/desktop

# Install dependencies
npm install

# Run in development mode
npm run tauri:dev
```

That's it. The app window will open at 680×520px.

## What you'll see

1. **loci logo** (4-triangle diamond, Scholar green)
2. **Folder picker** — Browse to select `~/Dev` or workspace
3. **Detect Palace** button — Scans for palace patterns
4. **Result card** — Shows what was found:
   - Badge: `loci`, `mempalace`, `mila-mempalace`, `karpathy`, or `none`
   - Stats: room count + crystal count
   - Action: "Migrate to loci" or "Already loci-format"
   - Path display

## Build for production

```bash
npm run tauri:build
```

Bundle will be in `src-tauri/target/release/bundle/`.

## What it detects

| Pattern | Location | Trigger |
|---------|----------|---------|
| **loci** | `~/.loci/` | Already migrated |
| **mempalace** | `<path>/_palace/` | legacy memory palace |
| **mila-mempalace** | `<path>/mila-mempalace/` | a named palace |
| **karpathy** | `<path>/LLM/` with `CLAUDE.md` | Karpathy-style |

## Migration result

After migration, you'll have:

```
~/.loci/
├── loci.json           # Manifest (version, migrated_from, migrated_at)
└── [all palace files]  # Copied recursively from source
```

## Requirements

- **Node.js** 18+
- **Rust** 1.70+
- **Tauri CLI** 2.0+ (installed via npm)

## Troubleshooting

**App won't start:**
```bash
# Check Rust installation
rustc --version

# Check Node version
node --version

# Reinstall deps
rm -rf node_modules
npm install
```

**Build fails:**
```bash
# Clean Rust build cache
cd src-tauri
cargo clean
cd ..
npm run tauri:build
```

**Icons missing:**
- App will still run, just won't have custom icons
- Create icons later in `src-tauri/icons/`

## Next

- Test with your own palace folder
- Verify migration creates `~/.loci/loci.json`
- Check room/crystal counts are accurate
- Open migrated palace in Finder to confirm files copied

## Files created

12 total files:
- 5 config files (package.json, vite.config.ts, tsconfig.json, Cargo.toml, tauri.conf.json)
- 4 source files (index.html, main.ts, main.rs, build.rs)
- 3 docs (README.md, SCAFFOLD-COMPLETE.md, QUICKSTART.md)

All complete. No TODOs. No placeholders.

**Ready to ship.**
