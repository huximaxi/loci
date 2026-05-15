/* nav-persona.js — Mini persona switcher for 2nd-level pages.
 * Writes localStorage['loci-theme'] then routes back to '/'.
 * The landing page's setTheme() reads this key on init, so the
 * user returns home in the persona they picked. */
(function () {
  'use strict';

  function setPersonaAndReturn(theme) {
    try { localStorage.setItem('loci-theme', theme); }
    catch (_) { /* private mode or storage disabled — still navigate */ }
    window.location.href = '/';
  }

  function markActive() {
    var current;
    try { current = localStorage.getItem('loci-theme') || 'scholar'; }
    catch (_) { current = 'scholar'; }
    var match = document.querySelector('.persona-mini-btn[data-theme="' + current + '"]');
    if (match) match.classList.add('active');
  }

  function init() {
    var btns = document.querySelectorAll('.persona-mini-btn');
    if (!btns.length) return;
    btns.forEach(function (btn) {
      btn.addEventListener('click', function () {
        setPersonaAndReturn(btn.dataset.theme);
      });
    });
    markActive();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
