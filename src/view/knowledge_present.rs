//! V1 KnowledgePresentation model — structured facts shared by body, outline, and detail.
//!
//! Semantic facts come from SemanticProject; this module does not invent verification.

use crate::sema::{SemanticArtifact, SemanticProject};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct KnowledgeNode {
    pub node_id: String,
    pub artifact_id: Option<String>,
    pub display_name: String,
    pub kind: String,
    pub source_file: String,
    pub source_line: u32,
    pub signature: Option<String>,
    pub effects: Vec<String>,
    pub evidence: String,
    pub dependencies: Vec<String>,
    pub artifact_digest: String,
}

#[derive(Debug, Clone)]
pub struct KnowledgePresentation {
    pub document_id: String,
    pub title: String,
    pub source_digest: String,
    /// V2: semantic generation / run matching token (empty when not compiled under CURRENT).
    pub semantic_generation: String,
    /// V2: page token for SSE stale-packet rejection (digest of document facts).
    pub page_token: String,
    pub execution_state: String,
    pub nodes: Vec<KnowledgeNode>,
    pub diagnostics: Vec<String>,
}

impl KnowledgePresentation {
    pub fn from_project(project: &SemanticProject, rel: &str, title: &str) -> Self {
        let mut nodes = Vec::new();
        for artifact in &project.artifacts {
            if artifact.source.file != rel {
                continue;
            }
            if artifact.kind == "module" {
                continue;
            }
            nodes.push(node_from_artifact(artifact));
        }
        let mut hasher_parts: Vec<&str> = project
            .artifacts
            .iter()
            .filter(|a| a.source.file == rel)
            .map(|a| a.hashes.artifact.0.as_str())
            .collect();
        hasher_parts.sort();
        let mut h = Sha256::new();
        h.update(project.project_id.as_bytes());
        h.update(rel.as_bytes());
        for part in &hasher_parts {
            h.update(part.as_bytes());
        }
        let digest_hex = format!("{:x}", h.finalize());
        let source_digest = format!("sha256:{digest_hex}");
        // page_token is filled by caller from raw source so SSE can match without recompile.
        let page_token = String::new();
        let semantic_generation = {
            let mut gh = Sha256::new();
            gh.update(project.project_id.as_bytes());
            for a in &project.artifacts {
                gh.update(a.hashes.artifact.0.as_bytes());
            }
            format!("{:x}", gh.finalize())[..16].to_string()
        };
        let mut diagnostics = Vec::new();
        for n in &nodes {
            if n.evidence == "sourced" {
                // Never imply sourced == verified (V2).
                diagnostics.push(format!(
                    "{}: 已关联来源，内容尚未核验",
                    n.display_name
                ));
            }
        }
        Self {
            document_id: rel.to_string(),
            title: title.to_string(),
            source_digest,
            semantic_generation,
            page_token,
            execution_state: "not_run".into(),
            nodes,
            diagnostics,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({
            "schema_version": 1,
            "document_id": self.document_id,
            "title": self.title,
            "source_digest": self.source_digest,
            "semantic_generation": self.semantic_generation,
            "page_token": self.page_token,
            "execution_state": self.execution_state,
            "nodes": self.nodes.iter().map(|n| json!({
                "node_id": n.node_id,
                "artifact_id": n.artifact_id,
                "display_name": n.display_name,
                "kind": n.kind,
                "source": {"file": n.source_file, "line": n.source_line},
                "signature": n.signature,
                "effects": n.effects,
                "evidence": n.evidence,
                "dependencies": n.dependencies,
                "artifact_digest": n.artifact_digest,
                "verified_claim": false,
            })).collect::<Vec<_>>(),
            "local_deps": self.nodes.iter().flat_map(|n| {
                n.dependencies.iter().map(|d| json!({
                    "from": n.node_id,
                    "to": d,
                })).collect::<Vec<_>>()
            }).collect::<Vec<_>>(),
            "diagnostics": self.diagnostics,
        })
    }
}

/// V2: whether an SSE result packet may update the live document structure.
pub fn sse_accepts_packet(page_token: &str, packet_token: Option<&str>) -> bool {
    match packet_token {
        None | Some("") => true, // legacy packets without token still apply
        Some(_tok) if page_token.is_empty() => true,
        Some(tok) => tok == page_token,
    }
}

/// Stable page token from raw source bytes (used by SSE run path).
pub fn page_token_from_source(rel: &str, source: &str) -> String {
    let mut h = Sha256::new();
    h.update(rel.as_bytes());
    h.update(source.as_bytes());
    let hex = format!("{:x}", h.finalize());
    hex[..hex.len().min(32)].to_string()
}

fn node_from_artifact(artifact: &SemanticArtifact) -> KnowledgeNode {
    let signature = artifact.signature.as_ref().map(|sig| {
        let params = sig
            .params
            .iter()
            .map(|p| format!("{}:{}", p.name, p.ty))
            .collect::<Vec<_>>()
            .join(", ");
        format!("({params}) → {}", sig.returns)
    });
    KnowledgeNode {
        node_id: artifact
            .symbol_id
            .as_ref()
            .map(|id| id.as_str().to_string())
            .unwrap_or_else(|| artifact.artifact_id.0.clone()),
        artifact_id: Some(artifact.artifact_id.0.clone()),
        display_name: artifact.display_name.clone(),
        kind: artifact.kind.clone(),
        source_file: artifact.source.file.clone(),
        source_line: artifact.source.line,
        signature,
        effects: artifact.effects.values(),
        evidence: artifact.verification.evidence.clone(),
        dependencies: artifact.dependencies.iter().map(|d| d.0.clone()).collect(),
        artifact_digest: artifact.hashes.artifact.0.clone(),
    }
}

/// Pull `title:` from YAML frontmatter without a full YAML parser.
pub fn title_from_source(source: &str, fallback: &str) -> String {
    let mut lines = source.lines();
    if lines.next().map(|l| l.trim_end()) != Some("---") {
        return fallback.to_string();
    }
    for line in lines {
        let t = line.trim_end();
        if t.trim() == "---" {
            break;
        }
        let trimmed = t.trim();
        if let Some(rest) = trimmed.strip_prefix("title:") {
            let title = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            if !title.is_empty() {
                return title;
            }
        }
    }
    fallback.to_string()
}
