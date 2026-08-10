//! OKF agent-kb helpers (plugin-side). No dependency on marqdo `src/host`.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

pub fn normalize_goal(raw: &str) -> String {
    let t = raw.trim();
    let mut out = String::with_capacity(t.len());
    let mut prev_space = false;
    for c in t.chars() {
        if c.is_whitespace() {
            if !prev_space && !out.is_empty() {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

pub fn goal_sig_hex(material: &str) -> String {
    let mut h: u64 = 5381;
    for b in material.as_bytes() {
        h = h.wrapping_mul(33).wrapping_add(u64::from(*b));
    }
    format!("{h:016x}")[..12].to_string()
}

pub fn goal_slug_str(raw: &str) -> String {
    let n = normalize_goal(raw);
    let mut out = String::new();
    for c in n.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !c.is_ascii()
            && !c.is_whitespace()
            && !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        {
            if c.is_alphanumeric() {
                out.push(c);
            } else if !out.is_empty() && !out.ends_with('-') {
                out.push('-');
            }
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    let mut truncated = String::new();
    for c in out.chars() {
        if truncated.chars().count() >= 48 {
            break;
        }
        truncated.push(c);
    }
    while truncated.ends_with('-') {
        truncated.pop();
    }
    if truncated.is_empty() {
        "task".into()
    } else {
        truncated
    }
}

fn tools_fingerprint(tools: Option<&Value>) -> String {
    match tools {
        None | Some(Value::Null) => String::new(),
        Some(Value::Array(items)) => {
            let mut names: Vec<String> = items
                .iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s.trim().to_string()),
                    Value::Object(entries) => entries
                        .get("工具")
                        .or_else(|| entries.get("name"))
                        .or_else(|| entries.get("tool"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.trim().to_string()),
                    _ => None,
                })
                .filter(|s| !s.is_empty())
                .collect();
            names.sort();
            names.join("\n")
        }
        Some(Value::String(s)) => {
            let t = s.trim();
            if t.is_empty() {
                String::new()
            } else {
                t.to_string()
            }
        }
        _ => String::new(),
    }
}

fn material_for(goal: &str, tools: Option<&Value>) -> String {
    let mut material = normalize_goal(goal);
    let fp = tools_fingerprint(tools);
    if !fp.is_empty() {
        material.push('\n');
        material.push_str(&fp);
    }
    material
}

fn resolve_path(rel: &str) -> PathBuf {
    let p = PathBuf::from(rel);
    if p.is_absolute() {
        return p;
    }
    // Prefer host cwd (entry file directory) via host_query when available.
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

/// Parse Task FM `aliases:` as inline list `[a, b]`, comma string, or YAML `- item` block.
fn extract_fm_aliases(source: &str) -> Vec<String> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut i = 1usize;
    while i < lines.len() {
        let t = lines[i].trim();
        if t == "---" {
            break;
        }
        if let Some(rest) = t.strip_prefix("aliases:") {
            let rest = rest.trim();
            if rest.is_empty() {
                i += 1;
                while i < lines.len() {
                    let body = lines[i];
                    let bt = body.trim();
                    if bt == "---" {
                        break;
                    }
                    if let Some(item) = bt.strip_prefix("- ") {
                        let a = normalize_goal(item.trim().trim_matches('"'));
                        if !a.is_empty() {
                            out.push(a);
                        }
                        i += 1;
                        continue;
                    }
                    if bt == "-" {
                        i += 1;
                        continue;
                    }
                    if !body.starts_with(' ') && !body.starts_with('\t') && !bt.is_empty() {
                        break;
                    }
                    i += 1;
                }
                break;
            }
            let inline = rest.trim_matches(|c| c == '[' || c == ']');
            for part in inline.split(',') {
                let a = normalize_goal(part.trim().trim_matches('"'));
                if !a.is_empty() {
                    out.push(a);
                }
            }
            break;
        }
        i += 1;
    }
    out
}

fn format_aliases_yaml(aliases: &[String]) -> String {
    if aliases.is_empty() {
        return String::new();
    }
    let mut s = String::from("aliases:\n");
    for a in aliases {
        s.push_str("  - ");
        s.push_str(&yaml_escape(a));
        s.push('\n');
    }
    s
}

fn set_fm_field(source: &str, key: &str, value: &str) -> String {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let lines: Vec<&str> = source.lines().collect();
    if lines.first().map(|l| l.trim()) != Some("---") {
        return source.to_string();
    }
    let prefix = format!("{key}:");
    let mut out = Vec::new();
    let mut replaced = false;
    let mut in_fm = true;
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            out.push((*line).to_string());
            continue;
        }
        if in_fm && line.trim() == "---" {
            if !replaced {
                out.push(format!("{key}: {value}"));
            }
            in_fm = false;
            out.push((*line).to_string());
            continue;
        }
        if in_fm && line.trim().starts_with(&prefix) {
            out.push(format!("{key}: {value}"));
            replaced = true;
        } else {
            out.push((*line).to_string());
        }
    }
    out.join("\n") + if source.ends_with('\n') { "\n" } else { "" }
}

fn kb_rel(kb_dir: &str, parts: &str) -> String {
    format!("{}/{}", kb_dir.trim_end_matches('/'), parts)
}

fn resolve_skill_paths(
    root: &Path,
    sig: &str,
    slug_hint: &str,
) -> Option<(PathBuf, PathBuf, PathBuf, String)> {
    let tasks_dir = root.join("concepts/tasks");
    let skills_dir = root.join("concepts/skills");
    let res_dir = root.join("resources");

    let sig4 = if sig.len() >= 4 { &sig[..4] } else { sig };
    for slug in [slug_hint.to_string(), format!("{slug_hint}-{sig4}")] {
        let task = tasks_dir.join(format!("{slug}.md"));
        if task.is_file() {
            if let Ok(src) = fs::read_to_string(&task) {
                let fm_sig = extract_fm_field(&src, "sig");
                if fm_sig.as_deref() == Some(sig) || fm_sig.is_none() {
                    let skill = skills_dir.join(format!("{slug}.md"));
                    let res = res_dir.join(format!("{slug}.mq.md"));
                    if skill.is_file() && res.is_file() {
                        return Some((task, skill, res, slug));
                    }
                }
            }
        }
    }

    if tasks_dir.is_dir() {
        if let Ok(rd) = fs::read_dir(&tasks_dir) {
            for ent in rd.flatten() {
                let p = ent.path();
                if p.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let Ok(src) = fs::read_to_string(&p) else {
                    continue;
                };
                if extract_fm_field(&src, "sig").as_deref() == Some(sig) {
                    let stem = p.file_stem()?.to_string_lossy().into_owned();
                    let skill = skills_dir.join(format!("{stem}.md"));
                    let res = res_dir.join(format!("{stem}.mq.md"));
                    if skill.is_file() && res.is_file() {
                        return Some((p, skill, res, stem));
                    }
                }
            }
        }
    }

    let task = tasks_dir.join(format!("{sig}.md"));
    let skill = skills_dir.join(format!("{sig}.md"));
    let res = res_dir.join(format!("{sig}.mq.md"));
    if task.is_file() && skill.is_file() && res.is_file() {
        return Some((task, skill, res, sig.to_string()));
    }
    None
}

/// Second-pass exact match: `normalize_goal(query)` equals a Task FM alias.
fn resolve_skill_paths_by_alias(
    root: &Path,
    goal_norm: &str,
) -> Option<(PathBuf, PathBuf, PathBuf, String)> {
    if goal_norm.is_empty() {
        return None;
    }
    let tasks_dir = root.join("concepts/tasks");
    let skills_dir = root.join("concepts/skills");
    let res_dir = root.join("resources");
    if !tasks_dir.is_dir() {
        return None;
    }
    let rd = fs::read_dir(&tasks_dir).ok()?;
    for ent in rd.flatten() {
        let p = ent.path();
        if p.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        let aliases = extract_fm_aliases(&src);
        if !aliases.iter().any(|a| a == goal_norm) {
            continue;
        }
        let stem = p.file_stem()?.to_string_lossy().into_owned();
        let skill = skills_dir.join(format!("{stem}.md"));
        let res = res_dir.join(format!("{stem}.mq.md"));
        if skill.is_file() && res.is_file() {
            return Some((p, skill, res, stem));
        }
    }
    None
}

fn unique_slug(root: &Path, base: &str, sig: &str) -> String {
    let tasks = root.join("concepts/tasks");
    let candidate = tasks.join(format!("{base}.md"));
    if !candidate.is_file() {
        return base.to_string();
    }
    if let Ok(src) = fs::read_to_string(&candidate) {
        if extract_fm_field(&src, "sig").as_deref() == Some(sig) {
            return base.to_string();
        }
    }
    let sig4 = if sig.len() >= 4 { &sig[..4] } else { sig };
    format!("{base}-{sig4}")
}

fn llm_free_score(src: &str) -> (bool, i64) {
    let lower = src.to_ascii_lowercase();
    let has_step = src.contains(".step") || src.contains(".单步") || src.contains("agent.step");
    let has_llm = lower.contains("llm.llm")
        || lower.contains("llm.complete")
        || src.contains("大模型")
        || src.contains(".complete");
    let free = !has_step && !has_llm;
    let len = src.len() as i64;
    let quality = if free {
        1_000_000 - len.min(999_999)
    } else {
        100_000 - len.min(99_999)
    };
    (free, quality)
}

fn write_task_concept(
    path: &Path,
    sig: &str,
    slug: &str,
    goal: &str,
    now: &str,
    aliases: &[String],
) -> Result<(), String> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| format!("mkdir: {e}"))?;
    }
    let title: String = goal.chars().take(48).collect();
    let mut merged: Vec<String> = Vec::new();
    if path.is_file() {
        if let Ok(old) = fs::read_to_string(path) {
            for a in extract_fm_aliases(&old) {
                if !merged.iter().any(|x| x == &a) {
                    merged.push(a);
                }
            }
        }
    }
    for a in aliases {
        let a = normalize_goal(a);
        if a.is_empty() || a == goal {
            continue;
        }
        if !merged.iter().any(|x| x == &a) {
            merged.push(a);
        }
    }
    let alias_block = format_aliases_yaml(&merged);
    let body = format!(
        r#"---
type: Marqdo Task
title: {title}
description: {goal_e}
sig: {sig}
status: stable
tags: [agent-task]
skill: ../skills/{slug}.md
{alias_block}generated:
  by: marqdo-agent/plan
  at: {now}
verified:
  by: marqdo-agent/spawn
  at: {now}
---

# Task

See [skill](../skills/{slug}.md).
"#,
        title = yaml_escape(&title),
        goal_e = yaml_escape(goal),
        sig = sig,
        slug = slug,
        alias_block = alias_block,
        now = now,
    );
    fs::write(path, body).map_err(|e| format!("write task: {e}"))
}

fn write_skill_concept(
    path: &Path,
    sig: &str,
    slug: &str,
    goal: &str,
    status: &str,
    llm_free: bool,
    quality: i64,
    hits: i64,
    now: &str,
) -> Result<(), String> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(|e| format!("mkdir: {e}"))?;
    }
    let title: String = goal.chars().take(48).collect();
    let body = format!(
        r#"---
type: Marqdo Agent Skill
title: {title}
sig: {sig}
resource: ../../resources/{slug}.mq.md
status: {status}
llm_free: {llm_free}
quality: {quality}
hits: {hits}
generated:
  by: marqdo-agent/plan
  at: {now}
verified:
  by: marqdo-agent/spawn
  at: {now}
---

# Skill

Prefer spawning `resource` over re-planning.
"#,
        title = yaml_escape(&title),
        sig = sig,
        slug = slug,
        status = status,
        llm_free = if llm_free { "true" } else { "false" },
        quality = quality,
        hits = hits,
        now = now,
    );
    fs::write(path, body).map_err(|e| format!("write skill: {e}"))
}

fn refresh_index(root: &Path, slug: &str, goal: &str) -> Result<(), String> {
    let index = root.join("index.md");
    let line = format!(
        "- [`{slug}`](concepts/tasks/{slug}.md) — {}\n",
        goal.chars().take(60).collect::<String>()
    );
    if index.is_file() {
        let mut cur = fs::read_to_string(&index).map_err(|e| format!("read index: {e}"))?;
        let marker = format!("concepts/tasks/{slug}.md");
        if !cur.contains(&marker) {
            if !cur.ends_with('\n') {
                cur.push('\n');
            }
            cur.push_str(&line);
            fs::write(&index, cur).map_err(|e| format!("write index: {e}"))?;
        }
    } else {
        let body = format!(
            "---\ntype: Marqdo Catalog\ntitle: agent-kb\ngenerated:\n  by: marqdo-agent/plan\n---\n\n# Agent knowledge base\n\n{line}"
        );
        fs::write(&index, body).map_err(|e| format!("write index: {e}"))?;
    }
    Ok(())
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

fn arg_text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("missing text `{key}`"))
}

fn opt_tools(args: &Value) -> Option<&Value> {
    args.get("tools").filter(|t| !t.is_null())
}

pub fn goal_sig(args: &Value) -> Result<Value, String> {
    let goal = arg_text(args, "goal")?;
    Ok(Value::String(goal_sig_hex(&material_for(
        goal,
        opt_tools(args),
    ))))
}

pub fn goal_slug(args: &Value) -> Result<Value, String> {
    let goal = arg_text(args, "goal")?;
    Ok(Value::String(goal_slug_str(goal)))
}

pub fn kb_lookup(args: &Value) -> Result<Value, String> {
    let kb = arg_text(args, "kb_dir")?;
    let goal = arg_text(args, "goal")?;
    let goal_n = normalize_goal(goal);
    let query_sig = goal_sig_hex(&material_for(goal, opt_tools(args)));
    let slug_hint = goal_slug_str(goal);
    let root = resolve_path(kb);
    let mut match_kind = "exact";
    let resolved = resolve_skill_paths(&root, &query_sig, &slug_hint).or_else(|| {
        match_kind = "alias";
        resolve_skill_paths_by_alias(&root, &goal_n)
    });
    let Some((_task, skill_path, _res, slug)) = resolved else {
        return Ok(Value::Null);
    };

    let skill_src =
        fs::read_to_string(&skill_path).map_err(|e| format!("kb_lookup read skill: {e}"))?;
    let mut status = extract_fm_field(&skill_src, "status").unwrap_or_else(|| "stable".into());
    if status == "deprecated" || status == "draft" {
        return Ok(Value::Null);
    }
    let mut llm_free = extract_fm_field(&skill_src, "llm_free")
        .map(|s| s == "true" || s == "True")
        .unwrap_or(false);
    let mut quality = extract_fm_field(&skill_src, "quality")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let hits = extract_fm_field(&skill_src, "hits")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0);
    let skill_sig = extract_fm_field(&skill_src, "sig").unwrap_or_else(|| query_sig.clone());

    let res_path = root.join("resources").join(format!("{slug}.mq.md"));
    if res_path.is_file() {
        if let Ok(res_src) = fs::read_to_string(&res_path) {
            let (res_free, res_q) = llm_free_score(&res_src);
            if res_free {
                llm_free = true;
                status = "stable".into();
            }
            if res_q > quality {
                quality = res_q;
            }
        }
    }

    Ok(json!({
        "hit": true,
        "match": match_kind,
        "sig": skill_sig,
        "query_sig": query_sig,
        "slug": slug,
        "status": status,
        "llm_free": llm_free,
        "quality": quality,
        "hits": hits,
        "resource": kb_rel(kb, &format!("resources/{slug}.mq.md")),
        "task": kb_rel(kb, &format!("concepts/tasks/{slug}.md")),
        "skill": kb_rel(kb, &format!("concepts/skills/{slug}.md")),
    }))
}

pub fn kb_record_hit(args: &Value) -> Result<Value, String> {
    let kb = arg_text(args, "kb_dir")?;
    let goal = arg_text(args, "goal")?;
    let goal_n = normalize_goal(goal);
    let sig = goal_sig_hex(&material_for(goal, opt_tools(args)));
    let slug_hint = goal_slug_str(goal);
    let root = resolve_path(kb);
    let Some((_t, skill_path, _r, slug)) = resolve_skill_paths(&root, &sig, &slug_hint)
        .or_else(|| resolve_skill_paths_by_alias(&root, &goal_n))
    else {
        return Ok(Value::Null);
    };
    let src = fs::read_to_string(&skill_path).map_err(|e| format!("read skill: {e}"))?;
    let status = extract_fm_field(&src, "status").unwrap_or_else(|| "stable".into());
    let llm_free = extract_fm_field(&src, "llm_free")
        .map(|s| s == "true" || s == "True")
        .unwrap_or(false);
    let hits = extract_fm_field(&src, "hits")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0)
        + 1;
    let next = set_fm_field(&src, "hits", &hits.to_string());
    fs::write(&skill_path, next).map_err(|e| format!("write skill hits: {e}"))?;

    let every = match args.get("improve_every") {
        None | Some(Value::Null) => 0i64,
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0),
        Some(Value::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    };
    let improve_due = every > 0
        && status == "candidate"
        && !llm_free
        && hits > 0
        && hits % every == 0;

    Ok(json!({
        "hits": hits,
        "slug": slug,
        "sig": sig,
        "status": status,
        "llm_free": llm_free,
        "improve_due": improve_due,
    }))
}

pub fn kb_promote(args: &Value) -> Result<Value, String> {
    let kb = arg_text(args, "kb_dir")?;
    let goal = arg_text(args, "goal")?;
    let workbook = arg_text(args, "workbook")?;
    let goal_n = normalize_goal(goal);
    let sig = goal_sig_hex(&material_for(goal, opt_tools(args)));
    let base_slug = goal_slug_str(&goal_n);
    let root = resolve_path(kb);
    let slug = unique_slug(&root, &base_slug, &sig);
    let src_path = resolve_path(workbook);
    let src = fs::read_to_string(&src_path)
        .map_err(|e| format!("kb_promote read workbook: {e}"))?;
    let (llm_free, quality) = llm_free_score(&src);
    let status = if llm_free { "stable" } else { "candidate" };

    let skill_path = root.join("concepts/skills").join(format!("{slug}.md"));
    let mut prev_hits = 0i64;
    if skill_path.is_file() {
        let old = fs::read_to_string(&skill_path).unwrap_or_default();
        prev_hits = extract_fm_field(&old, "hits")
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        let old_q = extract_fm_field(&old, "quality")
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        if quality < old_q {
            return Ok(json!({
                "promoted": false,
                "reason": "quality_worse",
                "sig": sig,
                "slug": slug,
            }));
        }
    }

    let res_path = root.join("resources").join(format!("{slug}.mq.md"));
    if let Some(p) = res_path.parent() {
        fs::create_dir_all(p).map_err(|e| format!("mkdir resources: {e}"))?;
    }
    fs::write(&res_path, &src).map_err(|e| format!("write resource: {e}"))?;

    let now = now_rfc3339();
    // Optional extra aliases from caller (JSON array or single string).
    let mut extra_aliases: Vec<String> = Vec::new();
    if let Some(Value::Array(items)) = args.get("aliases") {
        for v in items {
            if let Some(s) = v.as_str() {
                extra_aliases.push(s.to_string());
            }
        }
    } else if let Some(Value::String(s)) = args.get("aliases") {
        extra_aliases.push(s.clone());
    }
    write_task_concept(
        &root.join("concepts/tasks").join(format!("{slug}.md")),
        &sig,
        &slug,
        &goal_n,
        &now,
        &extra_aliases,
    )?;
    write_skill_concept(
        &skill_path,
        &sig,
        &slug,
        &goal_n,
        status,
        llm_free,
        quality,
        prev_hits,
        &now,
    )?;
    refresh_index(&root, &slug, &goal_n)?;

    Ok(json!({
        "promoted": true,
        "sig": sig,
        "slug": slug,
        "resource": kb_rel(kb, &format!("resources/{slug}.mq.md")),
        "llm_free": llm_free,
        "quality": quality,
        "status": status,
        "hits": prev_hits,
    }))
}

fn extract_result_text(observation: Option<&Value>) -> Option<String> {
    let obs = observation?;
    if let Some(v) = obs.get("value") {
        if let Some(t) = value_as_result_text(v) {
            return Some(t);
        }
    }
    let last_ok = obs.get("last_ok")?;
    let result_val = match last_ok {
        Value::Object(_) => last_ok.get("result")?.clone(),
        Value::String(s) => {
            let parsed: Value = serde_json::from_str(s).ok()?;
            match &parsed {
                Value::Object(_) => parsed.get("result")?.clone(),
                other => other.clone(),
            }
        }
        _ => return None,
    };
    value_as_result_text(&result_val)
}

fn value_as_result_text(result_val: &Value) -> Option<String> {
    match result_val {
        Value::Null => None,
        Value::String(t) => {
            let t = t.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        }
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(if *b { "true" } else { "false" }.into()),
        _ => None,
    }
}

fn marqdo_quoted_text(s: &str) -> String {
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

fn needs_step_solidify(src: &str) -> bool {
    src.contains(".step") || src.contains(".单步") || src.contains("worker.step")
}

pub fn workbook_solidify(args: &Value) -> Result<Value, String> {
    let path = arg_text(args, "path")?;
    let p = resolve_path(path);
    let src = fs::read_to_string(&p).map_err(|e| format!("solidify read: {e}"))?;
    if !needs_step_solidify(&src) {
        return Ok(json!({
            "solidified": false,
            "reason": "already_solid",
        }));
    }
    let Some(result) = extract_result_text(args.get("observation")) else {
        return Ok(json!({
            "solidified": false,
            "reason": "no_result",
        }));
    };
    let lit = marqdo_quoted_text(&result);
    let body = format!(
        "---\ntitle: agent workbook\n---\n\n# main\n\n*`result` = {lit} *\n**`result`**\n"
    );
    fs::write(&p, body).map_err(|e| format!("solidify write: {e}"))?;
    Ok(json!({
        "solidified": true,
        "result": result,
        "path": path,
    }))
}

pub fn kb_task_files(args: &Value) -> Result<Value, String> {
    let kb = arg_text(args, "kb_dir")?;
    let goal = arg_text(args, "goal")?;
    let _sig = goal_sig_hex(&material_for(goal, opt_tools(args)));
    let slug = goal_slug_str(goal);
    let root = resolve_path(kb);
    let mut files: Vec<Value> = Vec::new();

    let res = root.join("resources").join(format!("{slug}.mq.md"));
    if res.is_file() {
        files.push(Value::String(res.to_string_lossy().into_owned()));
    }

    let explore = root.join("explore").join(&slug);
    if explore.is_dir() {
        if let Ok(rd) = fs::read_dir(&explore) {
            for ent in rd.flatten() {
                let p = ent.path();
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.ends_with(".mq.md") {
                    files.push(Value::String(p.to_string_lossy().into_owned()));
                }
            }
        }
    }

    let runs = match args.get("runs_dir").and_then(|v| v.as_str()) {
        None => resolve_path(".marqdo/agent-runs"),
        Some(s) => resolve_path(s),
    };
    if runs.is_dir() {
        let prefix = format!("workbook-{slug}-");
        if let Ok(rd) = fs::read_dir(&runs) {
            for ent in rd.flatten() {
                let p = ent.path();
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with(&prefix) && name.ends_with(".mq.md") {
                    files.push(Value::String(p.to_string_lossy().into_owned()));
                }
            }
        }
    }

    let count = files.len() as i64;
    Ok(json!({
        "count": count,
        "slug": slug,
        "files": files,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_readable() {
        let s = goal_slug_str("Reply with exactly the word pong and nothing else.");
        assert!(s.contains("pong"), "{s}");
        assert!(!s.contains(' '));
    }

    #[test]
    fn sig_stable() {
        let a = goal_sig_hex("hello");
        assert_eq!(a, goal_sig_hex("hello"));
        assert_eq!(a.len(), 12);
    }
}
