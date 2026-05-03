// ─── loci core types ─────────────────────────────────────────────────────────

export type Platform = 'claude' | 'chatgpt' | 'gemini' | 'unknown';

export type TurnRole = 'user' | 'assistant';

export interface Turn {
  role: TurnRole;
  text: string;
  timestamp: number;
}

export interface Conversation {
  id: string;           // platform:uuid  e.g. "claude:abc123"
  platform: Platform;
  url: string;
  title: string;        // First user message (truncated to 80 chars) or page title
  turns: Turn[];
  tags: string[];
  roomId?: string;      // Assigned room (wizard+ tier)
  createdAt: number;
  updatedAt: number;
  indexed: boolean;
  sanitized?: boolean;  // Content has been sanitized for prompt injection
}

export interface SearchResult {
  id: string;
  title: string;
  platform: Platform;
  excerpt: string;
  score: number;
  createdAt: number;
  tags: string[];
}

export interface Tag {
  name: string;
  color?: string;
  conversationCount: number;
}

export interface Locus {
  id: string;           // locus-{date}-{slug}
  slug: string;
  title: string;
  content: string;      // Markdown body
  roomId?: string;
  tags: string[];
  sourceConversationId?: string;
  createdAt: number;
}

export interface Room {
  id: string;
  name: string;
  displayName: string;
  color: string;        // CSS hex colour
  glowColor: string;
  contextMd: string;    // Contents of room's CLAUDE.md / context.md
  conversationCount: number;
  locusCount: number;
  createdAt: number;
}

export interface LociConfig {
  version: string;
  tier: 'scholar' | 'wizard' | 'llmage';
  index: {
    autoSync: boolean;
    syncInterval: string;
    fullText: boolean;
    semantic: boolean;
  };
  llm?: {
    provider: 'local' | 'claude' | 'openai';
    endpoint?: string;
    model?: string;
  };
  mcp?: {
    exposeRooms: string[];
    port: number;
  };
}

// ─── IndexedDB schema ────────────────────────────────────────────────────────

export const DB_NAME = 'loci-db';
export const DB_VERSION = 1;

export const STORES = {
  CONVERSATIONS: 'conversations',
  TAGS: 'tags',
  LOCI: 'loci',
} as const;

// ─── Chrome storage keys ─────────────────────────────────────────────────────

export const STORAGE_KEYS = {
  SEARCH_INDEX: 'loci-search-index',
  TAG_MAP: 'loci-tag-map',
  CONFIG: 'loci-config',
  LAST_SYNC: 'loci-last-sync',
} as const;

// ─── Message types (extension internal) ──────────────────────────────────────

export type ExtensionMessage =
  | { type: 'CONVERSATION_INDEXED'; conversation: Conversation }
  | { type: 'SEARCH'; query: string; tags?: string[] }
  | { type: 'SEARCH_RESULTS'; results: SearchResult[] }
  | { type: 'TAG_CONVERSATION'; conversationId: string; tag: string }
  | { type: 'UNTAG_CONVERSATION'; conversationId: string; tag: string }
  | { type: 'GET_TAGS' }
  | { type: 'TAGS_LIST'; tags: Tag[] }
  | { type: 'OPEN_CONVERSATION'; conversationId: string }
  | { type: 'SYNC_REQUEST' }
  | { type: 'SYNC_COMPLETE'; count: number };
