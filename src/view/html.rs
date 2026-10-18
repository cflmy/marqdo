//! HTML shell for `marqdo view` — Apple-inspired monochrome, system fonts only.

use std::path::{Component, Path, PathBuf};

use crate::view::render::FileViewModel;

pub fn escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".into(),
            '<' => "&lt;".into(),
            '>' => "&gt;".into(),
            '"' => "&quot;".into(),
            '\'' => "&#39;".into(),
            _ => c.to_string(),
        })
        .collect()
}

/// How navigation hrefs are built.
#[derive(Debug, Clone)]
pub enum LinkMode {
    /// Live server: `/file?path=…`
    Live,
    /// Static export: relative links under `pages/`.
    /// `from` is the source rel path of the current page (`None` = index.html).
    Static { from: Option<String> },
}

fn stylesheet() -> &'static str {
    // Inspired by Apple HIG clarity + docs-site layouts (sticky sidebar, readable
    // center column, system grays). Monochrome only; no remote fonts.
    r#"
:root {
  --bg: #f5f5f7;
  --surface: #ffffff;
  --ink: #1d1d1f;
  --muted: #6e6e73;
  --line: #d2d2d7;
  --line-strong: #1d1d1f;
  --fill: #f5f5f7;
  --hover: #e8e8ed;
  --focus: #0071e3;
  --fail-bg: #fff5f5;
  --ok-bg: #f5f5f7;
  --radius: 12px;
  --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  --sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
  --syn-fg: #f0f0f0;
  --syn-comment: #6a9955;
  --syn-keyword: #569cd6;
  --syn-string: #ce9178;
  --syn-number: #b5cea8;
  --syn-variable: #9cdcfe;
  --syn-function: #dcdcaa;
  --syn-type: #4ec9b0;
  --syn-meta: #c586c0;
}
* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body {
  margin: 0;
  min-height: 100vh;
  font-family: var(--sans);
  color: var(--ink);
  background: var(--bg);
  line-height: 1.47;
  font-size: 17px;
  -webkit-font-smoothing: antialiased;
  text-rendering: optimizeLegibility;
}
a { color: var(--ink); text-decoration: none; }
a:hover { opacity: 0.72; }
a:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; }
#nav-toggle { position: absolute; opacity: 0; pointer-events: none; }
.topbar {
  display: none;
  align-items: center;
  gap: 0.75rem;
  padding: 0.7rem 1rem;
  border-bottom: 1px solid var(--line);
  background: rgba(255,255,255,0.86);
  backdrop-filter: saturate(180%) blur(16px);
  -webkit-backdrop-filter: saturate(180%) blur(16px);
  position: sticky;
  top: 0;
  z-index: 20;
}
.topbar .brand { margin: 0; font-size: 1.05rem; font-weight: 600; letter-spacing: -0.022em; }
.brand-row {
  display: flex;
  align-items: center;
  gap: 0.55rem;
}
.brand-row .brand { margin: 0; }
.logo {
  width: 1.75rem;
  height: 1.75rem;
  object-fit: contain;
  flex-shrink: 0;
  border-radius: 6px;
}
.topbar .logo { width: 1.45rem; height: 1.45rem; }
.nav-brand .logo { width: 2rem; height: 2rem; }
.nav-btn {
  appearance: none;
  border: 1px solid var(--line);
  background: var(--surface);
  color: var(--ink);
  border-radius: 10px;
  width: 2.35rem;
  height: 2.35rem;
  font-size: 1.05rem;
  line-height: 1;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s ease, border-color 0.15s ease;
}
.nav-btn:hover { background: var(--hover); }
.shell {
  display: grid;
  grid-template-columns: 280px minmax(0, 1fr);
  min-height: 100vh;
  max-width: 1680px;
  margin: 0 auto;
}
.nav {
  border-right: 1px solid var(--line);
  padding: 1.75rem 1rem 2.5rem 1.15rem;
  overflow: auto;
  background: var(--bg);
  position: sticky;
  top: 0;
  align-self: start;
  height: 100vh;
}
.nav-brand { display: block; padding: 0 0.35rem; margin-bottom: 1.15rem; }
.nav-brand .tagline { margin: 0.1rem 0 0; }
.brand {
  font-size: 1.35rem;
  font-weight: 700;
  letter-spacing: -0.03em;
  margin: 0 0 0.1rem;
}
.tagline {
  color: var(--muted);
  font-size: 0.78rem;
  margin: 0;
  font-weight: 500;
}
.nav > h2:first-of-type { margin-top: 0.35rem; }
.nav h2 {
  font-size: 0.68rem;
  text-transform: uppercase;
  letter-spacing: 0.09em;
  color: var(--muted);
  font-weight: 600;
  margin: 1.15rem 0.35rem 0.45rem;
}
.nav ul { list-style: none; padding: 0; margin: 0; }
.nav li { margin: 0.08rem 0; }
.nav a.file {
  display: block;
  padding: 0.42rem 0.65rem;
  border-radius: 8px;
  border-bottom: none;
  color: var(--ink);
  font-family: var(--mono);
  font-size: 0.78rem;
  transition: background 0.12s ease, color 0.12s ease;
}
.nav a.file:hover { background: var(--hover); opacity: 1; }
.nav a.file.active {
  background: var(--ink);
  color: #fff;
  opacity: 1;
  font-weight: 500;
}
.nav-root { margin: 0; padding: 0; list-style: none; }
.folder {
  margin: 0.55rem 0 0.2rem;
  border: none;
}
.folder > summary {
  list-style: none;
  cursor: pointer;
  user-select: none;
  color: var(--muted);
  font-size: 0.72rem;
  font-weight: 600;
  letter-spacing: 0.01em;
  padding: 0.35rem 0.45rem;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 0.35rem;
  transition: background 0.12s ease, color 0.12s ease;
}
.folder > summary::-webkit-details-marker { display: none; }
.folder > summary::before {
  content: "";
  width: 0;
  height: 0;
  border-style: solid;
  border-width: 4px 0 4px 6px;
  border-color: transparent transparent transparent var(--muted);
  opacity: 0.85;
  transition: transform 0.12s ease;
  flex-shrink: 0;
}
.folder[open] > summary::before {
  transform: rotate(90deg);
}
.folder > summary:hover {
  background: var(--hover);
  color: var(--ink);
}
.folder > ul {
  list-style: none;
  padding: 0.1rem 0 0.15rem 0.55rem;
  margin: 0;
}
.panel-wrap {
  min-width: 0;
  overflow: auto;
  background: var(--bg);
}
main.panel {
  padding: 2rem 2.25rem 3.5rem;
  margin: 0;
  width: 100%;
  max-width: 76rem;
}
h1.page {
  font-size: clamp(1.75rem, 2.4vw, 2.35rem);
  font-weight: 700;
  letter-spacing: -0.035em;
  margin: 0 0 0.3rem;
  line-height: 1.15;
}
.meta {
  color: var(--muted);
  font-size: 0.88rem;
  margin: 0 0 1.75rem;
  font-family: var(--mono);
}
.status-pill {
  display: inline-block;
  font-size: 0.68rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  padding: 0.18rem 0.5rem;
  border-radius: 999px;
  border: 1px solid var(--line);
  margin-left: 0.5rem;
  vertical-align: middle;
}
.status-pill.ok { background: var(--surface); color: var(--muted); }
.status-pill.pending { background: var(--surface); color: var(--ink); border-color: var(--ink); }
.status-pill.fail { background: var(--ink); color: #fff; border-color: var(--ink); }
section.block { margin: 2rem 0; }
section.block > h2 {
  font-size: 0.72rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--muted);
  font-weight: 600;
  margin: 0 0 0.85rem;
}
.stdin-form {
  margin: 0 0 1rem;
  display: grid;
  gap: 0.45rem;
  max-width: 36rem;
}
.stdin-form label {
  font-size: 0.85rem;
  color: var(--ink);
}
.stdin-form .hint {
  color: var(--muted);
  font-size: 0.8rem;
  margin-left: 0.35rem;
}
.stdin-form textarea {
  font: inherit;
  font-family: var(--mono);
  font-size: 0.9rem;
  padding: 0.65rem 0.75rem;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--surface);
  color: var(--ink);
  resize: vertical;
  min-height: 4.5rem;
}
.stdin-form button {
  justify-self: start;
  font: inherit;
  font-size: 0.9rem;
  font-weight: 600;
  padding: 0.5rem 1rem;
  border: 1px solid var(--ink);
  border-radius: 8px;
  background: var(--ink);
  color: #ffffff;
  cursor: pointer;
}
.stdin-form button:hover { opacity: 0.88; }
.stdin-form button:focus-visible {
  outline: 2px solid var(--focus);
  outline-offset: 2px;
}
.structure {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 0.35rem 0.15rem;
  box-shadow: 0 1px 0 rgba(0,0,0,0.02);
}
.card {
  border: none;
  border-bottom: 1px solid var(--line);
  border-radius: 0;
  padding: 0.85rem 1.1rem;
  margin: 0;
  background: transparent;
}
.card:last-child { border-bottom: none; }
.fun-card, .call-card, .branch-card, .loop-card, .ret-card {
  border-left: 2px solid var(--line-strong);
  padding-left: calc(1.1rem - 2px);
}
.comment-card {
  border-left: 2px solid var(--line);
  background: transparent;
  padding-left: calc(1.1rem - 2px);
}
/* Document surface (Browse Structure) */
.document-surface {
  padding: 0.5rem 1rem 1.25rem;
  color: var(--ink);
  line-height: 1.55;
}
.doc-imports { margin: 0 0 0.85rem; }
.doc-fun {
  margin: 1.1rem 0 0.65rem;
  padding-bottom: 0.35rem;
  border-bottom: 1px solid var(--line);
}
.doc-fun-title {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 650;
  letter-spacing: -0.01em;
}
.doc-fun-bar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem 1rem;
}
.call-picker-label {
  font-size: 0.78rem;
  color: var(--muted);
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}
.call-picker, .loop-picker {
  font-size: 0.78rem;
  padding: 0.2rem 0.4rem;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: var(--surface);
  color: var(--ink);
}
.loop-picker-label {
  font-size: 0.78rem;
  color: var(--muted);
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}
li.doc-list-meta {
  list-style: none;
  margin: 0.25rem 0 0.5rem;
  margin-left: -1.25rem;
}
.doc-block { position: relative; margin: 0.55rem 0 0.85rem; }
.doc-block > .annot-leaders {
  position: absolute;
  inset: 0;
  pointer-events: none;
  overflow: visible;
  z-index: 1; /* below .annot-chips */
}
.doc-block > .annot-leaders svg {
  position: absolute;
  left: 0;
  top: 0;
  overflow: visible;
}
.doc-block.is-keyed > .annot-leaders { display: none; }
.doc-block.doc-exec[style*="--doc-indent"] {
  margin-left: calc(var(--doc-indent, 1) * 1.5rem);
}
.doc-prose { color: var(--ink); }
.doc-prose > :first-child { margin-top: 0; }
.doc-prose > :last-child { margin-bottom: 0; }
.doc-line { margin: 0.15rem 0; }
.doc-prose-line { white-space: pre-wrap; }
.doc-stmt, .doc-call, .doc-cond {
  font-family: var(--mono);
  font-size: 0.92em;
}
.mq-call-gt { color: var(--muted); margin-right: 0.25rem; }
.mq-else { font-family: var(--mono); color: var(--muted); font-style: italic; }
.doc-table-wrap { overflow-x: auto; }
.doc-table {
  border-collapse: collapse;
  width: auto;
  min-width: 12rem;
  font-size: 0.92rem;
  margin: 0.35rem 0;
}
.doc-table th, .doc-table td {
  border: 1px solid var(--line);
  padding: 0.35rem 0.7rem;
  text-align: left;
}
.doc-table th {
  background: var(--fill);
  font-weight: 600;
}
ul.doc-loop, ol.doc-branch {
  margin: 0.5rem 0 0.75rem;
  padding-left: calc(var(--doc-indent, 1) * 1.5rem + 1.25rem);
}
.doc-list-item { margin: 0.45rem 0; }
.doc-list-head { font-family: var(--mono); font-size: 0.92em; }
.doc-list-body {
  margin: 0.35rem 0 0.2rem;
  padding-left: 1.5rem;
  border-left: 2px solid var(--line);
}
.doc-list-item.arm-taken {
  border-left: 3px solid #bf5af2;
  padding-left: 0.55rem;
  margin-left: -0.15rem;
}
.doc-list-item.arm-taken > .doc-list-head { font-weight: 600; }
.doc-list-item.arm-miss { opacity: 0.45; }
.doc-code {
  font-family: var(--mono);
  font-size: 0.9em;
  white-space: pre-wrap;
}
.mq-anchor {
  background: transparent;
  color: inherit;
  padding: 0 0.05em;
  border-radius: 0;
  border-bottom: 2px solid var(--line-strong);
  font-style: normal;
  font-weight: inherit;
  cursor: default;
}
.mq-anchor.mq-decl {
  border-bottom-color: #5ac8fa;
  font-family: var(--mono);
  font-size: 0.95em;
}
.mq-anchor.mq-code {
  border-bottom-color: #ff9f0a;
  font-family: var(--mono);
  font-size: 0.95em;
  font-weight: 600;
}
.mq-anchor.mq-return {
  border-bottom-color: #30d158;
  font-family: var(--mono);
  font-size: 0.95em;
  font-style: italic;
}
.mq-anchor.is-focus { background: rgba(0,0,0,0.04); }
.annot-band {
  position: relative;
  margin: 0.55rem 0 0.15rem;
  min-height: 1.6rem;
  padding-top: 1.15rem; /* room for mid→mid leaders (rail + stub) */
  z-index: 1;
}
/* Band-local leaders kept for keyed fallback; geometry uses .doc-block > .annot-leaders */
.annot-band > .annot-leaders { display: none; }
.annot-leader {
  fill: none;
  stroke-width: 1.5;
  stroke-linecap: round;
  stroke-linejoin: round;
  pointer-events: none;
}
.annot-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 0.45rem 0.7rem;
  position: relative;
  z-index: 3; /* chips above leaders */
}
.annot-chip {
  display: inline-flex;
  align-items: baseline;
  gap: 0.35rem;
  max-width: 100%;
  margin: 0;
  padding: 0.22rem 0.5rem;
  border: 1px solid var(--line);
  border-left-width: 3px;
  border-radius: 8px;
  background: var(--fill);
  color: var(--ink);
  font-family: var(--mono);
  font-size: 0.72rem;
  cursor: pointer;
  text-align: left;
}
/* Author `display:inline-flex` must not defeat frame filtering. */
.annot-chip[hidden],
.annot-chip.is-frame-hidden {
  display: none !important;
}
.annot-chip:hover,
.annot-chip.is-focus { background: #fff; }
.annot-chip.annot-bind { border-left-color: #5ac8fa; }
.annot-chip.annot-return { border-left-color: #30d158; }
.annot-chip.annot-call,
.annot-chip.annot-expr,
.annot-chip.annot-code { border-left-color: #ff9f0a; }
.annot-chip.annot-branch { border-left-color: #bf5af2; }
.annot-chip.annot-loop { border-left-color: #64d2ff; }
.annot-chip.annot-writeback { border-left-color: #8e8e93; }
.annot-chip.annot-chip-overflow { display: none !important; }
.annot-band.is-expanded .annot-chip.annot-chip-overflow:not([hidden]):not(.is-frame-hidden) {
  display: inline-flex !important;
}
.annot-label { color: var(--muted); font-weight: 600; }
.annot-value { color: var(--ink); word-break: break-word; }
.annot-more {
  border: 1px dashed var(--line);
  border-radius: 8px;
  background: transparent;
  color: var(--muted);
  font-size: 0.72rem;
  padding: 0.22rem 0.45rem;
  cursor: pointer;
}
.annot-band.is-keyed .annot-chip::before,
.doc-block.is-keyed .annot-chip::before {
  content: attr(data-annot-key) " ";
  color: var(--muted);
}
@media (max-width: 640px) {
  .annot-band { padding-top: 0.25rem; }
  .doc-block > .annot-leaders { display: none; }
}
@media (prefers-reduced-motion: reduce) {
  .doc-block > .annot-leaders { display: none; }
}
.badge {
  display: inline-block;
  font-family: var(--mono);
  font-size: 0.66rem;
  padding: 0.14rem 0.4rem;
  border-radius: 6px;
  border: 1px solid var(--line);
  background: var(--fill);
  color: var(--ink);
  margin-right: 0.4rem;
  font-weight: 600;
  letter-spacing: 0.02em;
}
.params { margin: 0.35rem 0 0; }
.chip {
  display: inline-block;
  font-family: var(--mono);
  font-size: 0.72rem;
  padding: 0.14rem 0.45rem;
  margin: 0.15rem 0.15rem 0 0;
  border-radius: 6px;
  background: var(--fill);
  border: 1px solid transparent;
  color: var(--ink);
}
.chip.inferred {
  border-style: dashed;
  border-color: var(--line);
  opacity: 0.92;
}
.chip.inferred::after {
  content: " ↑";
  font-size: 0.65em;
  opacity: 0.7;
}
.chip .chip-val {
  font-weight: 500;
  opacity: 0.78;
  font-family: var(--mono);
  font-size: 0.92em;
}
.chip .chip-val:empty {
  display: none;
}
.nested {
  margin: 0.45rem 0 0.15rem 0.15rem;
  padding-left: 0.85rem;
  border-left: 1px solid var(--line);
}
.arm {
  margin: 0.4rem 0;
  padding: 0.55rem 0.7rem;
  background: var(--fill);
  border-radius: 10px;
  border: 1px solid transparent;
}
.exec-clear {
  float: right;
  font: inherit;
  font-size: 0.72rem;
  padding: 0.2rem 0.6rem;
  border: 1px solid var(--line);
  border-radius: 999px;
  background: var(--surface);
  color: var(--muted);
  cursor: pointer;
}
.exec-clear:hover { color: var(--ink); border-color: var(--muted); }
body.notebook-page .exec-clear {
  border-color: var(--nb-border);
  background: var(--nb-surface-2);
  color: var(--nb-text);
}
body.notebook-page .exec-clear:hover { border-color: var(--nb-accent); color: var(--nb-strong); }
.out, .source {
  font-family: var(--mono);
  font-size: 0.82rem;
  white-space: pre-wrap;
  background: var(--surface);
  border-radius: var(--radius);
  padding: 1rem 1.15rem;
  border: 1px solid var(--line);
  line-height: 1.55;
  box-shadow: 0 1px 0 rgba(0,0,0,0.02);
}
.stmt { display: block; }
.stmt-body { min-width: 0; }
.stmt-body > .card { border-bottom: 1px solid var(--line); }
.structure > .stmt:last-child > .stmt-body > .card,
.structure > .card:last-child { border-bottom: none; }
.struct-wrap {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(9.5rem, 12.5rem);
  gap: 0.75rem;
  align-items: start;
}
@media (max-width: 960px) {
  .struct-wrap { grid-template-columns: 1fr; }
  .outline-rail, .outline-panel { position: static; }
}
.outline-rail {
  position: sticky;
  top: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
  min-width: 0;
}
.outline-section {
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 0.35rem 0.55rem 0.55rem;
  box-shadow: 0 1px 0 rgba(0,0,0,0.02);
}
.outline-summary {
  cursor: pointer;
  font: 600 0.72rem/1.3 var(--sans);
  color: var(--muted);
  letter-spacing: 0.02em;
  text-transform: uppercase;
  list-style: none;
  padding: 0.2rem 0 0.35rem;
}
.outline-summary::-webkit-details-marker { display: none; }
.outline-panel {
  position: static;
  background: transparent;
  border: none;
  border-radius: 0;
  padding: 0;
  box-shadow: none;
}
.vars-panel {
  max-height: min(52vh, 30rem);
  overflow: auto;
  font: 0.72rem/1.35 var(--sans);
  padding-right: 0.1rem;
}
.vars-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
}
.vars-card {
  display: block;
  margin: 0;
  padding: 0.45rem 0.5rem 0.5rem;
  border: 1px solid var(--line);
  border-radius: 10px;
  background: #fff;
  cursor: pointer;
  transition: border-color 0.12s ease, box-shadow 0.12s ease;
}
.vars-card:hover {
  border-color: #a1a1a6;
  box-shadow: 0 4px 14px rgba(0,0,0,0.06);
}
.vars-card:focus-visible {
  outline: 2px solid var(--focus);
  outline-offset: 2px;
}
.vars-head {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  justify-content: space-between;
  gap: 0.25rem 0.55rem;
  margin-bottom: 0.35rem;
}
.vars-name {
  font: 600 0.8rem/1.25 var(--mono);
  color: var(--ink);
  word-break: break-all;
}
.vars-kind {
  font: 500 0.65rem/1.2 var(--sans);
  color: var(--muted);
}
.vars-preview {
  max-height: 7.5rem;
  overflow: hidden;
  position: relative;
  border-radius: 8px;
  background: #fafafa;
  border: 1px solid var(--line);
  padding: 0.35rem;
  pointer-events: none;
}
.vars-preview::after {
  content: "";
  position: absolute;
  left: 0; right: 0; bottom: 0;
  height: 1.6rem;
  background: linear-gradient(transparent, #fafafa);
  pointer-events: none;
}
.vars-preview .matrix-wrap {
  border: none;
  padding: 0.25rem;
  background: transparent;
}
.vars-preview .matrix-cell {
  min-width: 2.4rem;
  padding: 0.28rem 0.35rem;
  font-size: 0.68rem;
}
.vars-muted { color: var(--muted); }
.vars-peek { font-size: 0.68rem; }
.math-sm .katex-display { font-size: 0.92em; margin: 0.1rem 0; }
.vars-table {
  width: 100%;
  border-collapse: collapse;
  font: 0.72rem/1.35 var(--mono);
}
.vars-table th,
.vars-table td {
  border: 1px solid var(--line);
  padding: 0.15rem 0.3rem;
  text-align: left;
  vertical-align: top;
  word-break: break-word;
}
.vars-table th {
  background: #f5f5f7;
  font-weight: 600;
  color: var(--muted);
}
.vars-idx, .vars-key { color: var(--muted); white-space: nowrap; }
.vars-scalar { white-space: pre-wrap; }
.vars-empty, .vars-skipped { color: var(--muted); margin: 0.2rem 0; }
.vars-nested > summary { cursor: pointer; color: var(--muted); }
.var-dialog {
  border: none;
  padding: 0;
  max-width: min(920px, 94vw);
  max-height: min(88vh, 900px);
  width: min(920px, 94vw);
  border-radius: 18px;
  background: transparent;
  box-shadow: none;
}
.var-dialog::backdrop {
  background: rgba(29, 29, 31, 0.42);
  backdrop-filter: blur(6px);
}
.var-dialog-sheet {
  background: linear-gradient(180deg, #fffef9 0%, #ffffff 42%);
  border: 1px solid var(--line);
  border-radius: 18px;
  box-shadow: 0 24px 80px rgba(0,0,0,0.22);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  max-height: min(88vh, 900px);
}
.var-dialog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem 1.15rem;
  border-bottom: 1px solid var(--line);
  background: rgba(255,255,255,0.85);
}
.var-dialog-head h2 {
  margin: 0;
  font: 650 1.05rem/1.25 var(--mono);
  color: var(--ink);
  word-break: break-all;
}
.var-dialog-close {
  appearance: none;
  width: 2rem;
  height: 2rem;
  border-radius: 999px;
  border: 1px solid var(--line);
  background: #f5f5f7;
  color: var(--ink);
  font-size: 1.15rem;
  line-height: 1;
  cursor: pointer;
}
.var-dialog-close:hover { background: var(--hover); }
.var-dialog-body {
  padding: 1.15rem 1.25rem 1.4rem;
  overflow: auto;
  font: 0.85rem/1.45 var(--sans);
}
.var-rich-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem;
  margin: 0 0 0.85rem;
}
.var-chip {
  display: inline-block;
  padding: 0.2rem 0.55rem;
  border-radius: 999px;
  background: #1d1d1f;
  color: #fff;
  font: 600 0.68rem/1.2 var(--sans);
}
.var-chip.muted {
  background: #e8e8ed;
  color: var(--muted);
}
.var-rich-more {
  margin-top: 1rem;
  color: var(--muted);
  font-size: 0.78rem;
}
.matrix-wrap {
  overflow: auto;
  border-radius: 14px;
  border: 1px solid var(--line);
  background:
    radial-gradient(120% 80% at 0% 0%, rgba(0,113,227,0.06), transparent 55%),
    #fff;
  padding: 0.75rem;
}
.matrix-cap {
  caption-side: top;
  text-align: left;
  font: 500 0.7rem/1.2 var(--sans);
  color: var(--muted);
  margin-bottom: 0.45rem;
}
.matrix-grid {
  border-collapse: separate;
  border-spacing: 0.35rem;
  margin: 0 auto;
  font: 0.78rem/1.2 var(--mono);
}
.matrix-cell {
  min-width: 3.6rem;
  padding: 0.55rem 0.65rem;
  text-align: center;
  border-radius: 10px;
  border: 1px solid var(--line);
  background: #f7f8fa;
  color: var(--ink);
  box-shadow: inset 0 1px 0 rgba(255,255,255,0.65);
}
.matrix-cell span { font-variant-numeric: tabular-nums; }
.math-lg .katex-display { font-size: 1.15em; }
.fn-search {
  width: 100%;
  box-sizing: border-box;
  margin: 0 0 0.45rem;
  padding: 0.4rem 0.5rem;
  border: 1px solid var(--line);
  border-radius: 8px;
  font: 0.78rem/1.2 var(--sans);
  color: var(--ink);
  background: #fff;
}
.fn-search:focus {
  outline: 2px solid var(--focus);
  outline-offset: 1px;
}
.outline-tree, .outline-tree ul {
  list-style: none;
  margin: 0;
  padding: 0;
}
.outline-tree ul {
  padding-left: 0.7rem;
  border-left: 1px solid var(--line);
  margin-left: 0.25rem;
}
.outline-item {
  margin: 0.12rem 0;
  font-size: 0.78rem;
  line-height: 1.35;
}
.outline-item.hidden { display: none; }
.outline-item > a {
  font-family: var(--mono);
  font-size: 0.76rem;
  color: var(--ink);
}
.outline-item > a:hover { opacity: 1; text-decoration: underline; }
.outline-item .ol-meta {
  color: var(--muted);
  font-size: 0.66rem;
  margin-left: 0.2rem;
}
.outline-item .ol-params {
  margin-left: 0.25rem;
  font-family: var(--mono);
  font-size: 0.62rem;
  color: var(--muted);
}
.outline-item .ol-param.inferred {
  border-bottom: 1px dashed var(--line);
}
.fun-card:target {
  outline: 2px solid var(--focus);
  outline-offset: 2px;
  background: #f0f7ff;
}
.out.ok { background: var(--ok-bg); }
.out.fail {
  background: var(--fail-bg);
  border-color: #e4c1c1;
  color: var(--ink);
  font-weight: 500;
}
.formula-card .math-block {
  display: block;
  margin: 0.35rem 0 0.1rem;
  overflow: visible;
  max-width: 100%;
}
.formula-card .katex-display {
  margin: 0.2rem 0;
  overflow-x: auto;
  overflow-y: hidden;
  padding-bottom: 0.15rem;
}
.code-card {
  padding: 0;
  overflow: hidden;
}
.output-card {
  border-left: 3px solid #2563eb;
  background: #f8fafc;
}
.output-card.output-card-image {
  border-left-color: #0d9488;
}
.output-card .wb-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
}
.wb-badge.wb-ok { background: #dcfce7; color: #166534; }
.wb-badge.wb-error { background: #fee2e2; color: #991b1b; }
.wb-badge.wb-trace { background: #e0e7ff; color: #3730a3; }
.output-card .output-body {
  margin: 0.35rem 0 0;
  padding: 0.5rem 0.65rem;
  font: 0.85rem/1.45 var(--mono);
  white-space: pre-wrap;
  color: #0f172a;
}
.wb-figure {
  margin: 0.5rem 0 0;
  padding: 0;
}
.wb-image {
  display: block;
  max-width: 100%;
  height: auto;
  border-radius: 6px;
  border: 1px solid var(--line);
  background: #fff;
}
.wb-image-url {
  margin: 0.35rem 0 0;
  font: 0.75rem/1.35 var(--mono);
  color: var(--muted);
  word-break: break-all;
}
.doc-writeback {
  margin: 0.35rem 0;
  padding: 0.45rem 0.6rem;
  border-radius: 8px;
  border: 1px dashed #94a3b8;
  background: #f1f5f9;
}
.doc-writeback[data-wb-kind="image"] {
  border-color: #0d9488;
  background: #f0fdfa;
}
.wb-source-badge {
  display: inline-block;
  margin-bottom: 0.28rem;
  font: 0.68rem/1.2 var(--sans);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: #334155;
  background: #e2e8f0;
  padding: 0.12rem 0.4rem;
  border-radius: 4px;
}
.doc-writeback[data-wb-kind="image"] .wb-source-badge {
  background: #ccfbf1;
  color: #115e59;
}
.wb-source-text {
  display: block;
  font: 0.82rem/1.4 var(--mono);
  color: #0f172a;
  white-space: pre-wrap;
  word-break: break-word;
}
.wb-outline {
  margin-top: 0.75rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--line);
}
.wb-outline-title {
  font: 0.72rem/1.2 var(--sans);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--muted);
  margin-bottom: 0.35rem;
}
.plan-card {
  margin: 0 0 0.85rem;
  padding: 0.75rem 0.9rem;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  background: #f8fafc;
  font: 0.88rem/1.45 var(--sans);
}
.plan-card .plan-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem 1rem;
  margin: 0.35rem 0 0.55rem;
  color: var(--muted);
  font: 0.8rem/1.35 var(--mono);
}
.plan-card a { color: #1d4ed8; }
.plan-card .plan-process {
  list-style: none;
  margin: 0.45rem 0 0;
  padding: 0;
  border-top: 1px solid var(--line);
}
.plan-card .plan-process li {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem 0.65rem;
  padding: 0.4rem 0;
  border-bottom: 1px solid #eef2f7;
  font: 0.8rem/1.35 var(--sans);
  color: var(--ink);
}
.plan-card .plan-process li:last-child { border-bottom: none; }
.plan-card .plan-process .pp-type {
  font: 0.72rem/1.2 var(--mono);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: #475569;
  min-width: 4.5rem;
}
.plan-card .plan-process .pp-detail {
  color: var(--muted);
  font: 0.78rem/1.35 var(--mono);
  word-break: break-word;
}
.stream-panel {
  margin: 0 0 0.85rem;
  border: 1px solid var(--line);
  border-radius: var(--radius);
  background: var(--surface);
  overflow: hidden;
  position: relative;
}
.stream-head {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  padding: 0.65rem 0.85rem;
  background: #fafafa;
  border-bottom: 1px solid var(--line);
  font: 0.85rem/1 var(--sans);
  color: var(--ink);
}
.stream-head button {
  font: 0.78rem/1 var(--sans);
  padding: 0.4rem 0.85rem;
  border-radius: 8px;
  border: 1px solid var(--line);
  background: #1d1d1f;
  color: #fff;
  cursor: pointer;
}
.stream-head button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.stream-status {
  margin-left: auto;
  color: var(--muted);
  font: 0.72rem/1 var(--mono);
}
.stream-head button.stream-stop {
  background: #fff;
  color: #1d1d1f;
  border-color: var(--line);
}
.stream-jump {
  position: absolute;
  left: 50%;
  bottom: 0.65rem;
  transform: translateX(-50%);
  z-index: 2;
  font: 0.72rem/1 var(--sans);
  padding: 0.35rem 0.7rem;
  border-radius: 999px;
  border: 1px solid var(--line);
  background: #fff;
  color: var(--ink);
  cursor: pointer;
  box-shadow: 0 1px 2px rgba(0,0,0,0.06);
}
.stream-jump[hidden] { display: none; }
.stream-body {
  position: relative;
  padding: 0.85rem 1rem 1rem;
  min-height: 5rem;
  max-height: 28rem;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.stream-empty {
  color: var(--muted);
  font: 0.85rem/1.4 var(--sans);
  margin: 0;
}
.stream-turn {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
}
.stream-thinking {
  border: 1px solid var(--line);
  border-radius: 10px;
  background: #f5f5f7;
  padding: 0.35rem 0.65rem 0.55rem;
}
.stream-thinking > summary {
  cursor: pointer;
  list-style: none;
  font: 0.78rem/1.4 var(--sans);
  color: var(--muted);
  user-select: none;
  padding: 0.25rem 0;
}
.stream-thinking > summary::-webkit-details-marker { display: none; }
.stream-thinking > summary::before {
  content: "▸ ";
  color: #aeaeb2;
}
.stream-thinking[open] > summary::before { content: "▾ "; }
.stream-thinking-body {
  margin: 0.15rem 0 0;
  font: 0.82rem/1.5 var(--sans);
  color: #6e6e73;
  white-space: pre-wrap;
  word-break: break-word;
}
.stream-answer {
  font: 0.95rem/1.55 var(--sans);
  color: var(--ink);
  white-space: pre-wrap;
  word-break: break-word;
}
.stream-answer:empty { display: none; }
.stream-decision {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.35rem 0.65rem;
  font: 0.78rem/1.35 var(--sans);
  color: var(--muted);
  padding: 0.15rem 0;
  border-top: 1px dashed var(--line);
}
.stream-decision strong {
  font-weight: 600;
  color: #1d1d1f;
  letter-spacing: 0.02em;
}
.stream-tool {
  border: 1px solid var(--line);
  border-left: 3px solid #94a3b8;
  border-radius: 8px;
  background: #f1f5f9;
  padding: 0.45rem 0.7rem;
  margin: 0.25rem 0;
}
.stream-tool-head {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.35rem 0.65rem;
  font: 0.78rem/1.3 var(--sans);
  color: var(--muted);
}
.stream-tool-head strong {
  color: #334155;
  font-weight: 600;
}
.stream-tool-meta {
  font: 0.72rem/1.3 var(--mono);
  color: #64748b;
}
.stream-tool-body {
  margin-top: 0.3rem;
  font: 0.78rem/1.4 var(--mono);
  color: #475569;
  white-space: pre-wrap;
  word-break: break-word;
}
.stream-child {
  border: 1px solid var(--line);
  border-left: 3px solid #2563eb;
  border-radius: 10px;
  background: #f8fafc;
  padding: 0.65rem 0.85rem;
}
.stream-child-await {
  border-left-color: #94a3b8;
  background: #f8fafc;
  opacity: 0.95;
}
.stream-child-await .stream-child-head strong {
  color: #64748b;
}
.stream-phase {
  display: flex;
  flex-wrap: wrap;
  align-items: baseline;
  gap: 0.35rem 0.65rem;
  font: 0.78rem/1.35 var(--sans);
  color: var(--muted);
  padding: 0.2rem 0;
  border-top: 1px dashed var(--line);
}
.stream-phase strong {
  font-weight: 600;
  color: #475569;
  letter-spacing: 0.02em;
}
.stream-child-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.4rem 0.75rem;
  font: 0.78rem/1.3 var(--sans);
  color: var(--muted);
  margin-bottom: 0.35rem;
}
.stream-child-head strong {
  color: #1d1d1f;
  font-weight: 600;
}
.stream-child-head a { color: #1d4ed8; }
.stream-child-meta {
  font: 0.72rem/1.3 var(--mono);
  color: var(--muted);
}
.stream-child-body {
  font: 0.88rem/1.5 var(--sans);
  color: var(--ink);
  white-space: pre-wrap;
  word-break: break-word;
}
.stream-child-body:empty { display: none; }
.stream-err {
  font: 0.85rem/1.45 var(--sans);
  color: #9b1c1c;
  background: #fff5f5;
  border: 1px solid #e4c1c1;
  border-radius: 8px;
  padding: 0.55rem 0.75rem;
  white-space: pre-wrap;
}
.code-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.45rem 0.65rem;
  padding: 0.65rem 0.85rem;
  border-bottom: 1px solid var(--line);
  background: #fafafa;
}
.code-lang {
  font: 0.72rem/1 var(--mono);
  color: var(--muted);
  text-transform: lowercase;
}
.code-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 0.4rem;
}
.code-foot {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.45rem 0.65rem;
  padding: 0.5rem 0.85rem;
  border-top: 1px solid var(--line);
  background: #fafafa;
}
.code-foot .code-actions { margin-left: auto; }
.code-cmd {
  min-width: 9rem;
  max-width: 16rem;
  padding: 0.35rem 0.55rem;
  border: 1px solid var(--line);
  border-radius: 8px;
  font: 0.78rem/1.2 var(--mono);
  color: var(--ink);
  background: #fff;
}
.code-cmd:focus {
  outline: 2px solid rgba(29,29,31,0.18);
  outline-offset: 1px;
}
.code-run {
  padding: 0.35rem 0.75rem;
  border: none;
  border-radius: 8px;
  background: var(--ink);
  color: #fff;
  font: 0.78rem/1.2 var(--sans);
  font-weight: 600;
  cursor: pointer;
}
.code-run:hover { opacity: 0.9; }
.code-run:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.code-body {
  margin: 0;
  padding: 0.85rem 1rem;
  overflow: auto;
  resize: vertical;
  min-height: 2.4rem;
  max-height: 40rem;
  background: #1e1e1e;
  color: var(--syn-fg);
  font: 0.82rem/1.45 var(--mono);
  min-height: 2.4rem;
}
.code-body code {
  font: inherit;
  background: none;
  padding: 0;
  color: inherit;
}
.code-body .hljs { color: var(--syn-fg); background: transparent; }
.code-body .hljs-comment,
.code-body .hljs-quote { color: var(--syn-comment); font-style: italic; }
.code-body .hljs-keyword,
.code-body .hljs-selector-tag,
.code-body .hljs-literal,
.code-body .hljs-section,
.code-body .hljs-link,
.code-body .hljs-doctag,
.code-body .hljs-type,
.code-body .hljs-tag,
.code-body .hljs-name { color: var(--syn-keyword); }
.code-body .hljs-string,
.code-body .hljs-regexp,
.code-body .hljs-addition,
.code-body .hljs-meta .hljs-string { color: var(--syn-string); }
.code-body .hljs-number,
.code-body .hljs-symbol,
.code-body .hljs-bullet { color: var(--syn-number); }
.code-body .hljs-variable,
.code-body .hljs-template-variable,
.code-body .hljs-attr,
.code-body .hljs-attribute,
.code-body .hljs-selector-attr,
.code-body .hljs-selector-pseudo { color: var(--syn-variable); }
.code-body .hljs-title,
.code-body .hljs-title.function_,
.code-body .hljs-function .hljs-title { color: var(--syn-function); }
.code-body .hljs-built_in,
.code-body .hljs-title.class_ { color: var(--syn-type); }
.code-body .hljs-meta,
.code-body .hljs-meta .hljs-keyword { color: var(--syn-meta); }
.code-body .hljs-deletion { color: var(--syn-string); }
.code-body .hljs-emphasis { font-style: italic; }
.code-body .hljs-strong { font-weight: 700; }
.code-out {
  margin: 0;
  padding: 0.75rem 1rem;
  border-top: 1px solid var(--line);
  background: #f0f0f2;
  font: 0.8rem/1.45 var(--mono);
  white-space: pre-wrap;
  word-break: break-word;
}
.code-out.err {
  background: #fff5f5;
  color: #9b1c1c;
}
.code-out:not([hidden]) { display: block; }
.plots {
  display: grid;
  gap: 1rem;
  margin-top: 1rem;
  justify-items: center;
}
.plot {
  overflow: auto;
  background: var(--surface);
  border: 1px solid var(--line);
  border-radius: var(--radius);
  padding: 0.75rem;
  width: 100%;
  display: flex;
  justify-content: center;
}
.plot:has(svg[data-theme="dark"]) {
  background: #070b10;
  border-color: #1c2838;
  padding: 1.1rem 1.25rem;
}
.plot:has(svg[data-theme="light"]),
.plot:has(svg[data-theme="bw"]) {
  background: #f7f9fc;
  border-color: #d5dde8;
}
.plot svg {
  display: block;
  max-width: 100%;
  height: auto;
  margin-inline: auto;
  min-width: min(100%, 28rem);
}
.comment-text {
  color: var(--muted);
  font-size: 0.92rem;
  display: block;
  margin-top: 0.35rem;
  line-height: 1.55;
  max-width: 42em;
}
.comment-text > :first-child { margin-top: 0; }
.comment-text > :last-child { margin-bottom: 0; }
.comment-text p { margin: 0.45em 0; }
.comment-text a {
  color: var(--ink);
  text-decoration: underline;
  text-underline-offset: 0.12em;
  border-bottom: none;
}
.comment-text a:hover { opacity: 0.72; }
.comment-text code {
  font-family: var(--mono);
  font-size: 0.86em;
  background: var(--fill);
  padding: 0.08em 0.28em;
  border-radius: 4px;
}
.comment-text ul, .comment-text ol {
  margin: 0.4em 0;
  padding-left: 1.35em;
}
code.expr {
  font-family: var(--mono);
  font-size: 0.84rem;
  color: var(--ink);
}
.err { color: var(--ink); font-weight: 600; padding: 0.75rem 1rem; }
.welcome {
  max-width: 40rem;
  margin: 4rem auto;
  text-align: center;
}
.welcome p { color: var(--muted); font-size: 1.05rem; }
.backdrop {
  display: none;
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.28);
  z-index: 25;
  animation: fade-in 0.18s ease;
}
@keyframes fade-in { from { opacity: 0; } to { opacity: 1; } }
@media (max-width: 900px) {
  .shell { grid-template-columns: 240px minmax(0, 1fr); }
  main.panel { max-width: 64rem; padding: 1.75rem 1.5rem 3rem; }
}
@media (max-width: 800px) {
  .topbar { display: flex; }
  .nav-brand { display: none; }
  .shell {
    grid-template-columns: 1fr;
    max-width: none;
    min-height: calc(100vh - 3.25rem);
  }
  .nav {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    width: min(82vw, 300px);
    height: 100vh;
    z-index: 30;
    transform: translateX(-105%);
    transition: transform 0.22s cubic-bezier(0.2, 0.8, 0.2, 1);
    border-right: 1px solid var(--line);
    background: var(--surface);
    box-shadow: 8px 0 32px rgba(0,0,0,0.08);
  }
  #nav-toggle:checked ~ .shell .nav { transform: translateX(0); }
  #nav-toggle:checked ~ .backdrop { display: block; }
  main.panel {
    max-width: none;
    padding: 1.4rem 1.15rem 3rem;
  }
}
"#
}

pub fn layout(title: &str, nav: &str, main: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{title} · marqdo</title>
<link rel="icon" href="https://s3.cflmy.cn/logo/Logo.ico" type="image/x-icon"/>
<link rel="shortcut icon" href="https://s3.cflmy.cn/logo/Logo.ico" type="image/x-icon"/>
<link rel="apple-touch-icon" href="https://s3.cflmy.cn/logo/Logo.png"/>
<link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css" crossorigin="anonymous"/>
<style>{css}</style>
</head>
<body>
<input type="checkbox" id="nav-toggle"/>
<label class="backdrop" for="nav-toggle" aria-hidden="true"></label>
<header class="topbar">
  <label class="nav-btn" for="nav-toggle" aria-label="Menu">☰</label>
  <div class="brand-row">
    <img class="logo" src="https://s3.cflmy.cn/logo/Logo.png" width="28" height="28" alt="marqdo"/>
    <p class="brand">marqdo</p>
  </div>
</header>
<div class="shell">
<aside class="nav">
  <div class="nav-brand">
    <div class="brand-row">
      <img class="logo" src="https://s3.cflmy.cn/logo/Logo.png" width="32" height="32" alt="marqdo"/>
      <div>
        <p class="brand">marqdo</p>
        <p class="tagline">view</p>
      </div>
    </div>
  </div>
  {nav}
</aside>
<div class="panel-wrap">
<main class="panel">
  {main}
</main>
</div>
</div>
<dialog class="var-dialog" id="var-dialog" aria-labelledby="var-dialog-title">
  <div class="var-dialog-sheet">
    <div class="var-dialog-head">
      <h2 id="var-dialog-title">Variable</h2>
      <button type="button" class="var-dialog-close" id="var-dialog-close" aria-label="Close">×</button>
    </div>
    <div class="var-dialog-body" id="var-dialog-body"></div>
  </div>
</dialog>
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js" crossorigin="anonymous"></script>
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/contrib/auto-render.min.js" crossorigin="anonymous"
  onload="renderMathInElement(document.body,{{delimiters:[{{left:'$$',right:'$$',display:true}},{{left:'\\(',right:'\\)',display:false}}],ignoredTags:['script','noscript','style','textarea','pre','code','kbd']}});"></script>
<script defer src="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.9.0/build/highlight.min.js" crossorigin="anonymous"
  onload="document.querySelectorAll('pre.code-body code').forEach(function(el){{hljs.highlightElement(el);}});"></script>
<script>
(function () {{
  function initForeignCards() {{
    var live = !!window.MARQDO_VIEW_LIVE;
    document.querySelectorAll(".code-card").forEach(function (card) {{
      var foot = card.querySelector(".code-foot");
      var btn = card.querySelector(".code-run");
      var cmd = card.querySelector(".code-cmd");
      var out = card.querySelector(".code-out");
      var body = card.querySelector(".code-body code");
      // Narrative ```text``` cards have no Run chrome — leave as-is.
      if (!btn) return;
      if (!live) return; // static actions are bound to the view-shell notice
      if (!out || !body) return;
      btn.addEventListener("click", function () {{
        btn.disabled = true;
        out.hidden = false;
        out.classList.remove("err");
        out.textContent = "Running…";
        fetch("api/foreign-run", {{
          method: "POST",
          headers: {{ "Content-Type": "application/json" }},
          body: JSON.stringify({{
            lang: card.getAttribute("data-lang") || "python",
            source: body.textContent,
            cmd: cmd ? cmd.value : ""
          }})
        }})
          .then(function (r) {{ return r.json(); }})
          .then(function (data) {{
            var parts = [];
            if (data.stdout) parts.push(data.stdout);
            if (data.stderr) parts.push("[stderr]\n" + data.stderr);
            if (data.error) parts.push(data.error);
            var meta = [];
            if (typeof data.code === "number") meta.push("exit " + data.code);
            if (typeof data.ms === "number") meta.push(data.ms + "ms");
            if (data.timed_out) meta.push("timed out");
            if (data.truncated) meta.push("truncated");
            if (meta.length) parts.push("— " + meta.join(" · "));
            if (data.ok) {{
              out.classList.remove("err");
            }} else {{
              out.classList.add("err");
            }}
            out.textContent = parts.length ? parts.join("\n\n") : "(no output)";
          }})
          .catch(function (e) {{
            out.classList.add("err");
            out.textContent = String(e);
          }})
          .finally(function () {{ btn.disabled = false; }});
      }});
    }});
  }}
  function initStreamPanel() {{
    if (!window.MARQDO_VIEW_LIVE) return;
    var body = document.getElementById("stream-body");
    var status = document.getElementById("stream-status");
    var runBtn = document.getElementById("stream-run");
    var stopBtn = document.getElementById("stream-stop");
    var jumpBtn = document.getElementById("stream-jump");
    if (!body) return;

    var turn = null;
    var thinkingEl = null;
    var thinkingBody = null;
    var answerEl = null;
    var reasonNode = null;
    var answerNode = null;
    var pinned = true;
    var ignoreScroll = false;
    var pendingReason = "";
    var pendingAnswer = "";
    var pendingChild = "";
    var raf = 0;
    // Industry pattern: coalesce tokens to one DOM write per animation frame.
    // Do NOT artificially drip/hold text — network pacing is the typing effect.
    var awaitCards = Object.create(null);
    var abortCtrl = null;

    function nearBottom() {{
      return body.scrollHeight - body.scrollTop - body.clientHeight < 72;
    }}
    function updateJump() {{
      if (!jumpBtn) return;
      jumpBtn.hidden = pinned;
    }}
    function scrollIfPinned() {{
      if (!pinned) {{
        updateJump();
        return;
      }}
      ignoreScroll = true;
      body.scrollTop = body.scrollHeight;
      // Ignore the scroll event our own assignment fires.
      requestAnimationFrame(function () {{
        ignoreScroll = false;
      }});
      updateJump();
    }}
    function onUserScrollIntent() {{
      // Re-evaluate after the browser applies the user scroll.
      requestAnimationFrame(function () {{
        if (ignoreScroll) return;
        pinned = nearBottom();
        updateJump();
      }});
    }}
    body.addEventListener("scroll", function () {{
      if (ignoreScroll) return;
      pinned = nearBottom();
      updateJump();
    }}, {{ passive: true }});
    body.addEventListener("wheel", onUserScrollIntent, {{ passive: true }});
    body.addEventListener("touchmove", onUserScrollIntent, {{ passive: true }});
    if (jumpBtn) {{
      jumpBtn.addEventListener("click", function () {{
        pinned = true;
        ignoreScroll = true;
        body.scrollTop = body.scrollHeight;
        requestAnimationFrame(function () {{ ignoreScroll = false; }});
        updateJump();
      }});
    }}

    function clearStream() {{
      body.innerHTML = "";
      turn = null;
      thinkingEl = null;
      thinkingBody = null;
      answerEl = null;
      reasonNode = null;
      answerNode = null;
      pendingReason = "";
      pendingAnswer = "";
      pendingChild = "";
      awaitCards = Object.create(null);
      if (raf) {{ cancelAnimationFrame(raf); raf = 0; }}
      pinned = true;
      updateJump();
    }}
    function ensureTurn() {{
      if (turn) return;
      turn = document.createElement("div");
      turn.className = "stream-turn";
      body.appendChild(turn);
    }}
    function ensureThinking() {{
      ensureTurn();
      if (thinkingEl) return;
      thinkingEl = document.createElement("details");
      thinkingEl.className = "stream-thinking";
      thinkingEl.open = true;
      var sum = document.createElement("summary");
      sum.textContent = "Thinking";
      thinkingBody = document.createElement("div");
      thinkingBody.className = "stream-thinking-body";
      reasonNode = document.createTextNode("");
      thinkingBody.appendChild(reasonNode);
      thinkingEl.appendChild(sum);
      thinkingEl.appendChild(thinkingBody);
      turn.appendChild(thinkingEl);
    }}
    function ensureAnswer() {{
      ensureTurn();
      if (answerEl) return;
      answerEl = document.createElement("div");
      answerEl.className = "stream-answer";
      answerNode = document.createTextNode("");
      answerEl.appendChild(answerNode);
      turn.appendChild(answerEl);
    }}
    function flushChildPending() {{
      if (!pendingChild) return;
      var keys = Object.keys(awaitCards);
      var card = keys.length ? awaitCards[keys[keys.length - 1]] : null;
      if (!card || !card.parentNode) {{
        pendingAnswer += pendingChild;
        pendingChild = "";
        return;
      }}
      var out = card.querySelector(".stream-child-body");
      if (!out) {{
        out = document.createElement("div");
        out.className = "stream-child-body";
        card.appendChild(out);
      }}
      out.textContent += pendingChild;
      pendingChild = "";
      if (status) status.textContent = "child streaming…";
    }}
    function flushPendingAll() {{
      if (raf) {{ cancelAnimationFrame(raf); raf = 0; }}
      if (pendingReason) {{
        ensureThinking();
        reasonNode.data += pendingReason;
        pendingReason = "";
      }}
      if (pendingAnswer) {{
        ensureAnswer();
        answerNode.data += pendingAnswer;
        pendingAnswer = "";
      }}
      flushChildPending();
      scrollIfPinned();
    }}
    function dripPending() {{
      raf = 0;
      if (pendingReason) {{
        ensureThinking();
        reasonNode.data += pendingReason;
        pendingReason = "";
      }}
      if (pendingAnswer) {{
        ensureAnswer();
        answerNode.data += pendingAnswer;
        pendingAnswer = "";
      }}
      flushChildPending();
      scrollIfPinned();
    }}
    function queueText(kind, text) {{
      if (!text) return;
      if (kind === "reasoning") pendingReason += text;
      else pendingAnswer += text;
      if (!raf) raf = requestAnimationFrame(dripPending);
    }}
    function appendChildStream(text) {{
      if (!text) return;
      pendingChild += text;
      if (!raf) raf = requestAnimationFrame(dripPending);
    }}
    function formatResult(v) {{
      if (v == null) return "";
      if (typeof v === "string") return v;
      try {{ return JSON.stringify(v, null, 2); }} catch (e) {{ return String(v); }}
    }}
    function resetTurnRefs() {{
      turn = null;
      thinkingEl = null;
      thinkingBody = null;
      answerEl = null;
      reasonNode = null;
      answerNode = null;
    }}
    function addDecision(ev) {{
      flushPendingAll();
      ensureTurn();
      var row = document.createElement("div");
      row.className = "stream-decision";
      var label = document.createElement("strong");
      label.textContent = "DECISION: " + (ev.decision || "?");
      row.appendChild(label);
      if (ev.summary) {{
        var s = document.createElement("span");
        s.textContent = ev.summary;
        row.appendChild(s);
      }}
      turn.appendChild(row);
      resetTurnRefs();
      scrollIfPinned();
    }}
    function addToolCard(ev) {{
      flushPendingAll();
      ensureTurn();
      var card = document.createElement("div");
      card.className = "stream-tool";
      var head = document.createElement("div");
      head.className = "stream-tool-head";
      var title = document.createElement("strong");
      var phase = (ev.type === "tool_end") ? "Tool end" : "Tool start";
      title.textContent = phase;
      head.appendChild(title);
      var meta = document.createElement("span");
      meta.className = "stream-tool-meta";
      var bits = [];
      if (ev.name) bits.push(ev.name);
      if (ev.kind) bits.push(ev.kind);
      meta.textContent = bits.join(" · ");
      head.appendChild(meta);
      card.appendChild(head);
      if (ev.type === "tool_end" && ev.result != null && ev.result !== "") {{
        var body = document.createElement("div");
        body.className = "stream-tool-body";
        var res = formatResult(ev.result);
        body.textContent = res.length > 1024 ? res.slice(0, 1024) + "…" : res;
        card.appendChild(body);
      }}
      turn.appendChild(card);
      scrollIfPinned();
    }}
    function addPhaseMark(label, ev) {{
      flushPendingAll();
      ensureTurn();
      var row = document.createElement("div");
      row.className = "stream-phase";
      var strong = document.createElement("strong");
      strong.textContent = label;
      row.appendChild(strong);
      if (ev && ev.workbook) {{
        var a = document.createElement("a");
        a.href = "/file?path=" + encodeURIComponent(ev.workbook);
        a.textContent = ev.workbook;
        row.appendChild(a);
      }}
      turn.appendChild(row);
      scrollIfPinned();
    }}
    function addAwaitCard(ev) {{
      flushPendingAll();
      var key = String(ev.round != null ? ev.round : "?");
      var existing = awaitCards[key];
      if (existing && existing.parentNode) {{
        existing.className = "stream-child stream-child-await";
        var t0 = existing.querySelector("strong");
        if (t0) t0.textContent = "Child running…";
        if (status) status.textContent = "awaiting child…";
        scrollIfPinned();
        return;
      }}
      var card = document.createElement("div");
      card.className = "stream-child stream-child-await";
      card.setAttribute("data-await-round", key);
      var head = document.createElement("div");
      head.className = "stream-child-head";
      var title = document.createElement("strong");
      title.textContent = "Child running…";
      head.appendChild(title);
      var meta = document.createElement("span");
      meta.className = "stream-child-meta";
      var bits = [];
      if (ev.round != null) bits.push("round " + ev.round);
      bits.push("waiting");
      meta.textContent = bits.join(" · ");
      head.appendChild(meta);
      if (ev.workbook) {{
        var a = document.createElement("a");
        a.href = "/file?path=" + encodeURIComponent(ev.workbook);
        a.textContent = ev.workbook;
        head.appendChild(a);
      }}
      card.appendChild(head);
      var live = document.createElement("div");
      live.className = "stream-child-body";
      card.appendChild(live);
      body.appendChild(card);
      awaitCards[key] = card;
      if (status) status.textContent = "awaiting child…";
      resetTurnRefs();
      scrollIfPinned();
    }}
    function addChildCard(ev) {{
      flushPendingAll();
      var key = String(ev.round != null ? ev.round : "?");
      var card = awaitCards[key];
      var liveText = "";
      if (card && card.parentNode) {{
        var live = card.querySelector(".stream-child-body");
        if (live) liveText = live.textContent || "";
        delete awaitCards[key];
        card.className = "stream-child";
        card.removeAttribute("data-await-round");
        card.innerHTML = "";
      }} else {{
        card = document.createElement("div");
        card.className = "stream-child";
        body.appendChild(card);
      }}
      var head = document.createElement("div");
      head.className = "stream-child-head";
      var title = document.createElement("strong");
      title.textContent = "Child workbook";
      head.appendChild(title);
      var meta = document.createElement("span");
      meta.className = "stream-child-meta";
      var bits = [];
      if (ev.round != null) bits.push("round " + ev.round);
      if (ev.exit_code != null) bits.push("exit " + ev.exit_code);
      meta.textContent = bits.join(" · ");
      head.appendChild(meta);
      if (ev.workbook) {{
        var a = document.createElement("a");
        a.href = "/file?path=" + encodeURIComponent(ev.workbook);
        a.textContent = ev.workbook;
        head.appendChild(a);
      }}
      card.appendChild(head);
      var res = formatResult(ev.result);
      if (!res && liveText) res = liveText;
      if (res) {{
        var out = document.createElement("div");
        out.className = "stream-child-body";
        out.textContent = res.length > 8000 ? res.slice(0, 8000) + "…" : res;
        card.appendChild(out);
      }}
      if (status) status.textContent = "running…";
      resetTurnRefs();
      scrollIfPinned();
    }}
    function addError(msg) {{
      flushPendingAll();
      var err = document.createElement("div");
      err.className = "stream-err";
      err.textContent = msg || "error";
      body.appendChild(err);
      scrollIfPinned();
    }}
    function handleEvent(ev) {{
      var t = ev.type || "?";
      if (t === "reasoning") {{
        if (ev.source === "child") appendChildStream(ev.text || "");
        else queueText("reasoning", ev.text || "");
      }} else if (t === "delta") {{
        if (ev.source === "child") appendChildStream(ev.text || "");
        else queueText("answer", ev.text || "");
      }} else if (t === "error") {{
        addError(ev.message || "error");
        if (status) status.textContent = "error";
      }} else if (t === "decision") {{
        addDecision(ev);
      }} else if (t === "tool_start" || t === "tool_end") {{
        addToolCard(ev);
      }} else if (t === "decompose") {{
        addPhaseMark("DECOMPOSE", ev);
        if (status) status.textContent = "planning…";
      }} else if (t === "await") {{
        addAwaitCard(ev);
      }} else if (t === "round") {{
        addChildCard(ev);
      }} else if (t === "run_start") {{
        if (status) status.textContent = "running…";
        ensureTurn();
        var mark = document.createElement("div");
        mark.className = "stream-decision";
        var strong = document.createElement("strong");
        strong.textContent = "RUN";
        mark.appendChild(strong);
        if (ev.path) {{
          var sp = document.createElement("span");
          sp.textContent = ev.path;
          mark.appendChild(sp);
        }}
        turn.appendChild(mark);
        scrollIfPinned();
      }} else if (t === "done") {{
        // Nested LLM `done` (no run_id): flush drip only; do not rewrite Answer/status.
        flushPendingAll();
        if (ev.run_id != null) {{
          if (status) status.textContent = "done";
          var hasAnswer = answerNode && answerNode.data;
          if (!hasAnswer && typeof ev.result === "string" && ev.result) {{
            ensureAnswer();
            answerNode.data += ev.result;
            scrollIfPinned();
          }}
          if (thinkingEl && (answerNode && answerNode.data)) thinkingEl.open = false;
          if (typeof ev.bindings_html === "string") {{
            var vp = document.getElementById("vars-panel");
            if (vp) {{
              vp.innerHTML = ev.bindings_html
                || '<p class="vars-empty">No bindings</p>';
              initVarsPanel();
            }}
          }}
          if (typeof ev.structure_html === "string" && ev.structure_html) {{
            var structEl = document.getElementById("structure");
            if (structEl) {{
              structEl.innerHTML = ev.structure_html;
              initAnnotLeaders();
              initCallPickers();
              initLoopPickers();
            }}
          }}
          if (typeof ev.trace_json === "string" && ev.trace_json) {{
            var tr = document.getElementById("mq-view-trace");
            if (tr) tr.textContent = ev.trace_json;
          }}
        }}
      }}
    }}
    function setRunning(on) {{
      if (runBtn) runBtn.disabled = on;
      var formBtn = document.querySelector(".stdin-form button[type='submit']");
      if (formBtn) formBtn.disabled = on;
      if (stopBtn) {{
        stopBtn.disabled = !on;
        stopBtn.hidden = !on;
      }}
    }}
    async function readSseStream(response) {{
      var ctype = (response.headers.get("content-type") || "").toLowerCase();
      if (ctype.indexOf("application/json") !== -1) {{
        var data = await response.json();
        throw new Error(data.error || data.message || JSON.stringify(data));
      }}
      if (!response.ok) {{
        var errText = await response.text();
        throw new Error(errText || ("HTTP " + response.status));
      }}
      if (!response.body || !response.body.getReader) {{
        throw new Error("ReadableStream not supported");
      }}
      var reader = response.body.getReader();
      var decoder = new TextDecoder();
      var buf = "";
      while (true) {{
        var chunk = await reader.read();
        if (chunk.done) break;
        buf += decoder.decode(chunk.value, {{ stream: true }});
        buf = buf.replace(/\r\n/g, "\n").replace(/\r/g, "\n");
        var parts = buf.split("\n\n");
        buf = parts.pop() || "";
        for (var i = 0; i < parts.length; i++) {{
          var block = parts[i];
          if (!block || block.charAt(0) === ":") continue;
          var lines = block.split("\n");
          var dataLines = [];
          for (var j = 0; j < lines.length; j++) {{
            var line = lines[j];
            if (line.indexOf("data:") === 0) {{
              dataLines.push(line.slice(5).replace(/^ /, ""));
            }}
          }}
          if (!dataLines.length) continue;
          try {{
            handleEvent(JSON.parse(dataLines.join("\n")));
          }} catch (e) {{
            addError(String(e));
          }}
        }}
      }}
      flushPendingAll();
    }}
    function startRun() {{
      if (abortCtrl) abortCtrl.abort();
      abortCtrl = new AbortController();
      setRunning(true);
      if (status) status.textContent = "running…";
      clearStream();
      pinned = true;
      var stdinEl = document.getElementById("stdin");
      fetch("api/run", {{
        method: "POST",
        headers: {{
          "Content-Type": "application/json",
          "Accept": "text/event-stream"
        }},
        body: JSON.stringify({{
          path: window.MARQDO_FILE_PATH || "",
          stdin: stdinEl ? stdinEl.value : ""
        }}),
        signal: abortCtrl.signal
      }})
        .then(readSseStream)
        .then(function () {{
          if (status && status.textContent === "running…") status.textContent = "done";
        }})
        .catch(function (e) {{
          if (e && e.name === "AbortError") {{
            if (status) status.textContent = "stopped";
            return;
          }}
          addError(String(e && e.message ? e.message : e));
          if (status) status.textContent = "error";
        }})
        .finally(function () {{
          setRunning(false);
          abortCtrl = null;
        }});
    }}
    window.marqdoStreamRun = startRun;
    if (stopBtn) {{
      stopBtn.addEventListener("click", function () {{
        if (abortCtrl) abortCtrl.abort();
      }});
    }}
    if (runBtn) {{
      runBtn.addEventListener("click", function () {{ startRun(); }});
    }}
    var form = document.querySelector(".stdin-form");
    if (form) {{
      form.addEventListener("submit", function (ev) {{
        ev.preventDefault();
        startRun();
      }});
    }}
    if (window.MARQDO_AUTO_STREAM) {{
      startRun();
    }}
  }}

  function initVarsPanel() {{
    var panel = document.getElementById("vars-panel");
    var dialog = document.getElementById("var-dialog");
    var title = document.getElementById("var-dialog-title");
    var body = document.getElementById("var-dialog-body");
    var closeBtn = document.getElementById("var-dialog-close");
    if (!panel || !dialog || !title || !body) return;

    function renderMath(root) {{
      if (window.renderMathInElement) {{
        try {{
          window.renderMathInElement(root, {{
            delimiters: [
              {{ left: "$$", right: "$$", display: true }},
              {{ left: "\\(", right: "\\)", display: false }}
            ],
            ignoredTags: ["script", "noscript", "style", "textarea", "pre", "code", "kbd", "button"]
          }});
        }} catch (e) {{}}
      }}
    }}

    function openVar(id) {{
      var store = document.getElementById("var-full-" + id);
      if (!store) return;
      var name = store.getAttribute("data-var-name") || ("#" + id);
      title.textContent = name;
      body.innerHTML = store.innerHTML;
      renderMath(body);
      try {{
        if (typeof dialog.showModal === "function") dialog.showModal();
        else dialog.setAttribute("open", "");
      }} catch (e) {{
        dialog.setAttribute("open", "");
      }}
    }}

    function closeVar() {{
      try {{
        if (typeof dialog.close === "function") dialog.close();
        else dialog.removeAttribute("open");
      }} catch (e) {{
        dialog.removeAttribute("open");
      }}
    }}

    // Event delegation: survives vars-panel innerHTML refresh after runs.
    if (panel.dataset.varsBound !== "1") {{
      panel.dataset.varsBound = "1";
      panel.addEventListener("click", function (ev) {{
        var card = ev.target.closest(".vars-card");
        if (!card || !panel.contains(card)) return;
        ev.preventDefault();
        openVar(card.getAttribute("data-var-id"));
      }});
      panel.addEventListener("keydown", function (ev) {{
        if (ev.key !== "Enter" && ev.key !== " ") return;
        var card = ev.target.closest(".vars-card");
        if (!card || !panel.contains(card)) return;
        ev.preventDefault();
        openVar(card.getAttribute("data-var-id"));
      }});
    }}

    if (closeBtn && closeBtn.dataset.bound !== "1") {{
      closeBtn.dataset.bound = "1";
      closeBtn.addEventListener("click", function (ev) {{
        ev.preventDefault();
        closeVar();
      }});
    }}
    if (dialog.dataset.bound !== "1") {{
      dialog.dataset.bound = "1";
      dialog.addEventListener("click", function (ev) {{
        if (ev.target === dialog) closeVar();
      }});
      document.addEventListener("keydown", function (ev) {{
        if (ev.key === "Escape" && dialog.open) closeVar();
      }});
    }}

    renderMath(panel);
  }}

  function readViewTrace() {{
    var el = document.getElementById("mq-view-trace");
    if (!el) return null;
    try {{ return JSON.parse(el.textContent || "{{}}"); }} catch (e) {{ return null; }}
  }}

  function parseArgsBrief(brief) {{
    var out = {{}};
    if (!brief) return out;
    String(brief).split(", ").forEach(function (part) {{
      var i = part.indexOf("=");
      if (i <= 0) return;
      out[part.slice(0, i)] = part.slice(i + 1);
    }});
    return out;
  }}

  function applyParamChipValues(fnName, callId, trace) {{
    if (!trace || !Array.isArray(trace.calls)) return;
    var brief = "";
    for (var i = 0; i < trace.calls.length; i++) {{
      var c = trace.calls[i];
      if (String(c.call_id) === String(callId) && String(c.fn || "") === String(fnName)) {{
        brief = c.args_brief || "";
        break;
      }}
    }}
    var args = parseArgsBrief(brief);
    document.querySelectorAll('.chip[data-fn="' + fnName + '"][data-param-name]').forEach(function (chip) {{
      var name = chip.getAttribute("data-param-name") || "";
      var slot = chip.querySelector(".chip-val");
      if (!slot) return;
      var v = args[name];
      slot.textContent = v ? ("=" + v) : "";
    }});
  }}

  var MAX_VISIBLE_ANNOTS = 4;

  function setFrameHidden(el, hide) {{
    el.hidden = !!hide;
    if (hide) el.classList.add("is-frame-hidden");
    else el.classList.remove("is-frame-hidden");
  }}

  /** After frame filter: overflow only among chips visible in the selected frame. */
  function recomputeAnnotOverflow(scope) {{
    var root = scope || document.getElementById("structure") || document;
    root.querySelectorAll("[data-annot-band]").forEach(function (band) {{
      band.classList.remove("is-expanded");
      var chips = band.querySelectorAll(".annot-chip");
      var visible = [];
      chips.forEach(function (chip) {{
        chip.classList.remove("annot-chip-overflow");
        if (chip.hidden || chip.classList.contains("is-frame-hidden")) return;
        visible.push(chip);
      }});
      var overflow = 0;
      visible.forEach(function (chip, i) {{
        if (i >= MAX_VISIBLE_ANNOTS) {{
          chip.classList.add("annot-chip-overflow");
          overflow++;
        }}
      }});
      var more = band.querySelector("[data-annot-more]");
      if (more) {{
        if (overflow > 0) {{
          more.hidden = false;
          more.textContent = "+" + overflow;
        }} else {{
          more.hidden = true;
        }}
      }}
    }});
  }}

  /** Source structure always stays visible; only chips + arm classes follow the frame. */
  function initCallPickers() {{
    document.querySelectorAll(".call-picker").forEach(function (sel) {{
      function apply() {{
        var callId = String(sel.value || "");
        var fnName = String(sel.getAttribute("data-fn") || "");
        var allowed = {{}};
        Array.prototype.forEach.call(sel.options, function (o) {{
          allowed[String(o.value)] = true;
        }});
        var root = document.getElementById("structure") || document;
        // Only touch chips whose call_id belongs to this function's calls.
        root.querySelectorAll(".annot-chip[data-call-id]").forEach(function (chip) {{
          var cid = String(chip.getAttribute("data-call-id") || "");
          if (!allowed[cid]) return;
          setFrameHidden(chip, cid !== callId);
        }});
        var trace = readViewTrace();
        applyParamChipValues(fnName, callId, trace);
        if (trace && Array.isArray(trace.events)) {{
          var takenByLine = {{}};
          trace.events.forEach(function (ev) {{
            if (ev.kind === "branch" && String(ev.call_id) === callId && ev.arm_index != null) {{
              takenByLine[ev.line] = ev.arm_index;
            }}
          }});
          root.querySelectorAll("ol.doc-branch").forEach(function (ol) {{
            var line = parseInt(ol.getAttribute("data-line") || "0", 10);
            var taken = takenByLine[line];
            if (taken === undefined) return;
            ol.querySelectorAll(":scope > li.doc-list-item[data-arm-index]").forEach(function (li) {{
              var idx = parseInt(li.getAttribute("data-arm-index") || "-1", 10);
              li.classList.remove("arm-taken", "arm-miss");
              if (idx === taken) li.classList.add("arm-taken");
              else li.classList.add("arm-miss");
            }});
          }});
        }}
        recomputeAnnotOverflow(root);
        initAnnotLeaders();
      }}
      if (sel.dataset.bound !== "1") {{
        sel.dataset.bound = "1";
        sel.addEventListener("change", apply);
      }}
      apply();
    }});
  }}

  function initLoopPickers() {{
    document.querySelectorAll(".loop-picker").forEach(function (sel) {{
      function apply() {{
        var iter = String(sel.value || "");
        var loopId = String(sel.getAttribute("data-loop-id") || "");
        var ul = sel.closest("ul.doc-loop");
        var scope = ul || document.getElementById("structure");
        if (!scope) return;
        // Only the selected iteration's chips; structure source stays visible.
        scope.querySelectorAll(".annot-chip").forEach(function (chip) {{
          var lid = chip.getAttribute("data-loop-id");
          var chipIter = chip.getAttribute("data-iter");
          if (loopId && lid && String(lid) !== loopId) return;
          if (chipIter == null) {{
            // No data-iter: leave alone (foreach uses BindRef chips; while may have one summary).
            return;
          }}
          // Chips with data-iter but no loop-id still filter by iter inside this ul.
          if (lid && loopId && String(lid) !== loopId) return;
          setFrameHidden(chip, String(chipIter) !== iter);
        }});
        recomputeAnnotOverflow(scope);
        initAnnotLeaders();
      }}
      if (sel.dataset.bound !== "1") {{
        sel.dataset.bound = "1";
        sel.addEventListener("change", apply);
      }}
      apply();
    }});
  }}

  function ensureBlockLeaders(block) {{
    var leaders = block.querySelector(":scope > .annot-leaders");
    if (!leaders) {{
      leaders = document.createElement("div");
      leaders.className = "annot-leaders";
      leaders.setAttribute("aria-hidden", "true");
      block.insertBefore(leaders, block.firstChild);
    }}
    return leaders;
  }}

  function leaderColor(kind) {{
    if (kind === "bind") return "#5ac8fa";
    if (kind === "return") return "#30d158";
    if (kind === "call" || kind === "expr" || kind === "code") return "#ff9f0a";
    if (kind === "branch") return "#bf5af2";
    if (kind === "loop") return "#64d2ff";
    return "#8e8e93";
  }}

  function initAnnotLeaders() {{
    var bands = document.querySelectorAll("[data-annot-band]");
    if (!bands.length) return;
    var reduce = window.matchMedia && window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    var narrow = window.matchMedia && window.matchMedia("(max-width: 640px)").matches;

    function layoutBand(band) {{
      var block = band.closest(".doc-block");
      if (!block) return;
      var leaders = ensureBlockLeaders(block);
      leaders.innerHTML = "";
      var chipsEl = band.querySelector(".annot-chips");
      var chips = Array.prototype.slice.call(
        band.querySelectorAll(".annot-chip:not(.annot-chip-overflow):not([hidden]):not(.is-frame-hidden)")
      );
      // Pair chips with anchors; reorder DOM left→right under underlines so
      // L-leaders do not cross (crowded reverse order looked like missing lines).
      var items = [];
      chips.forEach(function (chip) {{
        var id = chip.getAttribute("data-annot-id") || "";
        var kind = chip.getAttribute("data-annot-kind") || "";
        var anchor = id
          ? (block.querySelector('.mq-anchor[data-annot-id="' + id + '"]')
            || document.querySelector('.mq-anchor[data-annot-id="' + id + '"]'))
          : null;
        // Fallback: match mark kind on this block (frame chips may share a span).
        if (!anchor) {{
          var cls = kind === "bind" ? "mq-decl"
            : kind === "return" ? "mq-return"
            : (kind === "call" || kind === "expr" || kind === "code") ? "mq-code"
            : "";
          if (cls) anchor = block.querySelector(".mq-anchor." + cls);
        }}
        if (!anchor) return;
        if (id) anchor.setAttribute("data-annot-id", id);
        items.push({{
          chip: chip,
          anchor: anchor,
          kind: kind,
          x: anchor.getBoundingClientRect().left
        }});
      }});
      items.sort(function (a, b) {{ return a.x - b.x; }});
      if (chipsEl && items.length) {{
        items.forEach(function (it) {{ chipsEl.appendChild(it.chip); }});
        var more = band.querySelector("[data-annot-more]");
        if (more) chipsEl.appendChild(more);
      }}
      if (reduce || narrow || items.length > MAX_VISIBLE_ANNOTS) {{
        block.classList.add("is-keyed");
        band.classList.add("is-keyed");
        items.forEach(function (it, i) {{
          it.chip.setAttribute("data-annot-key", String(i + 1));
        }});
        return;
      }}
      block.classList.remove("is-keyed");
      band.classList.remove("is-keyed");
      // Mid-bottom of mark → mid-top of chip: vertical, horizontal rail, short stub.
      var blockRect = block.getBoundingClientRect();
      var bw = Math.max(1, block.offsetWidth);
      var bh = Math.max(1, block.offsetHeight);
      var svgNS = "http://www.w3.org/2000/svg";
      var svg = document.createElementNS(svgNS, "svg");
      svg.setAttribute("width", String(bw));
      svg.setAttribute("height", String(bh));
      svg.setAttribute("viewBox", "0 0 " + bw + " " + bh);
      svg.setAttribute("aria-hidden", "true");
      items.forEach(function (it) {{
        var a = it.anchor.getBoundingClientRect();
        var c = it.chip.getBoundingClientRect();
        var x0 = a.left + a.width / 2 - blockRect.left;
        var y0 = a.bottom - blockRect.top + 1;
        var x1 = c.left + c.width / 2 - blockRect.left;
        var y1 = c.top - blockRect.top;
        if (y1 < y0 + 8) {{
          y1 = y0 + 12;
        }}
        // Short stub into chip top-center; rail sits above that.
        var stub = Math.min(10, Math.max(6, (y1 - y0) * 0.28));
        var yRail = y1 - stub;
        if (yRail < y0 + 4) {{
          yRail = (y0 + y1) / 2;
        }}
        var path = document.createElementNS(svgNS, "path");
        path.setAttribute("class", "annot-leader annot-" + it.kind);
        path.setAttribute("stroke", leaderColor(it.kind));
        // Orthogonal: down from mark mid → across → short down to chip mid.
        path.setAttribute(
          "d",
          "M " + x0.toFixed(1) + " " + y0.toFixed(1) +
            " L " + x0.toFixed(1) + " " + yRail.toFixed(1) +
            " L " + x1.toFixed(1) + " " + yRail.toFixed(1) +
            " L " + x1.toFixed(1) + " " + y1.toFixed(1)
        );
        svg.appendChild(path);
      }});
      leaders.appendChild(svg);
    }}

    function layoutAll() {{
      bands.forEach(layoutBand);
    }}
    layoutAll();
    // Second pass after fonts/layout settle (first paint often has zero chip width).
    requestAnimationFrame(function () {{ layoutAll(); }});
    if (document.documentElement.dataset.annotLeadersRo === "1") return;
    document.documentElement.dataset.annotLeadersRo = "1";
    if (typeof ResizeObserver !== "undefined") {{
      var ro = new ResizeObserver(function () {{ layoutAll(); }});
      bands.forEach(function (b) {{
        ro.observe(b);
        var blk = b.closest(".doc-block");
        if (blk) ro.observe(blk);
      }});
      var surface = document.querySelector(".document-surface");
      if (surface) ro.observe(surface);
    }}
    window.addEventListener("resize", layoutAll);

    document.querySelectorAll("[data-annot-more]").forEach(function (btn) {{
      if (btn.dataset.bound === "1") return;
      btn.dataset.bound = "1";
      btn.addEventListener("click", function () {{
        var band = btn.closest("[data-annot-band]");
        if (!band) return;
        band.classList.add("is-expanded");
        btn.hidden = true;
        layoutAll();
      }});
    }});
    document.querySelectorAll(".annot-chip").forEach(function (chip) {{
      if (chip.dataset.focusBound === "1") return;
      chip.dataset.focusBound = "1";
      chip.addEventListener("mouseenter", function () {{
        var id = chip.getAttribute("data-annot-id");
        if (!id) return;
        var a = document.querySelector('.mq-anchor[data-annot-id="' + id + '"]');
        if (a) a.classList.add("is-focus");
        chip.classList.add("is-focus");
      }});
      chip.addEventListener("mouseleave", function () {{
        var id = chip.getAttribute("data-annot-id");
        if (!id) return;
        var a = document.querySelector('.mq-anchor[data-annot-id="' + id + '"]');
        if (a) a.classList.remove("is-focus");
        chip.classList.remove("is-focus");
      }});
    }});
  }}

  if (document.readyState === "loading") {{
    document.addEventListener("DOMContentLoaded", function () {{
      initForeignCards();
      initOutlineSearch();
      initStreamPanel();
      initVarsPanel();
      initAnnotLeaders();
      initCallPickers();
      initLoopPickers();
    }});
  }} else {{
    initForeignCards();
    initOutlineSearch();
    initStreamPanel();
    initVarsPanel();
    initAnnotLeaders();
    initCallPickers();
    initLoopPickers();
  }}

  function initOutlineSearch() {{
    var input = document.getElementById("fn-search");
    var tree = document.querySelector(".outline-tree");
    if (!input || !tree) return;
    function applyFilter() {{
      var q = (input.value || "").trim().toLowerCase();
      var items = tree.querySelectorAll(".outline-item");
      items.forEach(function (li) {{
        if (!q) {{
          li.classList.remove("hidden");
          return;
        }}
        var name = (li.getAttribute("data-fn") || "").toLowerCase();
        var path = (li.getAttribute("data-fn-path") || "").toLowerCase();
        var selfMatch = name.indexOf(q) !== -1 || path.indexOf(q) !== -1;
        var childMatch = false;
        li.querySelectorAll(".outline-item").forEach(function (c) {{
          var cn = (c.getAttribute("data-fn") || "").toLowerCase();
          var cp = (c.getAttribute("data-fn-path") || "").toLowerCase();
          if (cn.indexOf(q) !== -1 || cp.indexOf(q) !== -1) childMatch = true;
        }});
        li.classList.toggle("hidden", !(selfMatch || childMatch));
      }});
    }}
    input.addEventListener("input", applyFilter);
    document.querySelectorAll("a[href^='#fn-'], a[href^='#wb-']").forEach(function (a) {{
      a.addEventListener("click", function (ev) {{
        var id = (a.getAttribute("href") || "").slice(1);
        var el = id ? document.getElementById(id) : null;
        if (el) {{
          ev.preventDefault();
          el.scrollIntoView({{ behavior: "smooth", block: "start" }});
          history.replaceState(null, "", "#" + id);
        }}
      }});
    }});
  }}
}})();
</script>
</body>
</html>
"##,
        title = escape(title),
        css = stylesheet(),
        nav = nav,
        main = main
    )
}

pub fn nav_html(files: &[PathBuf], active: Option<&str>, links: &LinkMode) -> String {
    let mut out = String::from("<h2>Files</h2>");
    let mut last_folder: Option<String> = None;

    for f in files {
        let rel = f.to_string_lossy().replace('\\', "/");
        let folder = Path::new(&rel)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        if last_folder.as_deref() != Some(folder.as_str()) {
            close_nav_group(&mut out, last_folder.as_deref());
            open_nav_group(&mut out, &folder, active);
            last_folder = Some(folder);
        }

        let name = Path::new(&rel)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&rel);
        let active_cls = if active == Some(rel.as_str()) {
            " active"
        } else {
            ""
        };
        let href = href_file(links, &rel);
        out.push_str(&format!(
            "<li><a class=\"file{active_cls}\" href=\"{}\">{}</a></li>",
            escape(&href),
            escape(name)
        ));
    }
    close_nav_group(&mut out, last_folder.as_deref());

    // index link for static pages
    if matches!(links, LinkMode::Static { from: Some(_) }) {
        let home = href_index(links);
        out.insert_str(
            0,
            &format!(
                "<p style=\"margin:0 0 0.75rem\"><a href=\"{}\">Index</a></p>",
                escape(&home)
            ),
        );
    }
    out
}

fn close_nav_group(out: &mut String, folder: Option<&str>) {
    match folder {
        None => {}
        Some("") => out.push_str("</ul>"),
        Some(_) => out.push_str("</ul></details>"),
    }
}

fn open_nav_group(out: &mut String, folder: &str, active: Option<&str>) {
    if folder.is_empty() {
        out.push_str("<ul class=\"nav-root\">");
        return;
    }
    let contains_active = active.is_some_and(|a| {
        Path::new(a)
            .parent()
            .map(|p| p.to_string_lossy().replace('\\', "/") == folder)
            .unwrap_or(false)
    });
    // Index (no active): keep folders open for browsing; else open only the active folder.
    let open_attr = if active.is_none() || contains_active {
        " open"
    } else {
        ""
    };
    out.push_str(&format!(
        "<details class=\"folder\"{open_attr}><summary>{}</summary><ul>",
        escape(folder)
    ));
}

fn href_file(links: &LinkMode, target_rel: &str) -> String {
    match links {
        LinkMode::Live => format!("/file?path={}", urlencoding_encode(target_rel)),
        LinkMode::Static { from: None } => format!("pages/{}.html", target_rel),
        LinkMode::Static {
            from: Some(from_rel),
        } => relative_page_href(from_rel, target_rel),
    }
}

fn href_index(links: &LinkMode) -> String {
    match links {
        LinkMode::Live => "/".into(),
        LinkMode::Static { from: None } => "index.html".into(),
        LinkMode::Static {
            from: Some(from_rel),
        } => {
            let depth = Path::new(from_rel)
                .parent()
                .map(|p| p.components().filter(|c| matches!(c, Component::Normal(_))).count())
                .unwrap_or(0);
            // pages/<dirs...>/file.html → up (depth+1) to OUT_DIR
            format!("{}index.html", "../".repeat(depth + 1))
        }
    }
}

/// From `pages/<from>.html` to `pages/<to>.html`.
fn relative_page_href(from_rel: &str, to_rel: &str) -> String {
    let from_parent = Path::new(from_rel).parent().unwrap_or_else(|| Path::new(""));
    let to_path = PathBuf::from(format!("{to_rel}.html"));
    let mut ups = 0usize;
    for c in from_parent.components() {
        if matches!(c, Component::Normal(_)) {
            ups += 1;
        }
    }
    format!("{}{}", "../".repeat(ups), to_path.to_string_lossy().replace('\\', "/"))
}

fn urlencoding_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

pub fn page_index(files: &[PathBuf], only: Option<&Path>, links: &LinkMode) -> String {
    let nav = nav_html(files, None, links);
    let main = if let Some(p) = only {
        let rel = p.to_string_lossy().replace('\\', "/");
        format!(
            "<div class=\"welcome\"><h1 class=\"page\">marqdo</h1><p class=\"meta\">Open <a href=\"{}\">{}</a></p></div>",
            escape(&href_file(links, &rel)),
            escape(&rel)
        )
    } else if files.is_empty() {
        "<div class=\"welcome\"><h1 class=\"page\">marqdo</h1><p>No <code>.mq.md</code> files found in this path.</p></div>".into()
    } else {
        // Prefer callers to render the first file at `/` / index.html; this is a fallback.
        let first = files[0].to_string_lossy().replace('\\', "/");
        format!(
            "<div class=\"welcome\"><h1 class=\"page\">marqdo</h1><p>Opening <a href=\"{}\">{}</a>…</p></div>",
            escape(&href_file(links, &first)),
            escape(&first)
        )
    };
    layout("index", &nav, &main)
}

fn notebook_ui() -> &'static str {
    r#"<style>
/* ── VS Code / Jupyter notebook skin ───────────────────────────────── */
body.notebook-page {
  color-scheme: dark;
  --nb-bg: #1e1e1e;
  --nb-surface: #252526;
  --nb-surface-2: #2d2d30;
  --nb-surface-3: #333333;
  --nb-border: #3c3c3c;
  --nb-border-soft: #2a2a2a;
  --nb-text: #f0f0f0;
  --nb-strong: #f0f0f0;
  --nb-muted: #9d9d9d;
  --nb-accent: #0e639c;
  --nb-accent-hover: #1177bb;
  --nb-success: #4ec9b0;
  --nb-error: #f48771;
  --nb-warning: #cca700;
  --nb-selection: #04395e;
  --nb-on-accent: #ffffff;
  --nb-topbar: rgba(30,30,30,.94);
  --nb-code-bg: #1e1e1e;
  --nb-code-text: #f0f0f0;
  --nb-error-bg: #2a0f12;
  --nb-error-border: #5a1d1d;
  --nb-radius: 6px;
  /* Map every legacy view token onto the active theme. This fixes the
     document surface and annotation chips without duplicating rules. */
  --bg: var(--nb-bg);
  --surface: var(--nb-surface);
  --ink: var(--nb-text);
  --muted: var(--nb-muted);
  --line: var(--nb-border);
  --line-strong: var(--nb-strong);
  --fill: var(--nb-surface-2);
  --hover: var(--nb-surface-3);
  --focus: var(--nb-accent);
  --fail-bg: var(--nb-error-bg);
  --ok-bg: var(--nb-surface);
  --radius: var(--nb-radius);
  background: var(--nb-bg);
  color: var(--nb-text);
}
body.notebook-page[data-theme="light"] {
  color-scheme: light;
  --nb-bg: #ffffff;
  --nb-surface: #f7f7f8;
  --nb-surface-2: #efeff1;
  --nb-surface-3: #e4e4e7;
  --nb-border: #d8d8dc;
  --nb-border-soft: #e8e8eb;
  --nb-text: #24292f;
  --nb-strong: #111418;
  --nb-muted: #57606a;
  --nb-accent: #0969da;
  --nb-accent-hover: #0550ae;
  --nb-success: #1a7f37;
  --nb-error: #cf222e;
  --nb-warning: #9a6700;
  --nb-selection: #0969da;
  --nb-on-accent: #ffffff;
  --nb-topbar: rgba(255,255,255,.94);
  --nb-code-bg: #f6f8fa;
  --nb-code-text: #24292f;
  --nb-error-bg: #fff5f5;
  --nb-error-border: #ffc1c1;
  --syn-fg: #24292f;
  --syn-comment: #57606a;
  --syn-keyword: #0550ae;
  --syn-string: #0a3069;
  --syn-number: #0550ae;
  --syn-variable: #953800;
  --syn-function: #8250df;
  --syn-type: #116329;
  --syn-meta: #8250df;
}
body.notebook-page .panel-wrap { background: var(--nb-bg); }
body.notebook-page main.panel {
  max-width: 76rem;
  margin: 0;
  padding: 2rem 2.25rem 3.5rem;
}
/* Left file navigation: keep the original layout, styled for the dark skin. */
body.notebook-page .nav {
  background: var(--nb-bg);
  border-right: 1px solid var(--nb-border);
}
body.notebook-page .nav .brand { color: var(--nb-strong); }
body.notebook-page .nav .tagline,
body.notebook-page .nav h2,
body.notebook-page .folder > summary { color: var(--nb-muted); }
body.notebook-page .nav a.file { color: var(--nb-text); }
body.notebook-page .nav a.file:hover { background: var(--nb-surface-2); }
body.notebook-page .nav a.file.active {
  background: var(--nb-selection);
  color: var(--nb-on-accent);
}
body.notebook-page .folder > summary:hover {
  background: var(--nb-surface-2);
  color: var(--nb-strong);
}
body.notebook-page .folder > summary::before {
  border-color: transparent transparent transparent var(--nb-muted);
}
body.notebook-page .topbar {
  background: var(--nb-topbar);
  border-bottom-color: var(--nb-border);
}
body.notebook-page .topbar .brand,
body.notebook-page .nav-btn { color: var(--nb-text); }
body.notebook-page .nav-btn {
  background: var(--nb-surface);
  border-color: var(--nb-border);
}
body.notebook-page .nav-btn:hover { background: var(--nb-surface-2); }
body.notebook-page h1.page {
  font-size: clamp(1.5rem, 2.6vw, 1.9rem);
  line-height: 1.2;
  letter-spacing: -0.025em;
  font-weight: 600;
  color: var(--nb-strong);
  margin: 0 0 0.3rem;
}
body.notebook-page .meta {
  margin: 0 0 1.15rem;
  color: var(--nb-muted);
  font: 0.72rem/1.45 var(--mono);
}
body.notebook-page .status-pill {
  display: inline-block;
  margin-left: 0.55rem;
  padding: 0.12rem 0.42rem;
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-surface);
  color: var(--nb-muted);
  font: 600 0.6rem/1.2 var(--sans);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  vertical-align: 0.12em;
}
body.notebook-page .status-pill.ok { color: var(--nb-success); }
body.notebook-page .status-pill.pending { color: var(--nb-warning); }
body.notebook-page .status-pill.fail { color: var(--nb-error); }
body.notebook-page section.block { margin: 1.4rem 0 0; }
body.notebook-page section.block > h2 {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin: 0 0 0.65rem;
  padding-bottom: 0.35rem;
  border-bottom: 1px solid var(--nb-border);
  color: var(--nb-muted);
  font: 600 0.66rem/1.4 var(--sans);
  letter-spacing: 0.09em;
  text-transform: uppercase;
}
body.notebook-page .structure-block { margin-top: 0.7rem; }

/* ── Toolbar ───────────────────────────────────────────────────────── */
body.notebook-page #notebook-bar {
  position: sticky;
  top: 0.55rem;
  z-index: 60;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.2rem;
  margin: 0 0 0.6rem;
  padding: 0.3rem;
  border: 1px solid var(--nb-border);
  border-radius: var(--nb-radius);
  background: var(--nb-surface);
}
body.notebook-page #notebook-bar strong {
  padding: 0 0.5rem;
  color: var(--nb-strong);
  font: 600 0.76rem/1.6 var(--sans);
}
body.notebook-page .nb-stdin-bar {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  margin: 0 0 0.75rem;
  padding: 0.4rem 0.55rem;
  border: 1px solid var(--nb-border);
  border-radius: var(--nb-radius);
  background: var(--nb-surface);
}
body.notebook-page .nb-stdin-bar label {
  color: var(--nb-muted);
  font: 0.7rem/1.2 var(--mono);
  white-space: nowrap;
}
body.notebook-page .nb-stdin-bar textarea {
  flex: 1;
  min-height: 2.1rem;
  max-height: 7rem;
  resize: vertical;
  border: 1px solid var(--nb-border);
  border-radius: 6px;
  padding: 0.3rem 0.5rem;
  background: var(--nb-bg);
  color: var(--nb-text);
  font: 0.72rem/1.35 var(--mono);
}
body.notebook-page #notebook-bar button {
  appearance: none;
  padding: 0.34rem 0.58rem;
  border: 1px solid transparent;
  border-radius: 3px;
  background: transparent;
  color: var(--nb-text);
  font: 500 0.72rem/1.2 var(--sans);
  cursor: pointer;
  transition: background 0.12s ease, border-color 0.12s ease;
}
body.notebook-page #notebook-bar button:hover {
  background: var(--nb-surface-3);
  border-color: var(--nb-border);
}
body.notebook-page #nb-run-all {
  background: var(--nb-accent);
  border-color: var(--nb-accent);
  color: #ffffff;
}
body.notebook-page #nb-run-all:hover {
  background: var(--nb-accent-hover);
  border-color: var(--nb-accent-hover);
}
body.notebook-page #nb-run-all::before {
  content: "▶";
  margin-right: 0.38rem;
  font-size: 0.58em;
  vertical-align: 0.08em;
}
body.notebook-page #nb-restart,
body.notebook-page #nb-step,
body.notebook-page #nb-state,
body.notebook-page #nb-journal,
body.notebook-page #nb-reload,
body.notebook-page #nb-debug {
  color: var(--nb-text);
}
body.notebook-page #nb-status {
  margin-left: auto;
  padding: 0 0.45rem;
  color: var(--nb-muted);
  font: 500 0.68rem/1.6 var(--sans);
  white-space: nowrap;
}
body.notebook-page #nb-status::before {
  content: "";
  display: inline-block;
  width: 0.42rem;
  height: 0.42rem;
  margin-right: 0.38rem;
  border-radius: 50%;
  background: var(--nb-success);
  vertical-align: 0.04em;
}
body.notebook-page #nb-state-actions {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.3rem;
  margin: 0 0 0.55rem;
  padding: 0.38rem 0.45rem;
  border: 1px solid var(--nb-border);
  border-radius: var(--nb-radius);
  background: var(--nb-surface);
  color: var(--nb-muted);
  font: 500 0.68rem/1.2 var(--sans);
}
body.notebook-page #nb-state-actions span:first-child { color: var(--nb-text); }
body.notebook-page #nb-state-actions input {
  width: 6.8rem;
  padding: 0.28rem 0.4rem;
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-bg);
  color: var(--nb-text);
  font: 0.68rem/1.2 var(--mono);
}
body.notebook-page #nb-state-actions input:focus {
  outline: 1px solid var(--nb-accent);
  border-color: var(--nb-accent);
}
body.notebook-page #nb-state-actions button {
  padding: 0.3rem 0.48rem;
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-surface-2);
  color: var(--nb-text);
  font: 500 0.68rem/1.2 var(--sans);
  cursor: pointer;
}
body.notebook-page #nb-state-actions button:hover { background: var(--nb-surface-3); }
body.notebook-page #nb-pull-out { color: var(--nb-success); font: 0.68rem/1.3 var(--mono); }
body.notebook-page #nb-state-out {
  max-height: 22rem;
  overflow: auto;
  margin: 0 0 0.55rem;
  padding: 0.65rem 0.75rem;
  border: 1px solid var(--nb-border);
  border-radius: var(--nb-radius);
  background: var(--nb-bg);
  color: var(--nb-text);
  font: 0.7rem/1.5 var(--mono);
  white-space: pre-wrap;
}

/* ── Notebook cells ────────────────────────────────────────────────── */
body.notebook-page .structure {
  display: flex;
  flex-direction: column;
  padding: 0;
  border: 1px solid var(--nb-border);
  border-radius: var(--nb-radius);
  background: var(--nb-surface);
  overflow: hidden;
}
body.notebook-page .structure > .doc-block,
body.notebook-page .structure > .card {
  margin: 0;
  border: 0;
  border-bottom: 1px solid var(--nb-border);
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}
body.notebook-page .structure > .doc-block:last-child,
body.notebook-page .structure > .card:last-child { border-bottom: 0; }
body.notebook-page .structure > .doc-block:not(.doc-exec) { padding: 0.75rem 1rem; }
body.notebook-page .structure > .doc-block.doc-exec { padding: 0; }
body.notebook-page .doc-prose {
  color: var(--nb-text);
  font-size: 0.82rem;
  line-height: 1.6;
}
body.notebook-page .doc-table-wrap { overflow-x: auto; }
body.notebook-page .doc-table {
  width: 100%;
  min-width: 0;
  margin: 0;
  border-collapse: collapse;
  font-size: 0.74rem;
}
body.notebook-page .doc-table th,
body.notebook-page .doc-table td {
  padding: 0.38rem 0.5rem;
  border: 1px solid var(--nb-border);
  text-align: left;
}
body.notebook-page .doc-table th {
  background: var(--nb-surface-2);
  color: var(--nb-muted);
  font-weight: 600;
}
body.notebook-page .doc-table td { color: var(--nb-text); }

/* ── Code cell ─────────────────────────────────────────────────────── */
body.notebook-page .annot-chip:hover,
body.notebook-page .annot-chip.is-focus {
  background: var(--nb-surface-3);
}
body.notebook-page .mq-anchor.is-focus {
  background: rgba(255,255,255,.08);
}
body.notebook-page .code-card {
  display: flex;
  flex-direction: column;
  padding: 0;
  border: 0;
  border-radius: 0;
  background: var(--nb-surface);
  overflow: hidden;
}
body.notebook-page .code-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.35rem;
  min-height: 2rem;
  padding: 0.34rem 0.6rem;
  border-bottom: 1px solid var(--nb-border);
  background: var(--nb-surface-2);
}
body.notebook-page .code-head .badge { display: none; }
body.notebook-page .code-name {
  order: 1;
  color: var(--nb-strong);
  font: 600 0.78rem/1.3 var(--sans);
}
body.notebook-page .code-lang {
  order: 2;
  color: var(--nb-muted);
  font: 0.66rem/1.3 var(--mono);
}
body.notebook-page .code-head .nb-kernel {
  order: 3;
  max-width: 9rem;
  padding: 0.18rem 0.28rem;
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-bg);
  color: var(--nb-text);
  font: 0.66rem/1.2 var(--sans);
  cursor: pointer;
}
body.notebook-page .code-head .nb-cell-stdin {
  order: 4;
  width: 8rem;
  padding: 0.16rem 0.3rem;
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-bg);
  color: var(--nb-text);
  font: 0.62rem/1.2 var(--mono);
}
body.notebook-page .code-head .nb-clean {
  order: 5;
  padding: 0.14rem 0.4rem;
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-bg);
  color: var(--nb-muted);
  font: 0.62rem/1.2 var(--sans);
  cursor: pointer;
}
body.notebook-page .code-head .nb-clean:hover {
  border-color: var(--nb-accent);
  color: var(--nb-strong);
}
body.notebook-page .code-head .nb-badge {
  order: 6;
  padding: 0.14rem 0.34rem;
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-bg);
  color: var(--nb-muted);
  font: 600 0.58rem/1.2 var(--sans);
  text-transform: uppercase;
}
body.notebook-page .nb-badge.ok { color: var(--nb-success); }
body.notebook-page .nb-badge.bad { color: var(--nb-error); }
body.notebook-page .code-head .code-actions {
  order: 7;
  display: flex;
  align-items: center;
  gap: 0.3rem;
  margin-left: auto;
}
body.notebook-page .code-foot { display: none; }
body.notebook-page .code-cmd {
  width: 5.4rem;
  padding: 0.22rem 0.35rem;
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-bg);
  color: var(--nb-text);
  font: 0.64rem/1.2 var(--mono);
}
body.notebook-page .code-cmd:focus {
  outline: 1px solid var(--nb-accent);
  border-color: var(--nb-accent);
}
body.notebook-page .code-run {
  padding: 0.27rem 0.55rem;
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-surface-3);
  color: var(--nb-text);
  font: 500 0.66rem/1.2 var(--sans);
  cursor: pointer;
}
body.notebook-page .code-run:hover {
  background: var(--nb-accent);
  border-color: var(--nb-accent);
  color: #fff;
}
body.notebook-page .code-run:disabled { opacity: 0.45; cursor: not-allowed; }
body.notebook-page .code-body {
  margin: 0;
  padding: 0.9rem 1rem;
  min-height: 6rem;
  max-height: 70vh;
  overflow: auto;
  resize: vertical;
  border: 0;
  background: var(--nb-code-bg);
  color: var(--nb-code-text);
  font: 0.76rem/1.55 var(--mono);
  tab-size: 4;
}
body.notebook-page .code-body::-webkit-resizer,
body.notebook-page .code-out::-webkit-resizer {
  background: linear-gradient(135deg, transparent 0 48%, var(--nb-muted) 48% 100%);
}
body.notebook-page .code-body code {
  background: transparent;
  color: inherit;
  font: inherit;
}
body.notebook-page .code-body .hljs { background: transparent; padding: 0; }
body.notebook-page .code-out {
  margin: 0;
  padding: 0.7rem 1rem 0.85rem;
  min-height: 3rem;
  max-height: 60vh;
  overflow: auto;
  resize: vertical;
  border-top: 1px solid var(--nb-border);
  background: var(--nb-code-bg);
  color: var(--nb-code-text);
  font: 0.74rem/1.55 var(--mono);
  white-space: pre-wrap;
  word-break: break-word;
}
body.notebook-page .code-out::before {
  content: "Output";
  display: block;
  margin-bottom: 0.38rem;
  color: var(--nb-muted);
  font: 0.62rem/1.2 var(--mono);
}
body.notebook-page .code-out.err {
  background: var(--nb-error-bg);
  color: var(--nb-error);
}
body.notebook-page .code-card .code-out:not([hidden]) { display: block; }
body.notebook-page #notebook-bar[hidden],
body.notebook-page #nb-state-actions[hidden],
body.notebook-page #nb-state-out[hidden],
body.notebook-page .code-out[hidden] { display: none !important; }

/* ── Original right rail: Functions / Variables ───────────────────── */
body.notebook-page .outline-rail { gap: 0.5rem; }
body.notebook-page .outline-section {
  padding: 0.4rem 0.55rem 0.55rem;
  border: 1px solid var(--nb-border);
  border-radius: var(--nb-radius);
  background: var(--nb-surface);
  box-shadow: none;
}
body.notebook-page .outline-summary { color: var(--nb-muted); }
body.notebook-page .outline-panel { background: transparent; }
body.notebook-page .fn-search {
  margin-bottom: 0.4rem;
  padding: 0.3rem 0.4rem;
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-bg);
  color: var(--nb-text);
  font: 0.7rem/1.3 var(--sans);
}
body.notebook-page .fn-search:focus {
  outline: 1px solid var(--nb-accent);
  border-color: var(--nb-accent);
}
body.notebook-page .outline-tree ul { border-left-color: var(--nb-border); }
body.notebook-page .outline-item > a { color: var(--nb-text); }
body.notebook-page .outline-item .ol-meta,
body.notebook-page .outline-item .ol-params { color: var(--nb-muted); }
body.notebook-page .outline-item .ol-param.inferred { border-bottom-color: var(--nb-border); }
body.notebook-page .vars-panel { padding-right: 0.15rem; }
body.notebook-page .vars-card {
  padding: 0.42rem 0.48rem;
  border: 1px solid var(--nb-border);
  border-radius: 4px;
  background: var(--nb-surface-2);
}
body.notebook-page .vars-card:hover {
  border-color: var(--nb-accent);
  box-shadow: none;
}
body.notebook-page .vars-name { color: var(--nb-text); }
body.notebook-page .vars-kind { color: var(--nb-muted); }
body.notebook-page .vars-preview {
  border: 1px solid var(--nb-border);
  border-radius: 3px;
  background: var(--nb-bg);
}
body.notebook-page .vars-preview::after {
  background: linear-gradient(transparent, var(--nb-bg));
}
body.notebook-page .vars-table th {
  border-color: var(--nb-border);
  background: var(--nb-surface-2);
  color: var(--nb-muted);
}
body.notebook-page .vars-table td { border-color: var(--nb-border); }
body.notebook-page .vars-idx,
body.notebook-page .vars-key,
body.notebook-page .vars-muted,
body.notebook-page .vars-empty,
body.notebook-page .vars-skipped,
body.notebook-page .vars-nested > summary { color: var(--nb-muted); }

/* ── Execution / Source blocks that the original layout keeps ──────── */
body.notebook-page .out,
body.notebook-page .source {
  border: 1px solid var(--nb-border);
  border-radius: var(--nb-radius);
  background: var(--nb-surface);
  color: var(--nb-text);
  box-shadow: none;
}
body.notebook-page .source { background: var(--nb-bg); }
body.notebook-page .out.ok { background: var(--nb-surface); }
body.notebook-page .out.fail {
  border-color: var(--nb-error-border);
  background: var(--nb-error-bg);
  color: var(--nb-error);
}
body.notebook-page .stdin-form label { color: var(--nb-text); }
body.notebook-page .stdin-form textarea {
  border-color: var(--nb-border);
  background: var(--nb-bg);
  color: var(--nb-text);
}
body.notebook-page .stdin-form button {
  border-color: var(--nb-accent);
  background: var(--nb-accent);
  color: #fff;
}

body.notebook-page .output-card {
  background: var(--nb-surface-2);
  border-left-color: var(--nb-accent);
}
body.notebook-page .output-card.output-card-image { border-left-color: var(--nb-success); }
body.notebook-page .output-card .wb-badges { color: var(--nb-muted); }
body.notebook-page .output-card .output-body {
  color: var(--nb-text);
  background: var(--nb-bg);
}
body.notebook-page .wb-badge {
  background: var(--nb-surface-3);
  color: var(--nb-text);
  border-color: var(--nb-border);
}
body.notebook-page .wb-badge.wb-ok { background: #143d2b; color: #86efac; }
body.notebook-page .wb-badge.wb-error { background: #4a1d1d; color: #fca5a5; }
body.notebook-page .wb-badge.wb-trace { background: #2e2a5a; color: #c7d2fe; }
body.notebook-page .doc-writeback {
  background: var(--nb-surface-2);
  border-color: var(--nb-border);
}
body.notebook-page .wb-source-badge {
  background: var(--nb-surface-3);
  color: var(--nb-muted);
}
body.notebook-page .wb-source-text { color: var(--nb-text); }

/* ── Narrow preview panes ──────────────────────────────────────────── */
@media (max-width: 760px) {
  body.notebook-page main.panel { padding: 1rem 0.75rem 3.5rem; }
  body.notebook-page #notebook-bar { position: static; }
  body.notebook-page #notebook-bar strong { width: 100%; padding-bottom: 0.15rem; }
  body.notebook-page #nb-status { margin-left: 0; }
  body.notebook-page .code-name { width: 100%; }
  body.notebook-page .code-head .nb-kernel { margin-left: 0; }
  body.notebook-page .code-cmd { display: none; }
  body.notebook-page .code-body { max-height: 22rem; }
}
</style>
<div class="notebook-bar" id="notebook-bar" hidden>
  <strong>View</strong>
  <button type="button" id="nb-run-all">Run All</button>
  <button type="button" id="nb-restart">Restart &amp; Run All</button>
  <button type="button" id="nb-step">Step</button>
  <button type="button" id="nb-state">State</button>
  <button type="button" id="nb-journal">Journal</button>
  <button type="button" id="nb-clear">Clear outputs</button>
  <button type="button" id="nb-reload">Reload</button>
  <button type="button" id="nb-debug">Debug</button>
  <button type="button" id="nb-theme" title="切换浅色/深色主题">浅色</button>
  <span id="nb-status">opening…</span>
</div>
<div class="nb-stdin-bar" id="nb-stdin-bar">
  <label for="nb-stdin">Cell stdin / input</label>
  <textarea id="nb-stdin" rows="1" placeholder="raw stdin for stateless cells; JSON/string for persistent kernels" spellcheck="false"></textarea>
</div>
<div class="notebook-bar" id="nb-state-actions" hidden>
  <span>Bridge</span>
  <input id="nb-kernel-input" placeholder="kernel" autocomplete="off"/>
  <input id="nb-name-input" placeholder="name" autocomplete="off"/>
  <input id="nb-value-input" placeholder="JSON value" autocomplete="off"/>
  <button type="button" id="nb-push">Push</button>
  <button type="button" id="nb-pull">Pull</button>
  <span id="nb-pull-out"></span>
</div>
<pre id="nb-state-out" hidden></pre>
<script>
(function () {
  document.body.classList.add("notebook-page");
  var THEME_KEY = "marqdo.nb.theme";
  function applyTheme(theme) {
    theme = theme === "light" ? "light" : "dark";
    document.body.dataset.theme = theme;
    var btn = document.getElementById("nb-theme");
    if (btn) {
      btn.textContent = theme === "light" ? "深色" : "浅色";
      btn.title = theme === "light" ? "切换到深色主题" : "切换到浅色主题";
    }
    try { localStorage.setItem(THEME_KEY, theme); } catch (e) {}
  }
  var savedTheme = "dark";
  try { savedTheme = localStorage.getItem(THEME_KEY) || "dark"; } catch (e) {}
  applyTheme(savedTheme);
  var live = !!window.MARQDO_VIEW_LIVE;
  var path = window.MARQDO_FILE_PATH || "";
  if (!path) return;
  var state = { open: false, persistent: false, kernels: [], cells: {} };
  function post(url, body) {
    return fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(Object.assign({ path: path }, body || {}))
    }).then(function (r) { return r.json(); });
  }
  function status(t) { var e = document.getElementById("nb-status"); if (e) e.textContent = t; }
  function cellStdin() {
    var el = document.getElementById("nb-stdin");
    return el ? el.value : "";
  }
  function card(name) {
    var els = document.querySelectorAll(".code-card");
    for (var i = 0; i < els.length; i++) {
      if (els[i].getAttribute("data-code-name") === name) return els[i];
    }
    return null;
  }
  function badge(el, text, ok) {
    if (!el) return;
    var b = el.querySelector(".nb-badge");
    if (!b) {
      b = document.createElement("span");
      var head = el.querySelector(".code-head");
      if (!head) return;
      head.appendChild(b);
    }
    b.textContent = text;
    b.className = "nb-badge " + (ok ? "ok" : (text === "clean" || text === "never" ? "" : "bad"));
  }
  function ensureKernel(el, persistent) {
    var sel = el.querySelector(".nb-kernel");
    if (!sel) {
      sel = document.createElement("select");
      sel.className = "nb-kernel";
      var head = el.querySelector(".code-head");
      if (head) head.appendChild(sel);
    }
    if (!persistent) {
      sel.disabled = true;
      sel.title = "No live kernel; this cell is stateless";
      sel.innerHTML = "";
      var opt = document.createElement("option");
      opt.textContent = "one-shot";
      sel.appendChild(opt);
    } else {
      sel.disabled = false;
      sel.title = "Switch kernel";
    }
    return sel;
  }
  function ensureCellStdin(el, c) {
    var inp = el.querySelector(".nb-cell-stdin");
    if (!inp) {
      inp = document.createElement("input");
      inp.className = "nb-cell-stdin";
      inp.placeholder = "stdin";
      inp.autocomplete = "off";
      var head = el.querySelector(".code-head");
      if (head) head.appendChild(inp);
    }
    if (c && inp.dataset.loaded !== "1") {
      inp.value = c.stdin || "";
      inp.dataset.loaded = "1";
    }
    return inp;
  }
  function clearCard(el) {
    var out = el.querySelector(".code-out");
    if (out) {
      out.hidden = true;
      out.textContent = "";
      out.classList.remove("err");
    }
    var b = el.querySelector(".nb-badge");
    if (b) { b.textContent = "never"; b.className = "nb-badge"; }
  }
  function bindClean(el) {
    var btn = el.querySelector(".nb-clean");
    if (!btn || btn.dataset.bound) return;
    btn.dataset.bound = "1";
    btn.addEventListener("click", function (e) {
      e.preventDefault();
      var name = el.getAttribute("data-code-name") || "";
      if (!live) {
        clearCard(el);
        status("static export: local output cleared");
        return;
      }
      if (!name) return;
      post("api/session/cell/clear", { cell: name }).then(function (r) {
        clearCard(el);
        if (r.ok && r.session) renderSummary(r.session);
        status(r.ok ? "clean" : (r.error || "clean failed"));
      }).catch(function (e2) { status(String(e2)); });
    });
  }
  function renderRecord(name, rec) {
    var el = card(name);
    if (!el || !rec) return;
    var out = el.querySelector(".code-out");
    if (out) {
      out.hidden = false;
      var parts = [];
      if (rec.stdout) parts.push(rec.stdout);
      if (rec.stderr) parts.push("[stderr]\n" + rec.stderr);
      if (rec.error) parts.push(rec.error);
      var meta = [];
      if (typeof rec.code === "number") meta.push("exit " + rec.code);
      if (typeof rec.ms === "number") meta.push(rec.ms + "ms");
      if (rec.timed_out) meta.push("timed out");
      if (rec.truncated) meta.push("truncated");
      if (meta.length) parts.push("— " + meta.join(" · "));
      out.textContent = parts.length ? parts.join("\n\n") : "(no output)";
      out.classList.toggle("err", !rec.ok);
    }
    badge(el, rec.ok ? "clean" : "error", !!rec.ok);
  }
  function renderSummary(s) {
    if (!s) return;
    state.persistent = !!s.persistent;
    state.kernels = s.kernels || [];
    state.cells = {};
    (s.cells || []).forEach(function (c) { state.cells[c.name] = c; });
    (s.cells || []).forEach(function (c) {
      var el = card(c.name);
      if (!el) return;
      badge(el, c.status, c.status === "clean");
      var sel = ensureKernel(el, state.persistent);
      if (!state.persistent) return;
      if (!sel.dataset.bound) {
        sel.dataset.bound = "1";
        sel.addEventListener("change", function () {
          post("api/session/kernel/assign", { cell: c.name, kernel: sel.value }).then(function (r) {
            status(r.ok ? "kernel: " + sel.value : (r.error || "assign failed"));
          });
        });
      }
      var opts = state.kernels.filter(function (k) { return k.lang === c.lang; }).map(function (k) { return k.name; });
      if (!opts.length) opts = [c.kernel];
      sel.innerHTML = "";
      opts.forEach(function (n) {
        var o = document.createElement("option");
        o.value = n; o.textContent = n;
        if (n === c.kernel) o.selected = true;
        sel.appendChild(o);
      });
    });
  }
  function renderStaticSummary(s) {
    if (!s) return;
    state.persistent = !!s.persistent;
    state.kernels = s.kernels || [];
    state.cells = {};
    (s.cells || []).forEach(function (c) { state.cells[c.name] = c; });
    (s.cells || []).forEach(function (c) {
      var el = card(c.name);
      if (!el) return;
      badge(el, c.status, c.status === "clean");
      ensureCellStdin(el, c);
      if (!state.persistent) {
        ensureKernel(el, false);
        return;
      }
      var sel = ensureKernel(el, true);
      var opts = state.kernels.filter(function (k) { return k.lang === c.lang; }).map(function (k) { return k.name; });
      if (!opts.length) opts = [c.kernel];
      sel.innerHTML = "";
      opts.forEach(function (n) {
        var o = document.createElement("option");
        o.value = n; o.textContent = n;
        if (n === c.kernel) o.selected = true;
        sel.appendChild(o);
      });
      sel.disabled = true;
      sel.title = "Static export: kernel fixed to " + c.kernel;
    });
  }
  function openSession() {
    var bar = document.getElementById("notebook-bar");
    if (bar) bar.hidden = false;
    post("api/session/open", {}).then(function (r) {
      if (!r.ok) { status(r.error || "session open failed"); return; }
      state.open = true;
      renderSummary(r.session);
      status("session ready");
    });
  }
  function runOne(name) {
    var inputEl = document.querySelector('.code-card[data-code-name="' + name + '"] .nb-cell-stdin');
    var stdinValue = inputEl && inputEl.value ? inputEl.value : cellStdin();
    return post("api/session/cell/run", { cell: name, stdin: stdinValue }).then(function (r) {
      if (r.ok) { renderRecord(name, r.record); if (r.session) renderSummary(r.session); }
      else status(r.error || "run failed");
    });
  }
  function renderDocumentResult(data) {
    var out = document.getElementById("exec-main");
    if (out) {
      out.hidden = false;
      out.classList.toggle("err", !data.ok);
      out.textContent = data.ok ? (data.stdout || "(no output)") : (data.error || "run failed");
    }
    var block = document.getElementById("exec-block");
    if (block) block.hidden = false;
  }
  function runAll(restart) {
    status(restart ? "restarting…" : "running all…");
    if (!state.persistent && Object.keys(state.cells).length === 0) {
      post("api/session/run_document", {}).then(function (r) {
        renderDocumentResult(r);
        status(r.ok ? "document run done" : (r.error || "document run failed"));
      }).catch(function (e) { status(String(e)); });
      return;
    }
    var perCell = {};
    document.querySelectorAll(".code-card").forEach(function (el) {
      var inp = el.querySelector(".nb-cell-stdin");
      var name = el.getAttribute("data-code-name");
      if (inp && name && inp.value) perCell[name] = inp.value;
    });
    post("api/session/run_all", { restart: !!restart, stdin: cellStdin(), inputs: perCell }).then(function (r) {
      if (!r.ok) { status(r.error || "run all failed"); return; }
      if (r.session) renderSummary(r.session);
      var recs = (r.summary && r.summary.records) || [];
      recs.forEach(function (rec) { if (rec.cell) renderRecord(rec.cell, rec); });
      status("run all done");
    });
  }
  function clearDom() {
    document.querySelectorAll(".code-out").forEach(function (el) {
      el.hidden = true;
      el.textContent = "";
      el.classList.remove("err");
    });
    document.querySelectorAll(".nb-badge").forEach(function (el) {
      el.textContent = "never";
      el.className = "nb-badge";
    });
    var exec = document.getElementById("exec-main");
    if (exec) { exec.hidden = true; exec.textContent = ""; }
    var execBlock = document.getElementById("exec-block");
    if (execBlock) {
      execBlock.querySelectorAll(".plots, .plan-card").forEach(function (el) {
        el.hidden = true;
        el.innerHTML = "";
      });
    }
    var stateOut = document.getElementById("nb-state-out");
    if (stateOut) { stateOut.hidden = true; stateOut.textContent = ""; }
    var pullOut = document.getElementById("nb-pull-out");
    if (pullOut) pullOut.textContent = "";
  }
  function clearOutputs() {
    clearDom();
    post("api/session/clear", {}).then(function (r) {
      if (r.ok && r.session) renderSummary(r.session);
      status(r.ok ? "outputs cleared" : (r.error || "clear failed"));
    }).catch(function (e) { status(String(e)); });
  }
  function bind(id, fn) { var b = document.getElementById(id); if (b) b.addEventListener("click", fn); }
  function init() {
    var themeBtn = document.getElementById("nb-theme");
    if (themeBtn) {
      themeBtn.addEventListener("click", function () {
        applyTheme(document.body.dataset.theme === "light" ? "dark" : "light");
      });
    }
    document.querySelectorAll(".code-body, .code-out").forEach(function (el) {
      el.title = "拖动右下角调整高度";
    });
    document.querySelectorAll(".code-card").forEach(function (cardEl) {
      var head = cardEl.querySelector(".code-head");
      var foot = cardEl.querySelector(".code-foot");
      if (head && foot) {
        var actions = foot.querySelector(".code-actions");
        if (actions) head.appendChild(actions);
        foot.remove();
      }
    });
    if (!live) {
      var staticBar = document.getElementById("notebook-bar");
      if (staticBar) staticBar.hidden = false;
      var staticStdin = document.getElementById("nb-stdin-bar");
      if (staticStdin) staticStdin.hidden = false;
      function staticNotice() {
        status("静态页面无法真实执行 / static export: no live execution");
      }
      ["nb-run-all", "nb-restart", "nb-step", "nb-state", "nb-journal",
       "nb-clear", "nb-reload", "nb-debug", "exec-clear"].forEach(function (id) {
        bind(id, staticNotice);
      });
      document.querySelectorAll(".code-run").forEach(function (btn) {
        btn.addEventListener("click", function (e) {
          e.preventDefault();
          staticNotice();
        });
      });
      if (window.MARQDO_STATIC_SESSION) {
        renderStaticSummary(window.MARQDO_STATIC_SESSION);
      } else {
        document.querySelectorAll(".code-card").forEach(function (el) {
          ensureKernel(el, false);
        });
      }
      document.querySelectorAll(".code-card").forEach(bindClean);
      status("static export · preview only");
      return;
    }
    document.querySelectorAll(".code-card").forEach(bindClean);
    bind("nb-run-all", function () { runAll(false); });
    bind("nb-restart", function () { runAll(true); });
    bind("nb-step", function () {
      if (!state.persistent && Object.keys(state.cells).length === 0) {
        status("no cells; Run All runs the document");
        return;
      }
      post("api/session/cell/step", { stdin: cellStdin() }).then(function (r) {
        if (!r.ok) { status(r.error || "step failed"); return; }
        if (r.record) {
          if (r.record.cell) renderRecord(r.record.cell, r.record);
          if (r.record.done) status("done");
        }
        if (r.session) renderSummary(r.session);
        if (!r.record || !r.record.done) status("step");
      });
    });
    bind("nb-state", function () {
      post("api/session/state/list", {}).then(function (r) {
        var pre = document.getElementById("nb-state-out");
        var actions = document.getElementById("nb-state-actions");
        var ki = document.getElementById("nb-kernel-input");
        if (!pre) return;
        pre.hidden = false;
        pre.textContent = JSON.stringify(r.state || r, null, 2);
        if (actions) actions.hidden = false;
        if (ki && !ki.value && state.kernels.length) ki.value = state.kernels[0].name;
        if (!state.persistent) status("stateless: no kernel state");
      });
    });
    bind("nb-push", function () {
      var k = (document.getElementById("nb-kernel-input") || {}).value || "";
      var n = (document.getElementById("nb-name-input") || {}).value || "";
      var raw = (document.getElementById("nb-value-input") || {}).value || "";
      if (!k || !n) { status("kernel and name required"); return; }
      var value = null;
      if (raw.trim()) {
        try { value = JSON.parse(raw); } catch (e) { status("value must be JSON"); return; }
      }
      post("api/session/state/push", { kernel: k, name: n, value: value }).then(function (r) {
        status(r.ok ? "pushed " + n + " → " + k : (r.error || "push failed"));
      });
    });
    bind("nb-pull", function () {
      var k = (document.getElementById("nb-kernel-input") || {}).value || "";
      var n = (document.getElementById("nb-name-input") || {}).value || "";
      var out = document.getElementById("nb-pull-out");
      if (!k || !n) { status("kernel and name required"); return; }
      post("api/session/state/pull", { kernel: k, name: n }).then(function (r) {
        if (out) out.textContent = r.ok ? (n + " = " + JSON.stringify(r.value)) : (r.error || "pull failed");
        status(r.ok ? "pulled " + n + " ← " + k : "pull failed");
      });
    });
    bind("nb-journal", function () {
      post("api/session/journal", {}).then(function (r) {
        var pre = document.getElementById("nb-state-out");
        if (!pre) return;
        pre.hidden = false;
        pre.textContent = JSON.stringify(r.journal || r, null, 2);
        if (!state.persistent) status("stateless: journal is per one-shot run");
      });
    });
    bind("nb-debug", function () {
      window.open("debug?path=" + encodeURIComponent(path), "_blank");
    });
    bind("nb-clear", clearOutputs);
    bind("exec-clear", clearOutputs);
    bind("nb-reload", function () {
      post("api/session/reload", {}).then(function (r) {
        if (!r.ok) { status(r.error || "reload failed"); return; }
        renderSummary(r.session);
        status("reloaded");
      });
    });
    document.addEventListener("click", function (e) {
      var btn = e.target && e.target.closest ? e.target.closest(".code-run") : null;
      if (!btn || !state.open) return;
      var el = btn.closest(".code-card");
      if (!el) return;
      var name = el.getAttribute("data-code-name");
      if (!name) return;
      e.preventDefault();
      e.stopImmediatePropagation();
      btn.disabled = true;
      status("running " + name);
      runOne(name).finally(function () { btn.disabled = false; status("ready"); });
    }, true);
    openSession();
  }
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", init);
  else init();
})();
</script>"#
}

pub fn page_file(files: &[PathBuf], rel: &str, vm: &FileViewModel, links: &LinkMode) -> String {
    let nav = nav_html(files, Some(rel), links);
    let (status, status_label, out_text) = if vm.awaiting_input {
        (
            "pending",
            "input",
            if vm.show_stream {
                "Fill preset input, then <strong>Run with input</strong> — output streams in the panel (no page reload)."
                    .to_string()
            } else {
                "Fill preset input, then <strong>Run with input</strong>."
                    .to_string()
            },
        )
    } else if vm.auto_stream {
        (
            "pending",
            "stream",
            "Streaming in the panel below…".to_string(),
        )
    } else if vm.ok {
        (
            "ok",
            "ok",
            if vm.stdout.is_empty() {
                "(no stdout)".into()
            } else {
                escape(&vm.stdout)
            },
        )
    } else if vm.stderr.is_empty() {
        ("fail", "fail", "(skipped)".into())
    } else {
        ("fail", "fail", escape(&vm.stderr))
    };
    let stdin_panel = if vm.input_prompts.is_empty() {
        String::new()
    } else {
        let placeholder = vm.input_prompts.join("\n");
        let rows = vm.input_prompts.len().clamp(2, 8);
        match links {
            LinkMode::Live => {
                let hint = if vm.input_prompts.iter().any(|p| !p.is_empty()) {
                    format!(
                        "prompts: {}",
                        escape(&vm.input_prompts.join(" · "))
                    )
                } else {
                    "one line per <code>input</code>".into()
                };
                format!(
                    r#"<form class="stdin-form" method="get" action="/file">
  <input type="hidden" name="path" value="{path}"/>
  <label for="stdin">Preset input <span class="hint">{hint}</span></label>
  <textarea id="stdin" name="stdin" rows="{rows}" placeholder="{placeholder}">{stdin}</textarea>
  <button type="submit">Run with input</button>
</form>"#,
                    path = escape(rel),
                    hint = hint,
                    rows = rows,
                    placeholder = escape(&placeholder),
                    stdin = escape(&vm.preset_stdin),
                )
            }
            LinkMode::Static { .. } => {
                if vm.preset_stdin.is_empty() {
                    String::new()
                } else {
                    format!(
                        r#"<p class="hint">Demo stdin (static export cannot re-run interactively); prompts: {prompts}</p>
<pre class="source" style="margin-bottom:0.75rem">{stdin}</pre>"#,
                        prompts = escape(&vm.input_prompts.join(" · ")),
                        stdin = escape(&vm.preset_stdin),
                    )
                }
            }
        }
    };
    let title = Path::new(rel)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(rel)
        .trim_end_matches(".mq.md");
    let plots_html = if vm.plots.is_empty() {
        String::new()
    } else {
        let mut s = String::from(r#"<div class="plots">"#);
        for svg in &vm.plots {
            s.push_str(r#"<div class="plot">"#);
            s.push_str(svg);
            s.push_str("</div>");
        }
        s.push_str("</div>");
        s
    };
    let live_flag = match links {
        LinkMode::Live => "true",
        LinkMode::Static { .. } => "false",
    };
    let exec_clear = r#"<button type="button" class="exec-clear" id="exec-clear">Clear</button>"#;
    let outline_block = if vm.outline_html.is_empty() {
        String::new()
    } else {
        vm.outline_html.clone()
    };
    let structure_block = if outline_block.is_empty() {
        format!(
            r#"<div class="structure" id="structure">{structure}</div>"#,
            structure = vm.structure_html
        )
    } else {
        format!(
            r#"<div class="struct-wrap">
  <div class="structure" id="structure">{structure}</div>
  {outline}
</div>"#,
            structure = vm.structure_html,
            outline = outline_block,
        )
    };
    let plan_card = plan_result_card(&vm.stdout, links);
    let stream_panel = match links {
        LinkMode::Live if vm.show_stream => r#"<div class="stream-panel">
  <div class="stream-head">
    <strong>Stream</strong>
    <button type="button" id="stream-run">Run</button>
    <button type="button" class="stream-stop" id="stream-stop" hidden disabled>Stop</button>
    <span class="stream-status" id="stream-status">idle</span>
  </div>
  <div class="stream-body" id="stream-body">
    <p class="stream-empty">Use <strong>Run with input</strong> or <strong>Run</strong> — thinking and answer stream here.</p>
  </div>
  <button type="button" class="stream-jump" id="stream-jump" hidden>↓ Latest</button>
</div>"#
        .to_string(),
        _ => String::new(),
    };
    let trace_safe = vm
        .trace_json
        .replace("</", "<\\/");
    // Static exports share the same view shell. The script sees
    // MARQDO_VIEW_LIVE=false and leaves the interactive controls hidden.
    let notebook = notebook_ui().to_string();
    let main = format!(
        r#"
<script>
window.MARQDO_VIEW_LIVE = {live};
window.MARQDO_FILE_PATH = {path_js};
window.MARQDO_AUTO_STREAM = {auto_stream};
</script>
<script type="application/json" id="mq-view-trace">{trace}</script>
<h1 class="page">{title}<span class="status-pill {status}">{status_label}</span></h1>
<p class="meta">{rel}</p>
{notebook}
<section class="block structure-block">
  <h2>Structure</h2>
  {structure_block}
</section>
<section class="block exec-block" id="exec-block">
  <h2>Execution {exec_clear}</h2>
  {stdin_panel}
  {stream_panel}
  {plan_card}
  <div class="out {status}" id="exec-main">{out}</div>
  {plots}
</section>
<section class="block source-block">
  <h2>Source</h2>
  <pre class="source">{source}</pre>
</section>
"#,
        live = live_flag,
        path_js = escape_js_string(rel),
        auto_stream = if vm.auto_stream { "true" } else { "false" },
        trace = if trace_safe.is_empty() {
            "{}"
        } else {
            trace_safe.as_str()
        },
        title = escape(title),
        status = status,
        status_label = status_label,
        rel = escape(rel),
        notebook = notebook,
        exec_clear = exec_clear,
        structure_block = structure_block,
        stdin_panel = stdin_panel,
        stream_panel = stream_panel,
        plan_card = plan_card,
        out = out_text,
        plots = plots_html,
        source = escape(&vm.source),
    );
    layout(rel, &nav, &main)
}

/// Plan / agent result map → card with workbook link, cache/rounds, and process events.
fn plan_result_card(stdout: &str, links: &LinkMode) -> String {
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let candidate = trimmed
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .unwrap_or(trimmed);
    let Ok(v) = serde_json::from_str::<serde_json::Value>(candidate) else {
        return String::new();
    };
    let workbook = v.get("workbook").and_then(|x| x.as_str());
    let cache = v.get("cache").and_then(|x| x.as_str());
    let match_kind = v.get("match").and_then(|x| x.as_str());
    let rounds = v
        .get("rounds")
        .and_then(|x| x.as_u64())
        .or_else(|| v.get("round").and_then(|x| x.as_u64()));
    let status = v.get("status").and_then(|x| x.as_str());
    let events = v.get("events").and_then(|x| x.as_array());
    if workbook.is_none() && cache.is_none() && rounds.is_none() && events.is_none() {
        return String::new();
    }
    let mut meta = Vec::new();
    if let Some(s) = status {
        meta.push(format!("status={}", escape(s)));
    }
    if let Some(c) = cache {
        meta.push(format!("cache={}", escape(c)));
    }
    if let Some(m) = match_kind {
        meta.push(format!("match={}", escape(m)));
    }
    if let Some(r) = rounds {
        meta.push(format!("rounds={r}"));
    }
    let link = match (workbook, links) {
        (Some(wb), LinkMode::Live) => {
            let href = format!("/file?path={}", urlencoding_path(wb));
            format!(
                r#"<a href="{href}">Open workbook</a> <code>{}</code>"#,
                escape(wb)
            )
        }
        (Some(wb), LinkMode::Static { .. }) => {
            format!(r#"workbook <code>{}</code>"#, escape(wb))
        }
        _ => String::new(),
    };
    let process = plan_process_list(events);
    format!(
        r#"<div class="plan-card"><strong>Plan result</strong><div class="plan-meta">{}</div>{}{}</div>"#,
        meta.join(" · "),
        link,
        process
    )
}

fn plan_process_list(events: Option<&Vec<serde_json::Value>>) -> String {
    let Some(events) = events else {
        return String::new();
    };
    if events.is_empty() {
        return String::new();
    }
    let mut items = Vec::new();
    for ev in events {
        let t = ev.get("type").and_then(|x| x.as_str()).unwrap_or("event");
        // Skip raw token deltas in the static card — live Stream panel owns those.
        if matches!(t, "delta" | "reasoning") {
            continue;
        }
        let detail = match t {
            "decision" => {
                let d = ev.get("decision").and_then(|x| x.as_str()).unwrap_or("?");
                let sum = ev.get("summary").and_then(|x| x.as_str()).unwrap_or("");
                if sum.is_empty() {
                    escape(d)
                } else {
                    format!("{} — {}", escape(d), escape(sum))
                }
            }
            "round" => {
                let mut bits = Vec::new();
                if let Some(r) = ev.get("round") {
                    bits.push(format!("round={r}"));
                }
                if let Some(wb) = ev.get("workbook").and_then(|x| x.as_str()) {
                    bits.push(format!("workbook={}", escape(wb)));
                }
                if let Some(c) = ev.get("exit_code") {
                    bits.push(format!("exit={c}"));
                }
                if let Some(res) = ev.get("result").and_then(|x| x.as_str()) {
                    let brief: String = res.chars().take(80).collect();
                    bits.push(format!("result={}", escape(&brief)));
                }
                bits.join(" · ")
            }
            "tool_start" | "tool_end" => {
                let name = ev.get("name").and_then(|x| x.as_str()).unwrap_or("?");
                let kind = ev.get("kind").and_then(|x| x.as_str()).unwrap_or("");
                if kind.is_empty() {
                    escape(name)
                } else {
                    format!("{} ({})", escape(name), escape(kind))
                }
            }
            "done" => {
                if let Some(res) = ev.get("result").and_then(|x| x.as_str()) {
                    let brief: String = res.chars().take(80).collect();
                    escape(&brief)
                } else {
                    String::new()
                }
            }
            "error" => escape(ev.get("message").and_then(|x| x.as_str()).unwrap_or("error")),
            _ => String::new(),
        };
        items.push(format!(
            r#"<li><span class="pp-type">{}</span><span class="pp-detail">{}</span></li>"#,
            escape(t),
            detail
        ));
    }
    if items.is_empty() {
        return String::new();
    }
    format!(r#"<ul class="plan-process">{}</ul>"#, items.join(""))
}

fn urlencoding_path(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(*b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn escape_js_string(s: &str) -> String {
    let mut out = String::from('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod plan_card_tests {
    use super::{plan_result_card, LinkMode};

    #[test]
    fn plan_card_renders_process_events() {
        let stdout = r#"{"status":"ok","cache":"hit","match":"exact","rounds":1,"workbook":"wb.mq.md","events":[{"type":"decision","decision":"REUSE","summary":"OKF hit"},{"type":"round","round":1,"workbook":"wb.mq.md","exit_code":0,"result":"pong"},{"type":"delta","text":"skip"},{"type":"done","result":"pong"}]}"#;
        let html = plan_result_card(stdout, &LinkMode::Static { from: None });
        assert!(html.contains("plan-card"), "{html}");
        assert!(html.contains("cache=hit"), "{html}");
        assert!(html.contains("match=exact"), "{html}");
        assert!(html.contains("plan-process"), "{html}");
        assert!(html.contains("REUSE"), "{html}");
        assert!(html.contains("pp-type\">round"), "{html}");
        assert!(!html.contains("pp-type\">delta"), "{html}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notebook_ui_has_controls() {
        let ui = notebook_ui();
        for id in [
            "nb-run-all",
            "nb-restart",
            "nb-step",
            "nb-state",
            "nb-journal",
            "nb-reload",
            "nb-debug",
            "nb-theme",
            "nb-kernel-input",
            "nb-push",
            "nb-pull",
            "nb-state-out",
        ] {
            assert!(ui.contains(id), "notebook UI missing {id}");
        }
        assert!(
            !ui.contains("\"/api/session"),
            "notebook API URLs must stay relative for path-prefix proxy"
        );
        assert!(ui.contains("post(\"api/session/open\""));
        assert!(!ui.contains("In ["), "view cells must not mimic Jupyter In[] prompts");
        assert!(!ui.contains("Out ["), "view cells must not mimic Jupyter Out[] prompts");
        assert!(ui.contains("id=\"nb-clear\""), "view must expose Clear outputs");
        assert!(ui.contains("api/session/clear"), "Clear outputs must call the session API");
    }

}
