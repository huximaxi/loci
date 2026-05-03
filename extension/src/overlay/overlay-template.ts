// ─── loci overlay HTML template ───────────────────────────────────────────────
// Exported as a plain string so the overlay module can stamp it into the
// Shadow DOM without any framework dependency.

export const overlayHTML = /* html */ `
<div id="loci-backdrop">
  <div id="loci-modal" role="dialog" aria-modal="true" aria-label="loci search">

    <div id="loci-search-bar">
      <svg viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
        <circle cx="8.5" cy="8.5" r="5.75" stroke="currentColor" stroke-width="1.5"/>
        <path d="M13.25 13.25L17 17" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      <input
        id="loci-input"
        type="search"
        placeholder="Search your conversations…"
        autocomplete="off"
        autocorrect="off"
        autocapitalize="off"
        spellcheck="false"
      />
      <kbd>esc</kbd>
    </div>

    <div id="loci-results" role="listbox" aria-label="Search results"></div>

    <div id="loci-footer">
      <span><kbd>↑</kbd><kbd>↓</kbd> navigate</span>
      <span><kbd>↵</kbd> open</span>
      <span><kbd>esc</kbd> close</span>
    </div>

  </div>
</div>
`;
