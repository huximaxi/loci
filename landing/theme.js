/* theme.js — depth-dial switcher behaviour for partial-nav (secondary) pages.
 * The landing has its own inline setTheme; this mirrors it for every page that
 * includes partials/_nav.html, so the chosen theme can be CHANGED in place
 * (not bounced home, the old nav-persona.js behaviour). The head pre-paint
 * script already applied the stored theme before first paint; init() here just
 * re-affirms it and lights the correct button. */
(function () {
  'use strict';
  var THEMES = ['scholar', 'wizard', 'llmage'];

  function setTheme(theme) {
    if (THEMES.indexOf(theme) === -1) return;
    document.documentElement.setAttribute('data-theme', theme);
    THEMES.forEach(function (t) {
      var btn = document.getElementById('btn-' + t);
      if (btn) {
        btn.classList.toggle('active', t === theme);
        btn.setAttribute('aria-pressed', String(t === theme));
      }
    });
    try { localStorage.setItem('loci-theme', theme); } catch (e) {}
  }
  // exposed for the inline onclick="setTheme('…')" handlers in _nav.html
  window.setTheme = setTheme;

  function init() {
    var saved = 'scholar';
    try { saved = localStorage.getItem('loci-theme') || 'scholar'; } catch (e) {}
    if (THEMES.indexOf(saved) === -1) saved = 'scholar';
    setTheme(saved);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
