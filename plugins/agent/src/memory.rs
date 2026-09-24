//! Agent Framework v2 memory: episodes → experience → compile (plugin-side).
//! Layout under `.marqdo/agent-memory/` (default). Does not touch `src/host`.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Datelike;
use serde_json::{json, Value};

use crate::kb::{
    canonicalize_goal_str, char_bigram_tf, cosine_tf, goal_sig_hex, goal_slug_str, normalize_goal,
};

const DEFAULT_MEMORY_DIR: &str = ".marqdo/agent-memory";
const DEFAULT_KB_DIR: &str = ".marqdo/agent-kb";

fn resolve_path(rel: &str) -> PathBuf {
    let p = PathBuf::from(rel);
    if p.is_absolute() {
        return p;
    }
    // Already valid relative to process cwd (e.g. re-passed after prior join).
    if p.exists() {
        return p;
    }
    let base = (|| {
        let q = crate::host_query_json("cwd")?;
        q.as_str()
            .map(PathBuf::from)
            .ok_or_else(|| "cwd not text".to_string())
    })()
    .or_else(|_| std::env::current_dir().map_err(|e| e.to_string()))
    .unwrap_or_else(|_| PathBuf::from("."));
    let joined = base.join(&p);
    if joined.exists() || !base.as_os_str().is_empty() {
        return joined;
    }
    p
}

fn arg_text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("missing text `{key}`"))
}

fn opt_text<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(|v| v.as_str()).filter(|s| !s.is_empty())
}

fn opt_i64(args: &Value, key: &str, default: i64) -> i64 {
    match args.get(key) {
        None | Some(Value::Null) => default,
        Some(Value::Number(n)) => n.as_i64().unwrap_or(default),
        Some(Value::String(s)) => s.parse().unwrap_or(default),
        Some(Value::Bool(b)) => {
            if *b {
                1
            } else {
                0
            }
        }
        _ => default,
    }
}

fn opt_bool(args: &Value, key: &str, default: bool) -> bool {
    match args.get(key) {
        None | Some(Value::Null) => default,
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => matches!(s.as_str(), "true" | "True" | "1" | "yes"),
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
        _ => default,
    }
}

fn memory_root(args: &Value) -> PathBuf {
    let rel = opt_text(args, "memory_dir").unwrap_or(DEFAULT_MEMORY_DIR);
    resolve_path(rel)
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn yaml_escape(s: &str) -> String {
    if s.is_empty()
        || s.contains(':')
        || s.contains('#')
        || s.contains('"')
        || s.contains('\'')
        || s.contains('\n')
        || s.starts_with(' ')
    {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

fn marqdo_quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn value_brief(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => {
            if *b {
                "true".into()
            } else {
                "false".into()
            }
        }
        other => other.to_string(),
    }
}

fn ensure_layout(root: &Path) -> Result<(), String> {
    for sub in [
        "episodes",
        "experiences",
        "candidates",
        "skills",
        "policies",
        "tests",
    ] {
        fs::create_dir_all(root.join(sub)).map_err(|e| format!("mkdir {sub}: {e}"))?;
    }
    let index = root.join("index.mq.md");
    if !index.is_file() {
        let body = "---\ntype: agent-memory-index\ntitle: agent-memory\n---\n\n# Agent memory\n\nEpisodes, experiences, and compiled skills.\n";
        fs::write(&index, body).map_err(|e| format!("write index: {e}"))?;
    }
    Ok(())
}

/// Ensure `.marqdo/agent-memory/` layout exists.
pub fn memory_ensure(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    ensure_layout(&root)?;
    Ok(json!({
        "ok": true,
        "memory_dir": root.to_string_lossy(),
    }))
}

fn episode_dir(root: &Path) -> PathBuf {
    let now = chrono::Utc::now();
    root.join("episodes")
        .join(format!("{:04}", now.year()))
        .join(format!("{:02}", now.month()))
}

fn next_episode_id(dir: &Path) -> Result<(String, PathBuf), String> {
    fs::create_dir_all(dir).map_err(|e| format!("mkdir episodes: {e}"))?;
    let mut max = 0u64;
    if let Ok(rd) = fs::read_dir(dir) {
        for ent in rd.flatten() {
            let name = ent.file_name().to_string_lossy().into_owned();
            if let Some(rest) = name.strip_prefix("episode-") {
                if let Some(num) = rest.strip_suffix(".mq.md") {
                    if let Ok(n) = num.parse::<u64>() {
                        max = max.max(n);
                    }
                }
            }
        }
    }
    let id_n = max + 1;
    let id = format!("episode-{id_n:04}");
    let path = dir.join(format!("{id}.mq.md"));
    Ok((id, path))
}

fn section(title: &str, body: &str) -> String {
    let b = body.trim();
    if b.is_empty() {
        format!("# {title}\n\nnone\n")
    } else {
        format!("# {title}\n\n{b}\n")
    }
}

/// Record one execution as a readable `.mq.md` episode (v2 Phase 1).
pub fn record_episode(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    ensure_layout(&root)?;
    let task = arg_text(args, "task").or_else(|_| arg_text(args, "goal"))?;
    let task_n = normalize_goal(task);
    let task_hash = goal_sig_hex(&task_n);
    let slug = goal_slug_str(&task_n);
    let status = opt_text(args, "status").unwrap_or("success");
    let mode = opt_text(args, "mode").unwrap_or("explore");
    let skill = opt_text(args, "skill")
        .map(|s| s.to_string())
        .or_else(|| {
            args.get("skill")
                .filter(|v| !v.is_null())
                .map(value_brief)
                .filter(|s| !s.is_empty() && s != "null")
        })
        .unwrap_or_else(|| "none".into());
    let result = args
        .get("result")
        .map(value_brief)
        .unwrap_or_default();
    let context = args
        .get("context")
        .map(value_brief)
        .unwrap_or_default();
    let actions = args
        .get("actions")
        .map(value_brief)
        .unwrap_or_default();
    let observations = args
        .get("observations")
        .or_else(|| args.get("observation"))
        .map(value_brief)
        .unwrap_or_default();
    let verification = args
        .get("verification")
        .map(value_brief)
        .unwrap_or_else(|| {
            if status == "ok" || status == "success" {
                "status: passed".into()
            } else {
                "status: failed".into()
            }
        });
    let errors = args
        .get("errors")
        .or_else(|| args.get("error"))
        .map(value_brief)
        .unwrap_or_else(|| "none".into());
    let llm_calls = opt_i64(args, "llm_calls", 0);
    let tool_calls = opt_i64(args, "tool_calls", 0);
    let tokens = opt_i64(args, "tokens", 0);
    let latency_ms = opt_i64(args, "latency_ms", 0);
    let workbook = opt_text(args, "workbook").unwrap_or("");
    let cache = opt_text(args, "cache").unwrap_or("");
    let match_kind = opt_text(args, "match").unwrap_or("");

    let ep_status = if status == "ok" || status == "success" {
        "success"
    } else {
        "failure"
    };

    let dir = episode_dir(&root);
    let (id, path) = next_episode_id(&dir)?;
    let ts = now_rfc3339();

    let mut fm = format!(
        "---\ntype: episode\nid: {id}\ntask_hash: {task_hash}\nslug: {slug}\ntimestamp: {ts}\nstatus: {ep_status}\nskill: {}\nmode: {}\nllm_calls: {llm_calls}\ntool_calls: {tool_calls}\ntokens: {tokens}\nlatency_ms: {latency_ms}\n",
        yaml_escape(&skill),
        yaml_escape(mode),
    );
    if !workbook.is_empty() {
        fm.push_str(&format!("workbook: {}\n", yaml_escape(workbook)));
    }
    if !cache.is_empty() {
        fm.push_str(&format!("cache: {}\n", yaml_escape(cache)));
    }
    if !match_kind.is_empty() {
        fm.push_str(&format!("match: {}\n", yaml_escape(match_kind)));
    }
    fm.push_str("---\n\n");

    let body = format!(
        "{fm}{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        section("Task", &task_n),
        section("Context", &context),
        section("Decisions", &actions),
        section("Actions", &actions),
        section("Observations", &observations),
        section("Result", &result),
        section("Verification", &verification),
    );
    let body = if errors != "none" && !errors.is_empty() {
        format!("{body}\n{}\n", section("Errors", &errors))
    } else {
        format!("{body}\n{}\n", section("Errors", "none"))
    };

    fs::write(&path, body).map_err(|e| format!("write episode: {e}"))?;
    let rel = path
        .strip_prefix(resolve_path("."))
        .unwrap_or(&path)
        .to_string_lossy()
        .into_owned();

    Ok(json!({
        "ok": true,
        "id": id,
        "path": path.to_string_lossy(),
        "rel": rel,
        "task_hash": task_hash,
        "slug": slug,
        "status": ep_status,
        "mode": mode,
    }))
}

fn extract_fm_field(source: &str, key: &str) -> Option<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return None;
    }
    let prefix = format!("{key}:");
    for line in lines.iter().skip(1) {
        let t = line.trim();
        if t == "---" {
            break;
        }
        if let Some(rest) = t.strip_prefix(&prefix) {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn extract_section(source: &str, title: &str) -> String {
    let marker = format!("# {title}");
    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        if line.trim() == marker {
            let mut body = String::new();
            for l in lines.by_ref() {
                if l.starts_with("# ") {
                    break;
                }
                if !body.is_empty() {
                    body.push('\n');
                }
                body.push_str(l);
            }
            return body.trim().to_string();
        }
    }
    String::new()
}

#[derive(Clone)]
struct EpisodeMeta {
    id: String,
    path: PathBuf,
    task_hash: String,
    slug: String,
    status: String,
    mode: String,
    task: String,
    result: String,
    workbook: String,
    llm_calls: i64,
    tool_calls: i64,
}

fn walk_episodes(root: &Path) -> Vec<EpisodeMeta> {
    let mut out = Vec::new();
    let base = root.join("episodes");
    if !base.is_dir() {
        return out;
    }
    walk_mq(&base, &mut out);
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

fn walk_mq(dir: &Path, out: &mut Vec<EpisodeMeta>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            walk_mq(&p, out);
            continue;
        }
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !name.ends_with(".mq.md") {
            continue;
        }
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        if extract_fm_field(&src, "type").as_deref() != Some("episode") {
            continue;
        }
        let id = extract_fm_field(&src, "id").unwrap_or_else(|| name.trim_end_matches(".mq.md").into());
        let task_hash = extract_fm_field(&src, "task_hash").unwrap_or_default();
        let slug = extract_fm_field(&src, "slug").unwrap_or_default();
        let status = extract_fm_field(&src, "status").unwrap_or_default();
        let mode = extract_fm_field(&src, "mode").unwrap_or_default();
        let workbook = extract_fm_field(&src, "workbook").unwrap_or_default();
        let llm_calls = extract_fm_field(&src, "llm_calls")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let tool_calls = extract_fm_field(&src, "tool_calls")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let task = extract_section(&src, "Task");
        let result = extract_section(&src, "Result");
        out.push(EpisodeMeta {
            id,
            path: p,
            task_hash,
            slug,
            status,
            mode,
            task,
            result,
            workbook,
            llm_calls,
            tool_calls,
        });
    }
}

/// List episodes, optionally filtered by task / task_hash / status.
pub fn list_episodes(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    ensure_layout(&root)?;
    let mut eps = walk_episodes(&root);
    if let Some(th) = opt_text(args, "task_hash") {
        eps.retain(|e| e.task_hash == th);
    } else if let Ok(task) = arg_text(args, "task").or_else(|_| arg_text(args, "goal")) {
        let th = goal_sig_hex(&normalize_goal(task));
        eps.retain(|e| e.task_hash == th);
    }
    if let Some(st) = opt_text(args, "status") {
        eps.retain(|e| e.status == st);
    }
    let limit = opt_i64(args, "limit", 50).max(0) as usize;
    let total = eps.len();
    eps.truncate(limit);
    let items: Vec<Value> = eps
        .into_iter()
        .map(|e| {
            json!({
                "id": e.id,
                "path": e.path.to_string_lossy(),
                "task_hash": e.task_hash,
                "slug": e.slug,
                "status": e.status,
                "mode": e.mode,
                "llm_calls": e.llm_calls,
                "tool_calls": e.tool_calls,
            })
        })
        .collect();
    Ok(json!({ "count": total, "episodes": items }))
}

/// Read one episode by id or path.
pub fn get_episode(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    if let Some(path) = opt_text(args, "path") {
        let p = resolve_path(path);
        let src = fs::read_to_string(&p).map_err(|e| format!("read episode: {e}"))?;
        return Ok(json!({
            "ok": true,
            "path": p.to_string_lossy(),
            "source": src,
            "id": extract_fm_field(&src, "id"),
            "task_hash": extract_fm_field(&src, "task_hash"),
            "status": extract_fm_field(&src, "status"),
            "result": extract_section(&src, "Result"),
            "task": extract_section(&src, "Task"),
        }));
    }
    let id = arg_text(args, "id")?;
    for e in walk_episodes(&root) {
        if e.id == id {
            let src = fs::read_to_string(&e.path).map_err(|e| format!("read episode: {e}"))?;
            return Ok(json!({
                "ok": true,
                "path": e.path.to_string_lossy(),
                "source": src,
                "id": e.id,
                "task_hash": e.task_hash,
                "status": e.status,
                "result": e.result,
                "task": e.task,
            }));
        }
    }
    Ok(Value::Null)
}

fn near_score(a: &str, b: &str) -> f64 {
    let q = canonicalize_goal_str(a);
    let c = canonicalize_goal_str(b);
    if q.is_empty() || c.is_empty() {
        return 0.0;
    }
    cosine_tf(&char_bigram_tf(&q), &char_bigram_tf(&c))
}

/// Public near-score for Adaptive Routing policy match.
pub(crate) fn task_near_score(a: &str, b: &str) -> f64 {
    near_score(a, b)
}

/// Cheap cluster of episodes by task_hash first, then n-gram near-match (v2 Phase 2).
pub fn cluster_episodes(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    ensure_layout(&root)?;
    let threshold = match args.get("threshold") {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.82),
        Some(Value::String(s)) => s.parse().unwrap_or(0.82),
        _ => 0.82,
    };
    let mut eps = walk_episodes(&root);
    if let Ok(task) = arg_text(args, "task").or_else(|_| arg_text(args, "goal")) {
        let th = goal_sig_hex(&normalize_goal(task));
        // Prefer same hash; also keep near-hash neighbors via task text.
        let task_n = normalize_goal(task);
        eps.retain(|e| e.task_hash == th || near_score(&task_n, &e.task) >= threshold);
    }
    // Group by task_hash.
    let mut by_hash: Vec<(String, Vec<EpisodeMeta>)> = Vec::new();
    for e in eps {
        if let Some((_, g)) = by_hash.iter_mut().find(|(h, _)| *h == e.task_hash) {
            g.push(e);
        } else {
            by_hash.push((e.task_hash.clone(), vec![e]));
        }
    }
    // Merge near groups when representatives are similar.
    let mut clusters: Vec<Vec<EpisodeMeta>> = by_hash.into_iter().map(|(_, g)| g).collect();
    let mut merged = true;
    while merged {
        merged = false;
        'outer: for i in 0..clusters.len() {
            for j in (i + 1)..clusters.len() {
                let ai = clusters[i].first().map(|e| e.task.as_str()).unwrap_or("");
                let aj = clusters[j].first().map(|e| e.task.as_str()).unwrap_or("");
                if !ai.is_empty() && !aj.is_empty() && near_score(ai, aj) >= threshold {
                    let other = clusters.remove(j);
                    clusters[i].extend(other);
                    merged = true;
                    break 'outer;
                }
            }
        }
    }
    let out: Vec<Value> = clusters
        .into_iter()
        .enumerate()
        .map(|(i, g)| {
            let ids: Vec<Value> = g.iter().map(|e| Value::String(e.id.clone())).collect();
            let sample = g.first().map(|e| e.task.clone()).unwrap_or_default();
            let success = g.iter().filter(|e| e.status == "success").count() as i64;
            json!({
                "cluster": format!("cluster-{}", i + 1),
                "size": g.len() as i64,
                "success": success,
                "sample_task": sample,
                "task_hash": g.first().map(|e| e.task_hash.clone()).unwrap_or_default(),
                "episodes": ids,
            })
        })
        .collect();
    Ok(json!({ "count": out.len(), "clusters": out }))
}

/// Extract a shared pattern from successful episodes (heuristic; no LLM).
pub fn extract_pattern(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    let mut eps = walk_episodes(&root);
    if let Ok(task) = arg_text(args, "task").or_else(|_| arg_text(args, "goal")) {
        let th = goal_sig_hex(&normalize_goal(task));
        eps.retain(|e| e.task_hash == th);
    } else if let Some(th) = opt_text(args, "task_hash") {
        eps.retain(|e| e.task_hash == th);
    }
    eps.retain(|e| e.status == "success");
    if eps.is_empty() {
        return Ok(json!({
            "ok": false,
            "reason": "no_success_episodes",
        }));
    }
    let results: Vec<&str> = eps.iter().map(|e| e.result.as_str()).collect();
    let same_result = results.iter().all(|r| *r == results[0]) && !results[0].is_empty() && results[0] != "none";
    let workbooks: Vec<&str> = eps
        .iter()
        .map(|e| e.workbook.as_str())
        .filter(|w| !w.is_empty())
        .collect();
    let avg_llm = {
        let n = eps.len() as f64;
        eps.iter().map(|e| e.llm_calls as f64).sum::<f64>() / n
    };
    let steps = if same_result {
        vec![
            "Return cached solidified answer".to_string(),
            format!("result = {}", results[0]),
        ]
    } else if !workbooks.is_empty() {
        vec![
            "Spawn solidified workbook resource".to_string(),
            format!("workbook = {}", workbooks[0]),
        ]
    } else {
        vec!["Replay last successful procedure".to_string()]
    };
    Ok(json!({
        "ok": true,
        "evidence": eps.len() as i64,
        "same_result": same_result,
        "result": if same_result { results[0] } else { "" },
        "workbook": workbooks.first().copied().unwrap_or(""),
        "avg_llm_calls": avg_llm,
        "steps": steps,
        "invariants": if same_result { json!(["deterministic_result"]) } else { json!([]) },
        "variables": ["task"],
    }))
}

/// Write an experience note under `experiences/<slug>.mq.md`.
pub fn generalize(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    ensure_layout(&root)?;
    let pattern = extract_pattern(args)?;
    if pattern.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Ok(pattern);
    }
    let task = arg_text(args, "task")
        .or_else(|_| arg_text(args, "goal"))
        .unwrap_or("task");
    let slug = goal_slug_str(task);
    let path = root.join("experiences").join(format!("{slug}.mq.md"));
    let evidence = pattern.get("evidence").and_then(|v| v.as_i64()).unwrap_or(0);
    let steps = pattern
        .get("steps")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut steps_md = String::new();
    for (i, s) in steps.iter().enumerate() {
        if let Some(t) = s.as_str() {
            steps_md.push_str(&format!("{}. {t}\n", i + 1));
        }
    }
    let ts = now_rfc3339();
    let body = format!(
        "---\ntype: experience\nslug: {slug}\nevidence: {evidence}\ntimestamp: {ts}\nstatus: draft\n---\n\n# Experience: {slug}\n\n# Task\n\n{}\n\n# Pattern\n\n{steps_md}\n",
        normalize_goal(task),
    );
    fs::write(&path, body).map_err(|e| format!("write experience: {e}"))?;
    Ok(json!({
        "ok": true,
        "slug": slug,
        "path": path.to_string_lossy(),
        "evidence": evidence,
        "pattern": pattern,
    }))
}

fn write_compiled_mq(result: &str, title: &str) -> String {
    let lit = marqdo_quoted(result);
    format!(
        "---\ntitle: {title}\ntype: skill\nstatus: compiled\nllm_free: true\n---\n\nCompiled from successful episodes (deterministic return).\n\n# main\n\n**`result` = {lit}**\n*`result`*\n"
    )
}

/// Synthesize a candidate skill `.mq.md` from pattern / episodes.
pub fn synthesize_skill(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    ensure_layout(&root)?;
    let pattern = extract_pattern(args)?;
    if pattern.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Ok(pattern);
    }
    let task = arg_text(args, "task")
        .or_else(|_| arg_text(args, "goal"))
        .unwrap_or("task");
    let slug = goal_slug_str(task);
    let cand_dir = root.join("candidates").join(&slug);
    fs::create_dir_all(&cand_dir).map_err(|e| format!("mkdir candidates: {e}"))?;
    let path = cand_dir.join("candidate.mq.md");
    let evidence = pattern.get("evidence").and_then(|v| v.as_i64()).unwrap_or(0);
    let same = pattern.get("same_result").and_then(|v| v.as_bool()).unwrap_or(false);
    let result = pattern
        .get("result")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let workbook = pattern
        .get("workbook")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let (body, llm_free) = if same && !result.is_empty() && result != "none" {
        (write_compiled_mq(result, &slug), true)
    } else if !workbook.is_empty() {
        let wb_path = resolve_path(workbook);
        match fs::read_to_string(&wb_path) {
            Ok(src) => {
                let lf = !src.contains(".step")
                    && !src.contains(".单步")
                    && !src.contains("worker.step");
                (src, lf)
            }
            Err(_) => (
                format!(
                    "---\ntitle: {slug}\ntype: skill\nstatus: candidate\nllm_free: false\n---\n\n# main\n\n*\"candidate pending solidify\"*\n"
                ),
                false,
            ),
        }
    } else {
        (
            format!(
                "---\ntitle: {slug}\ntype: skill\nstatus: candidate\nllm_free: false\n---\n\n# Purpose\n\nCandidate skill — needs further exploration.\n\n# Main\n\n# main\n\n*\"not yet compiled\"*\n"
            ),
            false,
        )
    };
    fs::write(&path, &body).map_err(|e| format!("write candidate: {e}"))?;
    Ok(json!({
        "ok": true,
        "slug": slug,
        "path": path.to_string_lossy(),
        "llm_free": llm_free,
        "evidence": evidence,
        "status": if llm_free { "compiled" } else { "candidate" },
    }))
}

/// Validate + compile candidate into `skills/<slug>/vN.mq.md`, optionally promote to agent-kb.
pub fn compile_skill(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    ensure_layout(&root)?;
    let task = arg_text(args, "task")
        .or_else(|_| arg_text(args, "goal"))
        .unwrap_or("task");
    let slug = opt_text(args, "slug")
        .map(|s| s.to_string())
        .unwrap_or_else(|| goal_slug_str(task));
    let cand = opt_text(args, "candidate")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("candidates").join(&slug).join("candidate.mq.md"));
    let cand = if cand.is_absolute() {
        cand
    } else {
        resolve_path(&cand.to_string_lossy())
    };
    if !cand.is_file() {
        // Try synthesize first.
        let syn = synthesize_skill(args)?;
        if syn.get("ok").and_then(|v| v.as_bool()) != Some(true) {
            return Ok(json!({
                "ok": false,
                "reason": "no_candidate",
                "synthesize": syn,
            }));
        }
    }
    let cand_path = if cand.is_file() {
        cand
    } else {
        root.join("candidates").join(&slug).join("candidate.mq.md")
    };
    let src = fs::read_to_string(&cand_path).map_err(|e| format!("read candidate: {e}"))?;
    let llm_free = extract_fm_field(&src, "llm_free")
        .map(|s| s == "true" || s == "True")
        .unwrap_or_else(|| {
            !src.contains(".step") && !src.contains(".单步") && !src.contains("worker.step")
        });
    if !llm_free && !opt_bool(args, "force", false) {
        return Ok(json!({
            "ok": false,
            "reason": "not_llm_free",
            "slug": slug,
            "candidate": cand_path.to_string_lossy(),
        }));
    }
    // Held-out / historical gate: require min evidence unless forced.
    let min_evidence = opt_i64(args, "min_evidence", 3);
    let pattern = extract_pattern(args)?;
    let evidence = pattern.get("evidence").and_then(|v| v.as_i64()).unwrap_or(0);
    if evidence < min_evidence && !opt_bool(args, "force", false) {
        return Ok(json!({
            "ok": false,
            "reason": "insufficient_evidence",
            "evidence": evidence,
            "min_evidence": min_evidence,
        }));
    }

    let skill_dir = root.join("skills").join(&slug);
    fs::create_dir_all(&skill_dir).map_err(|e| format!("mkdir skills: {e}"))?;
    let mut ver = 1i64;
    if let Ok(rd) = fs::read_dir(&skill_dir) {
        for ent in rd.flatten() {
            let name = ent.file_name().to_string_lossy().into_owned();
            if let Some(rest) = name.strip_prefix('v') {
                if let Some(num) = rest.strip_suffix(".mq.md") {
                    if let Ok(n) = num.parse::<i64>() {
                        ver = ver.max(n + 1);
                    }
                }
            }
        }
    }
    let out_path = skill_dir.join(format!("v{ver}.mq.md"));
    let mut body = src.clone();
    if !body.contains("llm_free:") {
        body = body.replacen("---\n", "---\nllm_free: true\nstatus: compiled\n", 1);
    } else {
        // Force compiled markers.
        body = body
            .lines()
            .map(|l| {
                if l.starts_with("status:") {
                    "status: compiled".to_string()
                } else if l.starts_with("llm_free:") {
                    "llm_free: true".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        if !body.ends_with('\n') {
            body.push('\n');
        }
    }
    fs::write(&out_path, &body).map_err(|e| format!("write skill: {e}"))?;
    // Absolute workbook path so kb_promote's resolve_path does not re-join cwd.
    let wb_abs = out_path
        .canonicalize()
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default().join(&out_path));

    let mut promoted = Value::Null;
    if opt_bool(args, "promote", true) {
        let kb = opt_text(args, "kb_dir").unwrap_or(DEFAULT_KB_DIR);
        let prom_args = json!({
            "kb_dir": kb,
            "goal": task,
            "workbook": wb_abs.to_string_lossy(),
        });
        match crate::kb::kb_promote(&prom_args) {
            Ok(p) => promoted = p,
            Err(e) => {
                return Ok(json!({
                    "ok": true,
                    "compiled": true,
                    "slug": slug,
                    "version": ver,
                    "path": out_path.to_string_lossy(),
                    "llm_free": true,
                    "evidence": evidence,
                    "promote_error": e,
                }));
            }
        }
    }

    Ok(json!({
        "ok": true,
        "compiled": true,
        "slug": slug,
        "version": ver,
        "path": out_path.to_string_lossy(),
        "llm_free": true,
        "evidence": evidence,
        "promoted": promoted,
    }))
}

/// If enough successful episodes exist, generalize → synthesize → compile (v2 learn loop).
pub fn maybe_learn(args: &Value) -> Result<Value, String> {
    let min_evidence = opt_i64(args, "improve_every", opt_i64(args, "min_evidence", 3));
    let root = memory_root(args);
    ensure_layout(&root)?;
    let mem_arg = opt_text(args, "memory_dir").unwrap_or(DEFAULT_MEMORY_DIR);
    let task = match arg_text(args, "task").or_else(|_| arg_text(args, "goal")) {
        Ok(t) => t.to_string(),
        Err(_) => {
            return Ok(json!({
                "learned": false,
                "reason": "missing_task",
            }));
        }
    };
    let th = goal_sig_hex(&normalize_goal(&task));
    let eps: Vec<_> = walk_episodes(&root)
        .into_iter()
        .filter(|e| e.task_hash == th && e.status == "success")
        .collect();
    let evidence = eps.len() as i64;
    if evidence < min_evidence {
        return Ok(json!({
            "learned": false,
            "reason": "insufficient_evidence",
            "evidence": evidence,
            "min_evidence": min_evidence,
            "task_hash": th,
        }));
    }
    // Skip if already llm_free in kb.
    let kb = opt_text(args, "kb_dir").unwrap_or(DEFAULT_KB_DIR);
    let lookup = crate::kb::kb_lookup(&json!({
        "kb_dir": kb,
        "goal": task,
        "near_match": true,
        "near_threshold": 0.78,
    }))
    .unwrap_or(Value::Null);
    if lookup.get("llm_free").and_then(|v| v.as_bool()) == Some(true) {
        return Ok(json!({
            "learned": false,
            "reason": "already_llm_free",
            "evidence": evidence,
            "skill": lookup.get("skill").cloned().unwrap_or(Value::Null),
        }));
    }

    // Pass the original memory_dir string (not resolved root) so nested
    // resolve_path does not double-join against host cwd.
    let exp = generalize(&json!({
        "memory_dir": mem_arg,
        "task": task,
    }))?;
    let syn = synthesize_skill(&json!({
        "memory_dir": mem_arg,
        "task": task,
    }))?;
    if syn.get("llm_free").and_then(|v| v.as_bool()) != Some(true)
        && !opt_bool(args, "force", false)
    {
        return Ok(json!({
            "learned": false,
            "reason": "candidate_not_llm_free",
            "evidence": evidence,
            "experience": exp,
            "candidate": syn,
        }));
    }
    let compiled = compile_skill(&json!({
        "memory_dir": mem_arg,
        "task": task,
        "kb_dir": kb,
        "min_evidence": min_evidence,
        "promote": opt_bool(args, "promote", true),
        "force": opt_bool(args, "force", false),
    }))?;
    Ok(json!({
        "learned": compiled.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
        "evidence": evidence,
        "experience": exp,
        "candidate": syn,
        "compile": compiled,
    }))
}

/// Reasoning-amortization metrics over episodes (+ optional kb skill counts).
pub fn metrics(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    ensure_layout(&root)?;
    let eps = walk_episodes(&root);
    let total = eps.len() as i64;
    let success = eps.iter().filter(|e| e.status == "success").count() as i64;
    let compiled_runs = eps
        .iter()
        .filter(|e| e.mode == "compiled" || e.llm_calls == 0)
        .count() as i64;
    let explore_runs = eps.iter().filter(|e| e.mode == "explore").count() as i64;
    let sum_llm: i64 = eps.iter().map(|e| e.llm_calls).sum();
    let sum_tools: i64 = eps.iter().map(|e| e.tool_calls).sum();
    let avg_llm = if total > 0 {
        sum_llm as f64 / total as f64
    } else {
        0.0
    };
    let mut by_mode: serde_json::Map<String, Value> = serde_json::Map::new();
    for e in &eps {
        let entry = by_mode.entry(e.mode.clone()).or_insert_with(|| {
            json!({ "count": 0, "llm_calls": 0 })
        });
        if let Some(obj) = entry.as_object_mut() {
            let c = obj.get("count").and_then(|v| v.as_i64()).unwrap_or(0) + 1;
            let l = obj.get("llm_calls").and_then(|v| v.as_i64()).unwrap_or(0) + e.llm_calls;
            obj.insert("count".into(), json!(c));
            obj.insert("llm_calls".into(), json!(l));
        }
    }
    Ok(json!({
        "episodes": total,
        "success": success,
        "compiled_runs": compiled_runs,
        "explore_runs": explore_runs,
        "llm_calls_total": sum_llm,
        "tool_calls_total": sum_tools,
        "avg_llm_calls": avg_llm,
        "reasoning_amortization": if success > 0 {
            sum_llm as f64 / success as f64
        } else {
            0.0
        },
        "by_mode": by_mode,
        "memory_dir": root.to_string_lossy(),
    }))
}

/// Resolve task → skill (thin wrapper around kb_lookup with mode labels).
pub fn resolve_task(args: &Value) -> Result<Value, String> {
    let kb = opt_text(args, "kb_dir").unwrap_or(DEFAULT_KB_DIR);
    let goal = arg_text(args, "task").or_else(|_| arg_text(args, "goal"))?;
    let near = opt_bool(args, "near_match", true);
    let thr = match args.get("threshold").or_else(|| args.get("near_threshold")) {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(0.78),
        Some(Value::String(s)) => s.parse().unwrap_or(0.78),
        _ => 0.78,
    };
    let hit = crate::kb::kb_lookup(&json!({
        "kb_dir": kb,
        "goal": goal,
        "near_match": near,
        "near_threshold": thr,
        "tools": args.get("tools").cloned().unwrap_or(Value::Null),
    }))?;
    if hit.is_null() {
        return Ok(json!({
            "mode": "unknown",
            "skill": null,
            "resource": null,
            "confidence": 0.0,
        }));
    }
    let match_kind = hit
        .get("match")
        .and_then(|v| v.as_str())
        .unwrap_or("exact");
    let llm_free = hit.get("llm_free").and_then(|v| v.as_bool()).unwrap_or(false);
    let mode = if llm_free {
        "exact"
    } else if match_kind == "near" || match_kind == "soft" {
        "near"
    } else {
        match_kind
    };
    let confidence = hit
        .get("score")
        .and_then(|v| v.as_f64())
        .unwrap_or(if llm_free { 0.99 } else { 0.85 });
    Ok(json!({
        "mode": mode,
        "match": match_kind,
        "skill": hit.get("skill").cloned().unwrap_or(Value::Null),
        "resource": hit.get("resource").cloned().unwrap_or(Value::Null),
        "slug": hit.get("slug").cloned().unwrap_or(Value::Null),
        "status": hit.get("status").cloned().unwrap_or(Value::Null),
        "llm_free": llm_free,
        "score": hit.get("score").cloned().unwrap_or(Value::Null),
        "confidence": confidence,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_and_cluster_and_compile() {
        let dir = std::env::temp_dir().join(format!(
            "marqdo-mem-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mem = dir.join("agent-memory");
        let kb = dir.join("agent-kb");
        let mem_s = mem.to_string_lossy().into_owned();
        let kb_s = kb.to_string_lossy().into_owned();
        let task = "Reply with exactly the word pong and nothing else.";
        for _ in 0..3 {
            let out = record_episode(&json!({
                "memory_dir": mem_s,
                "task": task,
                "status": "ok",
                "mode": "explore",
                "result": "pong",
                "llm_calls": 2,
            }))
            .unwrap();
            assert_eq!(out.get("ok").and_then(|v| v.as_bool()), Some(true));
        }
        let listed = list_episodes(&json!({
            "memory_dir": mem_s,
            "task": task,
        }))
        .unwrap();
        assert_eq!(listed.get("count").and_then(|v| v.as_i64()), Some(3));
        let cl = cluster_episodes(&json!({
            "memory_dir": mem_s,
            "task": task,
        }))
        .unwrap();
        assert!(cl.get("count").and_then(|v| v.as_i64()).unwrap_or(0) >= 1);
        let learned = maybe_learn(&json!({
            "memory_dir": mem_s,
            "kb_dir": kb_s,
            "task": task,
            "improve_every": 3,
            "promote": true,
        }))
        .unwrap();
        assert_eq!(
            learned.get("learned").and_then(|v| v.as_bool()),
            Some(true),
            "{learned}"
        );
        let m = metrics(&json!({ "memory_dir": mem_s })).unwrap();
        assert_eq!(m.get("episodes").and_then(|v| v.as_i64()), Some(3));
        let _ = fs::remove_dir_all(&dir);
    }
}
