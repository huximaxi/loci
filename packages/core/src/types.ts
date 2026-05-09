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

// ─── Ollama local inference config (1A) ──────────────────────────────────────
//
// OllamaConfig is the source-of-truth type for the Ollama integration.
// Stored in LociConfig and read by the Tauri backend on every command invocation.
//
// Cipher gate:
//   - base_url is validated in Rust before any HTTP call (localhost / Tailscale only)
//   - offline_mode: true → Rust returns Err("ollama_offline") immediately; no external fallback
//   - private key / API key: never present — Ollama is keyless by design

export interface OllamaConfig {
  /** Whether the Ollama integration is enabled. Default: false. */
  enabled: boolean;
  /** Ollama base URL. Default: "http://localhost:11434". Tailscale IPs (100.x.x.x) also accepted. */
  base_url: string;
  /** Chat/generation model. Default: "llama3". */
  chat_model: string;
  /** Embedding model. Default: "nomic-embed-text". */
  embed_model: string;
  /**
   * If true, Loci never falls back to an external API when Ollama is unreachable.
   * Prefer this for maximum sovereignty. Default: true.
   */
  offline_mode: boolean;
}

export const OLLAMA_DEFAULTS: OllamaConfig = {
  enabled: false,
  base_url: 'http://localhost:11434',
  chat_model: 'llama3',
  embed_model: 'nomic-embed-text',
  offline_mode: true,
} as const;

export interface LociConfig {
  version: string;
  tier: 'scholar' | 'wizard' | 'llmage';
  index: {
    autoSync: boolean;
    syncInterval: string;
    fullText: boolean;
    semantic: boolean;
  };
  /** Ollama local inference config. Replaces the legacy llm stub. */
  ollama?: OllamaConfig;
  /** Legacy: kept for config migration compatibility. Use ollama instead. */
  llm?: {
    provider: 'local' | 'claude' | 'openai';
    endpoint?: string;
    model?: string;
  };
  /** MCP server config. Replaces inline mcp stub (1B). */
  mcp?: McpConfig;
}

// ─── MCP server config (1B) ──────────────────────────────────────────────────
//
// McpConfig governs the embedded MCP server (localhost:3456).
// Served by the Tauri backend via axum. Goose, Continue.dev, Claude Code compatible.
//
// Cipher gate:
//   - port must be in range 1024–65535 (validated in Rust before bind)
//   - bind_host is always localhost — no 0.0.0.0 exposure
//   - expose_rooms: empty = expose all rooms. Non-empty = allowlist.
//   - Conversation objects NEVER exposed — THREAT-01 gate.
//   - All MCP responses carry `X-Loci-Content-Trust: user-authored` header.

export interface McpConfig {
  /** Whether the MCP server is enabled. Default: false. */
  enabled: boolean;
  /** Port to bind on localhost. Default: 3456. Must be 1024–65535. */
  port: number;
  /**
   * Room IDs to expose via MCP. Empty array = expose all rooms.
   * Set to specific IDs to restrict which rooms agents can query.
   */
  expose_rooms: string[];
}

export const MCP_DEFAULTS: McpConfig = {
  enabled: false,
  port: 3456,
  expose_rooms: [],
} as const;

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
