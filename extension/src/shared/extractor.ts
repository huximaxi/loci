// ─── loci shared extraction utilities ────────────────────────────────────────
//
// Used by both claude.ts and chatgpt.ts content scripts.
// Responsible for DOM → Turn[] scraping, Conversation assembly, and the
// fire-and-forget message dispatch to the service worker.

import type { Turn, TurnRole, Conversation, Platform } from '@loci/core/types';
import type { PlatformConfig } from '../platforms/platforms.config';

// ─── Content sanitization ─────────────────────────────────────────────────────

/**
 * Sanitizes extracted text to mitigate prompt injection risks.
 * Strips residual HTML and neutralizes patterns that could confuse downstream LLMs.
 */
function sanitizeText(raw: string): string {
  return raw
    .replace(/<[^>]*>/g, '')                    // strip any HTML tags
    .replace(/\[IGNORE\s/gi, '[')               // defang injection attempts
    .replace(/\bSYSTEM\s*:/gi, 'SYSTEM_MSG:')   // neutralize system prompts
    .trim();
}

// ─── extractTurns ─────────────────────────────────────────────────────────────

/**
 * Scrapes all user and assistant turns from a conversation container element.
 *
 * Querying is done with the selectors defined in `PlatformConfig`, so this
 * function is platform-agnostic — the same logic works for Claude.ai and
 * ChatGPT.
 *
 * Deduplication: if the last turn already in the accumulator has the same
 * trimmed text as the candidate, the candidate is skipped. This prevents
 * phantom duplicates that arise from streaming (the element gets mutated in
 * place, but the observer may fire multiple times).
 *
 * @param container - The conversation container element returned by
 *   `document.querySelector(config.conversationContainerSelector)`.
 * @param config    - Platform configuration containing the turn selectors.
 * @returns An ordered array of `Turn` objects. Returns an empty array when the
 *   container has no matching elements.
 */
export function extractTurns(container: Element, config: PlatformConfig): Turn[] {
  const turns: Turn[] = [];

  // Build a map of element → role by querying each selector separately.
  // We need to preserve DOM order, so we walk the full container tree once
  // and assign roles based on which selector set the element matches.
  const userElements = new Set(
    Array.from(container.querySelectorAll(config.userTurnSelector))
  );
  const assistantElements = new Set(
    Array.from(container.querySelectorAll(config.assistantTurnSelector))
  );

  // Walk the container in DOM order so turns come out chronologically.
  // querySelectorAll guarantees document order within the container.
  const allTurnElements = Array.from(
    container.querySelectorAll(
      `${config.userTurnSelector}, ${config.assistantTurnSelector}`
    )
  );

  for (const el of allTurnElements) {
    // Determine role — user takes precedence if an element somehow matches both
    // (shouldn't happen in practice, but defensive).
    let role: TurnRole;
    if (userElements.has(el)) {
      role = 'user';
    } else if (assistantElements.has(el)) {
      role = 'assistant';
    } else {
      // Element matched the compound selector but not either individual set —
      // shouldn't be possible, but skip rather than crash.
      continue;
    }

    const rawText = (el as HTMLElement).innerText?.trim() ?? '';

    // Skip empty turns (collapsed, loading, or hidden elements).
    if (!rawText) continue;

    // Sanitize to mitigate prompt injection risks
    const text = sanitizeText(rawText);

    // Deduplication: skip if last accumulated turn has the same text.
    // This handles streaming — the same element fires multiple mutations while
    // tokens arrive; we only want the final settled state.
    if (turns.length > 0 && turns[turns.length - 1].text === text) {
      continue;
    }

    turns.push({
      role,
      text,
      // Real timestamps aren't available from the DOM; use now() as a stable
      // sequential marker. The service worker merges on timestamp, so we derive
      // a per-turn offset from the index to keep each turn unique.
      timestamp: Date.now() + turns.length,
    });
  }

  return turns;
}

// ─── buildConversation ────────────────────────────────────────────────────────

/**
 * Assembles a `Conversation` object from extracted turns and metadata.
 *
 * ID strategy:
 *   - If the URL contains a platform-specific conversation ID (e.g. a UUID in
 *     the path), the id is formatted as `platform:uuid`.
 *   - Otherwise (new chat, `/` root, etc.) we fall back to
 *     `platform:crypto.randomUUID()`.
 *
 * Title strategy:
 *   - First user turn text truncated to 80 characters.
 *   - Fallback: "Untitled conversation".
 *
 * @param platform   - Platform key (`'claude'` or `'chatgpt'`).
 * @param url        - Current `window.location.href`.
 * @param turns      - Turns returned by `extractTurns`.
 * @param existingId - If we already assigned an ID for this conversation in a
 *   prior extraction cycle, pass it here so the ID stays stable across
 *   incremental updates.
 */
export function buildConversation(
  platform: Platform,
  url: string,
  turns: Turn[],
  existingId?: string
): Conversation {
  const now = Date.now();

  // Derive a stable conversation ID from the URL path segment.
  // Each platform config exposes `conversationIdFromUrl` for this.
  // We can't call that here without a config reference, so we do a generic
  // UUID-segment parse and let callers pass existingId to ensure stability.
  const urlId = conversationIdFromUrl(url);
  const id = existingId ?? (urlId ? `${platform}:${urlId}` : `${platform}:${crypto.randomUUID()}`);

  // First user turn → title (truncated to 80 chars).
  const firstUserTurn = turns.find((t) => t.role === 'user');
  const title = firstUserTurn
    ? firstUserTurn.text.slice(0, 80)
    : 'Untitled conversation';

  return {
    id,
    platform,
    url,
    title,
    turns,
    tags: [],
    indexed: false,
    sanitized: true,
    createdAt: now,
    updatedAt: now,
  };
}

// ─── sendToBackground ─────────────────────────────────────────────────────────

/**
 * Fire-and-forget dispatch of a `CONVERSATION_INDEXED` message to the service
 * worker.  Errors are logged to the console but never rethrown — a failure here
 * must never interrupt the host page.
 *
 * @param conversation - The assembled conversation to persist.
 */
export function sendToBackground(conversation: Conversation): void {
  chrome.runtime.sendMessage({ type: 'CONVERSATION_INDEXED', conversation })
    .catch((err: unknown) => {
      // Expected on pages where the extension context is invalidated (e.g.
      // extension updated mid-session) or when the service worker is starting up.
      console.warn('[loci] sendToBackground failed:', err);
    });
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

/**
 * Attempts to extract a UUID-shaped or hex conversation ID from a URL path.
 * Returns an empty string when the URL doesn't contain a recognisable ID
 * (e.g. `https://chatgpt.com/` or `https://claude.ai/new`).
 *
 * This is a generic fallback used inside `buildConversation`. Content scripts
 * that have access to a `PlatformConfig` should prefer
 * `config.conversationIdFromUrl(url)` which is selector-aware.
 */
function conversationIdFromUrl(url: string): string {
  try {
    const { pathname } = new URL(url);
    // Match a UUID (chatgpt /c/<uuid>) or a long hex/alphanumeric slug (claude)
    const match = pathname.match(
      /\/(?:c\/)?([0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}|[0-9a-zA-Z_-]{20,})/i
    );
    return match ? match[1] : '';
  } catch {
    return '';
  }
}
