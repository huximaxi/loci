# loci — Chrome Extension

Chrome MV3 browser extension. Local-first search + tagging for AI conversations.

## Supported platforms
- Claude.ai
- ChatGPT (chatgpt.com / chat.openai.com)

## Build

```bash
npm install
npm run build    # outputs to dist/
npm run typecheck
```

## Load in Chrome

1. Open chrome://extensions
2. Enable Developer Mode
3. "Load unpacked" → select dist/

## Architecture

See [architecture docs](https://docs.loci.garden/architecture).

## Security

loci is designed with privacy-first principles:

- **Zero network calls** — the extension makes no external requests
- **Local storage only** — all conversation data stays in your browser (IndexedDB)
- **Content sanitization** — indexed text is sanitized before storage
- **Sender validation** — internal messages are validated to prevent injection
- **Open source** — verify the build yourself: `npm run build` from source

For security concerns: hux@nymtech.net
