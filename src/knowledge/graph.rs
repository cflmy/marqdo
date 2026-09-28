//! Knowledge graph edge construction.

use crate::knowledge::ir::{
    CapabilityInfo, Edge, EdgeKind, KnowledgeItem, KnowledgeKind, ModuleInfo, SymbolInfo,
};

pub fn build_edges(
    modules: &[ModuleInfo],
    symbols: &[SymbolInfo],
    capabilities: &[CapabilityInfo],
    knowledge: &[KnowledgeItem],
) -> Vec<Edge> {
    let mut edges = Vec::new();

    // Module depends_on
    for m in modules {
        for imp in &m.imports {
            let path = imp
                .split(':')
                .nth(1)
                .unwrap_or(imp)
                .trim()
                .to_string();
            if let Some(dep) = modules.iter().find(|o| {
                o.resource == path
                    || o.resource.ends_with(&path)
                    || o.resource.ends_with(&format!("/{path}"))
                    || o.id == path.trim_end_matches(".mq.md")
            }) {
                edges.push(Edge {
                    from: format!("module:{}", m.id),
                    to: format!("module:{}", dep.id),
                    kind: EdgeKind::DependsOn,
                    note: Some(imp.clone()),
                });
            }
        }
    }

    // Symbol uses (calls)
    for s in symbols {
        for c in &s.calls {
            let short = c.rsplit('.').next().unwrap_or(c);
            if let Some(target) = symbols.iter().find(|t| t.name == short || t.name == *c || t.id == *c)
            {
                edges.push(Edge {
                    from: format!("symbol:{}", s.id),
                    to: format!("symbol:{}", target.id),
                    kind: EdgeKind::Uses,
                    note: None,
                });
            }
        }
        for t in &s.tests {
            edges.push(Edge {
                from: format!("symbol:{}", s.id),
                to: format!("symbol:{t}"),
                kind: EdgeKind::TestedBy,
                note: None,
            });
        }
        for r in &s.related {
            edges.push(Edge {
                from: format!("symbol:{}", s.id),
                to: r.clone(),
                kind: EdgeKind::RelatedTo,
                note: None,
            });
        }
        for r in &s.reuse {
            edges.push(Edge {
                from: format!("symbol:{}", s.id),
                to: r.clone(),
                kind: EdgeKind::DerivedFrom,
                note: Some("reuse".into()),
            });
        }
    }

    // Capability implements
    for c in capabilities {
        for sid in &c.implemented_by {
            edges.push(Edge {
                from: format!("capability:{}", c.id),
                to: format!("symbol:{sid}"),
                kind: EdgeKind::Implements,
                note: None,
            });
        }
        for r in &c.related {
            edges.push(Edge {
                from: format!("capability:{}", c.id),
                to: r.clone(),
                kind: EdgeKind::RelatedTo,
                note: None,
            });
        }
    }

    // Knowledge edges
    for k in knowledge {
        let kid = format!("knowledge:{}", k.id);
        for a in &k.applies_to {
            let kind = match k.kind {
                KnowledgeKind::Constraint => EdgeKind::ConstrainedBy,
                KnowledgeKind::Decision => EdgeKind::DecidedBy,
                KnowledgeKind::Failure | KnowledgeKind::AntiPattern => EdgeKind::FailedBy,
                _ => EdgeKind::DocumentedBy,
            };
            // Edge direction: subject --constrained_by--> knowledge (from subject to knowledge)
            // Or capability constrained_by knowledge: from applies_to to knowledge
            edges.push(Edge {
                from: normalize_ref(a, capabilities, symbols),
                to: kid.clone(),
                kind,
                note: Some(k.title.clone()),
            });
        }
        for s in &k.supersedes {
            edges.push(Edge {
                from: kid.clone(),
                to: format!("knowledge:{s}"),
                kind: EdgeKind::Supersedes,
                note: None,
            });
        }
        for s in &k.conflicts_with {
            edges.push(Edge {
                from: kid.clone(),
                to: format!("knowledge:{s}"),
                kind: EdgeKind::ConflictsWith,
                note: None,
            });
        }
        for r in &k.related {
            edges.push(Edge {
                from: kid.clone(),
                to: normalize_ref(r, capabilities, symbols),
                kind: EdgeKind::RelatedTo,
                note: None,
            });
        }
        if k.status == crate::knowledge::ir::Lifecycle::Deprecated {
            // no target
        }
    }

    edges.sort_by(|a, b| {
        (&a.from, a.kind.as_str(), &a.to).cmp(&(&b.from, b.kind.as_str(), &b.to))
    });
    edges.dedup_by(|a, b| a.from == b.from && a.to == b.to && a.kind == b.kind);
    edges
}

fn normalize_ref(r: &str, capabilities: &[CapabilityInfo], symbols: &[SymbolInfo]) -> String {
    if r.starts_with("capability:")
        || r.starts_with("symbol:")
        || r.starts_with("module:")
        || r.starts_with("knowledge:")
    {
        return r.to_string();
    }
    if let Some(c) = capabilities.iter().find(|c| c.id == r || c.name == r) {
        return format!("capability:{}", c.id);
    }
    if let Some(s) = symbols.iter().find(|s| s.id == r || s.name == r) {
        return format!("symbol:{}", s.id);
    }
    r.to_string()
}
