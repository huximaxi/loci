import type { Platform } from '@loci/core/types';

// ─── PlatformConfig ───────────────────────────────────────────────────────────

export interface PlatformConfig {
  /** Human-readable platform name */
  name: string;
  /** Regex tested against the full URL to decide if this platform is active */
  urlPattern: RegExp;
  /** Selector for the scrollable conversation container */
  conversationContainerSelector: string;
  /** Selector for user-turn elements within the container */
  userTurnSelector: string;
  /** Selector for assistant-turn elements within the container */
  assistantTurnSelector: string;
  /** Derive a stable conversation ID from the current URL */
  conversationIdFromUrl: (url: string) => string;
}

// ─── Platform definitions ─────────────────────────────────────────────────────

const claudeConfig: PlatformConfig = {
  name: 'Claude',
  urlPattern: /^https:\/\/claude\.ai\//,
  conversationContainerSelector: '[data-testid="conversation-turn-list"]',
  userTurnSelector: '[data-testid="user-message"]',
  assistantTurnSelector: '[data-testid="assistant-message"]',
  conversationIdFromUrl(url: string): string {
    const segments = new URL(url).pathname.split('/').filter(Boolean);
    return segments[segments.length - 1] ?? '';
  },
};

const chatgptConfig: PlatformConfig = {
  name: 'ChatGPT',
  urlPattern: /^https:\/\/(chatgpt\.com|chat\.openai\.com)\//,
  // Prefer the semantic test-id; fall back to <main> if absent
  conversationContainerSelector: '[data-testid="conversation-turns"], main',
  // Prefer the explicit turn attribute; fall back to the legacy author-role attr
  userTurnSelector:
    'article[data-turn="user"], [data-message-author-role="user"]',
  assistantTurnSelector:
    'article[data-turn="assistant"], [data-message-author-role="assistant"]',
  conversationIdFromUrl(url: string): string {
    const match = new URL(url).pathname.match(/\/c\/([0-9a-f-]{36})/i);
    return match ? match[1] : '';
  },
};

// ─── PLATFORMS map ────────────────────────────────────────────────────────────

export const PLATFORMS: Record<Exclude<Platform, 'gemini' | 'unknown'>, PlatformConfig> = {
  claude: claudeConfig,
  chatgpt: chatgptConfig,
};

// ─── Helper ───────────────────────────────────────────────────────────────────

/**
 * Returns the active Platform key for a given URL, or null if no platform
 * config matches.
 */
export function getActivePlatform(url: string): Platform | null {
  for (const [key, config] of Object.entries(PLATFORMS) as [Platform, PlatformConfig][]) {
    if (config.urlPattern.test(url)) {
      return key;
    }
  }
  return null;
}
