/* ─── LOCI.GARDEN — Skin Toggle ──────────────────────────────────────────────
   Toggles between 'scholar' (default) and 'wizard' (pixel art) skins.
   Persists via localStorage. Zero dependencies.
   ──────────────────────────────────────────────────────────────────────────── */

(function () {
  const STORAGE_KEY = 'loci-skin';
  const html = document.documentElement;

  // Read saved preference
  const saved = localStorage.getItem(STORAGE_KEY) || 'scholar';
  if (saved === 'wizard') html.setAttribute('data-skin', 'wizard');

  function currentSkin() {
    return html.getAttribute('data-skin') || 'scholar';
  }

  function setSkin(skin) {
    if (skin === 'wizard') {
      html.setAttribute('data-skin', 'wizard');
    } else {
      html.removeAttribute('data-skin');
    }
    localStorage.setItem(STORAGE_KEY, skin);
    updateToggleLabel();
  }

  function toggle() {
    setSkin(currentSkin() === 'wizard' ? 'scholar' : 'wizard');
  }

  function updateToggleLabel() {
    const btn = document.getElementById('skin-toggle-btn');
    if (!btn) return;
    const isWizard = currentSkin() === 'wizard';
    btn.innerHTML = isWizard
      ? '<span class="toggle-crystal"></span> Scholar'
      : '<span class="toggle-crystal"></span> Wizard';
    btn.title = isWizard ? 'Switch to Scholar skin' : 'Switch to Wizard skin';
  }

  // Inject toggle button
  document.addEventListener('DOMContentLoaded', function () {
    const btn = document.createElement('button');
    btn.id = 'skin-toggle-btn';
    btn.className = 'skin-toggle';
    btn.addEventListener('click', toggle);
    document.body.appendChild(btn);
    updateToggleLabel();
  });
})();
