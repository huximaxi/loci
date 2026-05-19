import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { openPath } from '@tauri-apps/plugin-opener';

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

// ── Loci config persistence (Tauri) ─────────────────────────────────────────
// Bridge: localStorage = sync in-memory state. Tauri = cross-restart persistence.
// loadConfig() pulls from Tauri on boot and seeds localStorage.
// saveConfig() writes current state to Tauri on every settings change.

async function loadConfig(): Promise<void> {
  try {
    const config = await invoke<{
      ollama?: {
        enabled?: boolean;
        base_url?: string;
        chat_model?: string;
        embed_model?: string;
        fail_closed?: boolean;
      };
    }>('read_loci_config');

    if (config.ollama) {
      const o = config.ollama;
      if (o.enabled !== undefined)   setOllamaEnabled(o.enabled);
      if (o.fail_closed !== undefined) setFailClosed(o.fail_closed);
      if (o.chat_model)  localStorage.setItem('loci.ollama.chat_model',  o.chat_model);
      if (o.embed_model) localStorage.setItem('loci.ollama.embed_model', o.embed_model);
    }
  } catch {
    // No config yet — defaults from localStorage apply
  }
}

async function saveConfig(): Promise<void> {
  try {
    await invoke('write_loci_config', {
      config: {
        ollama: {
          enabled:     getOllamaEnabled(),
          base_url:    'http://localhost:11434',
          chat_model:  localStorage.getItem('loci.ollama.chat_model')  ?? 'llama3',
          embed_model: localStorage.getItem('loci.ollama.embed_model') ?? 'nomic-embed-text',
          offline_mode: false,
          fail_closed: getFailClosed(),
        },
      },
    });
  } catch (err) {
    console.warn('write_loci_config failed:', err);
  }
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

// ── View routing ────────────────────────────────────────────────────────
// Views: onboarding (first-launch) · welcome · wizard (migrate) ·
//        setup-wizard (palace A/B) · dashboard · settings

type AppView = 'onboarding' | 'welcome' | 'wizard' | 'setup-wizard' | 'dashboard' | 'map' | 'eigenlayer' | 'settings';
let previousView: AppView = 'onboarding';

// Holds the path of the currently loaded palace so map/dashboard can share it.
let currentPalacePath = '';

function showView(view: AppView): void {
  const onboarding  = document.getElementById('onboarding-view');
  const welcome     = document.getElementById('welcome-view');
  const wizard      = document.getElementById('main-view');
  const setupWizard = document.getElementById('setup-wizard-view');
  const dashboard   = document.getElementById('dashboard-view');
  const map         = document.getElementById('map-view');
  const eigen       = document.getElementById('eigenlayer-view');
  const settings    = document.getElementById('settings-view');
  if (onboarding)  onboarding.style.display  = view === 'onboarding'   ? 'flex'  : 'none';
  if (welcome)     welcome.style.display     = view === 'welcome'      ? 'flex'  : 'none';
  if (wizard)      wizard.style.display      = view === 'wizard'       ? 'block' : 'none';
  if (setupWizard) setupWizard.style.display = view === 'setup-wizard' ? 'block' : 'none';
  if (dashboard)   dashboard.style.display   = view === 'dashboard'    ? 'flex'  : 'none';
  if (map)         map.style.display         = view === 'map'          ? 'flex'  : 'none';
  if (eigen)       eigen.style.display       = view === 'eigenlayer'   ? 'flex'  : 'none';
  if (settings)    settings.style.display    = view === 'settings'     ? 'flex'  : 'none';
}

// ── Settings panel ──────────────────────────────────────────────────────

function openSettings(): void {
  const current = document.getElementById('settings-view')?.style.display;
  if (current !== 'flex') {
    // remember where we came from — check all non-settings views
    if (document.getElementById('map-view')?.style.display === 'flex') {
      previousView = 'map';
    } else if (document.getElementById('dashboard-view')?.style.display === 'flex') {
      previousView = 'dashboard';
    } else if (document.getElementById('onboarding-view')?.style.display === 'flex') {
      previousView = 'onboarding';
    } else if (document.getElementById('setup-wizard-view')?.style.display === 'block') {
      previousView = 'setup-wizard';
    } else if (document.getElementById('welcome-view')?.style.display === 'flex') {
      previousView = 'welcome';
    } else {
      previousView = 'wizard';
    }
  }
  showView('settings');
  syncSettingsUI();
  checkOllamaHealth();
  if (getOllamaEnabled() && ollamaState === 'online') populateModels();
  syncMcpStatus();
}

function closeSettings(): void {
  showView(previousView);
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
    void saveConfig();
    setDependentsEnabled(on);
    checkOllamaHealth();
    if (on && ollamaState === 'online') populateModels();
  });

(document.getElementById('failClosedToggle') as HTMLInputElement | null)
  ?.addEventListener('change', (e) => {
    const on = (e.target as HTMLInputElement).checked;
    setFailClosed(on);
    void saveConfig();
    const warning = document.getElementById('failOpenWarning');
    if (warning) warning.style.display = on ? 'none' : 'block';
  });

(document.getElementById('chatModelSelect') as HTMLSelectElement | null)
  ?.addEventListener('change', (e) => {
    localStorage.setItem('loci.ollama.chat_model', (e.target as HTMLSelectElement).value);
    void saveConfig();
  });

(document.getElementById('embedModelSelect') as HTMLSelectElement | null)
  ?.addEventListener('change', (e) => {
    localStorage.setItem('loci.ollama.embed_model', (e.target as HTMLSelectElement).value);
    void saveConfig();
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

document.getElementById('titlebarSettingsBtn')?.addEventListener('click', openSettings);

// Welcome screen choices
document.getElementById('diveInBtn')?.addEventListener('click', openSettings);
document.getElementById('migrateBtn')?.addEventListener('click', () => {
  showView('setup-wizard');
});

// Wizard back button → welcome
document.getElementById('wizardBackBtn')?.addEventListener('click', () => {
  showView('welcome');
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

// ── Onboarding slide controller ─────────────────────────────────────────

let currentSlide = 0;
const SLIDE_COUNT = 3;

function showSlide(n: number): void {
  document.querySelectorAll<HTMLElement>('.slide').forEach((el, i) => {
    el.classList.toggle('active', i === n);
  });
  document.querySelectorAll<HTMLElement>('.slide-dot').forEach((el, i) => {
    el.classList.toggle('active', i === n);
  });
  const prevBtn = document.getElementById('slidePrev') as HTMLButtonElement | null;
  const nextBtn = document.getElementById('slideNext') as HTMLButtonElement | null;
  if (prevBtn) prevBtn.style.visibility = n === 0 ? 'hidden' : 'visible';
  if (nextBtn) nextBtn.textContent = n === SLIDE_COUNT - 1 ? 'get started →' : 'next →';
  currentSlide = n;
}

document.getElementById('slidePrev')?.addEventListener('click', () => {
  if (currentSlide > 0) showSlide(currentSlide - 1);
});

document.getElementById('slideNext')?.addEventListener('click', () => {
  if (currentSlide < SLIDE_COUNT - 1) {
    showSlide(currentSlide + 1);
  } else {
    showView('setup-wizard');
  }
});

document.querySelectorAll<HTMLElement>('.slide-dot').forEach((dot, i) => {
  dot.addEventListener('click', () => showSlide(i));
});

// ── Dashboard ────────────────────────────────────────────────────────────

interface CronJobState {
  job: string;
  status: string;
  last_run?: string;
  summary?: string;
  ciq?: number;
  ciq_delta?: number;
}

interface PalaceState {
  palace_path: string;
  room_count: number;
  cron_jobs: CronJobState[];
  current_focus: string;
  pending_tasks: string[];
  generated_at: string;
}

async function refreshDashboard(palacePath: string): Promise<void> {
  currentPalacePath = palacePath;
  try {
    const state = await invoke<PalaceState>('read_palace_state', { palacePath });
    renderDashboard(state);
  } catch (err) {
    const focus = document.getElementById('db-focus');
    if (focus) focus.textContent = `error loading palace: ${err}`;
  }
}

function renderDashboard(state: PalaceState): void {
  const ok    = state.cron_jobs.filter(j => j.status === 'ok').length;
  const total = state.cron_jobs.length;
  const ts    = new Date(state.generated_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

  const meta = document.getElementById('db-meta');
  if (meta) meta.textContent = `${ts} · ${total} jobs · ${ok}/${total} ok · ${state.room_count} rooms`;

  const dot = document.getElementById('db-status-dot');
  if (dot) {
    dot.className = 'db-status-dot ' + (
      total === 0 || ok === total ? 'db-status-ok' :
      ok > 0 ? 'db-status-warn' : 'db-status-error'
    );
  }

  const grid = document.getElementById('db-cron-grid');
  if (grid) {
    grid.innerHTML = '';
    if (state.cron_jobs.length === 0) {
      const empty = document.createElement('p');
      empty.className = 'db-empty';
      empty.textContent = 'no cron jobs found';
      grid.appendChild(empty);
    } else {
      for (const job of state.cron_jobs) {
        const card = document.createElement('div');
        card.className = `cron-card ${job.status}`;

        const lastRun = job.last_run
          ? new Date(job.last_run).toLocaleString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
          : 'never';

        const ciqText = job.ciq !== undefined
          ? ` · CIQ ${job.ciq.toFixed(1)}` + (
              job.ciq_delta !== undefined && job.ciq_delta !== 0
                ? (job.ciq_delta > 0 ? ` (+${job.ciq_delta.toFixed(1)})` : ` (${job.ciq_delta.toFixed(1)})`)
                : '')
          : '';

        const nameEl = document.createElement('div');
        nameEl.className = 'cron-card-name';
        nameEl.textContent = job.job;

        const summaryEl = document.createElement('div');
        summaryEl.className = 'cron-card-summary';
        // Cipher: textContent — job summaries are local but may contain pasted prose
        summaryEl.textContent = (job.summary ?? job.status) + ciqText;

        const metaEl = document.createElement('div');
        metaEl.className = 'cron-card-meta';
        metaEl.textContent = lastRun;

        card.appendChild(nameEl);
        card.appendChild(summaryEl);
        card.appendChild(metaEl);
        grid.appendChild(card);
      }
    }
  }

  // Cipher: textContent only — palace text is local-first but may contain pasted external prose
  const focus = document.getElementById('db-focus');
  if (focus) focus.textContent = state.current_focus || '(no current focus)';

  const pendingList    = document.getElementById('db-pending');
  const pendingSection = document.getElementById('db-pending-section');
  if (pendingList && pendingSection) {
    if (state.pending_tasks.length === 0) {
      pendingSection.style.display = 'none';
    } else {
      pendingSection.style.display = 'block';
      pendingList.innerHTML = '';
      for (const task of state.pending_tasks) {
        const el = document.createElement('div');
        el.className = 'pending-item';
        el.textContent = task; // Cipher: textContent
        pendingList.appendChild(el);
      }
    }
  }
}

function showInvalidPathBanner(path: string): void {
  const picker = document.getElementById('setup-branch-picker');
  if (!picker) return;
  const banner = document.createElement('div');
  banner.style.cssText = 'font-size:9px;color:#FFCC33;padding:8px 0 4px;margin-bottom:4px;letter-spacing:0.03em;';
  banner.textContent = `⚠ previous palace path no longer valid: ${path}`;
  picker.insertBefore(banner, picker.firstChild);
}

// ── Setup wizard ─────────────────────────────────────────────────────────

document.getElementById('setupWizardBackBtn')?.addEventListener('click', () => {
  showView('onboarding');
  showSlide(2); // return to last slide
});

document.getElementById('createPalaceBtn')?.addEventListener('click', () => {
  document.getElementById('setup-branch-picker')!.style.display = 'none';
  document.getElementById('setup-create-flow')!.style.display = 'block';
});

document.getElementById('loadPalaceBtn')?.addEventListener('click', () => {
  document.getElementById('setup-branch-picker')!.style.display = 'none';
  document.getElementById('setup-load-flow')!.style.display = 'block';
});

document.getElementById('createBackBtn')?.addEventListener('click', () => {
  document.getElementById('setup-create-flow')!.style.display = 'none';
  document.getElementById('setup-branch-picker')!.style.display = 'block';
});

document.getElementById('loadBackBtn')?.addEventListener('click', () => {
  document.getElementById('setup-load-flow')!.style.display = 'none';
  document.getElementById('setup-branch-picker')!.style.display = 'block';
});

// Branch A — create
document.getElementById('createBrowseBtn')?.addEventListener('click', async () => {
  const selected = await open({ directory: true, multiple: false, title: 'Choose palace parent folder' });
  if (selected && typeof selected === 'string') {
    (document.getElementById('createPalacePath') as HTMLInputElement).value = selected;
    (document.getElementById('createConfirmBtn') as HTMLButtonElement).disabled = false;
  }
});

document.getElementById('createConfirmBtn')?.addEventListener('click', async () => {
  const pathInput = document.getElementById('createPalacePath') as HTMLInputElement;
  const errEl     = document.getElementById('createError') as HTMLElement;
  const btn       = document.getElementById('createConfirmBtn') as HTMLButtonElement;
  btn.disabled = true;
  btn.textContent = 'scaffolding…';
  errEl.style.display = 'none';
  try {
    const palacePath = await invoke<string>('scaffold_palace', { parentPath: pathInput.value });
    showView('dashboard');
    void refreshDashboard(palacePath);
  } catch (err) {
    errEl.textContent = String(err);
    errEl.style.display = 'block';
    btn.disabled = false;
    btn.textContent = 'scaffold palace →';
  }
});

// Branch B — load
document.getElementById('loadBrowseBtn')?.addEventListener('click', async () => {
  const selected = await open({ directory: true, multiple: false, title: 'Select palace folder' });
  if (!selected || typeof selected !== 'string') return;
  (document.getElementById('loadPalacePath') as HTMLInputElement).value = selected;
  const errEl = document.getElementById('loadError') as HTMLElement;
  errEl.style.display = 'none';
  try {
    const manifest = await invoke<{ path: string; rooms: Array<{ name: string }>; cron_job_count: number }>(
      'load_palace', { path: selected }
    );
    const badge = document.getElementById('loadBadge') as HTMLElement;
    badge.textContent = `${manifest.rooms.length} rooms`;
    badge.className = 'badge badge-green';
    (document.getElementById('loadResultText') as HTMLElement).textContent =
      `Found ${manifest.rooms.length} rooms · ${manifest.cron_job_count} cron jobs. Ready to load.`;
    document.getElementById('loadResult')!.style.display = 'block';
    (document.getElementById('loadConfirmBtn') as HTMLButtonElement).disabled = false;
  } catch (err) {
    errEl.textContent = String(err);
    errEl.style.display = 'block';
    (document.getElementById('loadConfirmBtn') as HTMLButtonElement).disabled = true;
  }
});

document.getElementById('loadConfirmBtn')?.addEventListener('click', async () => {
  const pathInput = document.getElementById('loadPalacePath') as HTMLInputElement;
  const errEl     = document.getElementById('loadError') as HTMLElement;
  const btn       = document.getElementById('loadConfirmBtn') as HTMLButtonElement;
  btn.disabled = true;
  btn.textContent = 'loading…';
  errEl.style.display = 'none';
  try {
    await invoke('load_palace', { path: pathInput.value });
    showView('dashboard');
    void refreshDashboard(pathInput.value);
  } catch (err) {
    errEl.textContent = String(err);
    errEl.style.display = 'block';
    btn.disabled = false;
    btn.textContent = 'load palace →';
  }
});

// Dashboard → map
document.getElementById('dbMapBtn')?.addEventListener('click', () => {
  showView('map');
  if (currentPalacePath) void renderMap(currentPalacePath);
});

// Dashboard → change palace
document.getElementById('dbChangePalaceBtn')?.addEventListener('click', () => {
  showView('setup-wizard');
});

// Map → dashboard
document.getElementById('mapBackBtn')?.addEventListener('click', () => {
  showView('dashboard');
});

// Dashboard → eigenlayer
document.getElementById('dbEigenBtn')?.addEventListener('click', () => {
  showView('eigenlayer');
});

// Eigenlayer → dashboard
document.getElementById('eigenBackBtn')?.addEventListener('click', () => {
  showView('dashboard');
});

// ── Map view (Phase 4) ───────────────────────────────────────────────────

const ROOM_GLYPHS: Record<string, string> = {
  'dev-room':     '🖥',
  'design-room':  '🎨',
  'hatchery':     '🥚',
  'engine-room':  '🛠',
  'library':      '📚',
  'cave':         '🪨',
  'anon-agent':   '🕵',
  'marketing-room': '🔮',
  'trust-cp':     '🔬',
};

async function renderMap(palacePath: string): Promise<void> {
  const svg = document.getElementById('palace-map-svg') as SVGSVGElement | null;
  const mapMeta = document.getElementById('map-meta');
  if (!svg) return;

  while (svg.firstChild) svg.removeChild(svg.firstChild);

  let state: PalaceState;
  try {
    state = await invoke<PalaceState>('read_palace_state', { palacePath });
  } catch (err) {
    if (mapMeta) mapMeta.textContent = `error: ${err}`;
    return;
  }

  let rooms: Array<{ name: string; file_count: number }> = [];
  try {
    const manifest = await invoke<{ rooms: Array<{ name: string; file_count: number }> }>(
      'load_palace', { path: palacePath }
    );
    rooms = manifest.rooms;
  } catch {
    rooms = Array.from({ length: state.room_count }, (_, i) => ({ name: `room-${i + 1}`, file_count: 0 }));
  }

  if (mapMeta) mapMeta.textContent = `${rooms.length} rooms · ${state.cron_jobs.length} cron`;

  const W = 640, H = 400;
  svg.setAttribute('viewBox', `0 0 ${W} ${H}`);
  const svgNS = 'http://www.w3.org/2000/svg';

  // Defs: glow filters + dungeon floor grid (trusted static markup — no palace data here)
  const defs = document.createElementNS(svgNS, 'defs');
  defs.innerHTML = `
    <filter id="glow-ok" x="-70%" y="-70%" width="240%" height="240%">
      <feGaussianBlur stdDeviation="5" result="b"/>
      <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
    <filter id="glow-err" x="-70%" y="-70%" width="240%" height="240%">
      <feGaussianBlur stdDeviation="5" result="b"/>
      <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
    <filter id="glow-dim" x="-50%" y="-50%" width="200%" height="200%">
      <feGaussianBlur stdDeviation="3" result="b"/>
      <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
    <filter id="glow-gold" x="-100%" y="-100%" width="300%" height="300%">
      <feGaussianBlur stdDeviation="8" result="b"/>
      <feMerge><feMergeNode in="b"/><feMergeNode in="SourceGraphic"/></feMerge>
    </filter>
    <pattern id="dungeon-grid" width="40" height="40" patternUnits="userSpaceOnUse">
      <rect width="40" height="40" fill="none"/>
      <path d="M40 0L0 0 0 40" fill="none" stroke="rgba(255,255,255,0.03)" stroke-width="0.5"/>
    </pattern>`;
  svg.appendChild(defs);

  // Dungeon floor background
  const bg = document.createElementNS(svgNS, 'rect');
  bg.setAttribute('width', String(W));
  bg.setAttribute('height', String(H));
  bg.setAttribute('fill', 'url(#dungeon-grid)');
  svg.appendChild(bg);

  const cx = W / 2, cy = H / 2 - 10;
  const ringR = Math.min(cx, cy) - 70;
  const n = rooms.length;

  const cronMap: Record<string, string> = {};
  for (const job of state.cron_jobs) {
    cronMap[job.job.replace(/-daily$|-weekly$/, '')] = job.status;
  }

  const AURA_COLOR: Record<string, string> = {
    ok: '#4BDF8F', error: '#E03030', never: 'rgba(200,160,80,0.45)'
  };
  const GLOW_ID: Record<string, string> = {
    ok: 'glow-ok', error: 'glow-err', never: 'glow-dim'
  };

  // Pointy-top hexagon: first vertex at top (-π/2), then step π/3 each
  const hexPts = (hx: number, hy: number, hr: number) =>
    Array.from({ length: 6 }, (_, k) => {
      const a = -Math.PI / 2 + (Math.PI / 3) * k;
      return `${(hx + hr * Math.cos(a)).toFixed(1)},${(hy + hr * Math.sin(a)).toFixed(1)}`;
    }).join(' ');

  // Rhombus diamond (for center crystal)
  const diamondPts = (dx: number, dy: number, dr: number) => {
    const w = dr * 0.62;
    return `${dx.toFixed(1)},${(dy - dr).toFixed(1)} ${(dx + w).toFixed(1)},${dy.toFixed(1)} ${dx.toFixed(1)},${(dy + dr).toFixed(1)} ${(dx - w).toFixed(1)},${dy.toFixed(1)}`;
  };

  // Connection lines — drawn first so they sit behind all nodes
  rooms.forEach((_, i) => {
    const angle = (2 * Math.PI * i) / n - Math.PI / 2;
    const rx = cx + ringR * Math.cos(angle);
    const ry = cy + ringR * Math.sin(angle);
    const line = document.createElementNS(svgNS, 'line');
    line.setAttribute('x1', cx.toFixed(1));
    line.setAttribute('y1', cy.toFixed(1));
    line.setAttribute('x2', rx.toFixed(1));
    line.setAttribute('y2', ry.toFixed(1));
    line.setAttribute('stroke', 'rgba(180,130,50,0.13)');
    line.setAttribute('stroke-width', '1.5');
    line.setAttribute('stroke-dasharray', '4 6');
    svg.appendChild(line);
  });

  // Center crystal — animated pulsing diamond sigil
  const centerG = document.createElementNS(svgNS, 'g');

  const glowRing = document.createElementNS(svgNS, 'polygon');
  glowRing.setAttribute('id', 'crystal-glow');
  glowRing.setAttribute('points', diamondPts(cx, cy, 30));
  glowRing.setAttribute('fill', 'none');
  glowRing.setAttribute('stroke', '#c87c2c');
  glowRing.setAttribute('stroke-width', '1.2');
  glowRing.setAttribute('filter', 'url(#glow-gold)');
  centerG.appendChild(glowRing);

  const innerDiamond = document.createElementNS(svgNS, 'polygon');
  innerDiamond.setAttribute('points', diamondPts(cx, cy, 17));
  innerDiamond.setAttribute('fill', 'rgba(200,124,44,0.10)');
  innerDiamond.setAttribute('stroke', 'rgba(200,124,44,0.55)');
  innerDiamond.setAttribute('stroke-width', '1.2');
  centerG.appendChild(innerDiamond);

  const centerGlyph = document.createElementNS(svgNS, 'text');
  centerGlyph.setAttribute('x', String(cx));
  centerGlyph.setAttribute('y', String(cy + 5));
  centerGlyph.setAttribute('text-anchor', 'middle');
  centerGlyph.setAttribute('font-size', '12');
  centerGlyph.setAttribute('fill', 'rgba(200,124,44,0.9)');
  centerGlyph.textContent = '◆';
  centerG.appendChild(centerGlyph);

  const centerLabel = document.createElementNS(svgNS, 'text');
  centerLabel.setAttribute('x', String(cx));
  centerLabel.setAttribute('y', String(cy + 34));
  centerLabel.setAttribute('text-anchor', 'middle');
  centerLabel.setAttribute('font-size', '7');
  centerLabel.setAttribute('fill', 'rgba(200,160,80,0.4)');
  centerLabel.setAttribute('letter-spacing', '0.2em');
  centerLabel.textContent = 'PALACE';
  centerG.appendChild(centerLabel);
  svg.appendChild(centerG);

  // Room nodes — hexagonal, with glow aura matching cron status
  rooms.forEach((room, i) => {
    const angle = (2 * Math.PI * i) / n - Math.PI / 2;
    const rx = cx + ringR * Math.cos(angle);
    const ry = cy + ringR * Math.sin(angle);

    const statusKey = Object.keys(cronMap).find(k => room.name.includes(k) || k.includes(room.name));
    const status = statusKey ? cronMap[statusKey] : 'never';
    const auraColor = AURA_COLOR[status] ?? AURA_COLOR.never;
    const glowFilter = GLOW_ID[status] ?? GLOW_ID.never;

    const g = document.createElementNS(svgNS, 'g');

    // Glowing aura hex — bloom via SVG filter
    const aura = document.createElementNS(svgNS, 'polygon');
    aura.setAttribute('points', hexPts(rx, ry, 34));
    aura.setAttribute('fill', 'none');
    aura.setAttribute('stroke', auraColor);
    aura.setAttribute('stroke-width', '1.2');
    aura.setAttribute('opacity', '0.55');
    aura.setAttribute('filter', `url(#${glowFilter})`);
    g.appendChild(aura);

    // Room hex body
    const hex = document.createElementNS(svgNS, 'polygon');
    hex.setAttribute('points', hexPts(rx, ry, 24));
    hex.setAttribute('fill', 'rgba(12,9,5,0.88)');
    hex.setAttribute('stroke', 'rgba(180,140,60,0.22)');
    hex.setAttribute('stroke-width', '1');
    g.appendChild(hex);

    // Glyph — textContent (Cipher discipline)
    const glyph = document.createElementNS(svgNS, 'text');
    glyph.setAttribute('x', rx.toFixed(1));
    glyph.setAttribute('y', (ry + 5).toFixed(1));
    glyph.setAttribute('text-anchor', 'middle');
    glyph.setAttribute('font-size', '14');
    glyph.textContent = ROOM_GLYPHS[room.name] ?? '◈';
    g.appendChild(glyph);

    // Room label — uppercase, parchment tone
    const labelY = ry + (ry > cy ? 46 : -34);
    const label = document.createElementNS(svgNS, 'text');
    label.setAttribute('x', rx.toFixed(1));
    label.setAttribute('y', labelY.toFixed(1));
    label.setAttribute('text-anchor', 'middle');
    label.setAttribute('font-size', '7.5');
    label.setAttribute('fill', 'rgba(200,175,120,0.6)');
    label.setAttribute('letter-spacing', '0.12em');
    label.textContent = room.name.toUpperCase(); // textContent — Cipher discipline
    g.appendChild(label);

    // File count badge
    if (room.file_count > 0) {
      const dot = document.createElementNS(svgNS, 'circle');
      dot.setAttribute('cx', (rx + 18).toFixed(1));
      dot.setAttribute('cy', (ry - 18).toFixed(1));
      dot.setAttribute('r', '7');
      dot.setAttribute('fill', 'rgba(200,124,44,0.12)');
      dot.setAttribute('stroke', 'rgba(200,124,44,0.5)');
      dot.setAttribute('stroke-width', '1');
      g.appendChild(dot);

      const dotLabel = document.createElementNS(svgNS, 'text');
      dotLabel.setAttribute('x', (rx + 18).toFixed(1));
      dotLabel.setAttribute('y', (ry - 14).toFixed(1));
      dotLabel.setAttribute('text-anchor', 'middle');
      dotLabel.setAttribute('font-size', '6.5');
      dotLabel.setAttribute('fill', 'rgba(200,124,44,0.85)');
      dotLabel.textContent = String(room.file_count);
      g.appendChild(dotLabel);
    }

    svg.appendChild(g);
  });
}

// ── Boot ─────────────────────────────────────────────────────────────────

async function boot(): Promise<void> {
  await loadConfig();
  startOllamaPolling();
  try {
    const config = await invoke<{ palace_path?: string }>('read_loci_config');
    if (config.palace_path) {
      const valid = await invoke<boolean>('validate_palace_path', { path: config.palace_path });
      if (valid) {
        showView('dashboard');
        void refreshDashboard(config.palace_path);
        return;
      }
      // Path moved or deleted — route to setup with a warning
      showView('setup-wizard');
      showInvalidPathBanner(config.palace_path);
      return;
    }
  } catch {
    // config unreadable — treat as first launch
  }
  showView('onboarding');
  showSlide(0);
}

void boot();
