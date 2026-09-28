//! Engineering preflight + Context Pack artifacts.

use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::knowledge::ir::KnowledgeGraph;
use crate::knowledge::reuse::{resolve, ReuseBudget};
use crate::knowledge::util::slugify;

#[derive(Debug, Clone)]
pub struct PreflightResult {
    pub status: String,
    pub intent_capability: String,
    pub existing: Vec<String>,
    pub constraints: Vec<String>,
    pub decisions: Vec<String>,
    pub failures: Vec<String>,
    pub recommended: Vec<String>,
    pub create_allowed: bool,
    pub decision: String,
    pub context_path: Option<String>,
    pub pack_markdown: String,
}

pub fn preflight(
    graph: &KnowledgeGraph,
    task: &str,
    budget: &ReuseBudget,
    out_dir: Option<&Path>,
) -> Result<PreflightResult> {
    let reuse = resolve(graph, task, budget);
    let intent = reuse
        .capability
        .clone()
        .unwrap_or_else(|| infer_intent(task));

    let existing: Vec<String> = reuse
        .candidates
        .iter()
        .filter(|c| c.kind == "Capability" || c.kind == "Function")
        .take(8)
        .map(|c| c.title.clone())
        .collect();

    let pack = render_context_pack(task, &intent, &reuse, graph);
    let mut context_path = None;
    if let Some(out) = out_dir {
        let contexts = out.join("agent/contexts");
        fs::create_dir_all(&contexts)?;
        let id = task_id(task);
        let path = contexts.join(format!("task-{id}.mq.md"));
        fs::write(&path, &pack)?;
        context_path = Some(path.display().to_string());
    }

    let status = if reuse.create_allowed {
        "ready_create"
    } else if reuse.decision == crate::knowledge::ir::ReuseDecision::Reuse
        || reuse.decision == crate::knowledge::ir::ReuseDecision::Adapt
    {
        "ready"
    } else {
        "blocked"
    };

    Ok(PreflightResult {
        status: status.into(),
        intent_capability: intent,
        existing,
        constraints: reuse.constraints.clone(),
        decisions: reuse.decisions.clone(),
        failures: reuse.failures.clone(),
        recommended: reuse.recommended.clone(),
        create_allowed: reuse.create_allowed,
        decision: reuse.decision.as_str().into(),
        context_path,
        pack_markdown: pack,
    })
}

fn task_id(task: &str) -> String {
    let mut h = Sha256::new();
    h.update(task.as_bytes());
    format!("{:x}", h.finalize())[..12].to_string()
}

fn infer_intent(task: &str) -> String {
    let tokens: Vec<_> = task
        .split_whitespace()
        .filter(|t| t.len() > 2)
        .take(4)
        .collect();
    if tokens.is_empty() {
        "unknown".into()
    } else {
        slugify(&tokens.join(" "))
    }
}

fn render_context_pack(
    task: &str,
    intent: &str,
    reuse: &crate::knowledge::ir::ReuseResult,
    graph: &KnowledgeGraph,
) -> String {
    let mut apis = Vec::new();
    for c in reuse.candidates.iter().take(5) {
        if let Some(cap) = graph.capability_by_id(
            c.id.strip_prefix("capability:")
                .unwrap_or(&c.id),
        ) {
            for sid in &cap.implemented_by {
                if let Some(s) = graph.symbol_by_id(sid) {
                    apis.push(format!("{} ({})", s.name, s.resource));
                }
            }
        } else if c.kind == "Function" {
            apis.push(format!(
                "{} ({})",
                c.title,
                c.resource.as_deref().unwrap_or("?")
            ));
        }
    }
    apis.sort();
    apis.dedup();

    let mut forbidden = Vec::new();
    for f in &reuse.failures {
        forbidden.push(format!("Do not repeat: {f}"));
    }
    for c in graph.knowledge.iter().filter(|k| {
        matches!(
            k.kind,
            crate::knowledge::ir::KnowledgeKind::AntiPattern
                | crate::knowledge::ir::KnowledgeKind::Constraint
        )
    }) {
        if reuse.constraints.iter().any(|x| x.contains(&c.id))
            || reuse.failures.iter().any(|x| x.contains(&c.id))
        {
            forbidden.push(c.title.clone());
        }
    }

    format!(
        r#"---
type: Marqdo Context Pack
title: Engineering Context
task: {task_esc}
intent: {intent}
decision: {decision}
create_allowed: {create_allowed}
generated:
  by: marqdo/knowledge
---

# Engineering Context

## Task

{task}

## Existing Capabilities

{existing}

## Relevant APIs

{apis}

## Constraints

{constraints}

## Decisions

{decisions}

## Known Failures

{failures}

## Recommended Reuse

{recommended}

## Forbidden

Do not implement:
{forbidden}
"#,
        task_esc = task.replace('\n', " "),
        intent = intent,
        decision = reuse.decision.as_str(),
        create_allowed = reuse.create_allowed,
        task = task,
        existing = bullet_list(
            &reuse
                .candidates
                .iter()
                .filter(|c| c.kind == "Capability" || c.kind == "Function")
                .take(8)
                .map(|c| format!("{} ({:.2})", c.title, c.score))
                .collect::<Vec<_>>()
        ),
        apis = bullet_list(&apis),
        constraints = bullet_list(&reuse.constraints),
        decisions = bullet_list(&reuse.decisions),
        failures = bullet_list(&reuse.failures),
        recommended = numbered_list(&reuse.recommended),
        forbidden = bullet_list(&forbidden),
    )
}

fn bullet_list(items: &[String]) -> String {
    if items.is_empty() {
        "- _none_".into()
    } else {
        items.iter().map(|i| format!("- {i}")).collect::<Vec<_>>().join("\n")
    }
}

fn numbered_list(items: &[String]) -> String {
    if items.is_empty() {
        "1. _none_".into()
    } else {
        items
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {s}", i + 1))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn preflight_json(p: &PreflightResult) -> Value {
    json!({
        "status": p.status,
        "intent": { "capability": p.intent_capability },
        "existing": p.existing,
        "constraints": p.constraints,
        "decisions": p.decisions,
        "failures": p.failures,
        "recommended": p.recommended,
        "create_allowed": p.create_allowed,
        "decision": p.decision,
        "context_path": p.context_path,
    })
}

pub fn format_preflight_yaml(p: &PreflightResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("status: {}\n\n", p.status));
    out.push_str("intent:\n");
    out.push_str(&format!("  capability: {}\n\n", p.intent_capability));
    out.push_str("existing:\n");
    for e in &p.existing {
        out.push_str(&format!("  - {e}\n"));
    }
    if p.existing.is_empty() {
        out.push_str("  []\n");
    }
    out.push_str("\nconstraints:\n");
    for c in &p.constraints {
        out.push_str(&format!("  - {c}\n"));
    }
    if p.constraints.is_empty() {
        out.push_str("  []\n");
    }
    out.push_str("\ndecisions:\n");
    for d in &p.decisions {
        out.push_str(&format!("  - {d}\n"));
    }
    if p.decisions.is_empty() {
        out.push_str("  []\n");
    }
    out.push_str("\nfailures:\n");
    for f in &p.failures {
        out.push_str(&format!("  - {f}\n"));
    }
    if p.failures.is_empty() {
        out.push_str("  []\n");
    }
    out.push_str("\nrecommended:\n");
    for r in &p.recommended {
        out.push_str(&format!("  - {r}\n"));
    }
    if p.recommended.is_empty() {
        out.push_str("  []\n");
    }
    out.push_str(&format!("\ncreate_allowed: {}\n", p.create_allowed));
    if let Some(ref pth) = p.context_path {
        out.push_str(&format!("context_pack: {pth}\n"));
    }
    out
}
