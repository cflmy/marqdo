//! Reviewable semantic link lock and reverse-impact queries.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use super::ir::{SemanticProject, SCHEMA_VERSION};

#[derive(Debug, Clone, Default)]
pub struct LockReport {
    pub status: String,
    pub missing: Vec<String>,
    pub changed: Vec<String>,
    pub removed: Vec<String>,
    /// Per-artifact which digest class drifted (G08).
    pub classes: Vec<Value>,
}

impl LockReport {
    pub fn is_clean(&self) -> bool {
        self.status == "clean"
    }

    pub fn to_json(&self) -> Value {
        json!({
            "schema_version": SCHEMA_VERSION,
            "status": self.status,
            "missing": self.missing,
            "changed": self.changed,
            "removed": self.removed,
            "classes": self.classes,
        })
    }
}

pub fn update_links_lock(out_dir: &Path, project: &SemanticProject) -> Result<()> {
    fs::create_dir_all(out_dir)?;
    let target = out_dir.join("links.lock.json");
    let staged = out_dir.join(".links.lock.json.tmp");
    fs::write(&staged, serde_json::to_vec_pretty(&lock_json(project))?)
        .with_context(|| format!("write {}", staged.display()))?;
    fs::rename(&staged, &target)
        .with_context(|| format!("replace {}", target.display()))?;
    Ok(())
}

pub fn check_links_lock(out_dir: &Path, project: &SemanticProject) -> Result<LockReport> {
    let path = out_dir.join("links.lock.json");
    if !path.exists() {
        return Ok(LockReport {
            status: "missing".into(),
            missing: project.artifacts.iter().map(|a| a.artifact_id.0.clone()).collect(),
            ..LockReport::default()
        });
    }
    let raw: Value = serde_json::from_str(&fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?)
        .with_context(|| format!("parse {}", path.display()))?;
    if raw.get("schema_version").and_then(Value::as_u64) != Some(SCHEMA_VERSION as u64) {
        return Ok(LockReport {
            status: "schema_mismatch".into(),
            changed: vec!["schema_version".into()],
            ..LockReport::default()
        });
    }
    let expected = artifact_lock_entries(project);
    let actual = raw.get("artifacts").and_then(Value::as_object).cloned().unwrap_or_default();
    let mut report = LockReport::default();
    for (id, entry) in &expected {
        match actual.get(id).and_then(Value::as_object) {
            None => report.missing.push(id.clone()),
            Some(found) => {
                let mut drifted = Vec::new();
                for key in ["interface", "implementation", "evidence", "artifact", "closure"] {
                    let want = entry.get(key).and_then(Value::as_str);
                    let got = found.get(key).and_then(Value::as_str);
                    if want != got {
                        drifted.push(key.to_string());
                    }
                }
                if !drifted.is_empty() {
                    report.changed.push(id.clone());
                    report.classes.push(json!({
                        "artifact_id": id,
                        "changed": drifted,
                    }));
                }
            }
        }
    }
    for id in actual.keys() {
        if !expected.contains_key(id) {
            report.removed.push(id.clone());
        }
    }
    report.missing.sort();
    report.changed.sort();
    report.removed.sort();
    report.status = if report.missing.is_empty()
        && report.changed.is_empty()
        && report.removed.is_empty()
    {
        "clean".into()
    } else {
        "outdated".into()
    };
    Ok(report)
}

/// Static reverse dependency closure.  Queries can be an artifact/symbol URI,
/// a declaration suffix, a project-relative source filename, or a corpus
/// provenance URI (`supports` relation — M02).
pub fn impact(project: &SemanticProject, query: &str) -> Value {
    let mut seeds = BTreeSet::new();
    for artifact in &project.artifacts {
        if artifact.artifact_id.0 == query
            || artifact.symbol_id.as_ref().is_some_and(|id| id.as_str() == query)
            || artifact.display_name == query
            || artifact.source.file == query
            || artifact.source.file.ends_with(query)
            || std::path::Path::new(&artifact.source.file)
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|stem| stem == query || query.ends_with(stem))
            || artifact.artifact_id.0.ends_with(query)
            || artifact.provenance.iter().any(|p| {
                p.relation == "supports"
                    && (p.uri == query
                        || p.uri.ends_with(query)
                        || query.ends_with(&p.uri)
                        || p.locator
                            .as_ref()
                            .is_some_and(|loc| format!("{}#{}", p.uri, loc) == query))
            })
        {
            seeds.insert(artifact.artifact_id.0.clone());
        }
    }
    let mut reverse: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for artifact in &project.artifacts {
        for dependency in &artifact.dependencies {
            reverse
                .entry(dependency.0.clone())
                .or_default()
                .push(artifact.artifact_id.0.clone());
        }
        // Corpus URI → citing artifact (provenance supports edges).
        for prov in &artifact.provenance {
            if prov.relation != "supports" {
                continue;
            }
            reverse
                .entry(prov.uri.clone())
                .or_default()
                .push(artifact.artifact_id.0.clone());
            if let Some(loc) = &prov.locator {
                reverse
                    .entry(format!("{}#{}", prov.uri, loc))
                    .or_default()
                    .push(artifact.artifact_id.0.clone());
            }
        }
    }
    // NarrativeRef edges: step → symbol; impact includes the step's owning artifact.
    for step in &project.steps {
        if step.kind != "narrate" {
            continue;
        }
        let owner = step
            .id
            .0
            .split("#step-")
            .next()
            .unwrap_or(step.id.0.as_str())
            .to_string();
        for target in &step.calls {
            reverse
                .entry(target.as_str().to_string())
                .or_default()
                .push(owner.clone());
        }
    }
    // Also seed reverse walk from corpus URI keys when query matches them.
    if reverse.contains_key(query) {
        seeds.insert(query.to_string());
    }
    let mut queue: VecDeque<String> = seeds.iter().cloned().collect();
    let mut affected = BTreeSet::new();
    while let Some(current) = queue.pop_front() {
        if !affected.insert(current.clone()) {
            continue;
        }
        if let Some(users) = reverse.get(&current) {
            for user in users {
                queue.push_back(user.clone());
            }
        }
    }
    // Drop pure corpus URI keys that are not artifacts from the affected set display.
    let entries = affected
        .iter()
        .filter_map(|id| project.artifacts.iter().find(|a| &a.artifact_id.0 == id))
        .map(|artifact| {
            let classes = {
                let mut c = Vec::new();
                if !artifact.hashes.interface.0.is_empty() {
                    c.push("interface");
                }
                if !artifact.hashes.implementation.0.is_empty() {
                    c.push("implementation");
                }
                if !artifact.hashes.evidence.0.is_empty() {
                    c.push("evidence");
                }
                if !artifact.dependencies.is_empty() {
                    c.push("dependency");
                }
                c
            };
            json!({
                "artifact_id": artifact.artifact_id.0,
                "symbol_id": artifact.symbol_id.as_ref().map(|id| id.as_str()),
                "display_name": artifact.display_name,
                "kind": artifact.kind,
                "source": {"file": artifact.source.file, "line": artifact.source.line},
                "hash_classes": classes,
                "evidence": artifact.verification.evidence,
            })
        })
        .collect::<Vec<_>>();
    json!({"schema_version":SCHEMA_VERSION,"query":query,"resolved":!seeds.is_empty(),"seeds":seeds,"affected":entries})
}

fn artifact_lock_entries(project: &SemanticProject) -> BTreeMap<String, Value> {
    project
        .artifacts
        .iter()
        .map(|artifact| {
            (
                artifact.artifact_id.0.clone(),
                json!({
                    "interface": artifact.hashes.interface.0,
                    "implementation": artifact.hashes.implementation.0,
                    "evidence": artifact.hashes.evidence.0,
                    "artifact": artifact.hashes.artifact.0,
                    "closure": artifact.hashes.closure.0,
                }),
            )
        })
        .collect()
}

fn lock_json(project: &SemanticProject) -> Value {
    let mut artifacts = serde_json::Map::new();
    for artifact in &project.artifacts {
        artifacts.insert(artifact.artifact_id.0.clone(), json!({
            "interface": artifact.hashes.interface.0,
            "implementation": artifact.hashes.implementation.0,
            "evidence": artifact.hashes.evidence.0,
            "artifact": artifact.hashes.artifact.0,
            "closure": artifact.hashes.closure.0,
            "dependencies": artifact.dependencies.iter().map(|item| &item.0).collect::<Vec<_>>(),
        }));
    }
    json!({"schema_version":SCHEMA_VERSION,"project":project.project_id,"artifacts":artifacts})
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sema::{compile_project, CompileOptions};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn root() -> PathBuf { let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos(); let path = std::env::temp_dir().join(format!("marqdo-lock-{id}")); fs::create_dir_all(&path).unwrap(); path }

    #[test]
    fn lock_detects_semantic_change_and_impact_is_reverse_closed() {
        let root = root();
        fs::write(root.join("main.mq.md"), "# add\n\n*1*\n\n# use\n\n*add()*").unwrap();
        let options = CompileOptions::for_path(&root).unwrap();
        let first = compile_project(&options).unwrap();
        let out = root.join(".marqdo"); update_links_lock(&out, &first.semantics).unwrap();
        assert!(check_links_lock(&out, &first.semantics).unwrap().is_clean());
        let response = impact(&first.semantics, "add");
        assert!(response["affected"].as_array().unwrap().len() >= 1);
        fs::write(root.join("main.mq.md"), "# add\n\n*2*\n\n# use\n\n*add()*").unwrap();
        let second = compile_project(&options).unwrap();
        assert_eq!(check_links_lock(&out, &second.semantics).unwrap().status, "outdated");
        fs::remove_dir_all(root).unwrap();
    }
}
