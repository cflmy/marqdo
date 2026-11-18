//! Static HTML export: `marqdo view output`.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

use crate::host::foreign_doc::DocumentSession;
use crate::host::json::json_to_value;
use crate::value::Value;
use crate::view::html::{escape, page_file, page_index, LinkMode};
use crate::view::{build_file_view, build_file_view_with_exec, build_root, ViewRoot};

pub struct OutputOptions {
    pub path: PathBuf,
    pub out_dir: PathBuf,
    pub no_exec: bool,
    pub snapshot: Option<PathBuf>,
}

pub struct CaptureOptions {
    pub path: PathBuf,
    pub out: PathBuf,
}

#[derive(Debug, Clone)]
struct StaticCellOutput {
    ok: bool,
    text: String,
}

fn source_hash(source: &str) -> String {
    let mut h = Sha256::new();
    h.update(source.as_bytes());
    format!("{:x}", h.finalize())
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
fn static_page_data(
    root: &ViewRoot,
    abs: &std::path::Path,
    stdin: &str,
) -> (HashMap<String, StaticCellOutput>, Option<Value>) {
    let mut out = HashMap::new();
    let Ok(mut session) = DocumentSession::open(abs, &root.root) else {
        return (out, None);
    };
    let page_stdin = if stdin.trim().is_empty() { None } else { Some(stdin) };
    let Ok(summary) = session.run_all_with_stdin(false, page_stdin) else {
        session.close();
        return (out, None);
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
    let view_summary = session.summary();
    session.close();
    (out, Some(view_summary))
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

fn inject_static_session(html: &str, summary: Option<&Value>) -> String {
    let Some(summary) = summary else {
        return html.to_string();
    };
    let Ok(json) = crate::host::json::value_to_json(summary) else {
        return html.to_string();
    };
    let Ok(json) = serde_json::to_string(&json) else {
        return html.to_string();
    };
    let line = format!(
        "window.MARQDO_VIEW_LIVE = false;\nwindow.MARQDO_STATIC_SESSION = {json};"
    );
    html.replace("window.MARQDO_VIEW_LIVE = false;", &line)
}

/// Capture live-cell results into a snapshot consumed by `view output --snapshot`.
pub fn write_capture(opts: CaptureOptions) -> Result<()> {
    let root = build_root(&opts.path)?;
    let mut pages = serde_json::Map::new();
    for rel_pb in &root.files {
        let rel = rel_pb.to_string_lossy().replace('\\', "/");
        let abs = resolve_abs(&root, &rel)?;
        let vm = build_file_view(&root, &abs, &rel, &[], false)?;
        let (outputs, summary) = static_page_data(&root, &abs, &vm.preset_stdin);
        let mut out_map = serde_json::Map::new();
        for (name, output) in outputs {
            out_map.insert(
                name,
                serde_json::json!({ "ok": output.ok, "text": output.text }),
            );
        }
        let summary_json = summary
            .as_ref()
            .and_then(|v| crate::host::json::value_to_json(v).ok())
            .unwrap_or(serde_json::Value::Null);
        pages.insert(
            rel.clone(),
            serde_json::json!({
                "source_hash": source_hash(&vm.source),
                "summary": summary_json,
                "outputs": out_map,
            }),
        );
    }
    let doc = serde_json::json!({ "version": 1, "pages": pages });
    if let Some(parent) = opts.out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&opts.out, serde_json::to_string_pretty(&doc)?)
        .with_context(|| format!("write {}", opts.out.display()))?;
    eprintln!("marqdo view capture: {} page(s) → {}", root.files.len(), opts.out.display());
    Ok(())
}

fn snapshot_page(
    snapshot: &serde_json::Value,
    rel: &str,
    source: &str,
) -> Result<(HashMap<String, StaticCellOutput>, Option<Value>)> {
    let page = snapshot
        .get("pages")
        .and_then(|p| p.get(rel))
        .ok_or_else(|| anyhow::anyhow!("snapshot has no page `{rel}`"))?;
    let expected = page
        .get("source_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if expected != source_hash(source) {
        anyhow::bail!("snapshot source hash mismatch for `{rel}`");
    }
    let mut outputs = HashMap::new();
    if let Some(map) = page.get("outputs").and_then(|v| v.as_object()) {
        for (name, value) in map {
            outputs.insert(
                name.clone(),
                StaticCellOutput {
                    ok: value.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
                    text: value
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                },
            );
        }
    }
    let summary = page
        .get("summary")
        .filter(|v| !v.is_null())
        .and_then(|v| json_to_value(v).ok());
    Ok((outputs, summary))
}

pub fn write_static(opts: OutputOptions) -> Result<()> {
    let snapshot = match &opts.snapshot {
        Some(path) => Some(serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?,
        )?),
        None => None,
    };
    let root = build_root(&opts.path)?;
    fs::create_dir_all(&opts.out_dir)
        .with_context(|| format!("create {}", opts.out_dir.display()))?;

    let pages_root = opts.out_dir.join("pages");
    let mut first_page_html: Option<String> = None;

    for rel_pb in &root.files {
        let rel = rel_pb.to_string_lossy().replace('\\', "/");
        let abs = resolve_abs(&root, &rel)?;
        // V0: apply --no-exec before any capture run (do not execute then clear).
        let vm = build_file_view_with_exec(&root, &abs, &rel, &[], false, !opts.no_exec)?;
        let links = LinkMode::Static {
            from: Some(rel.clone()),
        };
        let (cell_outputs, static_summary) = if let Some(snapshot) = &snapshot {
            snapshot_page(snapshot, &rel, &vm.source)?
        } else if opts.no_exec {
            (HashMap::new(), None)
        } else {
            static_page_data(&root, &abs, &vm.preset_stdin)
        };
        let html = inject_static_session(
            &inject_cell_outputs(
                &page_file(&root.files, &rel, &vm, &links),
                &cell_outputs,
            ),
            static_summary.as_ref(),
        );
        let out_file = pages_root.join(format!("{rel}.html"));
        if let Some(parent) = out_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_file, html).with_context(|| format!("write {}", out_file.display()))?;

        if first_page_html.is_none() {
            // index.html = first file (same content, links rooted at OUT_DIR)
            first_page_html = Some(inject_static_session(
                &inject_cell_outputs(
                    &page_file(
                        &root.files,
                        &rel,
                        &vm,
                        &LinkMode::Static { from: None },
                    ),
                    &cell_outputs,
                ),
                static_summary.as_ref(),
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
    fn snapshot_page_rejects_source_hash_mismatch() {
        let snap = serde_json::json!({
            "pages": {
                "a.mq.md": {
                    "source_hash": "bad",
                    "summary": null,
                    "outputs": {}
                }
            }
        });
        assert!(snapshot_page(&snap, "a.mq.md", "# main\n").is_err());
    }

    #[test]
    fn snapshot_page_reads_outputs_and_summary() {
        let source = "# main\n";
        let hash = source_hash(source);
        let snap = serde_json::json!({
            "pages": {
                "a.mq.md": {
                    "source_hash": hash,
                    "summary": {"persistent": false, "cells": [], "kernels": []},
                    "outputs": {"a": {"ok": true, "text": "42"}}
                }
            }
        });
        let (outputs, summary) = snapshot_page(&snap, "a.mq.md", source).unwrap();
        assert_eq!(outputs["a"].text, "42");
        assert!(summary.is_some());
    }

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
