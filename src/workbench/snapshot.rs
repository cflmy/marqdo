//! Snapshot manifest for apps to retain historical evidence (M04).
//! Framework does not store history; missing snapshot → version_unavailable.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Build a retainable snapshot manifest for the current projection.
pub fn build_snapshot_manifest(
    project_root: &Path,
    document_rel: &str,
    source_bytes: &[u8],
    project: &crate::sema::SemanticProject,
    renderer_version: &str,
) -> Value {
    let source_revision = format!("sha256:{:x}", Sha256::digest(source_bytes));
    let mut dep_revs = Vec::new();
    for a in &project.artifacts {
        if a.source.file == document_rel {
            continue;
        }
        dep_revs.push(json!({
            "document_id": a.source.file,
            "artifact_id": a.artifact_id.0,
            "artifact_hash": a.hashes.artifact.0,
        }));
    }
    let mut artifact_hashes = Vec::new();
    for a in &project.artifacts {
        if a.source.file == document_rel {
            artifact_hashes.push(json!({
                "artifact_id": a.artifact_id.0,
                "hash": a.hashes.artifact.0,
            }));
        }
    }
    let mut gh = Sha256::new();
    gh.update(project.project_id.as_bytes());
    for a in &project.artifacts {
        gh.update(a.hashes.artifact.0.as_bytes());
    }
    let generation_id = format!("{:x}", gh.finalize())[..16].to_string();

    json!({
        "schema": "SnapshotManifestV1",
        "schema_version": 1,
        "project_id": project_root.display().to_string(),
        "document_id": document_rel,
        "source_revision": source_revision,
        "generation_id": generation_id,
        "dependency_revisions": dep_revs,
        "artifact_hashes": artifact_hashes,
        "renderer_version": renderer_version,
        "contract_schema": "DocumentViewV1",
        "retention": {
            "owner": "application",
            "note": "framework does not retain historical bodies; missing snapshot → version_unavailable"
        }
    })
}

/// When an expected revision/generation is requested but no app-provided snapshot
/// resolver is present, return the honest unavailable contract.
pub fn version_unavailable_result(
    locator: &Value,
    reason: &str,
    expected: &str,
    actual: Option<&str>,
) -> Value {
    json!({
        "schema": "KnowledgeLocatorV1",
        "schema_version": 1,
        "resolution": "version_unavailable",
        "precision": "none",
        "locator": locator,
        "target": Value::Null,
        "candidates": [],
        "diagnostics": [{
            "code": "snapshot_not_retained",
            "message": reason,
            "expected": expected,
            "actual": actual,
        }],
        "label": "引用版本不可用",
        "snapshot": {
            "available": false,
            "owner": "application"
        }
    })
}
