//! Engineering Knowledge IR (L0–L4) — projections of `.mq.md`, not a second source of truth.

use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Lifecycle: draft → verified → stable → deprecated → superseded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lifecycle {
    Draft,
    Verified,
    Stable,
    Deprecated,
    Superseded,
}

impl Lifecycle {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Verified => "verified",
            Self::Stable => "stable",
            Self::Deprecated => "deprecated",
            Self::Superseded => "superseded",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "verified" => Self::Verified,
            "stable" => Self::Stable,
            "deprecated" => Self::Deprecated,
            "superseded" => Self::Superseded,
            _ => Self::Draft,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Evidence {
    pub sources: Vec<String>,
    pub tests: Vec<String>,
    pub executions: Vec<String>,
    pub commits: Vec<String>,
}

impl Evidence {
    pub fn to_json(&self) -> Value {
        json!({
            "sources": self.sources,
            "tests": self.tests,
            "executions": self.executions,
            "commits": self.commits,
        })
    }
}

/// Graded confidence with explanation (not a bare float).
#[derive(Debug, Clone)]
pub struct Confidence {
    pub level: String, // high | medium | low
    pub implementation: String,
    pub tests: String,
    pub runtime: String,
}

impl Default for Confidence {
    fn default() -> Self {
        Self {
            level: "medium".into(),
            implementation: "extracted".into(),
            tests: "unknown".into(),
            runtime: "unknown".into(),
        }
    }
}

impl Confidence {
    pub fn to_json(&self) -> Value {
        json!({
            "level": self.level,
            "evidence": {
                "implementation": self.implementation,
                "tests": self.tests,
                "runtime": self.runtime,
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct RepositoryInfo {
    pub title: String,
    pub root: String,
    pub languages: Vec<String>,
    pub module_count: usize,
    pub symbol_count: usize,
    pub capability_count: usize,
    pub knowledge_count: usize,
}

#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub id: String,
    pub resource: String,
    pub title: String,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
    pub responsibility: Option<String>,
    pub verified_by: Option<String>,
    pub sources: Vec<String>,
    pub status: Lifecycle,
}

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub id: String,
    pub name: String,
    pub module: String,
    pub resource: String,
    pub kind: String, // fn | object | type
    pub visibility: String,
    pub params: Vec<String>,
    pub calls: Vec<String>,
    pub called_by: Vec<String>,
    pub tests: Vec<String>,
    pub fingerprint: String,
    pub fingerprint_text: String,
    pub use_when: Option<String>,
    pub do_not_use_when: Option<String>,
    pub related: Vec<String>,
    pub reuse: Vec<String>,
    pub capability: Option<String>,
    pub status: Lifecycle,
    pub evidence: Evidence,
    pub confidence: Confidence,
}

#[derive(Debug, Clone)]
pub struct CapabilityInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub implemented_by: Vec<String>,
    pub modules: Vec<String>,
    pub status: Lifecycle,
    pub use_when: Option<String>,
    pub do_not_use_when: Option<String>,
    pub related: Vec<String>,
    pub evidence: Evidence,
    pub confidence: Confidence,
    pub introduced: Option<String>,
    pub verified: Option<String>,
    pub last_used: Option<String>,
    pub last_tested: Option<String>,
    pub possible_duplicates: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeKind {
    Fact,
    Decision,
    Constraint,
    Pattern,
    AntiPattern,
    Failure,
    Migration,
    Function,
    Capability,
    Other(String),
}

impl KnowledgeKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Fact => "Fact",
            Self::Decision => "Decision",
            Self::Constraint => "Constraint",
            Self::Pattern => "Pattern",
            Self::AntiPattern => "AntiPattern",
            Self::Failure => "Failure",
            Self::Migration => "Migration",
            Self::Function => "Function",
            Self::Capability => "Capability",
            Self::Other(s) => s.as_str(),
        }
    }

    pub fn parse(s: &str) -> Self {
        let t = s.trim();
        let bare = t.strip_prefix("Marqdo ").unwrap_or(t);
        match bare {
            "Fact" => Self::Fact,
            "Decision" | "ADR" => Self::Decision,
            "Constraint" => Self::Constraint,
            "Pattern" => Self::Pattern,
            "AntiPattern" | "Anti-Pattern" => Self::AntiPattern,
            "Failure" => Self::Failure,
            "Migration" => Self::Migration,
            "Function" => Self::Function,
            "Capability" => Self::Capability,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn subdir(&self) -> &'static str {
        match self {
            Self::Fact => "facts",
            Self::Decision => "decisions",
            Self::Constraint => "constraints",
            Self::Pattern | Self::AntiPattern => "patterns",
            Self::Failure => "failures",
            Self::Migration => "migrations",
            Self::Function | Self::Capability | Self::Other(_) => "facts",
        }
    }
}

#[derive(Debug, Clone)]
pub struct KnowledgeItem {
    pub id: String,
    pub kind: KnowledgeKind,
    pub title: String,
    pub resource: String,
    pub status: Lifecycle,
    pub body_summary: String,
    pub applies_to: Vec<String>,
    pub supersedes: Vec<String>,
    pub conflicts_with: Vec<String>,
    pub related: Vec<String>,
    pub stale_after: Option<String>,
    pub evidence: Evidence,
    pub confidence: Confidence,
    /// Raw source text (copied into knowledge/ when authored under src).
    pub source_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeKind {
    Implements,
    Uses,
    DependsOn,
    TestedBy,
    DocumentedBy,
    ConstrainedBy,
    DecidedBy,
    Supersedes,
    ConflictsWith,
    DeprecatedBy,
    DerivedFrom,
    FailedBy,
    RelatedTo,
}

impl EdgeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Implements => "implements",
            Self::Uses => "uses",
            Self::DependsOn => "depends_on",
            Self::TestedBy => "tested_by",
            Self::DocumentedBy => "documented_by",
            Self::ConstrainedBy => "constrained_by",
            Self::DecidedBy => "decided_by",
            Self::Supersedes => "supersedes",
            Self::ConflictsWith => "conflicts_with",
            Self::DeprecatedBy => "deprecated_by",
            Self::DerivedFrom => "derived_from",
            Self::FailedBy => "failed_by",
            Self::RelatedTo => "related_to",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: EdgeKind,
    pub note: Option<String>,
}

impl Edge {
    pub fn to_json(&self) -> Value {
        let mut m = json!({
            "from": self.from,
            "to": self.to,
            "kind": self.kind.as_str(),
        });
        if let Some(ref n) = self.note {
            m["note"] = json!(n);
        }
        m
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReuseDecision {
    Reuse,
    Adapt,
    Create,
}

impl ReuseDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Reuse => "REUSE",
            Self::Adapt => "ADAPT",
            Self::Create => "CREATE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReuseCandidate {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub score: f64,
    pub resource: Option<String>,
    pub reason: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ReuseResult {
    pub decision: ReuseDecision,
    pub confidence: f64,
    pub capability: Option<String>,
    pub symbol: Option<String>,
    pub source: Option<String>,
    pub reason: Vec<String>,
    pub candidates: Vec<ReuseCandidate>,
    pub constraints: Vec<String>,
    pub decisions: Vec<String>,
    pub failures: Vec<String>,
    pub create_allowed: bool,
    pub recommended: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct KnowledgeGraph {
    pub repository: RepositoryInfo,
    pub modules: Vec<ModuleInfo>,
    pub symbols: Vec<SymbolInfo>,
    pub capabilities: Vec<CapabilityInfo>,
    pub knowledge: Vec<KnowledgeItem>,
    pub edges: Vec<Edge>,
    pub generated_by: String,
}

impl KnowledgeGraph {
    pub fn symbol_by_id(&self, id: &str) -> Option<&SymbolInfo> {
        self.symbols.iter().find(|s| s.id == id || s.name == id)
    }

    pub fn capability_by_id(&self, id: &str) -> Option<&CapabilityInfo> {
        self.capabilities
            .iter()
            .find(|c| c.id == id || c.name == id)
    }

    pub fn neighbors(&self, id: &str, kind: Option<EdgeKind>) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|e| {
                (e.from == id || e.to == id)
                    && kind.as_ref().map(|k| &e.kind == k).unwrap_or(true)
            })
            .collect()
    }

    pub fn to_graph_json(&self) -> Value {
        let mut nodes = Vec::new();
        nodes.push(json!({
            "id": "repo",
            "kind": "repository",
            "title": self.repository.title,
        }));
        for m in &self.modules {
            nodes.push(json!({
                "id": format!("module:{}", m.id),
                "kind": "module",
                "title": m.title,
                "resource": m.resource,
            }));
        }
        for s in &self.symbols {
            nodes.push(json!({
                "id": format!("symbol:{}", s.id),
                "kind": "symbol",
                "title": s.name,
                "resource": s.resource,
                "fingerprint": s.fingerprint,
            }));
        }
        for c in &self.capabilities {
            nodes.push(json!({
                "id": format!("capability:{}", c.id),
                "kind": "capability",
                "title": c.name,
                "status": c.status.as_str(),
            }));
        }
        for k in &self.knowledge {
            nodes.push(json!({
                "id": format!("knowledge:{}", k.id),
                "kind": k.kind.as_str(),
                "title": k.title,
                "status": k.status.as_str(),
            }));
        }
        json!({
            "type": "Marqdo Knowledge Graph",
            "generated": { "by": self.generated_by },
            "repository": {
                "title": self.repository.title,
                "root": self.repository.root,
                "modules": self.repository.module_count,
                "symbols": self.repository.symbol_count,
                "capabilities": self.repository.capability_count,
                "knowledge": self.repository.knowledge_count,
            },
            "nodes": nodes,
            "edge_count": self.edges.len(),
        })
    }

    pub fn to_edges_json(&self) -> Value {
        json!({
            "type": "Marqdo Knowledge Edges",
            "generated": { "by": self.generated_by },
            "edges": self.edges.iter().map(|e| e.to_json()).collect::<Vec<_>>(),
        })
    }

    pub fn index_map(&self) -> BTreeMap<String, Value> {
        let mut m = BTreeMap::new();
        for s in &self.symbols {
            m.insert(
                format!("symbol:{}", s.id),
                json!({
                    "name": s.name,
                    "module": s.module,
                    "resource": s.resource,
                    "capability": s.capability,
                    "fingerprint": s.fingerprint_text,
                }),
            );
        }
        for c in &self.capabilities {
            m.insert(
                format!("capability:{}", c.id),
                json!({
                    "name": c.name,
                    "description": c.description,
                    "implemented_by": c.implemented_by,
                }),
            );
        }
        for k in &self.knowledge {
            m.insert(
                format!("knowledge:{}", k.id),
                json!({
                    "kind": k.kind.as_str(),
                    "title": k.title,
                    "summary": k.body_summary,
                }),
            );
        }
        m
    }
}
