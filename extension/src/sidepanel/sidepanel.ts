// ─── loci side panel ─────────────────────────────────────────────────────────
//
// Manages the persistent side panel UI:
// - Loads tags from the background service worker and renders filter chips
// - Debounced search with active tag filter
// - Result rendering: icon, title, date, excerpt, tag chips
// - Keyboard navigation (↑↓ enter) through results
// - Click-to-open conversation in a new tab

import type { ExtensionMessage, SearchResult, Tag } from '@loci/core/types';

// ─── DOM refs ─────────────────────────────────────────────────────────────────

const searchInput  = document.getElementById('loci-search')    as HTMLInputElement;
const tagsRow      = document.getElementById('loci-tags-row')  as HTMLElement;
const resultsList  = document.getElementById('loci-results')   as HTMLElement;
const countEl      = document.getElementById('loci-count')     as HTMLElement;

// ─── State ────────────────────────────────────────────────────────────────────

let activeTags: Set<string> = new Set();
let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let currentResults: SearchResult[] = [];
let focusedIndex = -1;

// ─── Helpers ──────────────────────────────────────────────────────────────────

function escapeHTML(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function relativeTime(ts: number): string {
  const delta = Date.now() - ts;
  const s = Math.floor(delta / 1000);
  if (s < 60)  return 'just now';
  const m = Math.floor(s / 60);
  if (m < 60)  return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24)  return `${h}h ago`;
  const d = Math.floor(h / 24);
  if (d < 7)   return `${d}d ago`;
  if (d < 30)  return `${Math.floor(d / 7)}w ago`;
  if (d < 365) return `${Math.floor(d / 30)}mo ago`;
  return `${Math.floor(d / 365)}y ago`;
}

function platformEmoji(platform: string): string {
  switch (platform) {
    case 'claude':  return '✦';
    case 'chatgpt': return '⬡';
    case 'gemini':  return '◈';
    default:        return '○';
  }
}

function platformLabel(platform: string): string {
  switch (platform) {
    case 'claude':  return 'Claude';
    case 'chatgpt': return 'ChatGPT';
    case 'gemini':  return 'Gemini';
    default:        return 'Unknown';
  }
}

// ─── Tag chips ────────────────────────────────────────────────────────────────

function renderTags(tags: Tag[]): void {
  tagsRow.innerHTML = '';

  if (tags.length === 0) {
    // Hide the scrollable tag area if there are no tags
    const wrap = document.getElementById('loci-tags-wrap');
    if (wrap) wrap.style.display = 'none';
    return;
  }

  tags.forEach((tag) => {
    const chip = document.createElement('button');
    chip.className = 'loci-tag-chip' + (activeTags.has(tag.name) ? ' is-active' : '');
    chip.setAttribute('type', 'button');
    chip.setAttribute('aria-pressed', activeTags.has(tag.name) ? 'true' : 'false');
    chip.innerHTML = `
      ${escapeHTML(tag.name)}
      <span class="chip-count">${tag.conversationCount}</span>
    `;

    chip.addEventListener('click', () => {
      if (activeTags.has(tag.name)) {
        activeTags.delete(tag.name);
        chip.classList.remove('is-active');
        chip.setAttribute('aria-pressed', 'false');
      } else {
        activeTags.add(tag.name);
        chip.classList.add('is-active');
        chip.setAttribute('aria-pressed', 'true');
      }
      triggerSearch();
    });

    tagsRow.appendChild(chip);
  });
}

// ─── Search ───────────────────────────────────────────────────────────────────

function triggerSearch(): void {
  const query = searchInput.value.trim();
  const tags  = activeTags.size > 0 ? [...activeTags] : undefined;

  const msg: ExtensionMessage = { type: 'SEARCH', query, tags };
  chrome.runtime.sendMessage(msg, (response: ExtensionMessage | undefined) => {
    if (chrome.runtime.lastError) return;
    if (response && response.type === 'SEARCH_RESULTS') {
      currentResults = response.results;
      renderResults(response.results);
    }
  });
}

function handleSearchInput(): void {
  if (debounceTimer !== null) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(triggerSearch, 200);
}

// ─── Results ──────────────────────────────────────────────────────────────────

function renderResults(results: SearchResult[]): void {
  resultsList.innerHTML = '';
  focusedIndex = -1;

  if (results.length === 0) {
    const query = searchInput.value.trim();
    renderEmpty(query);
    return;
  }

  // Update header count
  countEl.textContent = `${results.length} result${results.length === 1 ? '' : 's'}`;

  results.forEach((r, i) => {
    const item = document.createElement('div');
    item.className = 'loci-result';
    item.setAttribute('role', 'listitem');
    item.setAttribute('tabindex', '-1');
    item.dataset.index = String(i);

    const tagChips = r.tags.length > 0
      ? `<div class="loci-result__tags">${r.tags
          .map((t) => `<span class="loci-result__tag">${escapeHTML(t)}</span>`)
          .join('')
        }</div>`
      : '';

    item.innerHTML = `
      <div class="loci-result__icon" aria-label="${platformLabel(r.platform)}">${platformEmoji(r.platform)}</div>
      <div class="loci-result__title">${escapeHTML(r.title)}</div>
      <div class="loci-result__meta">
        <span class="loci-result__date">${relativeTime(r.createdAt)}</span>
      </div>
      <div class="loci-result__excerpt">${escapeHTML(r.excerpt)}</div>
      ${tagChips}
    `;

    item.addEventListener('click', () => openConversation(r.id));
    item.addEventListener('mousemove', () => setFocus(i, false));

    resultsList.appendChild(item);
  });
}

function renderEmpty(query: string): void {
  countEl.textContent = '';

  const hasQuery  = query.length > 0;
  const hasFilter = activeTags.size > 0;

  let title = 'Nothing indexed yet';
  let body  = 'Start chatting on Claude.ai or ChatGPT and loci will remember your conversations.';
  let icon  = '✦';

  if (hasQuery || hasFilter) {
    title = 'No results found';
    body  = 'Try a different search term or remove the tag filter.';
    icon  = '○';
  }

  resultsList.innerHTML = `
    <div class="loci-empty" role="status" aria-live="polite">
      <div class="loci-empty__icon" aria-hidden="true">${icon}</div>
      <div class="loci-empty__title">${title}</div>
      <p class="loci-empty__body">${body}</p>
    </div>
  `;
}

// ─── Open conversation ────────────────────────────────────────────────────────

function openConversation(conversationId: string): void {
  // Ask the background to open the conversation. The service worker has the
  // full URL stored in IndexedDB — it can create the tab directly.
  chrome.runtime.sendMessage({
    type: 'OPEN_CONVERSATION',
    conversationId,
  } satisfies ExtensionMessage);
}

// ─── Keyboard navigation ──────────────────────────────────────────────────────

function setFocus(index: number, scroll = true): void {
  const items = resultsList.querySelectorAll<HTMLElement>('.loci-result');
  items.forEach((el, i) => {
    el.classList.toggle('is-focused', i === index);
  });
  focusedIndex = index;
  if (scroll && index >= 0 && index < items.length) {
    items[index].scrollIntoView({ block: 'nearest' });
  }
}

function handleKeydown(e: KeyboardEvent): void {
  const items = resultsList.querySelectorAll<HTMLElement>('.loci-result');
  const count = items.length;

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      if (count > 0) setFocus((focusedIndex + 1) % count);
      break;

    case 'ArrowUp':
      e.preventDefault();
      if (count > 0) setFocus(focusedIndex <= 0 ? count - 1 : focusedIndex - 1);
      break;

    case 'Enter':
      e.preventDefault();
      if (focusedIndex >= 0 && focusedIndex < count) {
        const result = currentResults[focusedIndex];
        if (result) openConversation(result.id);
      }
      break;
  }
}

// ─── Boot ─────────────────────────────────────────────────────────────────────

function init(): void {
  // Load tags
  chrome.runtime.sendMessage(
    { type: 'GET_TAGS' } satisfies ExtensionMessage,
    (response: ExtensionMessage | undefined) => {
      if (chrome.runtime.lastError) return;
      if (response && response.type === 'TAGS_LIST') {
        renderTags(response.tags);
      }
    }
  );

  // Load initial results (empty query = all conversations)
  triggerSearch();

  // Wire events
  searchInput.addEventListener('input', handleSearchInput);
  document.addEventListener('keydown', handleKeydown);

  // Focus search on load
  searchInput.focus();
}

document.addEventListener('DOMContentLoaded', init);
