// ─── loci IndexedDB wrapper ──────────────────────────────────────────────────
//
// All functions return Promises and are safe to call from the service worker
// context. The DB is opened lazily and the connection is cached for the
// lifetime of the service-worker activation cycle.

import {
  type Conversation,
  type Tag,
  type Locus,
  DB_NAME,
  DB_VERSION,
  STORES,
} from '@loci/core/types';

// ─── Internal connection cache ────────────────────────────────────────────────

let _db: IDBDatabase | null = null;

/**
 * Opens (or returns the cached) loci-db connection.
 * Creates the three object stores and their indices on first run.
 */
export function openDB(): Promise<IDBDatabase> {
  if (_db) return Promise.resolve(_db);

  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;

      // ── conversations ──────────────────────────────────────────────────────
      if (!db.objectStoreNames.contains(STORES.CONVERSATIONS)) {
        const convStore = db.createObjectStore(STORES.CONVERSATIONS, {
          keyPath: 'id',
        });
        convStore.createIndex('by_platform', 'platform', { unique: false });
        convStore.createIndex('by_updatedAt', 'updatedAt', { unique: false });
        convStore.createIndex('by_roomId', 'roomId', { unique: false });
      }

      // ── tags ───────────────────────────────────────────────────────────────
      if (!db.objectStoreNames.contains(STORES.TAGS)) {
        db.createObjectStore(STORES.TAGS, { keyPath: 'name' });
      }

      // ── loci ───────────────────────────────────────────────────────────────
      if (!db.objectStoreNames.contains(STORES.LOCI)) {
        db.createObjectStore(STORES.LOCI, { keyPath: 'id' });
      }
    };

    request.onsuccess = (event) => {
      _db = (event.target as IDBOpenDBRequest).result;

      // Reset the cache if the connection is unexpectedly closed
      _db.onclose = () => {
        _db = null;
      };

      resolve(_db);
    };

    request.onerror = () => {
      reject(new Error(`Failed to open ${DB_NAME}: ${request.error?.message}`));
    };
  });
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/** Wraps an IDBRequest in a Promise. */
function promisifyRequest<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

// ─── Conversations ────────────────────────────────────────────────────────────

/**
 * Persists a conversation to IndexedDB.
 *
 * If a record with the same `id` already exists, incoming turns are merged
 * with the stored turns — deduplication is based on `timestamp`. The
 * `updatedAt` field is always set to `Date.now()`.
 */
export async function saveConversation(conv: Conversation): Promise<void> {
  const db = await openDB();

  // Load the existing record first so we can merge turns
  const existing = await getConversation(conv.id);

  let merged: Conversation;
  if (existing) {
    // Build a Set of known timestamps to avoid duplicates
    const knownTimestamps = new Set(existing.turns.map((t) => t.timestamp));
    const newTurns = conv.turns.filter((t) => !knownTimestamps.has(t.timestamp));

    merged = {
      ...existing,
      ...conv,
      turns: [...existing.turns, ...newTurns],
      updatedAt: Date.now(),
    };
  } else {
    merged = { ...conv, updatedAt: Date.now() };
  }

  const tx = db.transaction(STORES.CONVERSATIONS, 'readwrite');
  await promisifyRequest(tx.objectStore(STORES.CONVERSATIONS).put(merged));
}

/** Returns a single conversation by its `id`, or `undefined` if not found. */
export async function getConversation(
  id: string
): Promise<Conversation | undefined> {
  const db = await openDB();
  const tx = db.transaction(STORES.CONVERSATIONS, 'readonly');
  const result = await promisifyRequest<Conversation | undefined>(
    tx.objectStore(STORES.CONVERSATIONS).get(id)
  );
  return result;
}

/** Returns every stored conversation. */
export async function getAllConversations(): Promise<Conversation[]> {
  const db = await openDB();
  const tx = db.transaction(STORES.CONVERSATIONS, 'readonly');
  return promisifyRequest<Conversation[]>(
    tx.objectStore(STORES.CONVERSATIONS).getAll()
  );
}

/**
 * Returns the `limit` most recently updated conversations, ordered by
 * `updatedAt` descending.
 */
export async function getRecentConversations(
  limit: number
): Promise<Conversation[]> {
  const db = await openDB();

  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORES.CONVERSATIONS, 'readonly');
    const index = tx.objectStore(STORES.CONVERSATIONS).index('by_updatedAt');

    // Open a cursor in descending order so the most recent comes first
    const request = index.openCursor(null, 'prev');
    const results: Conversation[] = [];

    request.onsuccess = () => {
      const cursor = request.result;
      if (!cursor || results.length >= limit) {
        resolve(results);
        return;
      }
      results.push(cursor.value as Conversation);
      cursor.continue();
    };

    request.onerror = () => reject(request.error);
  });
}

// ─── Tags ─────────────────────────────────────────────────────────────────────

/** Upserts a tag record. */
export async function saveTag(tag: Tag): Promise<void> {
  const db = await openDB();
  const tx = db.transaction(STORES.TAGS, 'readwrite');
  await promisifyRequest(tx.objectStore(STORES.TAGS).put(tag));
}

/** Returns all stored tags. */
export async function getAllTags(): Promise<Tag[]> {
  const db = await openDB();
  const tx = db.transaction(STORES.TAGS, 'readonly');
  return promisifyRequest<Tag[]>(tx.objectStore(STORES.TAGS).getAll());
}

// ─── Loci ─────────────────────────────────────────────────────────────────────

/** Upserts a locus record. */
export async function saveLocus(locus: Locus): Promise<void> {
  const db = await openDB();
  const tx = db.transaction(STORES.LOCI, 'readwrite');
  await promisifyRequest(tx.objectStore(STORES.LOCI).put(locus));
}

/** Returns all stored loci. */
export async function getAllLoci(): Promise<Locus[]> {
  const db = await openDB();
  const tx = db.transaction(STORES.LOCI, 'readonly');
  return promisifyRequest<Locus[]>(tx.objectStore(STORES.LOCI).getAll());
}
