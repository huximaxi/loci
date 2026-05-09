import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { open as openPath } from '@tauri-apps/plugin-opener';

interface DetectionResult {
  found: boolean;
  kind?: string;
  path?: string;
  rooms?: number;
  crystal_count?: number;
  suggestion: string;
}

let currentPath: string | null = null;
let currentResult: DetectionResult | null = null;

// DOM elements
const folderInput = document.getElementById('folderPath') as HTMLInputElement;
const browseBtn = document.getElementById('browseBtn') as HTMLButtonElement;
const detectBtn = document.getElementById('detectBtn') as HTMLButtonElement;
const resultCard = document.getElementById('resultCard') as HTMLDivElement;
const resultBadge = document.getElementById('resultBadge') as HTMLSpanElement;
const resultText = document.getElementById('resultText') as HTMLParagraphElement;
const statsSection = document.getElementById('statsSection') as HTMLDivElement;
const roomCount = document.getElementById('roomCount') as HTMLSpanElement;
const crystalCount = document.getElementById('crystalCount') as HTMLSpanElement;
const openFinderBtn = document.getElementById('openFinderBtn') as HTMLButtonElement;
const actionBtn = document.getElementById('actionBtn') as HTMLButtonElement;
const pathDisplay = document.getElementById('pathDisplay') as HTMLDivElement;

// Pick folder
async function pickFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Select workspace or Dev folder',
  });

  if (selected && typeof selected === 'string') {
    currentPath = selected;
    folderInput.value = selected;
    detectBtn.disabled = false;
    resultCard.classList.remove('show');
  }
}

// Detect palace
async function detectPalace() {
  if (!currentPath) return;

  detectBtn.disabled = true;
  detectBtn.textContent = 'Scanning...';

  try {
    const result = await invoke<DetectionResult>('detect_palace', {
      searchPath: currentPath,
    });

    currentResult = result;
    displayResult(result);
  } catch (error) {
    console.error('Detection error:', error);
    alert('Error during detection: ' + error);
  } finally {
    detectBtn.disabled = false;
    detectBtn.textContent = 'Detect Palace';
  }
}

// Display result
function displayResult(result: DetectionResult) {
  resultCard.classList.add('show');

  if (result.found && result.kind && result.path) {
    // Found a palace
    resultBadge.textContent = result.kind;
    resultBadge.className = 'badge badge-green';
    resultText.textContent = result.suggestion;

    if (result.rooms !== undefined && result.crystal_count !== undefined) {
      statsSection.style.display = 'flex';
      roomCount.textContent = result.rooms.toString();
      crystalCount.textContent = result.crystal_count.toString();
    } else {
      statsSection.style.display = 'none';
    }

    pathDisplay.textContent = result.path;

    if (result.kind === 'loci') {
      actionBtn.textContent = 'Already loci-format';
      actionBtn.disabled = true;
    } else {
      actionBtn.textContent = 'Migrate to loci';
      actionBtn.disabled = false;
      actionBtn.onclick = performAction;
    }
  } else {
    // No palace found
    resultBadge.textContent = 'none';
    resultBadge.className = 'badge badge-yellow';
    resultText.textContent = result.suggestion;
    statsSection.style.display = 'none';
    pathDisplay.textContent = currentPath || '';
    actionBtn.textContent = 'Create new palace';
    actionBtn.disabled = false;
    actionBtn.onclick = () => {
      alert('Palace creation coming soon!');
    };
  }
}

// Perform migration
async function performAction() {
  if (!currentResult || !currentResult.path) return;

  actionBtn.disabled = true;
  actionBtn.textContent = 'Migrating...';

  try {
    await invoke('migrate_to_loci', {
      sourcePath: currentResult.path,
    });

    alert('Migration complete! Palace migrated to ~/.loci/');

    // Re-scan to show new state
    await detectPalace();
  } catch (error) {
    console.error('Migration error:', error);
    alert('Migration failed: ' + error);
    actionBtn.disabled = false;
    actionBtn.textContent = 'Migrate to loci';
  }
}

// Open in Finder
async function openFinder() {
  if (!currentResult?.path && !currentPath) return;
  const pathToOpen = currentResult?.path || currentPath;
  if (pathToOpen) {
    try {
      await openPath(pathToOpen);
    } catch (error) {
      console.error('Failed to open Finder:', error);
    }
  }
}

// Event listeners
browseBtn.addEventListener('click', pickFolder);
detectBtn.addEventListener('click', detectPalace);
openFinderBtn.addEventListener('click', openFinder);

// ── Ollama status management ────────────────────────────────────────────

type OllamaState = 'online' | 'offline' | 'disabled';

let ollamaState: OllamaState = 'disabled';

// Config persisted in localStorage until a save_ollama_config Tauri command lands.
function getOllamaEnabled(): boolean {
  return localStorage.getItem('loci.ollama.enabled') === 'true';
}
function setOllamaEnabled(on: boolean): void {
  localStorage.setItem('loci.ollama.enabled', String(on));
}
function getFailClosed(): boolean {
  // default: true (sovereign setting)
  return localStorage.getItem('loci.ollama.fail_closed') !== 'false';
}
function setFailClosed(on: boolean): void {
  localStorage.setItem('loci.ollama.fail_closed', String(on));
}

async function checkOllamaHealth(): Promise<void> {
  if (!getOllamaEnabled()) {
    applyOllamaState('disabled');
    return;
  }
  try {
    const ok = await invoke<boolean>('check_ollama_health');
    applyOllamaState(ok ? 'online' : 'offline');
  } catch {
    applyOllamaState('offline');
  }
}

function applyOllamaState(state: OllamaState): void {
  ollamaState = state;

  // ── Status bar badge ───────────────────────────────────────────────
  const badge  = document.getElementById('ai-badge')  as HTMLElement | null;
  const dot    = document.getElementById('ai-dot')    as HTMLElement | null;
  const label  = document.getElementById('ai-label')  as HTMLElement | null;

  if (badge && dot && label) {
    if (state === 'disabled') {
      badge.style.display = 'none';
    } else {
      badge.style.display = 'inline-flex';
      dot.className = `ai-dot ${state}`;
      badge.classList.toggle('clickable', state === 'offline');

      // Re-create label span to trigger the 300ms crossfade
      const next = document.createElement('span');
      next.id = 'ai-label';
      next.className = 'ai-label';
      next.textContent = `local AI · ${state}`;
      label.replaceWith(next);
    }
  }

  // ── Settings panel status dot (if panel is open) ───────────────────
  const sDot   = document.getElementById('settingsDot')         as HTMLElement | null;
  const sLabel = document.getElementById('settingsStatusLabel') as HTMLElement | null;
  if (sDot && sLabel) {
    if (state === 'disabled') {
      sDot.className = 'settings-status-dot';
      sLabel.textContent = '—';
    } else {
      sDot.className = `settings-status-dot ${state}`;
      sLabel.textContent = `Ollama · ${state}`;
    }
  }
}

function startOllamaPolling(): void {
  checkOllamaHealth();
  window.setInterval(checkOllamaHealth, 30_000);
}

// ── Settings panel ──────────────────────────────────────────────────────

function openSettings(): void {
  const mainView     = document.getElementById('main-view');
  const settingsView = document.getElementById('settings-view');
  if (!mainView || !settingsView) return;
  mainView.style.display = 'none';
  settingsView.style.display = 'flex';
  syncSettingsUI();
  checkOllamaHealth();
  if (getOllamaEnabled() && ollamaState === 'online') populateModels();
  syncMcpStatus();
}

function closeSettings(): void {
  const mainView     = document.getElementById('main-view');
  const settingsView = document.getElementById('settings-view');
  if (!mainView || !settingsView) return;
  settingsView.style.display = 'none';
  mainView.style.display = '';
}

function syncSettingsUI(): void {
  const ollamaToggle     = document.getElementById('ollamaToggle')     as HTMLInputElement | null;
  const failClosedToggle = document.getElementById('failClosedToggle') as HTMLInputElement | null;
  if (ollamaToggle) {
    ollamaToggle.checked = getOllamaEnabled();
    setDependentsEnabled(ollamaToggle.checked);
  }
  if (failClosedToggle) {
    failClosedToggle.checked = getFailClosed();
    const warning = document.getElementById('failOpenWarning');
    if (warning) warning.style.display = failClosedToggle.checked ? 'none' : 'block';
  }
}

function setDependentsEnabled(on: boolean): void {
  document.querySelectorAll<HTMLElement>('.ollama-dependent').forEach(el => {
    el.classList.toggle('off', !on);
  });
}

async function populateModels(): Promise<void> {
  const chatSel  = document.getElementById('chatModelSelect')  as HTMLSelectElement | null;
  const embedSel = document.getElementById('embedModelSelect') as HTMLSelectElement | null;
  if (!chatSel || !embedSel) return;

  chatSel.disabled  = true;
  embedSel.disabled = true;
  chatSel.innerHTML  = '<option value="">Detecting models…</option>';
  embedSel.innerHTML = '<option value="">Detecting models…</option>';

  try {
    const models = await invoke<string[]>('list_ollama_models');
    const savedChat  = localStorage.getItem('loci.ollama.chat_model')  || 'llama3';
    const savedEmbed = localStorage.getItem('loci.ollama.embed_model') || 'nomic-embed-text';

    chatSel.innerHTML  = '';
    embedSel.innerHTML = '';

    if (models.length === 0) {
      chatSel.innerHTML  = '<option value="">— no models found</option>';
      embedSel.innerHTML = '<option value="">— no models found</option>';
    } else {
      (models as string[]).forEach((m: string) => {
        chatSel!.add(new Option(m, m, false, m === savedChat));
        embedSel!.add(new Option(m, m, false, m === savedEmbed));
      });
      chatSel.disabled  = false;
      embedSel.disabled = false;
    }
  } catch {
    chatSel.innerHTML  = '<option value="">— Ollama offline</option>';
    embedSel.innerHTML = '<option value="">— Ollama offline</option>';
  }
}

async function syncMcpStatus(): Promise<void> {
  try {
    const status = await invoke<{ running: boolean; port: number }>('mcp_server_status');
    const mcpToggle    = document.getElementById('mcpToggle')    as HTMLInputElement | null;
    const mcpStatusRow = document.getElementById('mcpStatusRow') as HTMLElement | null;
    const mcpLabel     = document.getElementById('mcpStatusLabel') as HTMLElement | null;
    if (mcpToggle)    mcpToggle.checked = status.running;
    if (mcpStatusRow) mcpStatusRow.style.display = status.running ? 'flex' : 'none';
    if (mcpLabel)     mcpLabel.textContent = status.running ? `running · :${status.port}` : '—';
  } catch {
    // MCP server not yet started — silently ignore
  }
}

// ── Settings event wiring ───────────────────────────────────────────────

document.getElementById('settingsBack')?.addEventListener('click', closeSettings);

document.getElementById('ai-badge')?.addEventListener('click', () => {
  if (ollamaState === 'offline') openSettings();
});

(document.getElementById('ollamaToggle') as HTMLInputElement | null)
  ?.addEventListener('change', (e) => {
    const on = (e.target as HTMLInputElement).checked;
    setOllamaEnabled(on);
    setDependentsEnabled(on);
    checkOllamaHealth();
    if (on && ollamaState === 'online') populateModels();
  });

(document.getElementById('failClosedToggle') as HTMLInputElement | null)
  ?.addEventListener('change', (e) => {
    const on = (e.target as HTMLInputElement).checked;
    setFailClosed(on);
    const warning = document.getElementById('failOpenWarning');
    if (warning) warning.style.display = on ? 'none' : 'block';
  });

(document.getElementById('chatModelSelect') as HTMLSelectElement | null)
  ?.addEventListener('change', (e) => {
    localStorage.setItem('loci.ollama.chat_model', (e.target as HTMLSelectElement).value);
  });

(document.getElementById('embedModelSelect') as HTMLSelectElement | null)
  ?.addEventListener('change', (e) => {
    localStorage.setItem('loci.ollama.embed_model', (e.target as HTMLSelectElement).value);
  });

(document.getElementById('mcpToggle') as HTMLInputElement | null)
  ?.addEventListener('change', async (e) => {
    const on = (e.target as HTMLInputElement).checked;
    try {
      if (on) {
        await invoke('start_mcp_server');
      } else {
        await invoke('stop_mcp_server');
      }
    } catch (err) {
      console.error('MCP toggle error:', err);
    } finally {
      await syncMcpStatus();
    }
  });

document.getElementById('copyGooseConfig')?.addEventListener('click', (e) => {
  const config = '{"mcpServers":{"loci":{"url":"http://localhost:3456"}}}';
  navigator.clipboard.writeText(config).then(() => {
    const btn = e.target as HTMLButtonElement;
    const orig = btn.textContent ?? '';
    btn.textContent = 'copied ✓';
    setTimeout(() => { btn.textContent = orig; }, 1500);
  });
});

// ⌘, or Ctrl+, to open settings; Esc to close
document.addEventListener('keydown', (e) => {
  if ((e.metaKey || e.ctrlKey) && e.key === ',') {
    e.preventDefault();
    const sv = document.getElementById('settings-view');
    if (sv?.style.display === 'none' || !sv?.style.display) {
      openSettings();
    } else {
      closeSettings();
    }
  }
  if (e.key === 'Escape') closeSettings();
});

// ── Boot ────────────────────────────────────────────────────────────────

startOllamaPolling();
