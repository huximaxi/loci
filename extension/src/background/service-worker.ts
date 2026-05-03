// ─── loci service worker ─────────────────────────────────────────────────────
//
// The brain of the loci extension. Handles all cross-context messaging,
// orchestrates IndexedDB persistence, and keeps the MiniSearch index in sync.
//
// Design principle: the service worker can be killed at any time by the browser.
// All durable state lives in IndexedDB or chrome.storage.local and is
// reconstructed on demand — nothing is assumed to survive between events.

import {
  type ExtensionMessage,
  type Conversation,
  type Tag,
} from '@loci/core/types';

import {
  openDB,
  saveConversation,
  getConversation,
  getAllConversations,
  saveTag,
  getAllTags,
} from '../shared/db';

import {
  getOrLoadIndex,
  saveIndex,
  buildIndex,
  searchIndex,
} from '../shared/search';

// ─── Helpers ──────────────────────────────────────────────────────────────────

/**
 * Derives a plain IndexDoc-compatible object for a single conversation so we
 * can patch it into the index without rebuilding everything from scratch.
 */
function convToIndexDoc(conv: Conversation) {
  const firstAssistant = conv.turns.find((t) => t.role === 'assistant');
  return {
    id: conv.id,
    title: conv.title,
    platform: conv.platform,
    excerpt: firstAssistant ? firstAssistant.text.slice(0, 200) : '',
    createdAt: conv.createdAt,
    tags: conv.tags.join(' '),
  };
}

/**
 * Updates or inserts a single conversation in the live index without rebuilding
 * the whole thing.  MiniSearch doesn't support "upsert" natively, so we remove
 * the old document (if it exists) and add the new one.
 */
async function patchIndex(conv: Conversation): Promise<void> {
  const index = await getOrLoadIndex();

  // Discard by id — MiniSearch v7 API. Safe to call even if the document isn't
  // in the index yet (no-op in that case, no throw).
  index.discard(conv.id);

  index.add(convToIndexDoc(conv));
  await saveIndex(index);
}

// ─── Message handler ──────────────────────────────────────────────────────────

chrome.runtime.onMessage.addListener(
  (
    message: ExtensionMessage,
    _sender: chrome.runtime.MessageSender,
    sendResponse: (response: unknown) => void
  ) => {
    // We need to return `true` to signal that sendResponse will be called
    // asynchronously (standard Chrome MV3 pattern).
    handleMessage(message, sendResponse);
    return true;
  }
);

async function handleMessage(
  message: ExtensionMessage,
  sendResponse: (response: unknown) => void
): Promise<void> {
  switch (message.type) {

    // ── CONVERSATION_INDEXED ─────────────────────────────────────────────────
    // A content script has scraped and serialised a conversation. Persist it
    // to IndexedDB and update the search index.
    case 'CONVERSATION_INDEXED': {
      try {
        const conv: Conversation = message.conversation;
        await saveConversation(conv);
        await patchIndex(conv);
        sendResponse({ success: true });
      } catch (err) {
        console.error('[loci:sw] CONVERSATION_INDEXED error:', err);
        sendResponse({ success: false, error: String(err) });
      }
      break;
    }

    // ── SEARCH ───────────────────────────────────────────────────────────────
    // The side panel or popup is asking for search results.
    // Empty query returns the 20 most recent conversations instead of running
    // MiniSearch (which would return nothing for an empty string).
    case 'SEARCH': {
      try {
        if (!message.query || message.query.trim() === '') {
          const allConvs = await getAllConversations();
          // Sort descending by createdAt and take the 20 most recent
          const recent = allConvs
            .sort((a, b) => b.createdAt - a.createdAt)
            .slice(0, 20);
          const results = recent.map((conv) => {
            const firstAssistant = conv.turns.find((t) => t.role === 'assistant');
            return {
              id: conv.id,
              title: conv.title,
              platform: conv.platform,
              excerpt: firstAssistant ? firstAssistant.text.slice(0, 200) : '',
              createdAt: conv.createdAt,
              tags: conv.tags,
              score: 1,
            };
          });
          sendResponse({ type: 'SEARCH_RESULTS', results });
        } else {
          const index = await getOrLoadIndex();
          const results = searchIndex(index, message.query, message.tags);
          sendResponse({ type: 'SEARCH_RESULTS', results });
        }
      } catch (err) {
        console.error('[loci:sw] SEARCH error:', err);
        sendResponse({ type: 'SEARCH_RESULTS', results: [] });
      }
      break;
    }

    // ── TAG_CONVERSATION ─────────────────────────────────────────────────────
    // Add a tag to a conversation and keep the index in sync.
    case 'TAG_CONVERSATION': {
      try {
        const { conversationId, tag } = message;
        const conv = await getConversation(conversationId);

        if (!conv) {
          sendResponse({ success: false, error: 'Conversation not found' });
          break;
        }

        // Idempotent — don't add duplicates
        if (!conv.tags.includes(tag)) {
          conv.tags = [...conv.tags, tag];
        }

        await saveConversation(conv);

        // Ensure there's a Tag record for this name
        const allTags = await getAllTags();
        const existingTag = allTags.find((t) => t.name === tag);
        if (existingTag) {
          await saveTag({
            ...existingTag,
            conversationCount: existingTag.conversationCount + 1,
          });
        } else {
          await saveTag({ name: tag, conversationCount: 1 });
        }

        // Patch the search index entry so tag filters work immediately
        await patchIndex(conv);
        sendResponse({ success: true });
      } catch (err) {
        console.error('[loci:sw] TAG_CONVERSATION error:', err);
        sendResponse({ success: false, error: String(err) });
      }
      break;
    }

    // ── UNTAG_CONVERSATION ───────────────────────────────────────────────────
    // Remove a tag from a conversation.
    case 'UNTAG_CONVERSATION': {
      try {
        const { conversationId, tag } = message;
        const conv = await getConversation(conversationId);

        if (!conv) {
          sendResponse({ success: false, error: 'Conversation not found' });
          break;
        }

        conv.tags = conv.tags.filter((t) => t !== tag);
        await saveConversation(conv);

        // Decrement tag counter (floor at 0)
        const allTags = await getAllTags();
        const existingTag = allTags.find((t) => t.name === tag);
        if (existingTag) {
          await saveTag({
            ...existingTag,
            conversationCount: Math.max(0, existingTag.conversationCount - 1),
          });
        }

        await patchIndex(conv);
        sendResponse({ success: true });
      } catch (err) {
        console.error('[loci:sw] UNTAG_CONVERSATION error:', err);
        sendResponse({ success: false, error: String(err) });
      }
      break;
    }

    // ── GET_TAGS ─────────────────────────────────────────────────────────────
    // Return the full tag list (used by the side panel tag picker).
    case 'GET_TAGS': {
      try {
        const tags: Tag[] = await getAllTags();
        sendResponse({ type: 'TAGS_LIST', tags });
      } catch (err) {
        console.error('[loci:sw] GET_TAGS error:', err);
        sendResponse({ type: 'TAGS_LIST', tags: [] });
      }
      break;
    }

    // ── OPEN_CONVERSATION ────────────────────────────────────────────────────
    // Open a stored conversation URL in a new tab.
    case 'OPEN_CONVERSATION': {
      try {
        const conv = await getConversation(message.conversationId);
        if (!conv) {
          sendResponse({ success: false, error: 'Conversation not found' });
          break;
        }
        await chrome.tabs.create({ url: conv.url });
        sendResponse({ success: true });
      } catch (err) {
        console.error('[loci:sw] OPEN_CONVERSATION error:', err);
        sendResponse({ success: false, error: String(err) });
      }
      break;
    }

    // ── SYNC_REQUEST ─────────────────────────────────────────────────────────
    // Rebuild the full search index from IndexedDB. Used after bulk imports or
    // when the index is suspected to be stale.
    case 'SYNC_REQUEST': {
      try {
        const conversations = await getAllConversations();
        const freshIndex = buildIndex(conversations);
        await saveIndex(freshIndex);
        sendResponse({ type: 'SYNC_COMPLETE', count: conversations.length });
      } catch (err) {
        console.error('[loci:sw] SYNC_REQUEST error:', err);
        sendResponse({ type: 'SYNC_COMPLETE', count: 0 });
      }
      break;
    }

    default: {
      // Unrecognised message type — log and ignore
      console.warn('[loci:sw] Unknown message type:', (message as { type: string }).type);
      sendResponse({ success: false, error: 'Unknown message type' });
    }
  }
}

// ─── Extension lifecycle ──────────────────────────────────────────────────────

chrome.runtime.onInstalled.addListener(async (details) => {
  console.log('[loci] installed — reason:', details.reason);

  // Warm the DB connection so the first message doesn't pay the open cost
  try {
    await openDB();
    console.log('[loci] IndexedDB ready');
  } catch (err) {
    console.error('[loci] Failed to initialise IndexedDB:', err);
  }
});

// ─── Side panel trigger ───────────────────────────────────────────────────────

// Open the loci side panel when the user clicks the extension action icon.
// chrome.sidePanel is available in Chrome 114+.
chrome.action.onClicked.addListener(async (tab) => {
  if (tab.id == null) return;

  try {
    await chrome.sidePanel.open({ tabId: tab.id });
  } catch (err) {
    // Graceful fallback — side panel API may not be enabled in all contexts
    console.error('[loci] Failed to open side panel:', err);
  }
});
