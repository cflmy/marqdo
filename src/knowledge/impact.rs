//! Knowledge impact analysis for changed paths.

use serde_json::{json, Value};

use crate::knowledge::ir::{EdgeKind, KnowledgeGraph};

#[derive(Debug, Clone)]
pub struct ImpactReport {
    pub changed: Vec<String>,
    pub capabilities: Vec<String>,
    pub decisions: Vec<String>,
    pub constraints: Vec<String>,
    pub tests: Vec<String>,
    pub documentation: Vec<String>,
    pub stale_knowledge: Vec<String>,
    pub symbols: Vec<String>,
}

pub fn impact(graph: &KnowledgeGraph, paths: &[String]) -> ImpactReport {
    let mut report = ImpactReport {
        changed: paths.to_vec(),
        capabilities: Vec::new(),
        decisions: Vec::new(),
        constraints: Vec::new(),
        tests: Vec::new(),
        documentation: Vec::new(),
        stale_knowledge: Vec::new(),
        symbols: Vec::new(),
    };

    for path in paths {
        let norm = path.replace('\\', "/");
        for s in &graph.symbols {
            if s.resource == norm
                || s.resource.ends_with(&norm)
                || norm.ends_with(&s.resource)
            {
                report.symbols.push(s.id.clone());
                if let Some(ref cap) = s.capability {
                    report.capabilities.push(cap.clone());
                }
                for c in &graph.capabilities {
                    if c.implemented_by.iter().any(|id| id == &s.id) {
                        report.capabilities.push(c.name.clone());
                    }
                }
                report.tests.extend(s.tests.clone());
            }
        }
        for m in &graph.modules {
            if m.resource == norm || m.resource.ends_with(&norm) || norm.ends_with(&m.resource)
            {
                for s in graph.symbols.iter().filter(|s| s.module == m.id) {
                    report.symbols.push(s.id.clone());
                }
            }
        }
        for k in &graph.knowledge {
            if k.resource == norm || k.resource.ends_with(&norm) {
                match k.kind {
                    crate::knowledge::ir::KnowledgeKind::Decision => {
                        report.decisions.push(k.id.clone())
                    }
                    crate::knowledge::ir::KnowledgeKind::Constraint => {
                        report.constraints.push(k.id.clone())
                    }
                    _ => report.documentation.push(k.resource.clone()),
                }
            }
            // applies_to matches changed symbols/modules
            for a in &k.applies_to {
                if report.symbols.iter().any(|s| s.contains(a) || a.contains(s))
                    || report.capabilities.iter().any(|c| c == a || a.contains(c))
                {
                    report.stale_knowledge.push(k.id.clone());
                    match k.kind {
                        crate::knowledge::ir::KnowledgeKind::Decision => {
                            report.decisions.push(k.id.clone())
                        }
                        crate::knowledge::ir::KnowledgeKind::Constraint => {
                            report.constraints.push(k.id.clone())
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Graph walk: from changed symbols follow edges
    let seed: Vec<String> = report
        .symbols
        .iter()
        .map(|s| format!("symbol:{s}"))
        .collect();
    for sid in &seed {
        for e in graph.neighbors(sid, None) {
            let other = if &e.from == sid { &e.to } else { &e.from };
            if other.starts_with("capability:") {
                report
                    .capabilities
                    .push(other.trim_start_matches("capability:").to_string());
            } else if other.starts_with("knowledge:") {
                report
                    .stale_knowledge
                    .push(other.trim_start_matches("knowledge:").to_string());
            } else if e.kind == EdgeKind::TestedBy {
                report.tests.push(other.clone());
            }
        }
    }

    report.capabilities.sort();
    report.capabilities.dedup();
    report.decisions.sort();
    report.decisions.dedup();
    report.constraints.sort();
    report.constraints.dedup();
    report.tests.sort();
    report.tests.dedup();
    report.documentation.sort();
    report.documentation.dedup();
    report.stale_knowledge.sort();
    report.stale_knowledge.dedup();
    report.symbols.sort();
    report.symbols.dedup();
    report
}

pub fn format_impact_text(r: &ImpactReport) -> String {
    let mut out = String::new();
    out.push_str("Changed:\n");
    for c in &r.changed {
        out.push_str(&format!("  {c}\n"));
    }
    out.push_str("\nAffected capabilities:\n");
    for c in &r.capabilities {
        out.push_str(&format!("  {c}\n"));
    }
    if r.capabilities.is_empty() {
        out.push_str("  (none)\n");
    }
    out.push_str("\nAffected decisions:\n");
    for d in &r.decisions {
        out.push_str(&format!("  {d}\n"));
    }
    if r.decisions.is_empty() {
        out.push_str("  (none)\n");
    }
    out.push_str("\nAffected constraints:\n");
    for c in &r.constraints {
        out.push_str(&format!("  {c}\n"));
    }
    out.push_str("\nAffected tests:\n");
    for t in &r.tests {
        out.push_str(&format!("  {t}\n"));
    }
    out.push_str("\nPotential stale knowledge:\n");
    for s in &r.stale_knowledge {
        out.push_str(&format!("  {s}\n"));
    }
    if r.stale_knowledge.is_empty() {
        out.push_str("  (none)\n");
    }
    out
}

pub fn impact_json(r: &ImpactReport) -> Value {
    json!({
        "changed": r.changed,
        "capabilities": r.capabilities,
        "decisions": r.decisions,
        "constraints": r.constraints,
        "tests": r.tests,
        "documentation": r.documentation,
        "stale_knowledge": r.stale_knowledge,
        "symbols": r.symbols,
    })
}
