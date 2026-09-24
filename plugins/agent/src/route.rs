//! Adaptive Routing / Policy Compilation (Phase 4).
//! Router is abstract: marqdo | small-llm | jev | llm | auto — Jev is optional.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::kb::{goal_sig_hex, goal_slug_str, normalize_goal};
use crate::memory::{self};

const DEFAULT_MEMORY_DIR: &str = ".marqdo/agent-memory";
const DEFAULT_KB_DIR: &str = ".marqdo/agent-kb";
const POLICY_NAME: &str = "router.mq.md";

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
        _ => default,
    }
}

fn opt_f64(args: &Value, key: &str, default: f64) -> f64 {
    match args.get(key) {
        None | Some(Value::Null) => default,
        Some(Value::Number(n)) => n.as_f64().unwrap_or(default),
        Some(Value::String(s)) => s.parse().unwrap_or(default),
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

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn yaml_escape(s: &str) -> String {
    if s.is_empty()
        || s.contains(':')
        || s.contains('#')
        || s.contains('|')
        || s.contains('"')
        || s.contains('\n')
    {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

fn resolve_path(rel: &str) -> PathBuf {
    let p = PathBuf::from(rel);
    if p.is_absolute() {
        return p;
    }
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
    base.join(p)
}

fn memory_root(args: &Value) -> PathBuf {
    let rel = opt_text(args, "memory_dir").unwrap_or(DEFAULT_MEMORY_DIR);
    resolve_path(rel)
}

fn policy_path(root: &Path) -> PathBuf {
    root.join("policies").join(POLICY_NAME)
}

#[derive(Clone, Debug)]
struct PolicyRule {
    task_hash: String,
    slug: String,
    skill: String,
    task: String,
    count: i64,
    confidence: f64,
}

fn parse_policy(src: &str) -> Vec<PolicyRule> {
    let mut rules = Vec::new();
    let mut in_table = false;
    let mut header_ok = false;
    for line in src.lines() {
        let t = line.trim();
        if t.starts_with("|") && t.contains("task_hash") {
            in_table = true;
            header_ok = true;
            continue;
        }
        if in_table && t.starts_with("|") && t.contains("---") {
            continue;
        }
        if in_table && t.starts_with("|") {
            let cols: Vec<&str> = t
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().trim_matches('"'))
                .collect();
            if cols.len() >= 5 {
                let count = cols[4].parse().unwrap_or(1);
                let confidence = if cols.len() >= 6 {
                    cols[5].parse().unwrap_or(0.9)
                } else {
                    0.9
                };
                let task = if cols.len() >= 7 {
                    cols[6].to_string()
                } else {
                    String::new()
                };
                rules.push(PolicyRule {
                    task_hash: cols[0].to_string(),
                    slug: cols[1].to_string(),
                    skill: cols[2].to_string(),
                    count,
                    confidence,
                    task,
                });
            }
            continue;
        }
        if in_table && header_ok && !t.starts_with('|') && !t.is_empty() {
            break;
        }
    }
    rules
}

fn write_policy(root: &Path, rules: &[PolicyRule]) -> Result<PathBuf, String> {
    fs::create_dir_all(root.join("policies")).map_err(|e| format!("mkdir policies: {e}"))?;
    let path = policy_path(root);
    let ts = now_rfc3339();
    let mut body = format!(
        "---\ntype: policy\nstatus: compiled\nbackend: marqdo\ngenerated_at: {ts}\nrules: {}\n---\n\n# Adaptive router (compiled Marqdo policy)\n\nDeterministic task → skill routing. No LLM required.\n\n# Rules\n\n| task_hash | slug | skill | resource | count | confidence | task |\n|-----------|------|-------|----------|------:|-----------:|------|\n",
        rules.len()
    );
    for r in rules {
        body.push_str(&format!(
            "| {} | {} | {} | {} | {} | {:.2} | {} |\n",
            r.task_hash,
            yaml_escape(&r.slug),
            yaml_escape(&r.skill),
            yaml_escape(&r.skill), // resource often == skill path for OKF
            r.count,
            r.confidence,
            yaml_escape(&r.task),
        ));
    }
    body.push_str("\n# Fallback\n\nllm\n");
    fs::write(&path, body).map_err(|e| format!("write policy: {e}"))?;
    Ok(path)
}

fn load_rules(root: &Path) -> Vec<PolicyRule> {
    let path = policy_path(root);
    let Ok(src) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    parse_policy(&src)
}

fn match_policy(rules: &[PolicyRule], task: &str, threshold: f64) -> Option<PolicyRule> {
    let th = goal_sig_hex(&normalize_goal(task));
    if let Some(r) = rules.iter().find(|r| r.task_hash == th) {
        return Some(r.clone());
    }
    // Near-match on stored task text / slug.
    let mut best: Option<(f64, PolicyRule)> = None;
    for r in rules {
        let score = memory::task_near_score(task, &r.task);
        if score >= threshold {
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, r.clone()));
            }
        }
        let slug_score = memory::task_near_score(task, &r.slug);
        if slug_score >= threshold {
            if best.as_ref().map(|(s, _)| slug_score > *s).unwrap_or(true) {
                let mut rr = r.clone();
                rr.confidence = slug_score;
                best = Some((slug_score, rr));
            }
        }
    }
    best.map(|(s, mut r)| {
        r.confidence = s;
        r
    })
}

fn jev_available() -> bool {
    if let Ok(p) = std::env::var("MARQDO_JEV") {
        if !p.is_empty() && p != "0" && p != "false" {
            return PathBuf::from(&p).is_file() || p == "1" || p == "true";
        }
    }
    false
}

fn build_route_prompt(task: &str, candidates: &[Value]) -> String {
    let mut s = String::from(
        "You are a Marqdo Agent router. Pick the best skill slug for the task, or say EXPLORE.\n\nReply with exactly one line:\nSKILL:<slug>\nor\nEXPLORE\n\nTask:\n",
    );
    s.push_str(task);
    s.push_str("\n\nCandidates:\n");
    for (i, c) in candidates.iter().enumerate() {
        let slug = c
            .get("slug")
            .or_else(|| c.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let title = c
            .get("title")
            .or_else(|| c.get("goal"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        s.push_str(&format!("{}. {slug} — {title}\n", i + 1));
    }
    s
}

fn candidate_list(args: &Value) -> Result<Vec<Value>, String> {
    if let Some(Value::Array(items)) = args.get("candidates") {
        return Ok(items.clone());
    }
    let kb = opt_text(args, "kb_dir").unwrap_or(DEFAULT_KB_DIR);
    let listed = crate::kb::kb_list_tasks(&json!({ "kb_dir": kb }))?;
    let tasks = listed
        .get("tasks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if !tasks.is_empty() {
        return Ok(tasks);
    }
    let goal = arg_text(args, "task").or_else(|_| arg_text(args, "goal"))?;
    let near = crate::kb::kb_near_match(&json!({
        "kb_dir": kb,
        "goal": goal,
    }))?;
    Ok(near
        .get("candidates")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default())
}

fn enrich_with_kb(rule: &PolicyRule, kb: &str, task: &str) -> Result<Value, String> {
    let hit = crate::kb::kb_lookup(&json!({
        "kb_dir": kb,
        "goal": task,
        "slug": rule.slug,
        "near_match": true,
        "near_threshold": 0.5,
    }))
    .unwrap_or(Value::Null);
    let resource = hit
        .get("resource")
        .cloned()
        .unwrap_or_else(|| Value::String(rule.skill.clone()));
    let skill = hit
        .get("skill")
        .cloned()
        .unwrap_or_else(|| Value::String(rule.skill.clone()));
    let llm_free = hit
        .get("llm_free")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    Ok(json!({
        "backend": "marqdo",
        "mode": "policy",
        "matched": true,
        "needs_llm": false,
        "slug": rule.slug,
        "skill": skill,
        "resource": resource,
        "task_hash": rule.task_hash,
        "confidence": rule.confidence,
        "llm_free": llm_free,
        "count": rule.count,
    }))
}

fn route_marqdo(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    let _ = memory::memory_ensure(&json!({
        "memory_dir": opt_text(args, "memory_dir").unwrap_or(DEFAULT_MEMORY_DIR),
    }));
    let task = arg_text(args, "task").or_else(|_| arg_text(args, "goal"))?;
    let threshold = opt_f64(args, "threshold", 0.78);
    let kb = opt_text(args, "kb_dir").unwrap_or(DEFAULT_KB_DIR);
    let rules = load_rules(&root);
    if let Some(rule) = match_policy(&rules, task, threshold) {
        if rule.confidence >= threshold {
            return enrich_with_kb(&rule, kb, task);
        }
    }
    Ok(json!({
        "backend": "marqdo",
        "mode": "miss",
        "matched": false,
        "needs_llm": false,
        "confidence": 0.0,
    }))
}

fn route_resolve_fallback(args: &Value) -> Result<Value, String> {
    let resolved = memory::resolve_task(args)?;
    let conf = resolved
        .get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let llm_free = resolved
        .get("llm_free")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if conf >= opt_f64(args, "threshold", 0.78) || llm_free {
        return Ok(json!({
            "backend": "kb",
            "mode": resolved.get("mode").cloned().unwrap_or(Value::String("near".into())),
            "matched": true,
            "needs_llm": false,
            "slug": resolved.get("slug").cloned().unwrap_or(Value::Null),
            "skill": resolved.get("skill").cloned().unwrap_or(Value::Null),
            "resource": resolved.get("resource").cloned().unwrap_or(Value::Null),
            "confidence": conf,
            "llm_free": llm_free,
        }));
    }
    Ok(json!({
        "backend": "kb",
        "mode": "miss",
        "matched": false,
        "needs_llm": false,
        "confidence": conf,
        "resolve": resolved,
    }))
}

fn route_model_request(args: &Value, level: &str) -> Result<Value, String> {
    let task = arg_text(args, "task").or_else(|_| arg_text(args, "goal"))?;
    let cands = candidate_list(args)?;
    let prompt = build_route_prompt(task, &cands);
    Ok(json!({
        "backend": level,
        "mode": "needs_llm",
        "matched": false,
        "needs_llm": true,
        "level": level,
        "prompt": prompt,
        "candidates": cands,
        "confidence": 0.0,
    }))
}

fn route_jev(args: &Value) -> Result<Value, String> {
    if !jev_available() {
        return Ok(json!({
            "backend": "jev",
            "mode": "unavailable",
            "matched": false,
            "needs_llm": false,
            "confidence": 0.0,
            "reason": "jev_not_configured",
        }));
    }
    // Optional external binary path; without a stable Jev ABI we surface a model-shaped request
    // so callers can treat Jev like a decision engine when wired later.
    route_model_request(args, "jev")
}

/// Cascade Adaptive Router (Phase 4).
///
/// `backend`: `auto` | `marqdo` | `small-llm` | `jev` | `llm`
///
/// `auto` order: marqdo policy → kb resolve → (optional jev) → small-llm request → llm request.
pub fn route(args: &Value) -> Result<Value, String> {
    let backend = opt_text(args, "backend").unwrap_or("auto");
    let threshold = opt_f64(args, "threshold", 0.78);

    match backend {
        "marqdo" => {
            let hit = route_marqdo(args)?;
            if hit.get("matched").and_then(|v| v.as_bool()) == Some(true) {
                return Ok(hit);
            }
            route_resolve_fallback(args)
        }
        "small-llm" | "small_llm" | "small" => route_model_request(args, "small-llm"),
        "llm" | "large" | "large-llm" => route_model_request(args, "llm"),
        "jev" => route_jev(args),
        "auto" | _ => {
            let mut trace: Vec<Value> = Vec::new();
            let m = route_marqdo(args)?;
            trace.push(json!({"try": "marqdo", "matched": m.get("matched")}));
            if m.get("matched").and_then(|v| v.as_bool()) == Some(true)
                && m.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0) >= threshold
            {
                let mut out = m;
                if let Some(obj) = out.as_object_mut() {
                    obj.insert("trace".into(), Value::Array(trace));
                    obj.insert("backend".into(), Value::String("auto/marqdo".into()));
                }
                return Ok(out);
            }
            let kb = route_resolve_fallback(args)?;
            trace.push(json!({"try": "kb", "matched": kb.get("matched")}));
            if kb.get("matched").and_then(|v| v.as_bool()) == Some(true) {
                let mut out = kb;
                if let Some(obj) = out.as_object_mut() {
                    obj.insert("trace".into(), Value::Array(trace));
                    obj.insert("backend".into(), Value::String("auto/kb".into()));
                }
                return Ok(out);
            }
            if jev_available() {
                let j = route_jev(args)?;
                trace.push(json!({"try": "jev", "mode": j.get("mode")}));
                if j.get("needs_llm").and_then(|v| v.as_bool()) == Some(true) {
                    let mut out = j;
                    if let Some(obj) = out.as_object_mut() {
                        obj.insert("trace".into(), Value::Array(trace));
                    }
                    return Ok(out);
                }
            }
            // Prefer small-llm request in auto when nothing matched (caller may use large).
            let mut req = route_model_request(args, "small-llm")?;
            if let Some(obj) = req.as_object_mut() {
                obj.insert("trace".into(), Value::Array(trace));
                obj.insert("backend".into(), Value::String("auto/small-llm".into()));
            }
            Ok(req)
        }
    }
}

/// Apply a router model reply (`SKILL:<slug>` / `EXPLORE`) to a concrete skill hit.
pub fn route_apply(args: &Value) -> Result<Value, String> {
    let reply = arg_text(args, "reply")?;
    let kb = opt_text(args, "kb_dir").unwrap_or(DEFAULT_KB_DIR);
    let task = arg_text(args, "task").or_else(|_| arg_text(args, "goal"))?;
    let line = reply
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let upper = line.to_ascii_uppercase();
    if upper.starts_with("EXPLORE") || upper == "LLM" || upper == "NEW" {
        return Ok(json!({
            "matched": false,
            "mode": "explore",
            "decision": "EXPLORE",
            "needs_llm": false,
        }));
    }
    let slug = if let Some(rest) = line.strip_prefix("SKILL:") {
        rest.trim()
    } else if let Some(rest) = line.strip_prefix("skill:") {
        rest.trim()
    } else if let Some(rest) = line.strip_prefix("调用：") {
        rest.trim()
    } else {
        line.trim()
    };
    if slug.is_empty() {
        return Ok(json!({
            "matched": false,
            "mode": "explore",
            "decision": "EXPLORE",
            "reason": "empty_slug",
        }));
    }
    let hit = crate::kb::kb_lookup(&json!({
        "kb_dir": kb,
        "goal": task,
        "slug": slug,
        "near_match": true,
        "near_threshold": 0.5,
    }))?;
    if hit.is_null() {
        return Ok(json!({
            "matched": false,
            "mode": "miss",
            "decision": "SKILL",
            "slug": slug,
            "reason": "slug_not_in_kb",
        }));
    }
    Ok(json!({
        "matched": true,
        "mode": "routed",
        "decision": "SKILL",
        "backend": opt_text(args, "backend").unwrap_or("llm"),
        "slug": slug,
        "skill": hit.get("skill").cloned().unwrap_or(Value::Null),
        "resource": hit.get("resource").cloned().unwrap_or(Value::Null),
        "llm_free": hit.get("llm_free").cloned().unwrap_or(Value::Bool(false)),
        "confidence": 0.85,
        "needs_llm": false,
    }))
}

/// Compile policy from successful episode skill selections (Policy Compilation).
pub fn compile_policy(args: &Value) -> Result<Value, String> {
    let root = memory_root(args);
    let mem_arg = opt_text(args, "memory_dir").unwrap_or(DEFAULT_MEMORY_DIR);
    let _ = memory::memory_ensure(&json!({ "memory_dir": mem_arg }));
    let min_evidence = opt_i64(args, "min_evidence", opt_i64(args, "improve_every", 3));
    let listed = memory::list_episodes(&json!({
        "memory_dir": mem_arg,
        "status": "success",
        "limit": 500,
    }))?;
    let eps = listed
        .get("episodes")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // task_hash → (slug → count, sample task, skill path)
    let mut by_hash: HashMap<String, HashMap<String, (i64, String, String)>> = HashMap::new();
    for e in &eps {
        let id = e.get("id").and_then(|v| v.as_str()).unwrap_or("");
        if id.is_empty() {
            continue;
        }
        let full = memory::get_episode(&json!({
            "memory_dir": mem_arg,
            "id": id,
        }))?;
        if full.is_null() {
            continue;
        }
        let src = full.get("source").and_then(|v| v.as_str()).unwrap_or("");
        let skill = extract_fm(src, "skill").unwrap_or_default();
        if skill.is_empty() || skill == "none" || skill == "null" {
            continue;
        }
        let task_hash = extract_fm(src, "task_hash")
            .or_else(|| e.get("task_hash").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .unwrap_or_default();
        if task_hash.is_empty() {
            continue;
        }
        let task = full
            .get("task")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let slug = extract_fm(src, "slug")
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| goal_slug_str(&task));
        // Prefer skill path as key when it looks like a path; else slug.
        let key = if skill.contains('/') || skill.ends_with(".md") {
            skill.clone()
        } else {
            slug.clone()
        };
        let entry = by_hash.entry(task_hash).or_default();
        let cell = entry.entry(key).or_insert((0, task, skill));
        cell.0 += 1;
    }

    let mut rules: Vec<PolicyRule> = Vec::new();
    for (task_hash, skills) in by_hash {
        let Some((skill_key, (count, task, skill_path))) =
            skills.into_iter().max_by_key(|(_, (c, _, _))| *c)
        else {
            continue;
        };
        if count < min_evidence {
            continue;
        }
        let slug = if skill_key.contains('/') {
            Path::new(&skill_key)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&skill_key)
                .trim_end_matches(".mq")
                .to_string()
        } else {
            skill_key
        };
        let confidence = (count as f64 / (count as f64 + 1.0)).min(0.99);
        rules.push(PolicyRule {
            task_hash,
            slug,
            skill: skill_path,
            task,
            count,
            confidence,
        });
    }
    rules.sort_by(|a, b| b.count.cmp(&a.count));

    if rules.is_empty() && !opt_bool(args, "force", false) {
        return Ok(json!({
            "ok": false,
            "compiled": false,
            "reason": "insufficient_stable_mappings",
            "min_evidence": min_evidence,
        }));
    }

    let path = write_policy(&root, &rules)?;
    Ok(json!({
        "ok": true,
        "compiled": true,
        "path": path.to_string_lossy(),
        "rules": rules.len() as i64,
        "min_evidence": min_evidence,
    }))
}

fn extract_fm(source: &str, key: &str) -> Option<String> {
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

/// Learn gate for policy: compile when enough skill-tagged episodes exist.
pub fn maybe_compile_policy(args: &Value) -> Result<Value, String> {
    let compiled = compile_policy(args)?;
    Ok(json!({
        "policy_learned": compiled.get("ok").and_then(|v| v.as_bool()).unwrap_or(false),
        "compile": compiled,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn compile_and_route_marqdo() {
        let dir = std::env::temp_dir().join(format!("marqdo-route-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mem = dir.join("agent-memory");
        let kb = dir.join("agent-kb");
        let mem_s = mem.to_string_lossy().into_owned();
        let kb_s = kb.to_string_lossy().into_owned();
        let task = "Reply with exactly the word pong and nothing else.";

        // Seed a solidified skill via memory compile path.
        for _ in 0..3 {
            memory::record_episode(&json!({
                "memory_dir": mem_s,
                "task": task,
                "status": "ok",
                "mode": "explore",
                "result": "pong",
                "skill": "concepts/skills/reply-with-exactly-the-word-pong-and-nothing-els.md",
                "llm_calls": 1,
            }))
            .unwrap();
        }
        let learned = memory::maybe_learn(&json!({
            "memory_dir": mem_s,
            "kb_dir": kb_s,
            "task": task,
            "improve_every": 3,
            "promote": true,
        }))
        .unwrap();
        assert_eq!(learned.get("learned").and_then(|v| v.as_bool()), Some(true));

        // Tag episodes with skill path for policy mining (re-record with skill).
        for _ in 0..3 {
            memory::record_episode(&json!({
                "memory_dir": mem_s,
                "task": task,
                "status": "ok",
                "mode": "compiled",
                "result": "pong",
                "skill": "concepts/skills/reply-with-exactly-the-word-pong-and-nothing-els.md",
                "llm_calls": 0,
            }))
            .unwrap();
        }

        let pol = compile_policy(&json!({
            "memory_dir": mem_s,
            "min_evidence": 3,
        }))
        .unwrap();
        assert_eq!(pol.get("ok").and_then(|v| v.as_bool()), Some(true), "{pol}");

        let routed = route(&json!({
            "memory_dir": mem_s,
            "kb_dir": kb_s,
            "task": task,
            "backend": "auto",
        }))
        .unwrap();
        assert_eq!(
            routed.get("matched").and_then(|v| v.as_bool()),
            Some(true),
            "{routed}"
        );
        assert_eq!(
            routed.get("needs_llm").and_then(|v| v.as_bool()),
            Some(false),
            "{routed}"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
