// ─── loci MiniSearch wrapper ─────────────────────────────────────────────────
//
// Provides a lightweight full-text search layer over stored conversations.
// The index is serialised to chrome.storage.local so it survives service-worker
// restarts without requiring a full rebuild from IndexedDB on every wake.

import MiniSearch from 'minisearch';
import type { Conversation, SearchResult } from '@loci/core/types';
import { STORAGE_KEYS } from '@loci/core/types';

// ─── Index document shape ─────────────────────────────────────────────────────

interface IndexDoc {
  id: string;
  title: string;
  platform: string;
  excerpt: string;
  createdAt: number;
  tags: string; // space-joined for full-text match
}

// ─── MiniSearch configuration (shared between create and load) ────────────────

const MINISEARCH_OPTIONS: ConstructorParameters<typeof MiniSearch<IndexDoc>>[0] = {
  fields: ['title', 'excerpt'],
  storeFields: ['id', 'title', 'platform', 'excerpt', 'createdAt', 'tags'],
  searchOptions: {
    fuzzy: 0.2,
    prefix: true,
    boost: { title: 2 },
  },
};

// ─── Index lifecycle ──────────────────────────────────────────────────────────

/** Creates and returns a fresh, empty MiniSearch index. */
export function createIndex(): MiniSearch<IndexDoc> {
  return new MiniSearch<IndexDoc>(MINISEARCH_OPTIONS);
}

/**
 * Builds a populated index from an array of conversations.
 * Derives an `excerpt` from the first 200 characters of the first assistant
 * turn (falls back to an empty string if no assistant turn exists).
 */
export function buildIndex(conversations: Conversation[]): MiniSearch<IndexDoc> {
  const index = createIndex();

  const docs: IndexDoc[] = conversations.map((conv) => {
    const firstAssistantTurn = conv.turns.find((t) => t.role === 'assistant');
    const excerpt = firstAssistantTurn
      ? firstAssistantTurn.text.slice(0, 200)
      : '';

    return {
      id: conv.id,
      title: conv.title,
      platform: conv.platform,
      excerpt,
      createdAt: conv.createdAt,
      tags: conv.tags.join(' '),
    };
  });

  index.addAll(docs);
  return index;
}

// ─── Serialisation ────────────────────────────────────────────────────────────

/** Serialises a MiniSearch index to a JSON string for storage. */
export function serializeIndex(index: MiniSearch<IndexDoc>): string {
  return JSON.stringify(index.toJSON());
}

/**
 * Deserialises a JSON string back into a live MiniSearch instance.
 * Uses the same options object so the index behaves identically.
 */
export function deserializeIndex(json: string): MiniSearch<IndexDoc> {
  return MiniSearch.loadJSON<IndexDoc>(json, MINISEARCH_OPTIONS);
}

// ─── Search ───────────────────────────────────────────────────────────────────

/**
 * Executes a search against `index`.
 *
 * @param index      - The live MiniSearch instance to query.
 * @param query      - The user's search string.
 * @param tagFilter  - Optional list of tags; only results that carry ALL of the
 *                     specified tags are returned.
 * @returns          - Typed SearchResult array, sorted by MiniSearch score.
 */
export function searchIndex(
  index: MiniSearch<IndexDoc>,
  query: string,
  tagFilter?: string[]
): SearchResult[] {
  const raw = index.search(query);

  return raw
    .filter((hit) => {
      if (!tagFilter || tagFilter.length === 0) return true;
      // `tags` is stored as a space-joined string — split back to array
      const hitTags = (hit.tags as string).split(' ').filter(Boolean);
      return tagFilter.every((t) => hitTags.includes(t));
    })
    .map((hit) => ({
      id: hit.id as string,
      title: hit.title as string,
      platform: hit.platform as SearchResult['platform'],
      excerpt: hit.excerpt as string,
      score: hit.score,
      createdAt: hit.createdAt as number,
      tags: (hit.tags as string).split(' ').filter(Boolean),
    }));
}

// ─── Chrome storage persistence ───────────────────────────────────────────────

/**
 * Loads the serialised index from `chrome.storage.local`.
 * Returns a fresh empty index if nothing has been saved yet.
 */
export async function getOrLoadIndex(): Promise<MiniSearch<IndexDoc>> {
  return new Promise((resolve) => {
    chrome.storage.local.get(STORAGE_KEYS.SEARCH_INDEX, (items) => {
      const stored = items[STORAGE_KEYS.SEARCH_INDEX] as string | undefined;
      if (stored) {
        try {
          resolve(deserializeIndex(stored));
        } catch (err) {
          // Corrupted index — start fresh rather than crashing
          console.error('[loci:search] Failed to deserialise index, resetting:', err);
          resolve(createIndex());
        }
      } else {
        resolve(createIndex());
      }
    });
  });
}

/**
 * Serialises `index` and writes it to `chrome.storage.local`.
 */
export async function saveIndex(index: MiniSearch<IndexDoc>): Promise<void> {
  return new Promise((resolve, reject) => {
    const serialised = serializeIndex(index);
    chrome.storage.local.set({ [STORAGE_KEYS.SEARCH_INDEX]: serialised }, () => {
      if (chrome.runtime.lastError) {
        reject(new Error(chrome.runtime.lastError.message));
      } else {
        resolve();
      }
    });
  });
}
