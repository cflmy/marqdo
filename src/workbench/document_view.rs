//! DocumentViewV1 — read-only embedded view (M03). Never executes the document.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::view::{render_knowledge_html, render_module_document, KnowledgePresentation};

#[derive(Debug, Clone)]
pub struct DocumentViewRequest {
    pub project: PathBuf,
    pub document: String,
    pub revision: Option<String>,
    pub generation: Option<String>,
    pub node_id: Option<String>,
    pub include: Vec<String>,
}

impl DocumentViewRequest {
    pub fn from_params(params: &Value) -> Result<Self, String> {
        let project = params
            .get("project")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "view.document requires params.project".to_string())?;
        let document = params
            .get("document")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "view.document requires params.document".to_string())?;
        let mode = params
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("read");
        if mode != "read" {
            return Err(format!("view.document unsupported mode={mode}; only read"));
        }
        let include = match params.get("include") {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            _ => vec![
                "fragment".into(),
                "outline".into(),
                "nodes".into(),
                "refs".into(),
                "diagnostics".into(),
            ],
        };
        Ok(Self {
            project: PathBuf::from(project),
            document: document.to_string(),
            revision: params
                .get("revision")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            generation: params
                .get("generation")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            node_id: params
                .get("node_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            include,
        })
    }
}

/// Pure read path: compile_project (no plugin load / no eval) + fragment render.
/// Must never call `build_file_view` with execute=true.
pub fn build_document_view(req: &DocumentViewRequest) -> Result<Value, Value> {
    let project_root = canonicalize_project(&req.project)?;
    let abs = resolve_under(&project_root, &req.document)?;
    let raw = fs::read(&abs).map_err(|e| {
        err(
            "view.document_read_failed",
            format!("cannot read {}: {e}", abs.display()),
        )
    })?;
    let source_revision = format!("sha256:{:x}", Sha256::digest(&raw));
    if let Some(expected) = &req.revision {
        if expected != &source_revision && !expected.is_empty() {
            return Err(err(
                "view.revision_mismatch",
                format!("requested {expected} but file is {source_revision}; refusing silent downgrade"),
            ));
        }
    }
    let source = String::from_utf8_lossy(&raw).into_owned();
    let title = title_from_source(&source, &req.document);

    // Semantic projection — compile_project does not load plugins / evaluate.
    // M-R5: Arc<Compilation> single-flight cache; followers reuse the same result.
    let compilation = crate::workbench::analysis_cache::get_or_compile(&project_root)
        .map_err(|e| err("view.compile_failed", e))?;

    let rel = req.document.replace('\\', "/");
    let mut presentation = KnowledgePresentation::from_project(&compilation.semantics, &rel, &title);
    presentation.page_token = {
        let mut h = Sha256::new();
        h.update(raw.as_slice());
        h.update(presentation.semantic_generation.as_bytes());
        format!("{:x}", h.finalize())
    };

    if let Some(gen) = &req.generation {
        if !gen.is_empty() && gen != &presentation.semantic_generation {
            return Err(err(
                "view.generation_mismatch",
                format!(
                    "requested generation {gen} but projection is {}; refusing silent downgrade",
                    presentation.semantic_generation
                ),
            ));
        }
    }

    let wants = |k: &str| req.include.iter().any(|x| x == k);

    let heading_outline = outline_from_headings(&source, &presentation);

    // Prefer document surface HTML when parseable; fall back to knowledge HTML.
    // Never call build_file_view (that path executes).
    let fragment = if wants("fragment") {
        let raw_html = match crate::load::parse_source_auto(&source) {
            Ok(module) => render_module_document(&module, &source, &[]),
            Err(_) => render_knowledge_html(&compilation.semantics, &rel),
        };
        wrap_fragment(&inject_mq_node_attrs(&raw_html, &heading_outline, &presentation))
    } else {
        String::new()
    };

    let outline = if wants("outline") {
        if heading_outline.is_empty() {
            outline_from_presentation(&presentation)
        } else {
            json!(heading_outline)
        }
    } else {
        json!([])
    };
    let nodes = if wants("nodes") {
        nodes_from_presentation_and_steps(
            &presentation,
            &compilation.semantics,
            &rel,
            &req.node_id,
        )
    } else {
        json!([])
    };
    let refs = if wants("refs") {
        refs_from_project(&compilation.semantics, &rel)
    } else {
        json!([])
    };

    let mut diag_msgs: Vec<String> = presentation.diagnostics.clone();
    for d in &compilation.diagnostics {
        diag_msgs.push(d.message.clone());
    }
    let diagnostics = if wants("diagnostics") {
        json!(diag_msgs)
    } else {
        json!([])
    };

    let completeness = if diag_msgs.is_empty() {
        "complete"
    } else {
        "partial"
    };

    let snapshot = crate::workbench::snapshot::build_snapshot_manifest(
        &project_root,
        &rel,
        &raw,
        &compilation.semantics,
        "1",
    );

    Ok(json!({
        "schema": "DocumentViewV1",
        "schema_version": 1,
        "document": {
            "id": rel,
            "title": title,
            "source_revision": source_revision,
            "semantic_digest": presentation.source_digest,
            "generation_id": if presentation.semantic_generation.is_empty() {
                Value::Null
            } else {
                json!(presentation.semantic_generation)
            },
            "presentation_token": presentation.page_token,
        },
        "execution": {"state": "not_run", "run_id": Value::Null},
        "render": {
            "format": "sanitized_html",
            "fragment": fragment,
            "renderer_version": "1"
        },
        "outline": outline,
        "nodes": nodes,
        "refs": refs,
        "diagnostics": diagnostics,
        "completeness": completeness,
        "snapshot": snapshot,
        "capabilities": {"read_only": true, "has_semantics": true, "executed": false}
    }))
}

fn wrap_fragment(inner: &str) -> String {
    // No html/body, no scripts, namespaced for host theming.
    let sanitized = strip_script_tags(inner);
    format!("<article class=\"mq-document\">{sanitized}</article>")
}

/// Remove `<script…>` blocks (case-insensitive) so embed fragments stay non-executable.
fn strip_script_tags(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut out = String::with_capacity(html.len());
    let mut i = 0usize;
    while i < html.len() {
        if lower[i..].starts_with("<script") {
            let after_open = lower[i..]
                .find('>')
                .map(|p| i + p + 1)
                .unwrap_or(html.len());
            if let Some(rel) = lower[after_open..].find("</script") {
                let close_at = after_open + rel;
                let end = lower[close_at..]
                    .find('>')
                    .map(|p| close_at + p + 1)
                    .unwrap_or(html.len());
                i = end;
            } else {
                // Truncate from the dangerous open tag onward.
                break;
            }
            continue;
        }
        // Advance one UTF-8 char so multi-byte text stays intact.
        let ch = html[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn title_from_source(source: &str, fallback: &str) -> String {
    for line in source.lines().take(40) {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("# ") {
            return rest.trim().to_string();
        }
        if let Some(rest) = t.strip_prefix("title:") {
            return rest.trim().trim_matches('"').to_string();
        }
    }
    Path::new(fallback)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(fallback)
        .to_string()
}

/// Outline entries from Markdown / Marqdo `#` headings; prefer matching node ids.
fn outline_from_headings(source: &str, p: &KnowledgePresentation) -> Vec<Value> {
    let mut items = Vec::new();
    let mut order = 0usize;
    for (idx, line) in source.lines().enumerate() {
        let line_no = (idx + 1) as u32;
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }
        let hashes = trimmed.chars().take_while(|c| *c == '#').count();
        if hashes == 0 || hashes > 6 {
            continue;
        }
        // `# Title`, `## Section`, or Marqdo `#main` / `##greet` (no space).
        let after = &trimmed[hashes..];
        let label = after.trim();
        if label.is_empty() {
            continue;
        }
        let matched = p.nodes.iter().find(|n| {
            n.source_line == line_no
                || n.display_name == label
                || n.node_id == label
                || n.artifact_id.as_deref() == Some(label)
        });
        let id = matched
            .map(|n| n.node_id.clone())
            .unwrap_or_else(|| format!("heading:{line_no}"));
        let artifact_id = matched.and_then(|n| n.artifact_id.clone());
        items.push(json!({
            "id": id,
            "parent_id": Value::Null,
            "kind": "heading",
            "label": label,
            "order": order,
            "source_span": {"line": line_no},
            "artifact_id": artifact_id,
        }));
        order += 1;
    }
    items
}

/// Stamp `data-mq-node` onto elements that already carry `data-line` matching a node/outline id.
fn inject_mq_node_attrs(html: &str, outline: &[Value], p: &KnowledgePresentation) -> String {
    let mut out = html.to_string();
    let mut bindings: Vec<(u32, String)> = Vec::new();
    for n in &p.nodes {
        bindings.push((n.source_line, n.node_id.clone()));
    }
    for item in outline {
        if let (Some(line), Some(id)) = (
            item.pointer("/source_span/line").and_then(|v| v.as_u64()),
            item.get("id").and_then(|v| v.as_str()),
        ) {
            bindings.push((line as u32, id.to_string()));
        }
    }
    bindings.sort_by_key(|(line, _)| *line);
    bindings.dedup_by_key(|(line, _)| *line);
    for (line, id) in bindings {
        let needle = format!("data-line=\"{line}\"");
        let attr = format!("data-line=\"{line}\" data-mq-node=\"{}\"", html_attr_escape(&id));
        if out.contains(&needle) && !out.contains(&format!("data-mq-node=\"{}\"", html_attr_escape(&id))) {
            out = out.replacen(&needle, &attr, 1);
        }
    }
    out
}

fn html_attr_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn outline_from_presentation(p: &KnowledgePresentation) -> Value {
    json!(p
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| json!({
            "id": n.node_id,
            "parent_id": Value::Null,
            "kind": "heading",
            "label": n.display_name,
            "order": i,
            "artifact_id": n.artifact_id,
        }))
        .collect::<Vec<_>>())
}

fn nodes_from_presentation_and_steps(
    p: &KnowledgePresentation,
    project: &crate::sema::SemanticProject,
    rel: &str,
    filter: &Option<String>,
) -> Value {
    let mut out = Vec::new();
    for n in &p.nodes {
        if filter
            .as_ref()
            .map(|f| &n.node_id == f || n.artifact_id.as_ref() == Some(f))
            .unwrap_or(true)
        {
            out.push(json!({
                "id": n.node_id,
                "parent_id": Value::Null,
                "kind": n.kind,
                "label": n.display_name,
                "order": n.source_line,
                "source_span": {"file": n.source_file, "line": n.source_line},
                "artifact_id": n.artifact_id,
                "ref_ids": [],
                "children": [],
                "evidence": n.evidence,
                "effects": n.effects,
                "dependencies": n.dependencies,
                "signature": n.signature,
            }));
        }
    }
    // F09: project KnowledgeStep kinds (observe/derive/call/iterate/…) into the read model.
    for step in &project.steps {
        if step.source.file != rel {
            continue;
        }
        let id = step.id.0.clone();
        if filter
            .as_ref()
            .map(|f| &id == f || &step.reference == f)
            .unwrap_or(true)
        {
            out.push(json!({
                "id": id,
                "parent_id": Value::Null,
                "kind": step.kind,
                "label": step.reference,
                "order": step.source.line,
                "source_span": {
                    "file": step.source.file,
                    "line": step.source.line,
                    "end_line": step.source.end_line
                },
                "artifact_id": Value::Null,
                "ref_ids": step.calls.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
                "children": [],
                "evidence": "unverified",
                "effects": step.effects.values(),
            }));
        }
    }
    json!(out)
}

fn refs_from_project(project: &crate::sema::SemanticProject, rel: &str) -> Value {
    let mut refs = Vec::new();
    for a in &project.artifacts {
        if a.source.file != rel {
            continue;
        }
        for dep in &a.dependencies {
            refs.push(json!({
                "id": format!("ref:{}:{}", a.artifact_id.0, dep.0),
                "from_artifact_id": a.artifact_id.0,
                "to": dep.0,
                "relation": "depends_on",
            }));
        }
    }
    json!(refs)
}

fn canonicalize_project(p: &Path) -> Result<PathBuf, Value> {
    let root = if p.is_file() {
        p.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
    } else {
        p.to_path_buf()
    };
    root.canonicalize().map_err(|e| {
        err(
            "view.project_invalid",
            format!("project {}: {e}", root.display()),
        )
    })
}

fn resolve_under(root: &Path, rel: &str) -> Result<PathBuf, Value> {
    let joined = root.join(rel);
    let canon = joined.canonicalize().map_err(|e| {
        err(
            "view.document_not_found",
            format!("{}: {e}", joined.display()),
        )
    })?;
    if !canon.starts_with(root) {
        return Err(err(
            "view.path_escape",
            "document path escapes project root",
        ));
    }
    Ok(canon)
}

fn err(code: &str, message: impl Into<String>) -> Value {
    json!({"code": code, "message": message.into()})
}
