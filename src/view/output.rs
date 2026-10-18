//! Static HTML export: `marqdo view output`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::host::foreign_doc::DocumentSession;
use crate::value::Value;
use crate::view::html::{escape, page_file, page_index, LinkMode};
use crate::view::{build_file_view, build_root, ViewRoot};

pub struct OutputOptions {
    pub path: PathBuf,
    pub out_dir: PathBuf,
    pub no_exec: bool,
}

#[derive(Debug, Clone)]
struct StaticCellOutput {
    ok: bool,
    text: String,
}

fn value_text(v: &Value) -> Option<String> {
    match v {
        Value::Text(s) => Some(s.clone()),
        Value::None => None,
        _ => v.reveal_text().map(|s| s.to_string()),
    }
}

fn value_bool(v: &Value) -> Option<bool> {
    match v {
        Value::Bool(b) => Some(*b),
        _ => None,
    }
}

fn record_field(record: &Value, key: &str) -> Option<Value> {
    match record {
        Value::Map(entries) => entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone()),
        _ => None,
    }
}

/// Run the document's cells at export time and keep their real outputs. This
/// is what makes a static page show foreign results instead of just source.
fn static_cell_outputs(
    root: &ViewRoot,
    abs: &std::path::Path,
    stdin: &str,
) -> HashMap<String, StaticCellOutput> {
    let mut out = HashMap::new();
    let Ok(mut session) = DocumentSession::open(abs, &root.root) else {
        return out;
    };
    let Ok(summary) = session.run_all_with_stdin(false, Some(stdin)) else {
        session.close();
        return out;
    };
    let records = match &summary {
        Value::Map(entries) => entries
            .iter()
            .find(|(k, _)| k == "records")
            .map(|(_, v)| v.clone()),
        _ => None,
    };
    if let Some(Value::List(records)) = records {
        for rec in records {
            let Some(cell) = record_field(&rec, "cell").and_then(|v| value_text(&v)) else {
                continue;
            };
            let ok = record_field(&rec, "ok")
                .and_then(|v| value_bool(&v))
                .unwrap_or(false);
            let mut parts = Vec::new();
            if let Some(stdout) = record_field(&rec, "stdout").and_then(|v| value_text(&v)) {
                if !stdout.is_empty() {
                    parts.push(stdout);
                }
            }
            if let Some(stderr) = record_field(&rec, "stderr").and_then(|v| value_text(&v)) {
                if !stderr.is_empty() {
                    parts.push(format!("[stderr]\n{stderr}"));
                }
            }
            if let Some(error) = record_field(&rec, "error").and_then(|v| value_text(&v)) {
                if !error.is_empty() {
                    parts.push(error);
                }
            }
            let has_body = !parts.is_empty();
            let mut meta = Vec::new();
            if let Some(code) = record_field(&rec, "code").and_then(|v| match v {
                Value::Int(n) => Some(n),
                _ => None,
            }) {
                if code != 0 {
                    meta.push(format!("exit {code}"));
                }
            }
            if let Some(ms) = record_field(&rec, "ms").and_then(|v| match v {
                Value::Int(n) => Some(n),
                _ => None,
            }) {
                if ms > 0 {
                    meta.push(format!("{ms}ms"));
                }
            }
            if !meta.is_empty() {
                parts.push(format!("— {}", meta.join(" · ")));
            }
            let text = if !has_body {
                "(no output)".to_string()
            } else {
                parts.join("\n\n")
            };
            out.insert(cell, StaticCellOutput { ok, text });
        }
    }
    session.close();
    out
}

/// Inject static cell outputs into code cards without running client JS.
fn inject_cell_outputs(html: &str, outputs: &HashMap<String, StaticCellOutput>) -> String {
    let mut html = html.to_string();
    for (name, out) in outputs {
        let marker = format!(r#"data-code-name="{name}""#);
        let Some(pos) = html.find(&marker) else {
            continue;
        };
        let tail = &html[pos..];
        let Some(out_pos) = tail.find(r#"<pre class="code-out" hidden></pre>"#) else {
            continue;
        };
        let abs_pos = pos + out_pos;
        let replacement = if out.ok {
            format!(r#"<pre class="code-out">{}</pre>"#, escape(&out.text))
        } else {
            format!(r#"<pre class="code-out err">{}</pre>"#, escape(&out.text))
        };
        html.replace_range(
            abs_pos..abs_pos + r#"<pre class="code-out" hidden></pre>"#.len(),
            &replacement,
        );
    }
    html
}

pub fn write_static(opts: OutputOptions) -> Result<()> {
    let root = build_root(&opts.path)?;
    fs::create_dir_all(&opts.out_dir)
        .with_context(|| format!("create {}", opts.out_dir.display()))?;

    let pages_root = opts.out_dir.join("pages");
    let mut first_page_html: Option<String> = None;

    for rel_pb in &root.files {
        let rel = rel_pb.to_string_lossy().replace('\\', "/");
        let abs = resolve_abs(&root, &rel)?;
        let mut vm = build_file_view(&root, &abs, &rel, &[], false)?;
        if opts.no_exec {
            vm.stdout.clear();
            vm.stderr = "execution skipped (--no-exec)".into();
            vm.ok = true;
            vm.plots.clear();
            vm.bindings_html = String::from(r#"<p class="vars-skipped">skipped</p>"#);
            vm.trace_json.clear();
            // Rebuild outline + Structure without run annotations.
            if let Ok(module) = crate::parse::parse_source(&vm.source) {
                let wb = crate::view::render::render_writeback_outline(&vm.source);
                vm.outline_html =
                    crate::view::render::render_right_rail(&module, &vm.bindings_html, &wb);
                vm.structure_html = crate::view::render::render_module_structure_with_annots(
                    &module,
                    &vm.source,
                    &[],
                );
            }
        }
        let links = LinkMode::Static {
            from: Some(rel.clone()),
        };
        let cell_outputs = if opts.no_exec {
            HashMap::new()
        } else {
            static_cell_outputs(&root, &abs, &vm.preset_stdin)
        };
        let html = inject_cell_outputs(
            &page_file(&root.files, &rel, &vm, &links),
            &cell_outputs,
        );
        let out_file = pages_root.join(format!("{rel}.html"));
        if let Some(parent) = out_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_file, html).with_context(|| format!("write {}", out_file.display()))?;

        if first_page_html.is_none() {
            // index.html = first file (same content, links rooted at OUT_DIR)
            first_page_html = Some(inject_cell_outputs(
                &page_file(
                    &root.files,
                    &rel,
                    &vm,
                    &LinkMode::Static { from: None },
                ),
                &cell_outputs,
            ));
        }
    }

    let index_html = first_page_html.unwrap_or_else(|| {
        page_index(&root.files, root.only_file.as_deref(), &LinkMode::Static { from: None })
    });
    let index_path = opts.out_dir.join("index.html");
    fs::write(&index_path, index_html)
        .with_context(|| format!("write {}", index_path.display()))?;

    eprintln!(
        "marqdo view output: {} page(s) → {}",
        root.files.len(),
        opts.out_dir.display()
    );
    Ok(())
}

fn resolve_abs(root: &ViewRoot, rel: &str) -> Result<PathBuf> {
    let abs = root.root.join(rel);
    Ok(abs.canonicalize().unwrap_or(abs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injects_visible_static_cell_outputs() {
        let html = r#"<div class="card" data-code-name="a"></div><pre class="code-out" hidden></pre>"#;
        let mut outputs = HashMap::new();
        outputs.insert("a".to_string(), StaticCellOutput { ok: true, text: "42".into() });
        let html = inject_cell_outputs(html, &outputs);
        assert!(html.contains(r#"<pre class="code-out">42</pre>"#), "{html}");
        assert!(!html.contains("hidden"), "{html}");
    }
}
