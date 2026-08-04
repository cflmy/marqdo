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
  max-width: 1400px;
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
.folder {
  color: var(--muted);
  font-size: 0.72rem;
  margin: 1rem 0.35rem 0.35rem;
  font-weight: 600;
  letter-spacing: 0.01em;
}
.panel-wrap {
  min-width: 0;
  overflow: auto;
  background: var(--bg);
}
main.panel {
  padding: 2.25rem 2.5rem 4rem;
  margin: 0 auto;
  width: 100%;
  max-width: 58rem;
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
  color: var(--text);
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
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--surface);
  color: var(--text);
  resize: vertical;
  min-height: 4.5rem;
}
.stdin-form button {
  justify-self: start;
  font: inherit;
  font-size: 0.9rem;
  padding: 0.45rem 0.9rem;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--text);
  color: var(--bg);
  cursor: pointer;
}
.stdin-form button:hover { opacity: 0.88; }
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
.out.ok { background: var(--ok-bg); }
.out.fail {
  background: var(--fail-bg);
  border-color: #e4c1c1;
  color: var(--ink);
  font-weight: 500;
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
  main.panel { max-width: 48rem; padding: 1.75rem 1.5rem 3rem; }
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
    let mut out = String::from("<h2>Files</h2><ul>");
    let mut last_folder = String::new();
    for f in files {
        let rel = f.to_string_lossy().replace('\\', "/");
        let folder = Path::new(&rel)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        if folder != last_folder {
            if !folder.is_empty() {
                out.push_str(&format!(
                    "</ul><div class=\"folder\">{}</div><ul>",
                    escape(&folder)
                ));
            }
            last_folder = folder;
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
    out.push_str("</ul>");
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

pub fn page_file(files: &[PathBuf], rel: &str, vm: &FileViewModel, links: &LinkMode) -> String {
    let nav = nav_html(files, Some(rel), links);
    let status = if vm.ok { "ok" } else { "fail" };
    let status_label = if vm.ok { "ok" } else { "fail" };
    let out_text = if vm.ok {
        if vm.stdout.is_empty() {
            "(no stdout)".into()
        } else {
            escape(&vm.stdout)
        }
    } else if vm.stderr.is_empty() {
        "(skipped)".into()
    } else {
        escape(&vm.stderr)
    };
    let stdin_panel = match links {
        LinkMode::Live => format!(
            r#"<form class="stdin-form" method="get" action="/file">
  <input type="hidden" name="path" value="{path}"/>
  <label for="stdin">Preset input <span class="hint">one line per input call</span></label>
  <textarea id="stdin" name="stdin" rows="3" placeholder="Alice">{stdin}</textarea>
  <button type="submit">Run with input</button>
</form>"#,
            path = escape(rel),
            stdin = escape(&vm.preset_stdin),
        ),
        LinkMode::Static { .. } => String::new(),
    };
    let title = Path::new(rel)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(rel)
        .trim_end_matches(".mq.md");
    let main = format!(
        r#"
<h1 class="page">{title}<span class="status-pill {status}">{status_label}</span></h1>
<p class="meta">{rel}</p>
<section class="block">
  <h2>Structure</h2>
  <div class="structure">{structure}</div>
</section>
<section class="block">
  <h2>Execution</h2>
  {stdin_panel}
  <div class="out {status}">{out}</div>
</section>
<section class="block">
  <h2>Source</h2>
  <pre class="source">{source}</pre>
</section>
"#,
        title = escape(title),
        status = status,
        status_label = status_label,
        rel = escape(rel),
        structure = vm.structure_html,
        stdin_panel = stdin_panel,
        out = out_text,
        source = escape(&vm.source),
    );
    layout(rel, &nav, &main)
}
