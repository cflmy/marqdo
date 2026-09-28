//! Engineering Knowledge preflight / reuse (reads `.marqdo` projections).
//! Does not call host_* — consumes compiled knowledge graph artifacts.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

fn arg_text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("missing string arg `{key}`"))
}

fn arg_f64(args: &Value, key: &str, default: f64) -> f64 {
    match args.get(key) {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(default),
        Some(Value::String(s)) => s.parse().unwrap_or(default),
        _ => default,
    }
}

fn arg_usize(args: &Value, key: &str, default: usize) -> usize {
    match args.get(key) {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(default as u64) as usize,
        Some(Value::String(s)) => s.parse().unwrap_or(default),
        _ => default,
    }
}

fn char_bigram_tf(text: &str) -> HashMap<String, f64> {
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    let mut m = HashMap::new();
    if chars.is_empty() {
        return m;
    }
    if chars.len() == 1 {
        m.insert(chars[0].to_string(), 1.0);
        return m;
    }
    for w in chars.windows(2) {
        let key: String = w.iter().collect();
        *m.entry(key).or_insert(0.0) += 1.0;
    }
    m
}

fn cosine_tf(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    let mut na = 0.0;
    let mut nb = 0.0;
    for v in a.values() {
        na += v * v;
    }
    for v in b.values() {
        nb += v * v;
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    let mut dot = 0.0;
    for (k, va) in a {
        if let Some(vb) = b.get(k) {
            dot += va * vb;
        }
    }
    dot / (na.sqrt() * nb.sqrt())
}


fn resolve_marqdo_root(start: &str) -> PathBuf {
    let mut cur = PathBuf::from(start);
    if cur.is_file() {
        if let Some(p) = cur.parent() {
            cur = p.to_path_buf();
        }
    }
    for _ in 0..8 {
        let cand = cur.join(".marqdo");
        if cand.join("engineering.yaml").exists() || cand.join("graph/graph.json").exists() {
            return cand;
        }
        if !cur.pop() {
            break;
        }
    }
    PathBuf::from(start).join(".marqdo")
}

#[derive(Clone)]
struct CapHit {
    id: String,
    name: String,
    score: f64,
    implemented_by: Vec<String>,
}

fn load_capabilities(marqdo: &Path) -> Vec<(String, String, Vec<String>, String)> {
    // returns (id, name, implemented_by, material)
    let mut out = Vec::new();
    let eng = marqdo.join("engineering.yaml");
    if let Ok(raw) = fs::read_to_string(&eng) {
        let mut cur_id = String::new();
        let mut cur_name = String::new();
        let mut impls = Vec::new();
        let mut in_caps = false;
        let mut in_impl = false;
        for line in raw.lines() {
            let t = line.trim_end();
            if t.trim() == "capabilities:" {
                in_caps = true;
                continue;
            }
            if in_caps && !line.starts_with(' ') && !line.starts_with('\t') && !t.trim().is_empty()
            {
                if t.trim() == "knowledge:" {
                    if !cur_id.is_empty() {
                        out.push((
                            cur_id.clone(),
                            cur_name.clone(),
                            impls.clone(),
                            format!("{cur_name} {cur_id}"),
                        ));
                    }
                    break;
                }
            }
            if !in_caps {
                continue;
            }
            let trimmed = t.trim();
            if let Some(rest) = trimmed.strip_prefix("- id:") {
                if !cur_id.is_empty() {
                    out.push((
                        cur_id.clone(),
                        cur_name.clone(),
                        impls.clone(),
                        format!("{cur_name} {cur_id}"),
                    ));
                }
                cur_id = rest.trim().trim_matches('"').to_string();
                cur_name.clear();
                impls.clear();
                in_impl = false;
            } else if let Some(rest) = trimmed.strip_prefix("name:") {
                cur_name = rest.trim().trim_matches('"').to_string();
            } else if trimmed == "implemented_by:" {
                in_impl = true;
            } else if in_impl {
                if let Some(rest) = trimmed.strip_prefix("- ") {
                    impls.push(rest.trim().trim_matches('"').to_string());
                } else if !trimmed.is_empty() && !line.starts_with(' ') {
                    in_impl = false;
                }
            }
        }
        if !cur_id.is_empty() {
            out.push((
                cur_id.clone(),
                cur_name.clone(),
                impls,
                format!("{cur_name} {cur_id}"),
            ));
        }
    }

    // Also scan catalog/capabilities/*.md
    let cap_dir = marqdo.join("catalog/capabilities");
    if let Ok(rd) = fs::read_dir(cap_dir) {
        for ent in rd.flatten() {
            let p = ent.path();
            if !p
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.ends_with(".md"))
                .unwrap_or(false)
            {
                continue;
            }
            let Ok(src) = fs::read_to_string(&p) else {
                continue;
            };
            let id = fm_field(&src, "id").unwrap_or_else(|| {
                p.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("cap")
                    .to_string()
            });
            let name = fm_field(&src, "name").unwrap_or_else(|| id.clone());
            if out.iter().any(|(i, _, _, _)| i == &id) {
                continue;
            }
            out.push((id.clone(), name.clone(), Vec::new(), format!("{name} {id} {src}")));
        }
    }
    out
}

fn fm_field(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    for line in source.lines() {
        let t = line.trim();
        if t == "---" {
            continue;
        }
        if let Some(rest) = t.strip_prefix(&prefix) {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    None
}

fn load_knowledge_titles(marqdo: &Path, kind_subdir: &str) -> Vec<(String, String)> {
    let dir = marqdo.join("knowledge").join(kind_subdir);
    let mut out = Vec::new();
    let Ok(rd) = fs::read_dir(dir) else {
        return out;
    };
    for ent in rd.flatten() {
        let p = ent.path();
        let Ok(src) = fs::read_to_string(&p) else {
            continue;
        };
        let id = fm_field(&src, "id").unwrap_or_else(|| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("k")
                .to_string()
        });
        let title = fm_field(&src, "title").unwrap_or_else(|| id.clone());
        out.push((id, title));
    }
    out
}

fn rank_caps(task: &str, caps: &[(String, String, Vec<String>, String)]) -> Vec<CapHit> {
    let qv = char_bigram_tf(task);
    let mut hits = Vec::new();
    for (id, name, impls, material) in caps {
        let mut score = cosine_tf(&qv, &char_bigram_tf(material));
        if name.to_ascii_lowercase().contains(&task.to_ascii_lowercase())
            || task
                .to_ascii_lowercase()
                .split_whitespace()
                .any(|t| t.len() > 2 && name.to_ascii_lowercase().contains(t))
        {
            score = (score + 0.35).min(1.0);
        }
        if score > 0.12 {
            hits.push(CapHit {
                id: id.clone(),
                name: name.clone(),
                score,
                implemented_by: impls.clone(),
            });
        }
    }
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits
}

fn relevant_knowledge(task: &str, items: &[(String, String)], cap_names: &[String]) -> Vec<String> {
    let task_l = task.to_ascii_lowercase();
    items
        .iter()
        .filter(|(id, title)| {
            let m = format!("{id} {title}").to_ascii_lowercase();
            task_l.split_whitespace().any(|t| t.len() > 3 && m.contains(t))
                || cap_names.iter().any(|c| m.contains(&c.to_ascii_lowercase()))
        })
        .map(|(id, title)| format!("{title} ({id})"))
        .collect()
}

/// `agent_eng_preflight` — Engineering First Information gate.
pub fn eng_preflight(args: &Value) -> Result<Value, String> {
    let task = arg_text(args, "task").or_else(|_| arg_text(args, "goal"))?;
    let root = args
        .get("root")
        .and_then(|v| v.as_str())
        .unwrap_or(".");
    let marqdo = args
        .get("marqdo_dir")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| resolve_marqdo_root(root));

    if !marqdo.join("engineering.yaml").exists() && !marqdo.join("graph/graph.json").exists() {
        return Ok(json!({
            "status": "missing_knowledge",
            "create_allowed": false,
            "decision": "CREATE",
            "error": "run `marqdo knowledge` first",
            "existing": [],
            "recommended": [],
        }));
    }

    let reuse_threshold = arg_f64(args, "reuse_threshold", 0.72);
    let adapt_threshold = arg_f64(args, "adapt_threshold", 0.45);
    let allow_create_after = arg_usize(args, "allow_create_after", 2);
    let min_candidates = arg_usize(args, "min_candidates", 3);

    let caps = load_capabilities(&marqdo);
    let ranked = rank_caps(task, &caps);
    let existing: Vec<String> = ranked.iter().take(8).map(|c| c.name.clone()).collect();
    let cap_names = existing.clone();

    let constraints = relevant_knowledge(
        task,
        &load_knowledge_titles(&marqdo, "constraints"),
        &cap_names,
    );
    let decisions = relevant_knowledge(
        task,
        &load_knowledge_titles(&marqdo, "decisions"),
        &cap_names,
    );
    let failures = relevant_knowledge(
        task,
        &load_knowledge_titles(&marqdo, "failures"),
        &cap_names,
    );

    let best = ranked.first();
    let best_score = best.map(|c| c.score).unwrap_or(0.0);
    let analyzed = ranked.len().max(1).min(min_candidates.max(allow_create_after));

    let (decision, create_allowed, recommended, reason) = if best_score >= reuse_threshold {
        let b = best.unwrap();
        (
            "REUSE",
            false,
            vec![format!("REUSE {}", b.name)],
            vec![
                "score meets REUSE threshold".into(),
                "same capability semantics".into(),
            ],
        )
    } else if best_score >= adapt_threshold {
        let b = best.unwrap();
        (
            "ADAPT",
            false,
            vec![format!("ADAPT {}", b.name)],
            vec!["near match — wrap/compose/extend".into()],
        )
    } else if analyzed >= allow_create_after {
        (
            "CREATE",
            true,
            vec!["CREATE new capability".into()],
            vec!["no existing capability satisfies constraints".into()],
        )
    } else {
        (
            "CREATE",
            false,
            vec![],
            vec![format!(
                "insufficient candidates ({analyzed} < {allow_create_after})"
            )],
        )
    };

    // Write context pack
    let contexts = marqdo.join("agent/contexts");
    let _ = fs::create_dir_all(&contexts);
    let mut h: u64 = 5381;
    for b in task.as_bytes() {
        h = h.wrapping_mul(33).wrapping_add(u64::from(*b));
    }
    let tid = format!("{h:016x}")[..12].to_string();
    let ctx_path = contexts.join(format!("task-{tid}.mq.md"));
    let pack = format!(
        "---\ntype: Marqdo Context Pack\ntitle: Engineering Context\ndecision: {decision}\ncreate_allowed: {create_allowed}\n---\n\n# Engineering Context\n\n## Task\n\n{task}\n\n## Existing Capabilities\n\n{existing}\n\n## Recommended Reuse\n\n{recommended}\n",
        existing = existing
            .iter()
            .map(|e| format!("- {e}"))
            .collect::<Vec<_>>()
            .join("\n"),
        recommended = recommended
            .iter()
            .enumerate()
            .map(|(i, r)| format!("{}. {r}", i + 1))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let _ = fs::write(&ctx_path, pack);

    let status = if create_allowed {
        "ready_create"
    } else if decision == "REUSE" || decision == "ADAPT" {
        "ready"
    } else if decision == "CREATE" && !create_allowed {
        "blocked"
    } else {
        "ready"
    };

    Ok(json!({
        "status": status,
        "intent": { "capability": best.map(|c| c.name.clone()).unwrap_or_else(|| task.to_string()) },
        "existing": existing,
        "constraints": constraints,
        "decisions": decisions,
        "failures": failures,
        "recommended": recommended,
        "create_allowed": create_allowed,
        "decision": decision,
        "confidence": best_score,
        "reason": reason,
        "context_path": ctx_path.display().to_string(),
        "candidates": ranked.iter().take(8).map(|c| json!({
            "id": c.id,
            "name": c.name,
            "score": c.score,
            "implemented_by": c.implemented_by,
        })).collect::<Vec<_>>(),
    }))
}

/// `agent_eng_reuse` — alias of preflight focused on decision payload.
pub fn eng_reuse(args: &Value) -> Result<Value, String> {
    let mut pf = eng_preflight(args)?;
    if let Some(obj) = pf.as_object_mut() {
        obj.insert(
            "protocol".into(),
            json!("REUSE|ADAPT|CREATE"),
        );
    }
    Ok(pf)
}

/// `agent_eng_record` — append reuse metrics under `.marqdo/agent/episodes`.
pub fn eng_record(args: &Value) -> Result<Value, String> {
    let decision = arg_text(args, "decision")?;
    let root = args
        .get("root")
        .and_then(|v| v.as_str())
        .unwrap_or(".");
    let marqdo = args
        .get("marqdo_dir")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| resolve_marqdo_root(root));
    let duplicated = matches!(args.get("duplicated"), Some(Value::Bool(true)));

    let path = marqdo.join("agent/episodes/reuse_metrics.json");
    if let Some(p) = path.parent() {
        let _ = fs::create_dir_all(p);
    }
    let mut m = if let Ok(raw) = fs::read_to_string(&path) {
        serde_json::from_str::<Value>(&raw).unwrap_or(json!({}))
    } else {
        json!({})
    };
    let get_u = |v: &Value, k: &str| v.get(k).and_then(|x| x.as_u64()).unwrap_or(0);
    let mut reused = get_u(&m, "reused");
    let mut adapted = get_u(&m, "adapted");
    let mut created = get_u(&m, "created");
    let mut dups = get_u(&m, "duplicated");
    match decision.to_ascii_uppercase().as_str() {
        "REUSE" => reused += 1,
        "ADAPT" => adapted += 1,
        "CREATE" => {
            created += 1;
            if duplicated {
                dups += 1;
            }
        }
        _ => {}
    }
    m = json!({
        "generated_functions": get_u(&m, "generated_functions") + 1,
        "reused": reused,
        "adapted": adapted,
        "created": created,
        "duplicated": dups,
    });
    let gen = m["generated_functions"].as_u64().unwrap_or(1) as f64;
    m["reuse_ratio"] = json!(reused as f64 / gen);
    m["adapt_ratio"] = json!(adapted as f64 / gen);
    m["novel_ratio"] = json!(created as f64 / gen);
    m["duplication_rate"] = json!(if created == 0 {
        0.0
    } else {
        dups as f64 / created as f64
    });
    fs::write(&path, serde_json::to_string_pretty(&m).unwrap_or_default())
        .map_err(|e| e.to_string())?;
    Ok(m)
}
