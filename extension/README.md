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

See [Cipher security analysis](../../loci-cipher-security-analysis.md).
