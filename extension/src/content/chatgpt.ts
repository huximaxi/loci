// ─── loci content script — ChatGPT ───────────────────────────────────────────
//
// Injected into every chatgpt.com / chat.openai.com tab. Observes the
// conversation container for DOM mutations and ships extracted turns to the
// service worker whenever the conversation settles.
//
// ChatGPT specifics that differ from the Claude.ai script:
//
//   1. Lazy rendering — React may not have committed the conversation container
//      when the URL change event fires. We therefore apply a 1 500 ms delay
//      before attaching the observer after each navigation (on top of the
//      normal 1 000 ms retry for a missing container).
//
//   2. Empty-route guard — navigating to `/` or `/new` produces a URL whose
//      `conversationIdFromUrl` returns an empty string. We skip extraction in
//      those cases rather than assigning a random ID to a blank page.

import { PLATFORMS } from '../platforms/platforms.config';
import { extractTurns, buildConversation, sendToBackground } from '../shared/extractor';

// ─── State ────────────────────────────────────────────────────────────────────

const config = PLATFORMS['chatgpt'];

/** The stable ID assigned to the conversation currently in view. */
let currentConversationId: string | null = null;

/** Pending debounce handle; cleared on each new mutation batch. */
let extractionTimeout: ReturnType<typeof setTimeout> | null = null;

/** Active MutationObserver; disconnected and replaced on each navigation. */
let observer: MutationObserver | null = null;

/** Last seen href — used by the URL polling loop to detect SPA navigation. */
let lastHref: string = location.href;

// ─── Empty-route guard ────────────────────────────────────────────────────────

/**
 * Returns `true` when the current URL represents a blank new-chat page where
 * there is nothing to extract.
 *
 * ChatGPT routes that produce an empty `conversationIdFromUrl`:
 *   - `https://chatgpt.com/`
 *   - `https://chatgpt.com/new`
 *   - `https://chat.openai.com/`
 */
function isNewChatPage(url: string): boolean {
  return config.conversationIdFromUrl(url) === '';
}

// ─── extractAndSend ───────────────────────────────────────────────────────────

/**
 * Reads the current DOM state, assembles a `Conversation`, and dispatches it to
 * the service worker.
 *
 * Skipped silently when on a new-chat / empty-route page.
 */
function extractAndSend(): void {
  // Don't attempt extraction on blank pages — there are no turns to capture and
  // assigning an ID here would create a spurious record.
  if (isNewChatPage(location.href)) return;

  const container = document.querySelector(config.conversationContainerSelector);
  if (!container) return;

  const turns = extractTurns(container, config);
  if (turns.length === 0) return;

  const conversation = buildConversation(
    'chatgpt',
    location.href,
    turns,
    currentConversationId ?? undefined
  );

  if (!currentConversationId) {
    currentConversationId = conversation.id;
  }

  sendToBackground(conversation);
  console.log(`[loci] indexed ${turns.length} turns from ChatGPT`);
}

// ─── observeConversation ──────────────────────────────────────────────────────

/**
 * Locates the conversation container and attaches a MutationObserver.
 *
 * If the container isn't present (React hasn't rendered yet) we retry after
 * 1 000 ms, up to a maximum of 20 attempts. The caller already waits 1 500 ms
 * before calling this after a navigation event, so the typical total wait is
 * ≤ 2 500 ms — acceptable given that the user is still reading or typing.
 */
function observeConversation(retryCount = 0): void {
  if (observer) {
    observer.disconnect();
    observer = null;
  }

  // Don't bother attaching on blank pages.
  if (isNewChatPage(location.href)) return;

  const container = document.querySelector(config.conversationContainerSelector);

  if (!container) {
    if (retryCount >= 20) {
      console.log('[loci] container not found after 20 retries, giving up');
      return;
    }
    setTimeout(() => observeConversation(retryCount + 1), 1_000);
    return;
  }

  // Capture any turns already rendered (e.g. returning to a previous chat via
  // the sidebar).
  extractAndSend();

  observer = new MutationObserver(() => {
    // Debounce: ChatGPT streams tokens aggressively; 800 ms gives the assistant
    // message time to settle before we snapshot it.
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
 * Called whenever a URL change is detected. Resets per-conversation state and
 * re-attaches the observer after giving React time to render the new route.
 *
 * The 1 500 ms delay is the key ChatGPT-specific behaviour — without it we
 * consistently attach before React has committed the conversation container and
 * end up in the retry loop anyway.
 */
function handleNavigation(): void {
  currentConversationId = null;
  if (extractionTimeout !== null) {
    clearTimeout(extractionTimeout);
    extractionTimeout = null;
  }

  // Disconnect the old observer immediately so we don't fire on the
  // unmounting/remounting churn that happens during React route transitions.
  if (observer) {
    observer.disconnect();
    observer = null;
  }

  // Wait for React to finish rendering the new conversation before attaching.
  setTimeout(observeConversation, 1_500);
}

/**
 * Starts the dual-mode navigation watcher: `popstate` for browser
 * back/forward, plus a 2 s poll for `history.pushState` navigations (sidebar
 * clicks, new-chat button, etc.).
 */
function startNavigationWatcher(): void {
  window.addEventListener('popstate', () => {
    lastHref = location.href;
    handleNavigation();
  });

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

  // On initial page load, apply the same 1 500 ms courtesy delay so we don't
  // race against the first React render.
  if (isNewChatPage(location.href)) {
    // New-chat or root page — nothing to observe yet; the navigation watcher
    // will fire when the user starts or opens a conversation.
    return;
  }

  setTimeout(observeConversation, 1_500);
}

if (document.readyState === 'complete' || document.readyState === 'interactive') {
  init();
} else {
  document.addEventListener('DOMContentLoaded', init, { once: true });
}
