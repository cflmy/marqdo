//! KnowledgeLocatorV1 + view.reference_resolve (M04).

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct KnowledgeLocator {
    pub project_id: String,
    pub document_id: String,
    pub source_revision: Option<String>,
    pub generation_id: Option<String>,
    pub artifact_id: Option<String>,
    pub node_id: Option<String>,
    pub source_ref_id: Option<String>,
    pub span: Option<Value>,
    pub relation: String,
}

impl KnowledgeLocator {
    pub fn from_params(params: &Value) -> Result<Self, String> {
        let loc = params.get("locator").unwrap_or(params);
        Ok(Self {
            project_id: loc
                .get("project_id")
                .or_else(|| params.get("project"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            document_id: loc
                .get("document_id")
                .or_else(|| params.get("document"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            source_revision: loc
                .get("source_revision")
                .or_else(|| loc.get("revision"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            generation_id: loc
                .get("generation_id")
                .or_else(|| loc.get("generation"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            artifact_id: loc
                .get("artifact_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            node_id: loc
                .get("node_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            source_ref_id: loc
                .get("source_ref_id")
                .or_else(|| loc.get("ref_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            span: loc.get("span").cloned(),
            relation: loc
                .get("relation")
                .and_then(|v| v.as_str())
                .unwrap_or("cites")
                .to_string(),
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "schema": "KnowledgeLocatorV1",
            "schema_version": 1,
            "project_id": self.project_id,
            "document_id": self.document_id,
            "source_revision": self.source_revision,
            "generation_id": self.generation_id,
            "artifact_id": self.artifact_id,
            "node_id": self.node_id,
            "source_ref_id": self.source_ref_id,
            "span": self.span,
            "relation": self.relation,
        })
    }
}

/// Resolve a locator or current-document ref_id against a project snapshot.
/// Never silently falls back to "latest" when expected revision/generation mismatch.
pub fn resolve_reference(params: &Value) -> Result<Value, Value> {
    let locator = KnowledgeLocator::from_params(params).map_err(|e| {
        json!({"code": "locator.bad_request", "message": e})
    })?;
    if locator.project_id.is_empty() {
        return Err(json!({
            "code": "locator.bad_request",
            "message": "project_id/project required"
        }));
    }
    let root = PathBuf::from(&locator.project_id);
    let root = root.canonicalize().map_err(|e| {
        json!({"code": "locator.project_invalid", "message": format!("{e}")})
    })?;

    let options = crate::sema::CompileOptions::for_path(&root).map_err(|e| {
        json!({"code": "locator.compile_failed", "message": format!("{e:#}")})
    })?;
    let compilation = crate::sema::compile_project(&options).map_err(|e| {
        json!({"code": "locator.compile_failed", "message": format!("{e:#}")})
    })?;

    // Generation mismatch → version_unavailable / stale (no silent current).
    if let Some(expected_gen) = &locator.generation_id {
        if !expected_gen.is_empty() {
            let current_gen = semantic_generation(&compilation.semantics);
            if &current_gen != expected_gen {
                return Ok(json!({
                    "schema": "KnowledgeLocatorV1",
                    "schema_version": 1,
                    "resolution": "version_unavailable",
                    "precision": "none",
                    "locator": locator.to_json(),
                    "target": Value::Null,
                    "candidates": [],
                    "diagnostics": [{
                        "code": "generation_mismatch",
                        "message": format!(
                            "expected generation {expected_gen}, current projection is {current_gen}; refusing silent downgrade"
                        )
                    }],
                    "label": "引用版本不可用"
                }));
            }
        }
    }

    // Source revision check when document is known.
    if let (Some(expected_rev), true) = (
        &locator.source_revision,
        !locator.document_id.is_empty(),
    ) {
        if !expected_rev.is_empty() {
            let abs = resolve_under(&root, &locator.document_id)?;
            let raw = std::fs::read(&abs).map_err(|e| {
                json!({"code": "locator.source_unreadable", "message": format!("{e}")})
            })?;
            use sha2::{Digest, Sha256};
            let actual = format!("sha256:{:x}", Sha256::digest(&raw));
            if &actual != expected_rev {
                return Ok(json!({
                    "schema": "KnowledgeLocatorV1",
                    "schema_version": 1,
                    "resolution": "stale",
                    "precision": "document",
                    "locator": locator.to_json(),
                    "target": {
                        "document_id": locator.document_id,
                        "source_revision": actual,
                        "note": "source changed since citation"
                    },
                    "candidates": [],
                    "diagnostics": [{
                        "code": "source_revision_mismatch",
                        "message": format!("expected {expected_rev}, actual {actual}")
                    }],
                    "label": "该来源已变化"
                }));
            }
        }
    }

    let query = locator
        .artifact_id
        .clone()
        .or_else(|| locator.node_id.clone())
        .or_else(|| locator.source_ref_id.clone());

    let Some(q) = query else {
        if locator.document_id.is_empty() {
            return Ok(json!({
                "schema": "KnowledgeLocatorV1",
                "schema_version": 1,
                "resolution": "unresolved",
                "precision": "none",
                "locator": locator.to_json(),
                "target": Value::Null,
                "candidates": [],
                "diagnostics": [{"code": "missing_target", "message": "no artifact/node/ref id"}],
                "label": "无法定位该知识"
            }));
        }
        return Ok(json!({
            "schema": "KnowledgeLocatorV1",
            "schema_version": 1,
            "resolution": "resolved",
            "precision": "document",
            "locator": locator.to_json(),
            "target": {
                "document_id": locator.document_id,
                "relation": locator.relation,
            },
            "candidates": [],
            "diagnostics": [],
            "label": locator.document_id,
        }));
    };

    let mut matches: Vec<&crate::sema::SemanticArtifact> = compilation
        .semantics
        .artifacts
        .iter()
        .filter(|a| {
            a.artifact_id.0 == q
                || a.display_name == q
                || a.symbol_id
                    .as_ref()
                    .map(|s| s.as_str() == q)
                    .unwrap_or(false)
        })
        .collect();

    if !locator.document_id.is_empty() {
        let scoped: Vec<_> = matches
            .iter()
            .copied()
            .filter(|a| a.source.file == locator.document_id)
            .collect();
        if !scoped.is_empty() {
            matches = scoped;
        }
    }

    match matches.len() {
        0 => Ok(json!({
            "schema": "KnowledgeLocatorV1",
            "schema_version": 1,
            "resolution": "unresolved",
            "precision": "none",
            "locator": locator.to_json(),
            "target": Value::Null,
            "candidates": [],
            "diagnostics": [{"code": "not_found", "message": format!("no artifact matching {q}")}],
            "label": "尚未定位到具体知识"
        })),
        1 => {
            let a = matches[0];
            Ok(json!({
                "schema": "KnowledgeLocatorV1",
                "schema_version": 1,
                "resolution": "resolved",
                "precision": "artifact",
                "locator": locator.to_json(),
                "target": {
                    "artifact_id": a.artifact_id.0,
                    "display_name": a.display_name,
                    "document_id": a.source.file,
                    "span": {"line": a.source.line, "end_line": a.source.end_line},
                    "evidence": a.verification.evidence,
                    "relation": locator.relation,
                },
                "candidates": [],
                "diagnostics": [],
                "label": a.display_name,
            }))
        }
        _ => {
            let candidates: Vec<Value> = matches
                .iter()
                .map(|a| {
                    json!({
                        "artifact_id": a.artifact_id.0,
                        "display_name": a.display_name,
                        "document_id": a.source.file,
                        "line": a.source.line,
                    })
                })
                .collect();
            Ok(json!({
                "schema": "KnowledgeLocatorV1",
                "schema_version": 1,
                "resolution": "ambiguous",
                "precision": "none",
                "locator": locator.to_json(),
                "target": Value::Null,
                "candidates": candidates,
                "diagnostics": [{"code": "ambiguous", "message": format!("{} candidates for {q}", matches.len())}],
                "label": "存在多个候选"
            }))
        }
    }
}

fn semantic_generation(project: &crate::sema::SemanticProject) -> String {
    use sha2::{Digest, Sha256};
    let mut gh = Sha256::new();
    gh.update(project.project_id.as_bytes());
    for a in &project.artifacts {
        gh.update(a.hashes.artifact.0.as_bytes());
    }
    format!("{:x}", gh.finalize())[..16].to_string()
}

fn resolve_under(root: &Path, rel: &str) -> Result<PathBuf, Value> {
    let joined = root.join(rel);
    let canon = joined.canonicalize().map_err(|e| {
        json!({"code": "locator.document_not_found", "message": format!("{}: {e}", joined.display())})
    })?;
    if !canon.starts_with(root) {
        return Err(json!({
            "code": "locator.path_escape",
            "message": "document path escapes project root"
        }));
    }
    Ok(canon)
}
