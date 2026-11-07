# View botanical reader

Implemented 2026-11-07 for the shared `marqdo view` and `marqdo view output` shell.

The default is warm paper, serif document text, olive runtime notes, gold accents,
and generated transparent blossom/feather ornaments. The M/book logo is embedded
SVG. Navigation and the floating toolbar are code-native SVG icons and CSS.

`src/view/reader.css` is the final theme layer after notebook component styles;
`src/view/reader.js` owns reader navigation, dialogs and annotation geometry.
Notebook APIs and controls remain available through More → advanced tools.
Welcome files get a larger introduction; ordinary files retain compact titles.

## Runtime notes

Annotations keep their existing owner band, stable IDs, source anchor, call frame
and loop iteration data. On viewports wider than 1000px with 180px available on
each side, notes alternate between margin rails with collision avoidance. Dense
bands that would drift more than 140px from their statement retain inline notes.
Compact screens, printing, and expanded overflow bands use ordinary document
flow. Connectors are CSS lines in the gutter, avoiding the paragraph text;
hover/focus highlights the precise source anchor. Clicking a note shows its full
captured detail. No explanatory prose or runtime values are invented.

Layout updates after window/font/structure resize, frame selection and live DOM
replacement. Static HTML includes the same code and embedded assets. All KaTeX
scripts, CSS and fonts are bundled for offline math. Optional syntax highlighting
retains the previous CDN progressive enhancement; code remains readable offline.

`api/session/run_document` now returns prose HTML, bindings and trace JSON from
its own capture. The reader applies that capture without rerunning the program.
Page tokens prevent a changed source from receiving mismatched annotations.

## Navigation

Document returns to prose. Functions/Variables move the real existing panels into
a modal and restore them on close, preserving their existing event handlers.
Memory shows the captured execution report, labeled as a snapshot on exports.
Run navigates to the output section. Search finds document, function and file text.
The bottom play button invokes actual live execution; static pages explain how to
start a live view. Static clear hides captured output locally; reload restores it.
Settings control light/dark theme, text size and annotation visibility. More keeps
source, advanced execution controls, file navigation and print within reach.

## Validation

Run `cargo test --offline --lib view::` and `cargo test --offline --test gold view_`.
The browser regression is `tests/view/reader.browser.cjs <URL>`; it accepts
`MARQDO_PLAYWRIGHT_MODULE`, `MARQDO_CHROMIUM`, and optional
`MARQDO_SCREENSHOT_DIR`. It blocks HTTPS resources to exercise offline reading,
checks six viewport widths, note bounds/collisions, real panel restoration,
search, frame selection, note inspection and live/static run behavior.
