// ─── loci content script — Claude.ai ─────────────────────────────────────────
//
// Injected into every claude.ai tab. Observes the conversation container for
// DOM mutations and ships extracted turns to the service worker whenever the
// conversation settles.
//
// Claude.ai is a React SPA. The conversation container persists across URL
// changes within the same "chat" route, but navigating to a new conversation
// replaces it entirely. We handle this with a lightweight URL-polling loop that
// re-attaches the observer whenever the page route changes.

import { PLATFORMS } from '../platforms/platforms.config';
import { extractTurns, buildConversation, sendToBackground } from '../shared/extractor';

// ─── State ────────────────────────────────────────────────────────────────────

const config = PLATFORMS['claude'];

/** The stable ID assigned to the conversation currently in view. */
let currentConversationId: string | null = null;

/** Pending debounce handle; cleared on each new mutation batch. */
let extractionTimeout: ReturnType<typeof setTimeout> | null = null;

/** Active MutationObserver; disconnected and replaced on each navigation. */
let observer: MutationObserver | null = null;

/** Last seen href — used by the URL polling loop to detect SPA navigation. */
let lastHref: string = location.href;

// ─── extractAndSend ───────────────────────────────────────────────────────────

/**
 * Reads the current DOM state, assembles a `Conversation`, and dispatches it to
 * the service worker. Called after every debounced mutation batch and on the
 * initial container attachment.
 */
function extractAndSend(): void {
  const container = document.querySelector(config.conversationContainerSelector);
  if (!container) return;

  const turns = extractTurns(container, config);
  if (turns.length === 0) return;

  // Re-use the ID we assigned earlier in this navigation so incremental updates
  // are merged correctly by the service worker.
  const conversation = buildConversation(
    'claude',
    location.href,
    turns,
    currentConversationId ?? undefined
  );

  // Lock in the ID for the duration of this conversation session.
  if (!currentConversationId) {
    currentConversationId = conversation.id;
  }

  sendToBackground(conversation);
  console.log(`[loci] indexed ${turns.length} turns from Claude.ai`);
}

// ─── observeConversation ──────────────────────────────────────────────────────

/**
 * Locates the conversation container and attaches a MutationObserver to it.
 *
 * If the container isn't present yet (page still loading, navigation in flight)
 * we retry after 1 000 ms, up to a maximum of 20 attempts. Each call
 * disconnects the previous observer first, so it's safe to call this on every
 * URL change.
 */
function observeConversation(retryCount = 0): void {
  // Tear down any previous observer before re-attaching.
  if (observer) {
    observer.disconnect();
    observer = null;
  }

  const container = document.querySelector(config.conversationContainerSelector);

  if (!container) {
    if (retryCount >= 20) {
      console.log('[loci] container not found after 20 retries, giving up');
      return;
    }
    // Container not ready — retry shortly. This covers the window between a
    // SPA navigation firing and React committing the new conversation DOM.
    setTimeout(() => observeConversation(retryCount + 1), 1_000);
    return;
  }

  // Do an immediate extraction so we capture any turns already in the DOM when
  // the observer first attaches (e.g. returning to an existing conversation).
  extractAndSend();

  observer = new MutationObserver(() => {
    // Debounce: wait for the mutation storm (streaming tokens) to settle before
    // extracting. 800 ms is long enough for normal token velocity; it can be
    // reduced if latency becomes noticeable.
    if (extractionTimeout !== null) {
      clearTimeout(extractionTimeout);
    }
    extractionTimeout = setTimeout(() => {
      extractionTimeout = null;
      extractAndSend();
    }, 800);
  });

  observer.observe(container, { childList: true, subtree: true });
}

// ─── Navigation handling ──────────────────────────────────────────────────────

/**
 * Called whenever we detect a URL change. Resets per-conversation state and
 * re-attaches the observer for the new conversation.
 */
function handleNavigation(): void {
  // Clear state for the previous conversation.
  currentConversationId = null;
  if (extractionTimeout !== null) {
    clearTimeout(extractionTimeout);
    extractionTimeout = null;
  }

  observeConversation();
}

/**
 * Polls `location.href` every 2 s to catch SPA navigations that don't emit
 * `popstate` (e.g. `history.pushState` calls from React Router).
 *
 * `popstate` covers browser back/forward; the poll covers programmatic
 * navigation. Both are cheap and together cover all cases.
 */
function startNavigationWatcher(): void {
  // popstate fires on browser history navigation (back/forward buttons, etc.)
  window.addEventListener('popstate', () => {
    lastHref = location.href;
    handleNavigation();
  });

  // Poll for pushState-based navigation (the common case in Claude.ai).
  setInterval(() => {
    if (location.href !== lastHref) {
      lastHref = location.href;
      handleNavigation();
    }
  }, 2_000);
}

// ─── Init ─────────────────────────────────────────────────────────────────────

function init(): void {
  startNavigationWatcher();
  observeConversation();
}

if (document.readyState === 'complete' || document.readyState === 'interactive') {
  init();
} else {
  document.addEventListener('DOMContentLoaded', init, { once: true });
}
