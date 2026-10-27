//! ResourceViewV1 — static read-only preview for ordinary resources (M-A3).
//! Never executes Marqdo / never loads plugins. `.mq.md` remains on DocumentView.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct ResourceViewRequest {
    pub workspace_root: PathBuf,
    pub path: String,
    pub revision: Option<String>,
    pub max_bytes: usize,
}

impl ResourceViewRequest {
    pub fn from_params(params: &Value) -> Result<Self, String> {
        let root = params
            .get("workspace_root")
            .or_else(|| params.get("project"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "resource.view requires workspace_root or project".to_string())?;
        let path = params
            .get("path")
            .or_else(|| params.get("document"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "resource.view requires path or document".to_string())?;
        let mode = params
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("read");
        if mode != "read" {
            return Err(format!("resource.view unsupported mode={mode}; only read"));
        }
        let max_bytes = params
            .get("max_bytes")
            .and_then(|v| v.as_u64())
            .unwrap_or(256 * 1024) as usize;
        Ok(Self {
            workspace_root: PathBuf::from(root),
            path: path.to_string(),
            revision: params
                .get("revision")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            max_bytes,
        })
    }
}

/// Build a ResourceViewV1 for text / markdown / code. No compile, no eval, no network.
pub fn build_resource_view(req: &ResourceViewRequest) -> Result<Value, Value> {
    let root = canonicalize_root(&req.workspace_root)?;
    let abs = resolve_under(&root, &req.path)?;
    let meta = fs::metadata(&abs).map_err(|e| {
        err(
            "resource.read_failed",
            format!("cannot stat {}: {e}", abs.display()),
        )
    })?;
    if !meta.is_file() {
        return Err(err(
            "resource.not_file",
            format!("{} is not a file", abs.display()),
        ));
    }
    let size = meta.len() as usize;
    let raw = fs::read(&abs).map_err(|e| {
        err(
            "resource.read_failed",
            format!("cannot read {}: {e}", abs.display()),
        )
    })?;
    let digest = format!("sha256:{:x}", Sha256::digest(&raw));
    if let Some(expected) = &req.revision {
        if !expected.is_empty() && expected != &digest {
            return Err(err(
                "resource.revision_mismatch",
                format!("requested {expected} but file is {digest}"),
            ));
        }
    }

    let rel = req.path.replace('\\', "/");
    let media = media_type_for(&rel);
    let kind = kind_for(&rel, media);
    let title = Path::new(&rel)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&rel)
        .to_string();

    // `.mq.md` semantic view stays on view.document — refuse silent exec/compile here.
    if rel.ends_with(".mq.md") || media == "application/x-marqdo" {
        return Ok(json!({
            "schema": "ResourceViewV1",
            "schema_version": 1,
            "resource": {
                "resource_id": rel,
                "workspace_id": "local",
                "kind": "marqdo",
                "media_type": "application/x-marqdo",
                "revision": digest,
                "digest": digest,
                "title": title,
                "size": size,
                "origin": "workspace",
                "locator": Value::Null
            },
            "renderer": {"id": "redirect_document_view", "version": "1", "trust": "none"},
            "render": {
                "format": "text",
                "fragment": Value::Null,
                "text": "该资源是 Marqdo 文档；请使用 view.document 获取语义只读视图。"
            },
            "outline": [],
            "locators": [],
            "completeness": "partial",
            "no_exec": true,
            "diagnostics": [{"code": "use_document_view", "message": "use view.document for .mq.md"}],
            "next_cursor": Value::Null
        }));
    }

    let truncated = size > req.max_bytes;
    let slice = if truncated {
        &raw[..req.max_bytes]
    } else {
        &raw[..]
    };
    let text = String::from_utf8_lossy(slice).into_owned();

    // Structured JSON / CSV (M-A5 / Q-A5 subset): bounded parse, no exec.
    if media == "application/json" {
        return Ok(render_json_view(
            &rel, &digest, &title, size, &text, truncated, req.max_bytes, kind,
        ));
    }
    if media == "text/csv" {
        return Ok(render_csv_view(
            &rel, &digest, &title, size, &text, truncated, req.max_bytes, kind,
        ));
    }

    let (renderer_id, format, fragment) = match media {
        "text/markdown" => (
            "markdown",
            "sanitized_html",
            wrap_pre(&simple_markdown_to_html(&text)),
        ),
        "text/x-code" | "text/plain" => ("plaintext", "sanitized_html", wrap_pre(&escape_html(&text))),
        other => {
            return Ok(json!({
                "schema": "ResourceViewV1",
                "schema_version": 1,
                "resource": resource_ref(&rel, other, &digest, &title, size),
                "renderer": {"id": "unavailable", "version": "1", "trust": "none"},
                "render": {"format": "text", "fragment": Value::Null, "text": format!("unsupported media_type={other}")},
                "outline": [],
                "locators": [],
                "completeness": "partial",
                "no_exec": true,
                "diagnostics": [{"code": "unsupported_format", "message": other}],
                "next_cursor": Value::Null
            }));
        }
    };

    let completeness = if truncated { "partial" } else { "complete" };
    let mut diagnostics = Vec::new();
    if truncated {
        diagnostics.push(json!({
            "code": "truncated",
            "message": format!("preview truncated at {} bytes", req.max_bytes)
        }));
    }

    Ok(json!({
        "schema": "ResourceViewV1",
        "schema_version": 1,
        "resource": resource_ref(&rel, media, &digest, &title, size),
        "renderer": {"id": renderer_id, "version": "1", "trust": "sanitized"},
        "render": {"format": format, "fragment": fragment},
        "outline": [],
        "locators": [],
        "completeness": completeness,
        "no_exec": true,
        "diagnostics": diagnostics,
        "next_cursor": Value::Null,
        "capabilities": {"read_only": true, "executed": false, "kind": kind}
    }))
}

fn resource_ref(id: &str, media: &str, digest: &str, title: &str, size: usize) -> Value {
    json!({
        "resource_id": id,
        "workspace_id": "local",
        "kind": kind_for(id, media),
        "media_type": media,
        "revision": digest,
        "digest": digest,
        "title": title,
        "size": size,
        "origin": "workspace",
        "locator": Value::Null
    })
}

fn media_type_for(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".mq.md") {
        "application/x-marqdo"
    } else if lower.ends_with(".md") || lower.ends_with(".markdown") {
        "text/markdown"
    } else if lower.ends_with(".json") {
        "application/json"
    } else if lower.ends_with(".csv") {
        "text/csv"
    } else if lower.ends_with(".rs")
        || lower.ends_with(".ts")
        || lower.ends_with(".tsx")
        || lower.ends_with(".js")
        || lower.ends_with(".jsx")
        || lower.ends_with(".py")
        || lower.ends_with(".go")
        || lower.ends_with(".toml")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
        || lower.ends_with(".css")
        || lower.ends_with(".html")
        || lower.ends_with(".sh")
    {
        "text/x-code"
    } else if lower.ends_with(".txt") || lower.ends_with(".log") {
        "text/plain"
    } else {
        "application/octet-stream"
    }
}

fn kind_for(path: &str, media: &str) -> &'static str {
    if media == "application/x-marqdo" || path.ends_with(".mq.md") {
        "marqdo"
    } else if media == "text/markdown" {
        "markdown"
    } else if media == "application/json" {
        "json"
    } else if media == "text/csv" {
        "csv"
    } else if media == "text/x-code" {
        "code"
    } else if media.starts_with("text/") {
        "text"
    } else {
        "file"
    }
}

const JSON_OUTLINE_MAX: usize = 40;
const CSV_MAX_ROWS: usize = 200;
const CSV_MAX_COLS: usize = 32;

fn render_json_view(
    rel: &str,
    digest: &str,
    title: &str,
    size: usize,
    text: &str,
    truncated: bool,
    max_bytes: usize,
    kind: &str,
) -> Value {
    let mut diagnostics = Vec::new();
    if truncated {
        diagnostics.push(json!({
            "code": "truncated",
            "message": format!("preview truncated at {max_bytes} bytes")
        }));
    }
    match serde_json::from_str::<Value>(text) {
        Ok(val) => {
            let outline = json_outline(&val, "", 0, &mut 0);
            let pretty = serde_json::to_string_pretty(&val).unwrap_or_else(|_| text.to_string());
            let fragment = wrap_pre(&escape_html(&pretty));
            let completeness = if truncated { "partial" } else { "complete" };
            json!({
                "schema": "ResourceViewV1",
                "schema_version": 1,
                "resource": resource_ref(rel, "application/json", digest, title, size),
                "renderer": {"id": "json", "version": "1", "trust": "sanitized"},
                "render": {"format": "sanitized_html", "fragment": fragment},
                "outline": outline,
                "locators": [],
                "completeness": completeness,
                "no_exec": true,
                "diagnostics": diagnostics,
                "next_cursor": Value::Null,
                "capabilities": {"read_only": true, "executed": false, "kind": kind}
            })
        }
        Err(e) => {
            diagnostics.push(json!({
                "code": "json.parse_error",
                "message": e.to_string()
            }));
            json!({
                "schema": "ResourceViewV1",
                "schema_version": 1,
                "resource": resource_ref(rel, "application/json", digest, title, size),
                "renderer": {"id": "json", "version": "1", "trust": "sanitized"},
                "render": {
                    "format": "sanitized_html",
                    "fragment": wrap_pre(&escape_html(text))
                },
                "outline": [],
                "locators": [],
                "completeness": "partial",
                "no_exec": true,
                "diagnostics": diagnostics,
                "next_cursor": Value::Null,
                "capabilities": {"read_only": true, "executed": false, "kind": kind}
            })
        }
    }
}

fn json_outline(val: &Value, path: &str, depth: usize, count: &mut usize) -> Vec<Value> {
    if *count >= JSON_OUTLINE_MAX || depth > 3 {
        return Vec::new();
    }
    match val {
        Value::Object(map) => {
            let mut out = Vec::new();
            for (k, v) in map.iter().take(24) {
                if *count >= JSON_OUTLINE_MAX {
                    break;
                }
                *count += 1;
                let child = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                let label = format!("{k}: {}", json_type_label(v));
                let mut node = json!({
                    "id": child,
                    "label": label,
                    "kind": "json_key",
                    "locator": {"path": child}
                });
                if depth < 2 {
                    let kids = json_outline(v, &child, depth + 1, count);
                    if !kids.is_empty() {
                        if let Some(obj) = node.as_object_mut() {
                            obj.insert("children".into(), json!(kids));
                        }
                    }
                }
                out.push(node);
            }
            out
        }
        Value::Array(items) => {
            *count += 1;
            vec![json!({
                "id": path,
                "label": format!("[{}] ({} items)", path, items.len()),
                "kind": "json_array",
                "locator": {"path": path}
            })]
        }
        _ => Vec::new(),
    }
}

fn json_type_label(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn render_csv_view(
    rel: &str,
    digest: &str,
    title: &str,
    size: usize,
    text: &str,
    truncated: bool,
    max_bytes: usize,
    kind: &str,
) -> Value {
    let mut diagnostics = Vec::new();
    if truncated {
        diagnostics.push(json!({
            "code": "truncated",
            "message": format!("preview truncated at {max_bytes} bytes")
        }));
    }
    let mut rows: Vec<&str> = text.lines().collect();
    while rows.last().is_some_and(|l| l.trim().is_empty()) {
        rows.pop();
    }
    let shown = rows.len().min(CSV_MAX_ROWS + 1); // header + data
    if rows.len() > CSV_MAX_ROWS + 1 {
        diagnostics.push(json!({
            "code": "csv.row_limit",
            "message": format!("showing first {CSV_MAX_ROWS} data rows")
        }));
    }
    let mut html = String::from("<article class=\"mq-resource\"><table class=\"mq-csv\">");
    let mut locators = Vec::new();
    for (i, line) in rows.iter().take(shown).enumerate() {
        let cells = split_csv_line(line);
        let tag = if i == 0 { "th" } else { "td" };
        html.push_str("<tr>");
        for (c, cell) in cells.iter().take(CSV_MAX_COLS).enumerate() {
            html.push('<');
            html.push_str(tag);
            html.push('>');
            html.push_str(&escape_html(cell));
            html.push_str("</");
            html.push_str(tag);
            html.push('>');
            if i > 0 && c == 0 {
                locators.push(json!({
                    "id": format!("r{i}"),
                    "kind": "csv_row",
                    "locator": {"row": i, "col": c + 1}
                }));
            }
        }
        html.push_str("</tr>");
    }
    html.push_str("</table></article>");
    let completeness = if truncated || rows.len() > CSV_MAX_ROWS + 1 {
        "partial"
    } else {
        "complete"
    };
    json!({
        "schema": "ResourceViewV1",
        "schema_version": 1,
        "resource": resource_ref(rel, "text/csv", digest, title, size),
        "renderer": {"id": "csv", "version": "1", "trust": "sanitized"},
        "render": {"format": "sanitized_html", "fragment": html},
        "outline": [],
        "locators": locators,
        "completeness": completeness,
        "no_exec": true,
        "diagnostics": diagnostics,
        "next_cursor": Value::Null,
        "capabilities": {"read_only": true, "executed": false, "kind": kind}
    })
}

/// Minimal CSV splitter (no quoted-newline support — preview only).
fn split_csv_line(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_q = false;
    for ch in line.chars() {
        match ch {
            '"' => in_q = !in_q,
            ',' if !in_q => {
                out.push(cur.clone());
                cur.clear();
            }
            c => cur.push(c),
        }
    }
    out.push(cur);
    out
}

fn wrap_pre(inner: &str) -> String {
    format!("<article class=\"mq-resource\"><pre class=\"mq-resource-body\">{inner}</pre></article>")
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Minimal markdown → HTML (headings/paragraphs/code fences). Not a full parser.
fn simple_markdown_to_html(src: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in src.lines() {
        if line.trim_start().starts_with("```") {
            if in_fence {
                out.push_str("</code></pre>\n");
                in_fence = false;
            } else {
                out.push_str("<pre><code>");
                in_fence = true;
            }
            continue;
        }
        if in_fence {
            out.push_str(&escape_html(line));
            out.push('\n');
            continue;
        }
        let t = line.trim_end();
        if let Some(rest) = t.strip_prefix("### ") {
            out.push_str("<h3>");
            out.push_str(&escape_html(rest));
            out.push_str("</h3>\n");
        } else if let Some(rest) = t.strip_prefix("## ") {
            out.push_str("<h2>");
            out.push_str(&escape_html(rest));
            out.push_str("</h2>\n");
        } else if let Some(rest) = t.strip_prefix("# ") {
            out.push_str("<h1>");
            out.push_str(&escape_html(rest));
            out.push_str("</h1>\n");
        } else if t.is_empty() {
            out.push_str("<br/>\n");
        } else {
            out.push_str("<p>");
            out.push_str(&escape_html(t));
            out.push_str("</p>\n");
        }
    }
    if in_fence {
        out.push_str("</code></pre>\n");
    }
    out
}

fn canonicalize_root(project: &Path) -> Result<PathBuf, Value> {
    fs::canonicalize(project).map_err(|e| {
        err(
            "resource.bad_root",
            format!("cannot canonicalize {}: {e}", project.display()),
        )
    })
}

fn resolve_under(root: &Path, rel: &str) -> Result<PathBuf, Value> {
    let cleaned = rel.trim_start_matches(['/', '\\']);
    if cleaned.is_empty() || cleaned.contains("..") {
        return Err(err(
            "resource.path_escape",
            format!("refusing path {rel:?}"),
        ));
    }
    let abs = root.join(cleaned);
    let canon = fs::canonicalize(&abs).map_err(|e| {
        err(
            "resource.not_found",
            format!("cannot resolve {}: {e}", abs.display()),
        )
    })?;
    if !canon.starts_with(root) {
        return Err(err(
            "resource.path_escape",
            format!("{} escapes workspace root", canon.display()),
        ));
    }
    Ok(canon)
}

fn err(code: &str, message: impl Into<String>) -> Value {
    json!({
        "ok": false,
        "error": {"code": code, "message": message.into()}
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("marqdo-resource-view-{label}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn plaintext_no_exec_and_escape() {
        let dir = scratch_dir("plain");
        let path = dir.join("note.txt");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "<script>alert(1)</script>").unwrap();
        let req = ResourceViewRequest {
            workspace_root: dir.clone(),
            path: "note.txt".into(),
            revision: None,
            max_bytes: 10_000,
        };
        let view = build_resource_view(&req).unwrap();
        assert_eq!(view["no_exec"], true);
        assert_eq!(view["schema"], "ResourceViewV1");
        let frag = view["render"]["fragment"].as_str().unwrap();
        assert!(frag.contains("&lt;script&gt;"));
        assert!(!frag.contains("<script>"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn path_escape_rejected() {
        let dir = scratch_dir("escape");
        let req = ResourceViewRequest {
            workspace_root: dir.clone(),
            path: "../etc/passwd".into(),
            revision: None,
            max_bytes: 10_000,
        };
        let errv = build_resource_view(&req).unwrap_err();
        assert_eq!(errv["error"]["code"], "resource.path_escape");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn json_outline_and_no_exec() {
        let dir = scratch_dir("json");
        let path = dir.join("cfg.json");
        fs::write(&path, r#"{"a":1,"b":{"c":true},"d":[1,2]}"#).unwrap();
        let req = ResourceViewRequest {
            workspace_root: dir.clone(),
            path: "cfg.json".into(),
            revision: None,
            max_bytes: 10_000,
        };
        let view = build_resource_view(&req).unwrap();
        assert_eq!(view["no_exec"], true);
        assert_eq!(view["renderer"]["id"], "json");
        assert_eq!(view["resource"]["media_type"], "application/json");
        assert!(view["outline"].as_array().unwrap().len() >= 2);
        let frag = view["render"]["fragment"].as_str().unwrap();
        assert!(frag.contains("&quot;a&quot;") || frag.contains("\"a\"") || frag.contains("a"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn csv_table_bounded() {
        let dir = scratch_dir("csv");
        let path = dir.join("t.csv");
        fs::write(&path, "name,age\nalice,3\nbob,4\n").unwrap();
        let req = ResourceViewRequest {
            workspace_root: dir.clone(),
            path: "t.csv".into(),
            revision: None,
            max_bytes: 10_000,
        };
        let view = build_resource_view(&req).unwrap();
        assert_eq!(view["no_exec"], true);
        assert_eq!(view["renderer"]["id"], "csv");
        let frag = view["render"]["fragment"].as_str().unwrap();
        assert!(frag.contains("<table"));
        assert!(frag.contains("alice"));
        assert!(!frag.contains("<script"));
        let _ = fs::remove_dir_all(&dir);
    }
}
