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
