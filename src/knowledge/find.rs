//! Hybrid knowledge find (lexical + symbol + capability + graph).

use crate::knowledge::fingerprint::text_similarity;
use crate::knowledge::ir::{KnowledgeGraph, KnowledgeKind};
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct FindHit {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub score: f64,
    pub resource: Option<String>,
    pub snippet: String,
}

pub fn find(graph: &KnowledgeGraph, query: &str, limit: usize) -> Vec<FindHit> {
    let q = query.trim();
    if q.is_empty() {
        return Vec::new();
    }
    let mut hits = Vec::new();

    for c in &graph.capabilities {
        let material = format!(
            "{} {} {} {} {}",
            c.name,
            c.description,
            c.use_when.as_deref().unwrap_or(""),
            c.do_not_use_when.as_deref().unwrap_or(""),
            c.implemented_by.join(" ")
        );
        let mut score = text_similarity(q, &material);
        if c.name.to_ascii_lowercase().contains(&q.to_ascii_lowercase())
            || q.to_ascii_lowercase()
                .split_whitespace()
                .any(|t| c.name.to_ascii_lowercase().contains(t))
        {
            score = (score + 0.35).min(1.0);
        }
        if score > 0.15 {
            hits.push(FindHit {
                id: format!("capability:{}", c.id),
                kind: "Capability".into(),
                title: c.name.clone(),
                score,
                resource: c
                    .implemented_by
                    .first()
                    .and_then(|sid| graph.symbol_by_id(sid))
                    .map(|s| s.resource.clone()),
                snippet: c.description.clone(),
            });
        }
    }

    for s in &graph.symbols {
        if s.kind == "test" {
            continue;
        }
        let material = format!(
            "{} {} {} {} {}",
            s.name,
            s.fingerprint_text,
            s.use_when.as_deref().unwrap_or(""),
            s.module,
            s.calls.join(" ")
        );
        let mut score = text_similarity(q, &material);
        if s.name.to_ascii_lowercase().contains(&q.to_ascii_lowercase()) {
            score = (score + 0.4).min(1.0);
        }
        if score > 0.18 {
            hits.push(FindHit {
                id: format!("symbol:{}", s.id),
                kind: "Function".into(),
                title: s.name.clone(),
                score,
                resource: Some(s.resource.clone()),
                snippet: s.fingerprint_text.clone(),
            });
        }
    }

    for k in &graph.knowledge {
        let material = format!("{} {} {}", k.title, k.body_summary, k.kind.as_str());
        let mut score = text_similarity(q, &material);
        if k.title.to_ascii_lowercase().contains(&q.to_ascii_lowercase()) {
            score = (score + 0.35).min(1.0);
        }
        // Boost engineering knowledge types for governance queries
        if matches!(
            k.kind,
            KnowledgeKind::Constraint
                | KnowledgeKind::Decision
                | KnowledgeKind::Failure
                | KnowledgeKind::AntiPattern
        ) && score > 0.1
        {
            score = (score + 0.1).min(1.0);
        }
        if score > 0.15 {
            hits.push(FindHit {
                id: format!("knowledge:{}", k.id),
                kind: k.kind.as_str().to_string(),
                title: k.title.clone(),
                score,
                resource: Some(k.resource.clone()),
                snippet: k.body_summary.clone(),
            });
        }
    }

    // Graph expansion: boost neighbors of top hits
    hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    let top_ids: Vec<String> = hits.iter().take(5).map(|h| h.id.clone()).collect();
    for tid in &top_ids {
        for e in graph.neighbors(tid, None) {
            let other = if e.from == *tid { &e.to } else { &e.from };
            if let Some(h) = hits.iter_mut().find(|h| h.id == *other) {
                h.score = (h.score + 0.05).min(1.0);
            }
        }
    }

    hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    hits.truncate(limit);
    hits
}

pub fn format_find_text(query: &str, hits: &[FindHit], graph: &KnowledgeGraph) -> String {
    let mut out = format!("Engineering First Information\n\nQuery: {query}\n\n");
    if hits.is_empty() {
        out.push_str("No matching capabilities, symbols, or knowledge.\n");
        return out;
    }

    let caps: Vec<_> = hits.iter().filter(|h| h.kind == "Capability").collect();
    let impls: Vec<_> = hits.iter().filter(|h| h.kind == "Function").collect();
    let constraints: Vec<_> = hits.iter().filter(|h| h.kind == "Constraint").collect();
    let decisions: Vec<_> = hits.iter().filter(|h| h.kind == "Decision").collect();
    let failures: Vec<_> = hits
        .iter()
        .filter(|h| h.kind == "Failure" || h.kind == "AntiPattern")
        .collect();
    let related: Vec<_> = hits
        .iter()
        .filter(|h| {
            h.kind != "Capability"
                && h.kind != "Function"
                && h.kind != "Constraint"
                && h.kind != "Decision"
                && h.kind != "Failure"
                && h.kind != "AntiPattern"
        })
        .collect();

    if !caps.is_empty() {
        out.push_str("Capability\n");
        for h in &caps {
            out.push_str(&format!("  {}  ({:.2})\n", h.title, h.score));
        }
        out.push('\n');
    }
    if !impls.is_empty() {
        out.push_str("Implementation\n");
        for h in &impls {
            out.push_str(&format!(
                "  {}\n",
                h.resource.as_deref().unwrap_or(&h.title)
            ));
        }
        out.push('\n');
    }
    if !constraints.is_empty() {
        out.push_str("Constraints\n");
        for h in &constraints {
            out.push_str(&format!("  {}\n", h.title));
        }
        out.push('\n');
    }
    if !decisions.is_empty() {
        out.push_str("Decisions\n");
        for h in &decisions {
            out.push_str(&format!("  {}\n", h.title));
        }
        out.push('\n');
    }
    if !failures.is_empty() {
        out.push_str("Failures\n");
        for h in &failures {
            out.push_str(&format!("  {}\n", h.title));
        }
        out.push('\n');
    }
    if !related.is_empty() {
        out.push_str("Related\n");
        for h in &related {
            out.push_str(&format!("  {} ({})\n", h.title, h.kind));
        }
        out.push('\n');
    }

    // Graph-related capabilities for top cap
    if let Some(top) = caps.first() {
        let id = top.id.strip_prefix("capability:").unwrap_or(&top.id);
        if let Some(c) = graph.capability_by_id(id) {
            if !c.related.is_empty() {
                out.push_str("Related capabilities\n");
                for r in &c.related {
                    out.push_str(&format!("  {r}\n"));
                }
            }
        }
    }
    out
}

pub fn find_json(query: &str, hits: &[FindHit]) -> Value {
    json!({
        "query": query,
        "hits": hits.iter().map(|h| json!({
            "id": h.id,
            "kind": h.kind,
            "title": h.title,
            "score": h.score,
            "resource": h.resource,
            "snippet": h.snippet,
        })).collect::<Vec<_>>(),
    })
}
