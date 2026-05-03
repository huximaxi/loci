/**
 * fold.js — Collapsible section toggles
 * loci.garden
 */
document.addEventListener('DOMContentLoaded', () => {
  document.querySelectorAll('.fold-trigger').forEach(btn => {
    btn.addEventListener('click', () => {
      const expanded = btn.getAttribute('aria-expanded') === 'true';
      btn.setAttribute('aria-expanded', String(!expanded));
      const target = document.getElementById(btn.getAttribute('aria-controls'));
      if (target) {
        target.classList.toggle('fold-section--open', !expanded);
        target.classList.toggle('fold-section--collapsed', expanded);
      }
    });
  });
});
