//! HTML shell for `marqdo view`.

use std::path::Path;

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

pub fn layout(title: &str, nav: &str, main: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{title} · marqdo view</title>
<link rel="preconnect" href="https://fonts.googleapis.com"/>
<link href="https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,600;9..144,700&family=IBM+Plex+Mono:wght@400;500&family=Sora:wght@400;600&display=swap" rel="stylesheet"/>
<style>
:root {{
  --bg0: #0f1c1a;
  --bg1: #162824;
  --panel: #1c322d;
  --ink: #e7f2ee;
  --muted: #9bb5ac;
  --accent: #3ecf8e;
  --accent-dim: #2a9b6a;
  --call: #5ec8ff;
  --branch: #f0b429;
  --loop: #ff7a59;
  --ret: #c4a7ff;
  --err: #ff6b6b;
  --line: rgba(255,255,255,0.08);
}}
* {{ box-sizing: border-box; }}
body {{
  margin: 0;
  min-height: 100vh;
  font-family: "Sora", system-ui, sans-serif;
  color: var(--ink);
  background:
    radial-gradient(1200px 600px at 10% -10%, #1a3d34 0%, transparent 55%),
    radial-gradient(900px 500px at 100% 0%, #243528 0%, transparent 50%),
    linear-gradient(165deg, var(--bg0), var(--bg1));
}}
a {{ color: var(--accent); text-decoration: none; }}
a:hover {{ text-decoration: underline; }}
.shell {{
  display: grid;
  grid-template-columns: minmax(220px, 280px) 1fr;
  min-height: 100vh;
}}
@media (max-width: 800px) {{
  .shell {{ grid-template-columns: 1fr; }}
}}
.nav {{
  border-right: 1px solid var(--line);
  background: rgba(0,0,0,0.22);
  padding: 1.25rem 1rem 2rem;
  overflow: auto;
}}
.brand {{
  font-family: "Fraunces", Georgia, serif;
  font-size: 1.45rem;
  font-weight: 700;
  letter-spacing: -0.02em;
  margin: 0 0 0.25rem;
}}
.brand span {{ color: var(--accent); }}
.tagline {{
  color: var(--muted);
  font-size: 0.8rem;
  margin: 0 0 1.25rem;
}}
.nav h2 {{
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  color: var(--muted);
  margin: 1rem 0 0.4rem;
}}
.nav ul {{ list-style: none; padding: 0; margin: 0; }}
.nav li {{ margin: 0.15rem 0; }}
.nav a.file {{
  display: block;
  padding: 0.35rem 0.5rem;
  border-radius: 6px;
  color: var(--ink);
  font-family: "IBM Plex Mono", monospace;
  font-size: 0.78rem;
}}
.nav a.file:hover, .nav a.file.active {{
  background: rgba(62, 207, 142, 0.12);
  text-decoration: none;
}}
.folder {{ color: var(--muted); font-size: 0.75rem; margin-top: 0.75rem; }}
main.panel {{
  padding: 1.5rem 1.75rem 3rem;
  overflow: auto;
}}
h1.page {{
  font-family: "Fraunces", Georgia, serif;
  font-size: 1.75rem;
  margin: 0 0 0.5rem;
}}
.meta {{ color: var(--muted); font-size: 0.85rem; margin-bottom: 1.5rem; }}
section.block {{
  margin: 1.25rem 0 1.75rem;
}}
section.block > h2 {{
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--muted);
  margin: 0 0 0.65rem;
  border-bottom: 1px solid var(--line);
  padding-bottom: 0.35rem;
}}
.card {{
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: 10px;
  padding: 0.85rem 1rem;
  margin: 0.55rem 0;
}}
.fun-card {{ border-left: 3px solid var(--accent); }}
.call-card {{ border-left: 3px solid var(--call); }}
.branch-card {{ border-left: 3px solid var(--branch); }}
.loop-card {{ border-left: 3px solid var(--loop); }}
.ret-card {{ border-left: 3px solid var(--ret); }}
.badge {{
  display: inline-block;
  font-family: "IBM Plex Mono", monospace;
  font-size: 0.7rem;
  padding: 0.1rem 0.4rem;
  border-radius: 4px;
  background: rgba(255,255,255,0.06);
  margin-right: 0.35rem;
}}
.params {{ margin: 0.35rem 0; }}
.chip {{
  display: inline-block;
  font-family: "IBM Plex Mono", monospace;
  font-size: 0.75rem;
  padding: 0.15rem 0.45rem;
  margin: 0.15rem;
  border-radius: 999px;
  background: rgba(62, 207, 142, 0.15);
  color: var(--accent);
}}
.nested {{ margin-left: 0.85rem; padding-left: 0.75rem; border-left: 1px dashed var(--line); }}
.arm {{
  margin: 0.4rem 0;
  padding: 0.5rem 0.65rem;
  background: rgba(0,0,0,0.2);
  border-radius: 8px;
}}
.out {{
  font-family: "IBM Plex Mono", monospace;
  font-size: 0.85rem;
  white-space: pre-wrap;
  background: #0a1210;
  border-radius: 8px;
  padding: 0.85rem 1rem;
  border: 1px solid var(--line);
}}
.out.ok {{ box-shadow: inset 3px 0 0 var(--accent-dim); }}
.out.fail {{ box-shadow: inset 3px 0 0 var(--err); color: #ffc9c9; }}
.source {{
  font-family: "IBM Plex Mono", monospace;
  font-size: 0.78rem;
  white-space: pre-wrap;
  background: #0a1210;
  border-radius: 8px;
  padding: 0.85rem 1rem;
  border: 1px solid var(--line);
  line-height: 1.45;
}}
.err {{ color: var(--err); }}
.comment-card {{
  border-left: 3px solid var(--muted);
  background: rgba(255,255,255,0.03);
}}
.comment-text {{
  color: var(--muted);
  font-size: 0.9rem;
  line-height: 1.45;
}}
code.expr {{
  font-family: "IBM Plex Mono", monospace;
  font-size: 0.82rem;
  color: var(--ink);
}}
.welcome {{ max-width: 36rem; }}
.welcome p {{ color: var(--muted); line-height: 1.55; }}
</style>
</head>
<body>
<div class="shell">
<aside class="nav">
  <p class="brand">marq<span>do</span> view</p>
  <p class="tagline">Markup structure · live output</p>
  {nav}
</aside>
<main class="panel">
  {main}
</main>
</div>
</body>
</html>
"##,
        title = escape(title),
        nav = nav,
        main = main
    )
}

pub fn nav_html(files: &[std::path::PathBuf], active: Option<&str>) -> String {
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
        let href = format!("/file?path={}", urlencoding_encode(&rel));
        out.push_str(&format!(
            "<li><a class=\"file{active_cls}\" href=\"{href}\">{}</a></li>",
            escape(name)
        ));
    }
    out.push_str("</ul>");
    out
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

pub fn page_index(files: &[std::path::PathBuf], only: Option<&Path>) -> String {
    let nav = nav_html(files, None);
    let main = if let Some(p) = only {
        let rel = p.to_string_lossy().replace('\\', "/");
        format!(
            "<div class=\"welcome\"><h1 class=\"page\">Single file</h1><p class=\"meta\">Open <a href=\"/file?path={}\">{}</a></p></div>",
            urlencoding_encode(&rel),
            escape(&rel)
        )
    } else {
        format!(
            "<div class=\"welcome\"><h1 class=\"page\">Browse</h1><p>Select a <code>.mq.md</code> file in the index. Structure is rendered from the AST; output comes from the same pipeline as <code>marqdo run</code>.</p><p class=\"meta\">{} file(s)</p></div>",
            files.len()
        )
    };
    layout("index", &nav, &main)
}

pub fn page_file(files: &[std::path::PathBuf], rel: &str, vm: &FileViewModel) -> String {
    let nav = nav_html(files, Some(rel));
    let status = if vm.ok { "ok" } else { "fail" };
    let out_text = if vm.ok {
        if vm.stdout.is_empty() {
            "(no stdout)".to_string()
        } else {
            escape(&vm.stdout)
        }
    } else {
        escape(&vm.stderr)
    };
    let main = format!(
        r#"
<h1 class="page">{title}</h1>
<p class="meta">{rel}</p>
<section class="block">
  <h2>Structure</h2>
  {structure}
</section>
<section class="block">
  <h2>Execution</h2>
  <div class="out {status}">{out}</div>
</section>
<section class="block">
  <h2>Source</h2>
  <pre class="source">{source}</pre>
</section>
"#,
        title = escape(
            Path::new(rel)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(rel)
        ),
        rel = escape(rel),
        structure = vm.structure_html,
        status = status,
        out = out_text,
        source = escape(&vm.source),
    );
    layout(rel, &nav, &main)
}
