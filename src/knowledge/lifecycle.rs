//! Lifecycle: stale, conflicts, verify.

use std::fs;
use std::path::Path;
use std::time::SystemTime;

use serde_json::{json, Value};

use crate::knowledge::ir::{EdgeKind, KnowledgeGraph, Lifecycle};

#[derive(Debug, Clone)]
pub struct StaleItem {
    pub id: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ConflictItem {
    pub left: String,
    pub right: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct VerifyIssue {
    pub severity: String, // error | warning
    pub message: String,
}

pub fn find_stale(graph: &KnowledgeGraph, root: &Path) -> Vec<StaleItem> {
    let mut out = Vec::new();
    let now = chrono::Utc::now().date_naive();

    for k in &graph.knowledge {
        if let Some(ref sa) = k.stale_after {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(sa, "%Y-%m-%d") {
                if d < now {
                    out.push(StaleItem {
                        id: k.id.clone(),
                        reason: format!("stale_after {sa} passed"),
                    });
                }
            }
        }
        if matches!(k.status, Lifecycle::Deprecated | Lifecycle::Superseded) {
            out.push(StaleItem {
                id: k.id.clone(),
                reason: format!("status={}", k.status.as_str()),
            });
        }
        // Source newer than projection evidence — mtime drift
        let src = root.join(&k.resource);
        if src.exists() {
            if let (Ok(meta), Ok(proj_meta)) = (
                fs::metadata(&src),
                fs::metadata(root.join(".marqdo/graph/graph.json"))
                    .or_else(|_| fs::metadata(root.join("graph/graph.json"))),
            ) {
                if let (Ok(st), Ok(pt)) = (meta.modified(), proj_meta.modified()) {
                    if st > pt {
                        out.push(StaleItem {
                            id: k.id.clone(),
                            reason: "source newer than knowledge graph — recompile".into(),
                        });
                    }
                }
            }
        }
    }

    for c in &graph.capabilities {
        if matches!(c.status, Lifecycle::Deprecated | Lifecycle::Superseded) {
            out.push(StaleItem {
                id: c.id.clone(),
                reason: format!("capability status={}", c.status.as_str()),
            });
        }
    }

    // Dedup
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out.dedup_by(|a, b| a.id == b.id && a.reason == b.reason);
    out
}

pub fn find_conflicts(graph: &KnowledgeGraph) -> Vec<ConflictItem> {
    let mut out = Vec::new();
    for e in &graph.edges {
        if e.kind == EdgeKind::ConflictsWith {
            out.push(ConflictItem {
                left: e.from.clone(),
                right: e.to.clone(),
                note: e.note.clone(),
            });
        }
    }
    // Knowledge items that claim supersession without edge already covered
    for k in &graph.knowledge {
        for c in &k.conflicts_with {
            out.push(ConflictItem {
                left: format!("knowledge:{}", k.id),
                right: format!("knowledge:{c}"),
                note: Some("authored conflicts_with".into()),
            });
        }
    }
    out.sort_by(|a, b| (&a.left, &a.right).cmp(&(&b.left, &b.right)));
    out.dedup_by(|a, b| a.left == b.left && a.right == b.right);
    out
}

pub fn verify(graph: &KnowledgeGraph) -> Vec<VerifyIssue> {
    let mut issues = Vec::new();

    // Symbols referenced by capabilities must exist
    for c in &graph.capabilities {
        for sid in &c.implemented_by {
            if graph.symbol_by_id(sid).is_none() {
                issues.push(VerifyIssue {
                    severity: "error".into(),
                    message: format!(
                        "capability `{}` implements missing symbol `{sid}`",
                        c.id
                    ),
                });
            }
        }
    }

    // Edges must reference known-ish nodes (soft: warn)
    let mut node_ids = std::collections::HashSet::new();
    node_ids.insert("repo".into());
    for m in &graph.modules {
        node_ids.insert(format!("module:{}", m.id));
    }
    for s in &graph.symbols {
        node_ids.insert(format!("symbol:{}", s.id));
    }
    for c in &graph.capabilities {
        node_ids.insert(format!("capability:{}", c.id));
    }
    for k in &graph.knowledge {
        node_ids.insert(format!("knowledge:{}", k.id));
    }

    for e in &graph.edges {
        if !node_ids.contains(&e.from) && !e.from.contains('/') {
            // allow raw applies_to strings
            if !e.from.starts_with("knowledge:")
                && !e.from.starts_with("symbol:")
                && !e.from.starts_with("capability:")
                && !e.from.starts_with("module:")
            {
                continue;
            }
            issues.push(VerifyIssue {
                severity: "warning".into(),
                message: format!("edge from unknown node `{}`", e.from),
            });
        }
    }

    // Duplicate high-similarity without documentation
    for c in &graph.capabilities {
        if !c.possible_duplicates.is_empty() {
            issues.push(VerifyIssue {
                severity: "warning".into(),
                message: format!(
                    "capability `{}` has possible_duplicates: {}",
                    c.id,
                    c.possible_duplicates.join(", ")
                ),
            });
        }
    }

    // Supersedes targets should exist
    for k in &graph.knowledge {
        for s in &k.supersedes {
            if !graph.knowledge.iter().any(|o| o.id == *s || o.id.ends_with(s)) {
                issues.push(VerifyIssue {
                    severity: "warning".into(),
                    message: format!("knowledge `{}` supersedes unknown `{s}`", k.id),
                });
            }
        }
    }

    issues
}

pub fn format_stale_text(items: &[StaleItem]) -> String {
    let mut out = String::from("Stale knowledge\n\n");
    if items.is_empty() {
        out.push_str("_None._\n");
        return out;
    }
    for i in items {
        out.push_str(&format!("- {} — {}\n", i.id, i.reason));
    }
    out
}

pub fn format_conflicts_text(items: &[ConflictItem]) -> String {
    let mut out = String::from("Knowledge conflicts\n\n");
    if items.is_empty() {
        out.push_str("_None._\n");
        return out;
    }
    for i in items {
        out.push_str(&format!(
            "- {} ↔ {}{}\n",
            i.left,
            i.right,
            i.note
                .as_ref()
                .map(|n| format!(" ({n})"))
                .unwrap_or_default()
        ));
    }
    out
}

pub fn format_verify_text(issues: &[VerifyIssue]) -> String {
    let mut out = String::from("Knowledge verify\n\n");
    if issues.is_empty() {
        out.push_str("OK — code and knowledge projections are consistent.\n");
        return out;
    }
    for i in issues {
        out.push_str(&format!("[{}] {}\n", i.severity, i.message));
    }
    out
}

pub fn stale_json(items: &[StaleItem]) -> Value {
    json!({ "stale": items.iter().map(|i| json!({"id": i.id, "reason": i.reason})).collect::<Vec<_>>() })
}

pub fn conflicts_json(items: &[ConflictItem]) -> Value {
    json!({ "conflicts": items.iter().map(|i| json!({"left": i.left, "right": i.right, "note": i.note})).collect::<Vec<_>>() })
}

pub fn verify_json(issues: &[VerifyIssue]) -> Value {
    json!({
        "ok": issues.iter().all(|i| i.severity != "error"),
        "issues": issues.iter().map(|i| json!({"severity": i.severity, "message": i.message})).collect::<Vec<_>>(),
    })
}

#[allow(dead_code)]
fn _system_time_now() -> SystemTime {
    SystemTime::now()
}
