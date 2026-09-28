//! Reuse protocol: REUSE → ADAPT → CREATE with reuse budget.

use serde_json::{json, Value};

use crate::knowledge::find::{find, FindHit};
use crate::knowledge::ir::{
    KnowledgeGraph, KnowledgeKind, ReuseCandidate, ReuseDecision, ReuseResult,
};

#[derive(Debug, Clone)]
pub struct ReuseBudget {
    pub min_candidates: usize,
    pub require_preflight: bool,
    pub require_reason: bool,
    pub allow_create_after: usize,
    pub reuse_threshold: f64,
    pub adapt_threshold: f64,
}

impl Default for ReuseBudget {
    fn default() -> Self {
        Self {
            min_candidates: 3,
            require_preflight: true,
            require_reason: true,
            allow_create_after: 2,
            reuse_threshold: 0.72,
            adapt_threshold: 0.45,
        }
    }
}

pub fn resolve(graph: &KnowledgeGraph, task: &str, budget: &ReuseBudget) -> ReuseResult {
    let hits = find(graph, task, 20);
    let mut candidates: Vec<ReuseCandidate> = hits
        .iter()
        .filter(|h| h.kind == "Capability" || h.kind == "Function")
        .map(|h| ReuseCandidate {
            id: h.id.clone(),
            kind: h.kind.clone(),
            title: h.title.clone(),
            score: h.score,
            resource: h.resource.clone(),
            reason: reason_for(h, task),
        })
        .collect();

    // Ensure min_candidates from broader search if needed
    if candidates.len() < budget.min_candidates {
        for h in &hits {
            if candidates.iter().any(|c| c.id == h.id) {
                continue;
            }
            candidates.push(ReuseCandidate {
                id: h.id.clone(),
                kind: h.kind.clone(),
                title: h.title.clone(),
                score: h.score,
                resource: h.resource.clone(),
                reason: reason_for(h, task),
            });
            if candidates.len() >= budget.min_candidates {
                break;
            }
        }
    }

    let constraints: Vec<String> = graph
        .knowledge
        .iter()
        .filter(|k| k.kind == KnowledgeKind::Constraint)
        .filter(|k| relevant_knowledge(k.title.as_str(), &k.body_summary, task, &hits))
        .map(|k| format!("{} ({})", k.title, k.id))
        .collect();

    let decisions: Vec<String> = graph
        .knowledge
        .iter()
        .filter(|k| k.kind == KnowledgeKind::Decision)
        .filter(|k| relevant_knowledge(k.title.as_str(), &k.body_summary, task, &hits))
        .map(|k| format!("{} ({})", k.title, k.id))
        .collect();

    let failures: Vec<String> = graph
        .knowledge
        .iter()
        .filter(|k| {
            matches!(
                k.kind,
                KnowledgeKind::Failure | KnowledgeKind::AntiPattern
            )
        })
        .filter(|k| relevant_knowledge(k.title.as_str(), &k.body_summary, task, &hits))
        .map(|k| format!("{} ({})", k.title, k.id))
        .collect();

    let best = candidates.first().cloned();
    let best_score = best.as_ref().map(|c| c.score).unwrap_or(0.0);
    let analyzed = candidates.len().min(budget.allow_create_after.max(budget.min_candidates));

    let (decision, create_allowed, reason, recommended) = if best_score >= budget.reuse_threshold {
        let b = best.as_ref().unwrap();
        let mut reasons = b.reason.clone();
        reasons.push("score meets REUSE threshold".into());
        if !constraints.is_empty() {
            reasons.push("active constraints favor existing capability".into());
        }
        (
            ReuseDecision::Reuse,
            false,
            reasons,
            vec![format!("REUSE {}", b.title)],
        )
    } else if best_score >= budget.adapt_threshold {
        let b = best.as_ref().unwrap();
        let mut reasons = b.reason.clone();
        reasons.push("near match — wrap/compose/extend rather than copy".into());
        (
            ReuseDecision::Adapt,
            false,
            reasons,
            vec![format!("ADAPT {}", b.title)],
        )
    } else if analyzed >= budget.allow_create_after {
        let mut reasons = vec![
            "no existing capability satisfies constraints".into(),
            format!("analyzed {analyzed} candidates below adapt threshold"),
        ];
        if budget.require_reason {
            reasons.push("CREATE allowed only after reuse budget exhausted".into());
        }
        (ReuseDecision::Create, true, reasons, vec!["CREATE new capability".into()])
    } else {
        (
            ReuseDecision::Create,
            false,
            vec![
                format!(
                    "insufficient candidates analyzed ({analyzed} < {})",
                    budget.allow_create_after
                ),
                "continue discovery before CREATE".into(),
            ],
            vec![],
        )
    };

    let capability = best.as_ref().and_then(|c| {
        if c.kind == "Capability" {
            Some(c.title.clone())
        } else {
            c.id.strip_prefix("capability:").map(|s| s.to_string())
        }
    });
    let symbol = best.as_ref().and_then(|c| {
        if c.kind == "Function" {
            Some(c.title.clone())
        } else if let Some(cap) = capability.as_ref() {
            graph
                .capability_by_id(cap)
                .and_then(|cap| cap.implemented_by.first().cloned())
        } else {
            None
        }
    });
    let source = best.as_ref().and_then(|c| c.resource.clone());

    ReuseResult {
        decision,
        confidence: best_score,
        capability,
        symbol,
        source,
        reason,
        candidates,
        constraints,
        decisions,
        failures,
        create_allowed,
        recommended,
    }
}

fn reason_for(h: &FindHit, task: &str) -> Vec<String> {
    let mut r = Vec::new();
    r.push(format!("matched query tokens against {}", h.kind));
    if h.score >= 0.7 {
        r.push("same input/output semantics likely".into());
    }
    if task.to_ascii_lowercase().split_whitespace().any(|t| {
        h.title.to_ascii_lowercase().contains(t) && t.len() > 2
    }) {
        r.push("title overlaps task terms".into());
    }
    r
}

fn relevant_knowledge(title: &str, body: &str, task: &str, hits: &[FindHit]) -> bool {
    let material = format!("{title} {body}").to_ascii_lowercase();
    let task_l = task.to_ascii_lowercase();
    if task_l.split_whitespace().any(|t| t.len() > 3 && material.contains(t)) {
        return true;
    }
    hits.iter().take(5).any(|h| {
        material.contains(&h.title.to_ascii_lowercase())
            || h.snippet
                .to_ascii_lowercase()
                .split_whitespace()
                .any(|t| t.len() > 4 && material.contains(t))
    })
}

pub fn format_reuse_text(task: &str, result: &ReuseResult) -> String {
    let mut out = String::from("Engineering First Information\n\n");
    out.push_str(&format!("Task: {task}\n\n"));
    out.push_str("Existing capabilities:\n");
    let caps: Vec<_> = result
        .candidates
        .iter()
        .filter(|c| c.kind == "Capability" || c.kind == "Function")
        .collect();
    if caps.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for c in caps.iter().take(8) {
            out.push_str(&format!("  {:<28} {:.2}\n", c.title, c.score));
        }
    }
    out.push('\n');
    if !result.constraints.is_empty() {
        out.push_str("Relevant constraints:\n");
        for c in &result.constraints {
            out.push_str(&format!("  {c}\n"));
        }
        out.push('\n');
    }
    if !result.decisions.is_empty() {
        out.push_str("Relevant decisions:\n");
        for d in &result.decisions {
            out.push_str(&format!("  {d}\n"));
        }
        out.push('\n');
    }
    if !result.failures.is_empty() {
        out.push_str("Known failures:\n");
        for f in &result.failures {
            out.push_str(&format!("  {f}\n"));
        }
        out.push('\n');
    }
    out.push_str("Recommended:\n");
    if result.recommended.is_empty() {
        out.push_str(&format!("  {}\n", result.decision.as_str()));
    } else {
        for r in &result.recommended {
            out.push_str(&format!("  {r}\n"));
        }
    }
    out.push('\n');
    out.push_str("New implementation:\n");
    if result.create_allowed {
        out.push_str("  ALLOWED (CREATE)\n");
    } else if result.decision == ReuseDecision::Create {
        out.push_str("  NOT YET — exhaust reuse budget first\n");
    } else {
        out.push_str("  NOT REQUIRED\n");
    }
    out.push('\n');
    out.push_str("Reason:\n");
    for r in &result.reason {
        out.push_str(&format!("  - {r}\n"));
    }
    out
}

pub fn reuse_json(task: &str, result: &ReuseResult) -> Value {
    json!({
        "task": task,
        "decision": result.decision.as_str(),
        "confidence": result.confidence,
        "capability": result.capability,
        "symbol": result.symbol,
        "source": result.source,
        "reason": result.reason,
        "create_allowed": result.create_allowed,
        "recommended": result.recommended,
        "constraints": result.constraints,
        "decisions": result.decisions,
        "failures": result.failures,
        "candidates": result.candidates.iter().map(|c| json!({
            "id": c.id,
            "kind": c.kind,
            "title": c.title,
            "score": c.score,
            "resource": c.resource,
            "reason": c.reason,
        })).collect::<Vec<_>>(),
    })
}
