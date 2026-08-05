//! Dedicated `marqdo debug` UI — VS Code / Chrome DevTools–style layout.
//!
//! Toolbar · Structure (breakpoints) · Variables / Breakpoints / Frame · Console

use std::path::{Path, PathBuf};

use crate::view::html::{escape, nav_html, LinkMode};
use crate::view::render::{render_function_outline, render_module_structure_mode, StructureMode};

pub struct DebugPageModel {
    #[allow(dead_code)]
    pub rel_path: String,
    #[allow(dead_code)]
    pub source: String,
    pub structure_html: String,
    pub outline_html: String,
    pub stdin_preset: String,
    pub has_input: bool,
}

pub fn build_debug_model(abs: &Path, rel: &str, source: &str) -> DebugPageModel {
    let _ = abs;
    let (structure, outline, has_input) = match crate::parse::parse_source(source) {
        Ok(module) => (
            render_module_structure_mode(&module, source, StructureMode::Debug),
            render_function_outline(&module),
            !crate::view::render::collect_input_prompts(&module).is_empty(),
        ),
        Err(e) => (
            format!(
                "<div class=\"err\">parse error: {}</div>",
                escape(&e.to_string())
            ),
            String::new(),
            false,
        ),
    };
    let stdin_preset = crate::input_feed::effective_stdin(source, &[]).join("\n");
    DebugPageModel {
        rel_path: rel.to_string(),
        source: source.to_string(),
        structure_html: structure,
        outline_html: outline,
        stdin_preset,
        has_input,
    }
}

pub fn page_debug(files: &[PathBuf], rel: &str, vm: &DebugPageModel) -> String {
    let nav = nav_html(files, Some(rel), &LinkMode::Live);
    let title = Path::new(rel)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(rel)
        .trim_end_matches(".mq.md");

    let stdin_box = if vm.has_input {
        format!(
            r#"<label class="dbg-stdin-label">stdin <textarea id="stdin" rows="2" spellcheck="false">{stdin}</textarea></label>"#,
            stdin = escape(&vm.stdin_preset),
        )
    } else {
        String::new()
    };

    let outline = if vm.outline_html.is_empty() {
        String::new()
    } else {
        format!(r#"<div class="dbg-outline">{}</div>"#, vm.outline_html)
    };

    let body = format!(
        r#"
<script>
window.MARQDO_FILE_PATH = {path_js};
</script>
<header class="dbg-toolbar">
  <div class="dbg-brand">
    <span class="dbg-mark">marqdo</span>
    <span class="dbg-mode">debug</span>
  </div>
  <div class="dbg-actions">
    <button type="button" class="dbg-btn primary" id="dbg-start" title="Start">Start</button>
    <button type="button" class="dbg-btn" id="dbg-continue" disabled title="Continue (F5)">Continue</button>
    <button type="button" class="dbg-btn" id="dbg-step" disabled title="Step Over (F10)">Step</button>
    <button type="button" class="dbg-btn danger" id="dbg-stop" disabled title="Stop">Stop</button>
  </div>
  <div class="dbg-status" id="dbg-status">Idle — set breakpoints, then Start · drag edges to resize panes</div>
</header>
<div class="dbg-work">
  <div class="dbg-shell">
    <aside class="dbg-nav" id="dbg-nav">
      <h2>Files</h2>
      {nav}
    </aside>
    <div class="dbg-split-v" id="split-nav" title="Drag to resize"></div>
    <main class="dbg-main">
      <div class="dbg-main-head">
        <h1>{title}</h1>
        <p class="meta">{rel}</p>
        {stdin}
      </div>
      <div class="dbg-center">
        <section class="dbg-structure-pane">
          <h2>Structure <span class="hint">gutter = breakpoint</span></h2>
          <div class="structure debug-surface" id="structure">{structure}</div>
        </section>
        {outline}
      </div>
    </main>
    <div class="dbg-split-v" id="split-side" title="Drag to resize"></div>
    <aside class="dbg-side" id="dbg-side">
      <section class="dbg-panel">
        <h2>Variables</h2>
        <div id="dbg-locals" class="dbg-vars">(idle)</div>
      </section>
      <section class="dbg-panel">
        <h2>Breakpoints</h2>
        <ul id="dbg-bps" class="dbg-bps"><li class="muted">none</li></ul>
      </section>
      <section class="dbg-panel">
        <h2>Call stack</h2>
        <div id="dbg-frame" class="dbg-frame">(idle)</div>
      </section>
    </aside>
  </div>
  <div class="dbg-split-h" id="split-console" title="Drag to resize console"></div>
  <section class="dbg-console" id="dbg-console">
    <h2>Debug Console</h2>
    <pre id="dbg-stdout" class="dbg-out">(idle)</pre>
  </section>
</div>
"#,
        path_js = escape_js_string(rel),
        nav = nav,
        title = escape(title),
        rel = escape(rel),
        stdin = stdin_box,
        structure = vm.structure_html,
        outline = outline,
    );

    layout_debug(rel, &body)
}

fn escape_js_string(s: &str) -> String {
    let mut out = String::from('"');
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn layout_debug(title: &str, main: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>debug · {title} · marqdo</title>
<style>{css}</style>
</head>
<body class="dbg-body">
{main}
<script>
{js}
</script>
</body>
</html>
"##,
        title = escape(title),
        css = debug_css(),
        main = main,
        js = debug_js(),
    )
}

fn debug_css() -> &'static str {
    r#"
:root {
  --bg: #1e1e1e;
  --panel: #252526;
  --ink: #d4d4d4;
  --muted: #858585;
  --line: #3c3c3c;
  --accent: #0e639c;
  --accent-hover: #1177bb;
  --current: #264f78;
  --bp: #e51400;
  --ok: #4ec9b0;
  --nav-w: 168px;
  --side-w: 220px;
  --console-h: 120px;
  --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  --sans: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
}
* { box-sizing: border-box; }
html, body { margin: 0; height: 100%; }
.dbg-body {
  font-family: var(--sans);
  font-size: 12.5px;
  color: var(--ink);
  background: var(--bg);
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}
.dbg-toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.5rem 0.85rem;
  padding: 0.35rem 0.65rem;
  background: #333;
  border-bottom: 1px solid #000;
  flex-shrink: 0;
}
.dbg-brand { display: flex; align-items: baseline; gap: 0.35rem; }
.dbg-mark { font-weight: 700; letter-spacing: -0.02em; color: #fff; font-size: 0.9rem; }
.dbg-mode {
  font-size: 0.65rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--ok);
  font-weight: 600;
}
.dbg-actions { display: flex; gap: 0.3rem; }
.dbg-btn {
  font: 0.72rem/1 var(--sans);
  padding: 0.32rem 0.6rem;
  border: 1px solid #555;
  border-radius: 3px;
  background: #3c3c3c;
  color: #fff;
  cursor: pointer;
}
.dbg-btn:hover:not(:disabled) { background: #505050; }
.dbg-btn:disabled { opacity: 0.4; cursor: default; }
.dbg-btn.primary { background: var(--accent); border-color: var(--accent); }
.dbg-btn.primary:hover:not(:disabled) { background: var(--accent-hover); }
.dbg-btn.danger { background: #5a1d1d; border-color: #7a2d2d; }
.dbg-status { color: var(--muted); font-family: var(--mono); font-size: 0.7rem; flex: 1; min-width: 8rem; }
.dbg-work {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}
.dbg-shell {
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
}
.dbg-nav, .dbg-side {
  flex-shrink: 0;
  background: var(--panel);
  overflow: auto;
  padding: 0.35rem 0.3rem 0.6rem;
}
.dbg-nav { width: var(--nav-w); border-right: none; }
.dbg-side { width: var(--side-w); }
.dbg-split-v {
  flex: 0 0 4px;
  background: var(--line);
  cursor: col-resize;
  position: relative;
  z-index: 2;
}
.dbg-split-v:hover, .dbg-split-v.dragging { background: var(--accent); }
.dbg-split-h {
  flex: 0 0 5px;
  background: var(--line);
  cursor: row-resize;
  position: relative;
  z-index: 2;
}
.dbg-split-h:hover, .dbg-split-h.dragging { background: var(--accent); }
.dbg-nav h2, .dbg-side h2, .dbg-structure-pane h2, .dbg-console h2 {
  margin: 0.2rem 0.35rem 0.3rem;
  font-size: 0.6rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--muted);
  font-weight: 600;
}
.dbg-nav ul { list-style: none; margin: 0; padding: 0; }
.dbg-nav a.file {
  display: block;
  padding: 0.22rem 0.4rem;
  border-radius: 3px;
  color: var(--ink);
  text-decoration: none;
  font-family: var(--mono);
  font-size: 0.68rem;
  line-height: 1.25;
}
.dbg-nav a.file:hover { background: #2a2d2e; }
.dbg-nav a.file.active { background: #094771; color: #fff; }
.dbg-nav .folder { margin: 0.2rem 0; }
.dbg-nav .folder > summary {
  color: var(--muted);
  cursor: pointer;
  font-size: 0.68rem;
  padding: 0.15rem 0.35rem;
  line-height: 1.25;
}
.dbg-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
  background: var(--bg);
}
.dbg-main-head {
  padding: 0.4rem 0.75rem 0.3rem;
  border-bottom: 1px solid var(--line);
  flex-shrink: 0;
}
.dbg-main-head h1 { margin: 0; font-size: 0.95rem; font-weight: 600; color: #fff; line-height: 1.2; }
.dbg-main-head .meta {
  margin: 0.1rem 0 0;
  color: var(--muted);
  font-family: var(--mono);
  font-size: 0.65rem;
  line-height: 1.2;
}
.dbg-stdin-label {
  display: block;
  margin-top: 0.3rem;
  color: var(--muted);
  font-size: 0.65rem;
}
.dbg-stdin-label textarea {
  display: block;
  width: 100%;
  margin-top: 0.15rem;
  background: #1e1e1e;
  border: 1px solid var(--line);
  color: var(--ink);
  font-family: var(--mono);
  font-size: 0.7rem;
  border-radius: 3px;
  padding: 0.25rem 0.35rem;
  line-height: 1.3;
  max-height: 4rem;
}
.dbg-center {
  flex: 1;
  display: flex;
  min-height: 0;
  overflow: hidden;
}
.dbg-structure-pane {
  flex: 1;
  overflow: auto;
  padding: 0.35rem 0.55rem 0.6rem;
  min-width: 0;
}
.dbg-outline {
  flex: 0 0 9.5rem;
  border-left: 1px solid var(--line);
  overflow: auto;
  padding: 0.35rem;
  background: var(--panel);
}
.outline-panel { background: transparent; border: none; padding: 0; box-shadow: none; }
.fn-search {
  width: 100%;
  margin-bottom: 0.3rem;
  padding: 0.25rem 0.35rem;
  background: #1e1e1e;
  border: 1px solid var(--line);
  color: var(--ink);
  border-radius: 3px;
  font-size: 0.68rem;
}
.outline-tree, .outline-tree ul { list-style: none; margin: 0; padding: 0; }
.outline-tree ul { padding-left: 0.5rem; border-left: 1px solid var(--line); }
.outline-item { margin: 0.05rem 0; font-size: 0.68rem; line-height: 1.25; }
.outline-item.hidden { display: none; }
.outline-item a { color: #9cdcfe; text-decoration: none; font-family: var(--mono); }
.outline-item .ol-meta { color: var(--muted); margin-left: 0.15rem; font-size: 0.6rem; }

/* Compact structure — dense like an editor, not a docs card list */
.structure.debug-surface {
  background: #1e1e1e;
  border: 1px solid var(--line);
  border-radius: 3px;
  padding: 0;
  line-height: 1.25;
}
.structure.debug-surface .card {
  border: none;
  border-bottom: 1px solid #2d2d2d;
  border-radius: 0;
  padding: 0.18rem 0.45rem;
  margin: 0;
  background: transparent;
  color: var(--ink);
  line-height: 1.25;
}
.structure.debug-surface .fun-card,
.structure.debug-surface .call-card,
.structure.debug-surface .branch-card,
.structure.debug-surface .loop-card,
.structure.debug-surface .ret-card {
  border-left: 2px solid #555;
  padding-left: 0.4rem;
}
.structure.debug-surface .badge {
  background: transparent;
  border: none;
  color: #808080;
  font-size: 0.58rem;
  padding: 0 0.2rem 0 0;
  margin-right: 0.25rem;
}
.structure.debug-surface .chip {
  background: #2d2d2d;
  color: #9cdcfe;
  font-size: 0.62rem;
  padding: 0.05rem 0.28rem;
  margin: 0.05rem 0.1rem 0 0;
  border-radius: 3px;
}
.structure.debug-surface .params { margin: 0.1rem 0 0; }
.structure.debug-surface code.expr {
  color: #dcdcaa;
  font-family: var(--mono);
  font-size: 0.72rem;
  line-height: 1.25;
}
.structure.debug-surface strong { font-size: 0.78rem; font-weight: 600; color: #fff; }
.structure.debug-surface .comment-card {
  border-left: 2px solid #3a4a30;
  background: #1a1f18;
  padding: 0.1rem 0.4rem;
}
.structure.debug-surface .comment-card .badge { display: none; }
.structure.debug-surface .comment-text {
  color: #6a9955;
  font-size: 0.68rem;
  line-height: 1.3;
  max-height: 2.6em;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}
.structure.debug-surface .comment-text p {
  margin: 0;
  display: inline;
}
.structure.debug-surface .comment-text p + p::before { content: " · "; color: #4a6a40; }
.structure.debug-surface .nested {
  margin: 0.08rem 0 0.08rem 0.15rem;
  padding-left: 0.4rem;
  border-left: 1px solid #333;
}
.structure.debug-surface .arm {
  margin: 0.12rem 0;
  padding: 0.15rem 0.35rem;
  background: #252525;
  border-radius: 2px;
}
/* Foreign code cards (Run → /api/foreign-run) */
.structure.debug-surface .code-card {
  padding: 0 !important;
  overflow: hidden;
}
.structure.debug-surface .code-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.35rem 0.5rem;
  padding: 0.3rem 0.45rem;
  border-bottom: 1px solid #2d2d2d;
  background: #252526;
}
.structure.debug-surface .code-lang {
  font: 0.62rem/1 var(--mono);
  color: var(--muted);
  text-transform: lowercase;
}
.structure.debug-surface .code-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 0.3rem;
}
.structure.debug-surface .code-cmd {
  min-width: 6rem;
  max-width: 12rem;
  padding: 0.2rem 0.35rem;
  border: 1px solid var(--line);
  border-radius: 3px;
  font: 0.68rem/1.2 var(--mono);
  color: var(--ink);
  background: #1e1e1e;
}
.structure.debug-surface .code-run {
  padding: 0.2rem 0.55rem;
  border: none;
  border-radius: 3px;
  background: var(--accent);
  color: #fff;
  font: 0.68rem/1.2 var(--sans);
  font-weight: 600;
  cursor: pointer;
}
.structure.debug-surface .code-run:disabled { opacity: 0.45; cursor: not-allowed; }
.structure.debug-surface .code-body {
  margin: 0;
  padding: 0.4rem 0.55rem;
  overflow: auto;
  max-height: 12rem;
  background: #1e1e1e;
  color: #d4d4d4;
  font: 0.7rem/1.35 var(--mono);
  white-space: pre;
}
.structure.debug-surface .code-body code {
  font: inherit;
  background: none;
  padding: 0;
  color: inherit;
}
.structure.debug-surface .code-out {
  margin: 0;
  padding: 0.4rem 0.55rem;
  border-top: 1px solid #2d2d2d;
  background: #1a1a1a;
  font: 0.7rem/1.35 var(--mono);
  white-space: pre-wrap;
  word-break: break-word;
  color: #b5cea8;
}
.structure.debug-surface .code-out.err { color: #f48771; background: #2a1515; }
.structure.debug-surface .code-out[hidden] { display: none !important; }
.stmt {
  display: grid;
  grid-template-columns: 1rem 1fr;
  align-items: stretch;
}
.stmt .bp-gutter {
  border: 0;
  background: transparent;
  cursor: pointer;
  padding: 0;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 0.28rem;
}
.stmt .bp-gutter:hover { background: #3a1d1d; }
.stmt.bp .bp-gutter::after {
  content: "";
  width: 0.42rem;
  height: 0.42rem;
  border-radius: 50%;
  background: var(--bp);
  display: block;
}
.stmt.current {
  background: var(--current);
  box-shadow: inset 2px 0 0 #75beff;
}
.stmt-body > .card { border-bottom: 1px solid #2d2d2d; }
.dbg-panel {
  margin-bottom: 0.45rem;
  padding-bottom: 0.35rem;
  border-bottom: 1px solid var(--line);
}
.dbg-vars, .dbg-frame {
  font-family: var(--mono);
  font-size: 0.7rem;
  padding: 0.15rem 0.4rem;
  white-space: pre-wrap;
  color: #b5cea8;
  min-height: 1.5rem;
  line-height: 1.3;
}
.dbg-bps { list-style: none; margin: 0; padding: 0 0.35rem; }
.dbg-bps li {
  font-family: var(--mono);
  font-size: 0.68rem;
  padding: 0.12rem 0;
  color: #f48771;
  cursor: pointer;
  line-height: 1.25;
}
.dbg-bps li.muted { color: var(--muted); cursor: default; }
.dbg-console {
  flex-shrink: 0;
  height: var(--console-h);
  border-top: none;
  background: var(--panel);
  display: flex;
  flex-direction: column;
  min-height: 48px;
  max-height: 55vh;
  overflow: hidden;
}
.dbg-out {
  margin: 0;
  padding: 0.3rem 0.55rem 0.45rem;
  overflow: auto;
  font-family: var(--mono);
  font-size: 0.72rem;
  color: var(--ink);
  white-space: pre-wrap;
  flex: 1;
  line-height: 1.35;
}
.hint { color: var(--muted); font-weight: 400; text-transform: none; letter-spacing: 0; font-size: 0.62rem; }
.err { color: #f48771; padding: 0.5rem; }
body.resizing-col { cursor: col-resize; user-select: none; }
body.resizing-row { cursor: row-resize; user-select: none; }
@media (max-width: 900px) {
  .dbg-shell { flex-wrap: wrap; }
  .dbg-nav, .dbg-side { width: 100% !important; max-height: 120px; }
  .dbg-split-v { display: none; }
  .dbg-outline { display: none; }
}
"#
}

fn debug_js() -> &'static str {
    r#"
(function () {
  function initResizers() {
    var root = document.documentElement;
    var KEY = "marqdo.debug.layout";
    try {
      var saved = JSON.parse(localStorage.getItem(KEY) || "{}");
      if (saved.nav) root.style.setProperty("--nav-w", saved.nav + "px");
      if (saved.side) root.style.setProperty("--side-w", saved.side + "px");
      if (saved.console) root.style.setProperty("--console-h", saved.console + "px");
    } catch (e) {}
    function persist() {
      var nav = parseInt(getComputedStyle(root).getPropertyValue("--nav-w"), 10) || 168;
      var side = parseInt(getComputedStyle(root).getPropertyValue("--side-w"), 10) || 220;
      var cons = parseInt(getComputedStyle(root).getPropertyValue("--console-h"), 10) || 120;
      try { localStorage.setItem(KEY, JSON.stringify({ nav: nav, side: side, console: cons })); } catch (e) {}
    }
    function bindCol(splitId, prop, min, max, fromRight) {
      var el = document.getElementById(splitId);
      if (!el) return;
      el.addEventListener("mousedown", function (ev) {
        ev.preventDefault();
        el.classList.add("dragging");
        document.body.classList.add("resizing-col");
        function move(e) {
          var x = e.clientX;
          var w;
          if (fromRight) {
            w = Math.min(max, Math.max(min, window.innerWidth - x));
          } else {
            w = Math.min(max, Math.max(min, x));
          }
          root.style.setProperty(prop, w + "px");
        }
        function up() {
          el.classList.remove("dragging");
          document.body.classList.remove("resizing-col");
          document.removeEventListener("mousemove", move);
          document.removeEventListener("mouseup", up);
          persist();
        }
        document.addEventListener("mousemove", move);
        document.addEventListener("mouseup", up);
      });
    }
    function bindRow(splitId, prop, min, max) {
      var el = document.getElementById(splitId);
      if (!el) return;
      el.addEventListener("mousedown", function (ev) {
        ev.preventDefault();
        el.classList.add("dragging");
        document.body.classList.add("resizing-row");
        function move(e) {
          var h = Math.min(max, Math.max(min, window.innerHeight - e.clientY));
          root.style.setProperty(prop, h + "px");
        }
        function up() {
          el.classList.remove("dragging");
          document.body.classList.remove("resizing-row");
          document.removeEventListener("mousemove", move);
          document.removeEventListener("mouseup", up);
          persist();
        }
        document.addEventListener("mousemove", move);
        document.addEventListener("mouseup", up);
      });
    }
    bindCol("split-nav", "--nav-w", 120, 360, false);
    bindCol("split-side", "--side-w", 140, 420, true);
    bindRow("split-console", "--console-h", 48, Math.floor(window.innerHeight * 0.55));
  }
  initResizers();

  var structure = document.getElementById("structure");
  var startBtn = document.getElementById("dbg-start");
  var contBtn = document.getElementById("dbg-continue");
  var stepBtn = document.getElementById("dbg-step");
  var stopBtn = document.getElementById("dbg-stop");
  var statusEl = document.getElementById("dbg-status");
  var localsEl = document.getElementById("dbg-locals");
  var stdoutEl = document.getElementById("dbg-stdout");
  var frameEl = document.getElementById("dbg-frame");
  var bpsEl = document.getElementById("dbg-bps");
  if (!structure || !startBtn) return;

  var bps = new Set();
  var session = null;
  var path = window.MARQDO_FILE_PATH || "";

  function bpList() {
    return Array.from(bps).sort(function (a, b) { return a - b; });
  }
  function setStatus(t) { if (statusEl) statusEl.textContent = t; }
  function setPausedUi(paused) {
    startBtn.disabled = !!session;
    contBtn.disabled = !paused;
    stepBtn.disabled = !paused;
    stopBtn.disabled = !session;
  }
  function clearCurrent() {
    structure.querySelectorAll(".stmt.current").forEach(function (el) {
      el.classList.remove("current");
    });
  }
  function renderBps() {
    if (!bpsEl) return;
    var list = bpList();
    if (!list.length) {
      bpsEl.innerHTML = '<li class="muted">none</li>';
      return;
    }
    var html = "";
    list.forEach(function (n) {
      html += '<li data-line="' + n + '">line ' + n + '</li>';
    });
    bpsEl.innerHTML = html;
    bpsEl.querySelectorAll("li[data-line]").forEach(function (li) {
      li.addEventListener("click", function () {
        var line = li.getAttribute("data-line");
        var row = structure.querySelector('.stmt[data-line="' + line + '"]');
        if (row) row.scrollIntoView({ block: "nearest", behavior: "smooth" });
      });
    });
  }
  function showLocals(obj) {
    if (!localsEl) return;
    if (!obj || typeof obj !== "object") {
      localsEl.textContent = "(none)";
      return;
    }
    var keys = Object.keys(obj).sort();
    if (!keys.length) {
      localsEl.textContent = "(none)";
      return;
    }
    localsEl.textContent = keys.map(function (k) {
      return k + " = " + obj[k];
    }).join("\n");
  }
  function applySnap(data) {
    if (!data) return;
    if (data.session) session = data.session;
    clearCurrent();
    if (data.status === "paused") {
      setStatus("Paused at line " + data.line + " · " + (data.fun || ""));
      var row = structure.querySelector('.stmt[data-line="' + data.line + '"]');
      if (row) {
        row.classList.add("current");
        row.scrollIntoView({ block: "nearest", behavior: "smooth" });
      }
      showLocals(data.locals);
      if (frameEl) frameEl.textContent = data.fun || "(unknown)";
      if (stdoutEl) stdoutEl.textContent = data.stdout === "" ? "(empty)" : data.stdout;
      setPausedUi(true);
    } else if (data.status === "done") {
      setStatus(data.ok === false ? "Terminated with error" : "Finished");
      showLocals(null);
      if (frameEl) frameEl.textContent = "(done)";
      if (stdoutEl) {
        var t = data.stdout || "";
        if (data.stderr) t = (t ? t + "\n" : "") + data.stderr;
        stdoutEl.textContent = t || "(empty)";
      }
      session = null;
      setPausedUi(false);
      startBtn.disabled = false;
      stopBtn.disabled = true;
    } else if (data.status === "running") {
      setStatus("Running…");
      setPausedUi(false);
      stopBtn.disabled = !session;
    } else {
      setStatus(data.status || "Idle");
      setPausedUi(false);
    }
    if (data.ok === false && data.error) setStatus(data.error);
  }
  function post(url, body) {
    return fetch(url, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body || {})
    }).then(function (r) { return r.json(); });
  }
  function stdinText() {
    var ta = document.getElementById("stdin");
    return ta ? ta.value : "";
  }

  structure.querySelectorAll(".stmt > .bp-gutter").forEach(function (btn) {
    btn.addEventListener("click", function (ev) {
      ev.preventDefault();
      ev.stopPropagation();
      var row = btn.closest(".stmt");
      if (!row) return;
      var line = parseInt(row.getAttribute("data-line"), 10);
      if (bps.has(line)) {
        bps.delete(line);
        row.classList.remove("bp");
      } else {
        bps.add(line);
        row.classList.add("bp");
      }
      renderBps();
      if (session) {
        post("/api/debug/breakpoints", {
          session: session,
          breakpoints: bpList()
        }).catch(function () {});
      }
    });
  });

  startBtn.addEventListener("click", function () {
    startBtn.disabled = true;
    setStatus("Starting…");
    clearCurrent();
    post("/api/debug/start", {
      path: path,
      breakpoints: bpList(),
      stdin: stdinText()
    }).then(applySnap).catch(function (e) {
      setStatus(String(e));
      startBtn.disabled = false;
      session = null;
    });
  });
  contBtn.addEventListener("click", function () {
    if (!session) return;
    setStatus("Continue…");
    setPausedUi(false);
    post("/api/debug/continue", {
      session: session,
      breakpoints: bpList()
    }).then(applySnap).catch(function (e) { setStatus(String(e)); });
  });
  stepBtn.addEventListener("click", function () {
    if (!session) return;
    setStatus("Step…");
    setPausedUi(false);
    post("/api/debug/step", {
      session: session,
      breakpoints: bpList()
    }).then(applySnap).catch(function (e) { setStatus(String(e)); });
  });
  stopBtn.addEventListener("click", function () {
    if (!session) return;
    post("/api/debug/stop", { session: session }).then(function () {
      session = null;
      clearCurrent();
      setStatus("Stopped");
      setPausedUi(false);
      startBtn.disabled = false;
      stopBtn.disabled = true;
    }).catch(function (e) { setStatus(String(e)); });
  });

  document.addEventListener("keydown", function (ev) {
    if (ev.key === "F5" && !contBtn.disabled) {
      ev.preventDefault();
      contBtn.click();
    } else if (ev.key === "F10" && !stepBtn.disabled) {
      ev.preventDefault();
      stepBtn.click();
    }
  });

  var input = document.getElementById("fn-search");
  var tree = document.querySelector(".outline-tree");
  if (input && tree) {
    input.addEventListener("input", function () {
      var q = (input.value || "").trim().toLowerCase();
      tree.querySelectorAll(".outline-item").forEach(function (li) {
        if (!q) { li.classList.remove("hidden"); return; }
        var name = (li.getAttribute("data-fn") || "").toLowerCase();
        var fpath = (li.getAttribute("data-fn-path") || "").toLowerCase();
        var selfMatch = name.indexOf(q) !== -1 || fpath.indexOf(q) !== -1;
        var childMatch = false;
        li.querySelectorAll(".outline-item").forEach(function (c) {
          var cn = (c.getAttribute("data-fn") || "").toLowerCase();
          var cp = (c.getAttribute("data-fn-path") || "").toLowerCase();
          if (cn.indexOf(q) !== -1 || cp.indexOf(q) !== -1) childMatch = true;
        });
        li.classList.toggle("hidden", !(selfMatch || childMatch));
      });
    });
  }

  renderBps();
  setPausedUi(false);

  // Foreign code cards (same API as view)
  document.querySelectorAll(".code-card").forEach(function (card) {
    var btn = card.querySelector(".code-run");
    var cmd = card.querySelector(".code-cmd");
    var out = card.querySelector(".code-out");
    var body = card.querySelector(".code-body code");
    if (!btn || !out || !body) return;
    btn.addEventListener("click", function () {
      btn.disabled = true;
      out.hidden = false;
      out.classList.remove("err");
      out.textContent = "Running…";
      fetch("/api/foreign-run", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          lang: card.getAttribute("data-lang") || "python",
          source: body.textContent,
          cmd: cmd ? cmd.value : ""
        })
      })
        .then(function (r) { return r.json(); })
        .then(function (data) {
          if (data.ok) {
            out.classList.remove("err");
            out.textContent = data.stdout === "" ? "(no stdout)" : data.stdout;
          } else {
            out.classList.add("err");
            out.textContent = data.error || "foreign run failed";
          }
        })
        .catch(function (e) {
          out.classList.add("err");
          out.textContent = String(e);
        })
        .finally(function () { btn.disabled = false; });
    });
  });
})();
"#
}
