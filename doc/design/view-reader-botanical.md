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
and loop iteration data. Chips serialize source line and byte spans so another
call/iteration frame can find the same source mark without guessing by kind.
On viewports wider than 1000px with 180px available on each side, notes use both
page margins, varying horizontal reach and vertical offset with collision
avoidance. Dense bands that would drift more than 170px retain inline notes.
Compact screens, printing, and expanded overflow bands use ordinary document
flow. A shared SVG overlay draws continuously curved connections from the actual
mark's source-facing lower edge to the annotation title. The default is a full
S-curve with restrained gold/olive gradient; candidate sweeps are checked against
visible source text and already drawn connections. Tight prose uses two joined
curves below the text baseline and through the free margin, with no straight rail.
Notes sit closer to the document with varying horizontal/vertical offsets, rather
than stretching every connector across the entire page. Wrapped source marks use
their actual line fragments. Hover/focus highlights the exact source and connector.
Unresolved source anchors retain inline results instead of inventing a connection.
Clicking a note shows its full captured detail. No explanatory prose or runtime
values are invented.

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
The floating toolbar labels its icons 刷新 / 设置 / 运行 / 清理 / 更多.
The bottom play button invokes actual live execution; static pages explain how to
start a live view. Static clear hides captured output locally; reload restores it.
Settings control light/dark theme, text size and annotation visibility. More keeps
source, advanced execution controls, file navigation and print within reach.

## Validation

Run `cargo test --offline --lib view::` and `cargo test --offline --test gold view_`.
The browser regression is `tests/view/reader.browser.cjs <URL>`; it accepts
`MARQDO_PLAYWRIGHT_MODULE`, `MARQDO_CHROMIUM`, and optional
`MARQDO_SCREENSHOT_DIR`. It blocks HTTPS resources to exercise offline reading,
checks seven viewport widths, note bounds/collisions, source/label curve endpoints, visible curvature and non-crossing paths,
shared-source frame binding, stable redraw, real panel restoration,
search, frame selection, note inspection and live/static run behavior.
