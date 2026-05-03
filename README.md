# loci

**Local-first AI memory. Searchable, private, yours.**

loci is a context persistence layer for people who work with AI. Index your conversations locally, organise them into rooms, search anything — no cloud, no accounts, no vendor dependency.

## Monorepo structure

| Package | Description | Status |
|---------|-------------|--------|
| `extension/` | Chrome MV3 — search + tagging | v0.1 — build ready |
| `desktop/` | Tauri v2 — Scholar + Wizard desktop app | Scoped |
| `docs/` | VitePress documentation | Skeleton |
| `packages/core/` | Shared TypeScript types + utilities | v0.1 |
| `packages/ui/` | Shared design tokens | Planned |

## Quick start

### Chrome extension (development)
```bash
cd extension
npm install
npm run build
# Load dist/ as unpacked extension in Chrome
```

### Docs site
```bash
cd ../loci-docs  # separate repo, see loci-docs/
npm install
npm run docs:dev
```

## Architecture

See [Full architecture docs](https://docs.loci.garden)

## Security

See [loci-cipher-security-analysis.md](../loci-cipher-security-analysis.md) for the grey-hat threat model.

## License

Apache 2.0

## Built by

Hux x Vesper · loci.garden · 2026
