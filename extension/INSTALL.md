# Installing loci Chrome Extension

## Developer Preview (v1.2)

Until loci is on the Chrome Web Store, install it in developer mode:

1. **Download the extension**
   - Click the green "Code" button on this page → "Download ZIP"
   - Extract the ZIP to a folder (e.g. `~/Downloads/loci-extension/`)

2. **Build the extension** (requires Node.js 18+)
   ```bash
   cd loci-extension/extension
   npm install
   npm run build
   ```
   This creates a `dist/` folder with the built extension.

3. **Load in Chrome**
   - Open Chrome and go to `chrome://extensions`
   - Enable **Developer Mode** (toggle in top-right corner)
   - Click **"Load unpacked"**
   - Select the `dist/` folder
   - loci should appear in your extensions list

4. **Start using loci**
   - Visit Claude.ai or ChatGPT
   - loci starts indexing automatically
   - Press `Cmd+K` (Mac) or `Ctrl+K` (Windows) to search

## Permissions explanation

loci requests:
- `storage` + `unlimitedStorage` — stores your conversation index locally
- `scripting` + `activeTab` — reads conversation text from AI chat pages
- `sidePanel` — enables the persistent side panel
- `tabs` — opens conversations in new tabs when you click search results
- Host permissions for claude.ai and chatgpt.com — reads conversations there

**Nothing is transmitted externally. All data stays in your browser.**
