// ─── loci overlay — ⌘K search ────────────────────────────────────────────────
//
// Injects a Shadow DOM element into the host page so styles are fully isolated.
// Communicates with the background service worker via chrome.runtime.sendMessage.

import type { ExtensionMessage, SearchResult } from '@loci/core/types';
import overlayCSS from './overlay.css?inline';
import { overlayHTML } from './overlay-template';

// ─── State ────────────────────────────────────────────────────────────────────

let shadowRoot: ShadowRoot | null = null;
let backdrop: HTMLElement | null = null;
let input: HTMLInputElement | null = null;
let resultsList: HTMLElement | null = null;
let focusedIndex = -1;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let isOpen = false;

// ─── Platform icons ───────────────────────────────────────────────────────────

function platformEmoji(platform: string): string {
  switch (platform) {
    case 'claude':  return '✦';
    case 'chatgpt': return '⬡';
    case 'gemini':  return '◈';
    default:        return '○';
  }
}

// ─── Relative time ────────────────────────────────────────────────────────────

function relativeTime(ts: number): string {
  const delta = Date.now() - ts;
  const s = Math.floor(delta / 1000);
  if (s < 60)   return 'just now';
  const m = Math.floor(s / 60);
  if (m < 60)   return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24)   return `${h}h ago`;
  const d = Math.floor(h / 24);
  if (d < 30)   return `${d}d ago`;
  const mo = Math.floor(d / 30);
  if (mo < 12)  return `${mo}mo ago`;
  return `${Math.floor(mo / 12)}y ago`;
}

// ─── Render ───────────────────────────────────────────────────────────────────

function renderResults(results: SearchResult[]): void {
  if (!resultsList) return;

  resultsList.innerHTML = '';
  focusedIndex = -1;

  if (results.length === 0) {
    const empty = document.createElement('div');
    empty.className = 'loci-empty';
    empty.textContent = 'No results found.';
    resultsList.appendChild(empty);
    return;
  }

  results.forEach((r, i) => {
    const item = document.createElement('div');
    item.className = 'loci-result';
    item.setAttribute('role', 'option');
    item.setAttribute('aria-selected', 'false');
    item.dataset.index = String(i);
    item.dataset.url = '';  // set below via result.id lookup — for now use id as anchor

    item.innerHTML = `
      <div class="loci-result__icon" aria-hidden="true">${platformEmoji(r.platform)}</div>
      <div class="loci-result__header">
        <span class="loci-result__title">${escapeHTML(r.title)}</span>
        <span class="loci-result__date">${relativeTime(r.createdAt)}</span>
      </div>
      <div class="loci-result__excerpt">${escapeHTML(r.excerpt)}</div>
    `;

    item.addEventListener('click', () => openResult(r));
    item.addEventListener('mousemove', () => setFocus(i));
    resultsList!.appendChild(item);
  });
}

function escapeHTML(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function setFocus(index: number): void {
  if (!resultsList) return;

  const items = resultsList.querySelectorAll<HTMLElement>('.loci-result');
  items.forEach((el, i) => {
    el.classList.toggle('is-focused', i === index);
    el.setAttribute('aria-selected', i === index ? 'true' : 'false');
  });
  focusedIndex = index;
}

function openResult(result: SearchResult): void {
  // Derive the URL from the conversation id (platform:uuid → platform URL)
  // The search result id is "platform:uuid". We can't reconstruct the URL here
  // without a full conversation record, so we send OPEN_CONVERSATION to the
  // background and let it handle the tab.
  chrome.runtime.sendMessage({
    type: 'OPEN_CONVERSATION',
    conversationId: result.id,
  } satisfies ExtensionMessage);
  closeOverlay();
}

// ─── Search ───────────────────────────────────────────────────────────────────

function search(query: string): void {
  const msg: ExtensionMessage = { type: 'SEARCH', query };
  chrome.runtime.sendMessage(msg, (response: ExtensionMessage | undefined) => {
    if (chrome.runtime.lastError) return;
    if (response && response.type === 'SEARCH_RESULTS') {
      renderResults(response.results);
    }
  });
}

function handleInput(): void {
  if (debounceTimer !== null) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => {
    const q = input?.value.trim() ?? '';
    if (q.length > 0) {
      search(q);
    } else {
      if (resultsList) resultsList.innerHTML = '';
    }
  }, 200);
}

// ─── Keyboard navigation ──────────────────────────────────────────────────────

function handleKeydown(e: KeyboardEvent): void {
  if (!isOpen) return;

  const items = resultsList?.querySelectorAll<HTMLElement>('.loci-result') ?? [];
  const count = items.length;

  switch (e.key) {
    case 'Escape':
      e.preventDefault();
      closeOverlay();
      break;

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
        items[focusedIndex].click();
      }
      break;
  }
}

// ─── Open / close ─────────────────────────────────────────────────────────────

function openOverlay(): void {
  if (!backdrop) return;
  isOpen = true;
  backdrop.classList.add('is-open');
  input?.focus();
}

function closeOverlay(): void {
  if (!backdrop) return;
  isOpen = false;
  backdrop.classList.remove('is-open');
  if (input) input.value = '';
  if (resultsList) resultsList.innerHTML = '';
  focusedIndex = -1;
}

// ─── Init ─────────────────────────────────────────────────────────────────────

export function initOverlay(): void {
  // Avoid double-initialisation (e.g. if content script runs twice)
  if (document.getElementById('loci-overlay-host')) return;

  // Create host element and attach shadow
  const host = document.createElement('div');
  host.id = 'loci-overlay-host';
  host.style.cssText = 'all: initial; position: fixed; z-index: 2147483647;';
  document.documentElement.appendChild(host);

  shadowRoot = host.attachShadow({ mode: 'open' });

  // Inject styles
  const style = document.createElement('style');
  style.textContent = overlayCSS;
  shadowRoot.appendChild(style);

  // Inject HTML
  const wrapper = document.createElement('div');
  wrapper.innerHTML = overlayHTML;
  shadowRoot.appendChild(wrapper.firstElementChild!);

  // Grab DOM refs
  backdrop    = shadowRoot.getElementById('loci-backdrop');
  input       = shadowRoot.getElementById('loci-input') as HTMLInputElement;
  resultsList = shadowRoot.getElementById('loci-results');

  // Wire events
  input?.addEventListener('input', handleInput);

  backdrop?.addEventListener('click', (e: MouseEvent) => {
    // Close if the click lands on the backdrop itself (not the modal)
    if (e.target === backdrop) closeOverlay();
  });

  // Global keyboard listener — must be on document to catch ⌘K anywhere
  document.addEventListener('keydown', (e: KeyboardEvent) => {
    const isMac = /Mac|iPhone|iPad/i.test(navigator.userAgent);
    const trigger = isMac
      ? (e.metaKey && e.key === 'k')
      : (e.ctrlKey && e.key === 'k');

    if (trigger) {
      e.preventDefault();
      if (isOpen) {
        closeOverlay();
      } else {
        openOverlay();
      }
      return;
    }

    handleKeydown(e);
  });
}

// Auto-initialise when the script is loaded as a content script
initOverlay();
