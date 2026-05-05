# Installing loci · AI memory

loci is a local-first search engine for your AI conversations. These instructions cover installation on Chrome, Chromium, or Edge browsers.

## Quick Install (from ZIP)

The fastest way to get started:

1. Download the latest release from [GitHub Releases](https://github.com/huximaxi/loci/releases)
2. Unzip the file to a folder you'll keep (don't move or delete it later — the extension reads from this location)
3. Open `chrome://extensions` in your browser
4. Toggle "Developer mode" on (switch in the top right corner)
5. Click "Load unpacked"
6. Select the unzipped folder and confirm
7. Pin loci to your toolbar for quick access (click the puzzle icon, find loci, click the pin)

You're ready. Navigate to Claude.ai or ChatGPT and open the loci side panel with the toolbar icon.

## Build from Source

If you want to build loci yourself:

**Prerequisites:**
- Node.js 18 or later

**Steps:**

1. Clone the repository:
   ```bash
   git clone https://github.com/huximaxi/loci.git
   cd loci
   ```

2. Install dependencies:
   ```bash
   cd extension
   npm install
   ```

3. Build the extension:
   ```bash
   npm run build
   ```

4. Load the extension:
   - Open `chrome://extensions`
   - Enable "Developer mode" (top right)
   - Click "Load unpacked"
   - Select the `extension/dist` folder
   - Pin to toolbar

## Permissions Explained

loci asks for a few permissions. Here's why, in plain English:

**Claude.ai, ChatGPT, and OpenAI Chat access**
- loci reads the text of your conversations on these sites so it can index them locally for search. This is the core feature: knowing what you've discussed without re-uploading data anywhere.

**Storage**
- loci stores an index of your conversations on your computer using browser storage. This is how search stays fast and private — everything lives in your browser, not on a server.

**Unlimited Storage**
- As you chat more, your conversation index grows. This permission lets loci store as much as needed without limits.

**Tabs**
- loci needs to know which conversation tab you're viewing so it can stay in sync when you switch between chats.

**Side Panel**
- This is the search interface itself — the side panel that opens when you click the loci icon.

**Active Tab**
- When you search, loci can highlight results by jumping to the conversation and highlighting the matching text. This needs to interact with the active tab.

**All data stays local.** Nothing is sent to external servers. Your conversations are indexed in your browser only. You can verify this by checking your network requests — loci makes no outbound calls related to your conversation data.

## Troubleshooting

**Extension doesn't appear in the toolbar**
- Refresh the page you're on (Claude.ai or ChatGPT)
- Check that Developer mode is still enabled at `chrome://extensions`
- Try unpinning and repinning the extension

**Conversations aren't showing up in search**
- Refresh the Claude.ai or ChatGPT page
- Open a conversation and wait a moment for loci to index it
- Check the side panel to confirm the conversation appears in the list

**Search results are empty or incomplete**
- Make sure you've scrolled through the conversation (loci indexes visible content)
- Try clicking "Rebuild Index" in the side panel settings
- Refresh the page and try again

**Extension says "Manifest error" or "Failed to load"**
- Go to `chrome://extensions`
- Remove loci
- Re-download the latest release ZIP
- Follow the "Quick Install" steps above

**Still stuck?**
- Open an issue on [GitHub](https://github.com/huximaxi/loci/issues) with a screenshot and description
- Include your browser version (chrome://version)

---

That's it. Enjoy searching through your conversations.
