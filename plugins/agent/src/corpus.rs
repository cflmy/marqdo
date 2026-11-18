//! Local corpus search + MCP fixture adapter (evidence tools; workbook stays authority).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

fn arg_text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("missing {key}"))
}

fn arg_usize(args: &Value, key: &str, default: usize) -> usize {
    match args.get(key) {
        None | Some(Value::Null) => default,
        Some(Value::Number(n)) => n.as_u64().unwrap_or(default as u64) as usize,
        Some(Value::String(s)) => s.parse().unwrap_or(default),
        _ => default,
    }
}

fn resolve_path(rel: &str) -> PathBuf {
    let p = PathBuf::from(rel);
    if p.is_absolute() {
        return p;
    }
    // Prefer host cwd (entry file directory) via host_query — same as kb.rs.
    let base = (|| {
        let q = crate::host_query_json("cwd")?;
        q.as_str()
            .map(PathBuf::from)
            .ok_or_else(|| "cwd not text".to_string())
    })()
    .or_else(|_| std::env::current_dir().map_err(|e| e.to_string()))
    .unwrap_or_else(|_| PathBuf::from("."));
    base.join(p)
}

fn is_corpus_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some("md" | "txt" | "mq.md") => true,
        Some(ext) if ext.eq_ignore_ascii_case("markdown") => true,
        _ => {
            // `.mq.md` is two extensions on some platforms → check suffix
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(".mq.md") || n.ends_with(".md") || n.ends_with(".txt"))
                .unwrap_or(false)
        }
    }
}

fn query_terms(q: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let flush = |buf: &mut String, out: &mut Vec<String>| {
        if !buf.is_empty() {
            out.push(buf.to_ascii_lowercase());
            buf.clear();
        }
    };
    for c in q.chars() {
        if c.is_alphanumeric() {
            buf.push(c);
        } else if !c.is_whitespace() && !c.is_ascii() {
            // CJK / other letters: treat each char as a term
            flush(&mut buf, &mut out);
            out.push(c.to_string());
        } else {
            flush(&mut buf, &mut out);
        }
    }
    flush(&mut buf, &mut out);
    out.retain(|t| !t.is_empty());
    out
}

fn score_doc(text: &str, terms: &[String]) -> f64 {
    if terms.is_empty() {
        return 0.0;
    }
    let lower = text.to_ascii_lowercase();
    let mut hits = 0usize;
    for t in terms {
        if lower.contains(t) {
            hits += 1;
        }
    }
    hits as f64 / terms.len() as f64
}

fn best_excerpt(text: &str, terms: &[String], max_chars: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut best_i = 0usize;
    let mut best_score = -1.0f64;
    for (i, line) in lines.iter().enumerate() {
        let s = score_doc(line, terms);
        if s > best_score {
            best_score = s;
            best_i = i;
        }
    }
    let start = best_i.saturating_sub(1);
    let end = (best_i + 2).min(lines.len());
    let mut chunk = lines[start..end].join("\n");
    if chunk.chars().count() > max_chars {
        chunk = chunk.chars().take(max_chars).collect();
        chunk.push('…');
    }
    chunk
}

fn walk_corpus(root: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth > 6 || out.len() >= 400 {
        return;
    }
    let Ok(rd) = fs::read_dir(root) else {
        return;
    };
    let mut ents: Vec<_> = rd.flatten().collect();
    ents.sort_by_key(|e| e.file_name());
    for ent in ents {
        let p = ent.path();
        if p.is_dir() {
            walk_corpus(&p, out, depth + 1);
        } else if is_corpus_file(&p) {
            out.push(p);
        }
    }
}

/// Keyword corpus search over local markdown/text. Evidence only — not authority.
pub fn corpus_search(args: &Value) -> Result<Value, String> {
    let query = arg_text(args, "query")?;
    let root_s = match args.get("root").and_then(|v| v.as_str()) {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => ".marqdo/agent-corpus",
    };
    let limit = arg_usize(args, "limit", 6).clamp(1, 40);
    let root = resolve_path(root_s);
    if !root.is_dir() {
        return Ok(json!({
            "hits": [],
            "count": 0,
            "query": query,
            "root": root_s,
            "authority": "workbook",
            "note": "Corpus miss or missing root. Evidence only — runnable .mq.md workbook remains authority.",
        }));
    }
    let terms = query_terms(query);
    let mut files = Vec::new();
    walk_corpus(&root, &mut files, 0);
    let mut ranked: Vec<(f64, PathBuf, String)> = Vec::new();
    for path in files {
        let Ok(src) = fs::read_to_string(&path) else {
            continue;
        };
        let score = score_doc(&src, &terms);
        if score <= 0.0 {
            continue;
        }
        let excerpt = best_excerpt(&src, &terms, 320);
        ranked.push((score, path, excerpt));
    }
    ranked.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.cmp(&b.1))
    });
    ranked.truncate(limit);
    let hits: Vec<Value> = ranked
        .into_iter()
        .map(|(score, path, excerpt)| {
            let rel = path
                .strip_prefix(&root)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| path.to_string_lossy().into_owned());
            json!({
                "path": rel,
                "score": (score * 1000.0).round() / 1000.0,
                "excerpt": excerpt,
            })
        })
        .collect();
    let count = hits.len() as i64;
    Ok(json!({
        "hits": hits,
        "count": count,
        "query": query,
        "root": root_s,
        "authority": "workbook",
        "note": "Corpus hits are external evidence only. Do not treat them as the runnable ground truth — the .mq.md workbook / OKF skill remains authority.",
    }))
}

fn load_fixture(path: &str) -> Result<Value, String> {
    let p = resolve_path(path);
    let src = fs::read_to_string(&p).map_err(|e| format!("mcp fixture read: {e}"))?;
    serde_json::from_str(&src).map_err(|e| format!("mcp fixture json: {e}"))
}

/// Offline / local MCP-shaped tool surface via a JSON fixture.
/// `action=list` → tools; `action=call` → results[name] (arguments ignored for fixture).
pub fn mcp_fixture(args: &Value) -> Result<Value, String> {
    let action = args
        .get("action")
        .and_then(|v| v.as_str())
        .unwrap_or("list");
    let fixture = arg_text(args, "fixture")?;
    let doc = load_fixture(fixture)?;
    let note = doc
        .get("note")
        .and_then(|v| v.as_str())
        .unwrap_or(
            "MCP/fixture evidence only. Workbook .mq.md remains authority; do not promote tool prose to ground truth.",
        )
        .to_string();
    match action {
        "list" => {
            let tools = doc.get("tools").cloned().unwrap_or_else(|| json!([]));
            Ok(json!({
                "tools": tools,
                "fixture": fixture,
                "authority": "workbook",
                "note": note,
            }))
        }
        "call" => {
            let name = arg_text(args, "name")?;
            let results = doc.get("results").or_else(|| doc.get("calls"));
            let Some(map) = results.and_then(|v| v.as_object()) else {
                return Err("mcp fixture missing results/calls object".into());
            };
            let Some(val) = map.get(name) else {
                return Ok(json!({
                    "ok": false,
                    "name": name,
                    "error": "tool not in fixture",
                    "authority": "workbook",
                    "note": note,
                }));
            };
            Ok(json!({
                "ok": true,
                "name": name,
                "result": val,
                "arguments": args.get("arguments").cloned().unwrap_or(Value::Null),
                "authority": "workbook",
                "note": note,
            }))
        }
        other => Err(format!("mcp fixture unknown action: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn corpus_search_ranks_matching_file() {
        let dir = std::env::temp_dir().join("marqdo-corpus-a4-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mut f = fs::File::create(dir.join("refunds.md")).unwrap();
        writeln!(f, "# Refunds\n\nCustomers may request a refund within 30 days.").unwrap();
        let mut f2 = fs::File::create(dir.join("shipping.md")).unwrap();
        writeln!(f2, "# Shipping\n\nShips in two business days.").unwrap();

        let args = json!({
            "query": "refund 30 days",
            "root": dir.to_string_lossy(),
            "limit": 4,
        });
        let out = corpus_search(&args).unwrap();
        assert_eq!(out["authority"], "workbook");
        assert!(out["count"].as_i64().unwrap() >= 1);
        let hits = out["hits"].as_array().unwrap();
        assert_eq!(hits[0]["path"], "refunds.md");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn mcp_fixture_list_and_call() {
        let dir = std::env::temp_dir().join("marqdo-mcp-a4-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("fixture.json");
        fs::write(
            &path,
            r#"{"tools":[{"name":"wiki_get","description":"wiki"}],"results":{"wiki_get":{"text":"ok-page"}}}"#,
        )
        .unwrap();
        let list = mcp_fixture(&json!({
            "action": "list",
            "fixture": path.to_string_lossy(),
        }))
        .unwrap();
        assert_eq!(list["tools"][0]["name"], "wiki_get");
        let call = mcp_fixture(&json!({
            "action": "call",
            "fixture": path.to_string_lossy(),
            "name": "wiki_get",
        }))
        .unwrap();
        assert_eq!(call["ok"], true);
        assert_eq!(call["result"]["text"], "ok-page");
        assert_eq!(call["authority"], "workbook");
        let _ = fs::remove_dir_all(&dir);
    }
}
