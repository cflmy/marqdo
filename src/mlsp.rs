//! Marqdo Language Server for AI（MLSP）—— AI 的可查询工具面（T3.1/T3.2，T2.6 并入 `schema`）。
//!
//! 目标（[three-problems.md](../../doc/roadmap/three-problems.md) §3）：**AI 不必背 Marqdo
//! 语法细节**——把「记语法」换成「查工具」。语法唯一事实来源是 `parse::CORE_CONSTRUCTS`
//! （与 `doc/design/core-surface.md` 由 CI 守卫同步）；本模块只加**检索别名**，不定义语法。
//!
//! 行分隔 JSON 协议：每行一个请求 `{"id":…,"method":…,"params":{…}}`，
//! 每行一个响应 `{"id":…,"ok":true,"result":{…}}` / `{"id":…,"ok":false,"error":{…}}`。
//!
//! | method | 作用 |
//! |--------|------|
//! | `locate` | 语法/符号定位：构造卡片，或定义位置 + 契约 |
//! | `syntax` | 语法问答：关键字/别名 → 构造卡片 |
//! | `validate` | 程序校验：结构化诊断数组（parse + 契约 check） |
//! | `repair_targets` | 修复靶点：错误 → **有界**修复目标（越界必拒，护栏） |
//! | `repair_apply` | 应用行编辑（全有或全无；越界 ⇒ `mlsp.repair_out_of_scope` abstain） |
//! | `schema` | 契约查询（T2.6）：单元名 → 形参/返回/字段契约 |

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::ast::{Function, Module};
use crate::check::check_module;
use crate::contract::{Contract, Ty};
use crate::diagnostics::Diagnostic;
use crate::load;
use crate::parse::CORE_CONSTRUCTS;

/// 行分隔 JSON：一行请求 → 一行响应（永不 panic；错误也是结构化 JSON）。
pub fn handle_line(line: &str) -> String {
    let trimmed = line.trim();
    let id = match serde_json::from_str::<Value>(trimmed) {
        Ok(req) => req.get("id").cloned().unwrap_or(Value::Null),
        Err(e) => {
            return json!({
                "id": null,
                "ok": false,
                "error": {"code": "mlsp.bad_request", "message": format!("请求不是合法 JSON: {e}")}
            })
            .to_string();
        }
    };
    let req: Value = serde_json::from_str(trimmed).expect("already parsed");
    match dispatch(&req) {
        Ok(result) => json!({"id": id, "ok": true, "result": result}).to_string(),
        Err(error) => json!({"id": id, "ok": false, "error": error}).to_string(),
    }
}

fn dispatch(req: &Value) -> Result<Value, Value> {
    let params = req.get("params").cloned().unwrap_or(json!({}));
    match req.get("method").and_then(|m| m.as_str()) {
        Some("locate") => locate(&params),
        Some("syntax") => syntax(&params),
        Some("validate") => validate(&params),
        Some("repair_targets") => repair_targets(&params),
        Some("repair_apply") => repair_apply(&params),
        Some("schema") => schema(&params),
        Some("artifact.find") => artifact_find(&params),
        Some("artifact.inspect") => artifact_inspect(&params),
        Some("link.resolve") => link_resolve(&params),
        Some("artifact.verify") => artifact_verify(&params),
        Some("artifact.impact") => artifact_impact(&params),
        Some("artifact.bind") => artifact_bind(&params),
        Some("artifact.execute") => artifact_execute(&params),
        Some("artifact.reuse_execute") => artifact_reuse_execute(&params),
        Some("document.extract") => document_extract(&params),
        Some("artifact.patch") => artifact_patch(&params),
        Some("artifact.extract_pure") => artifact_extract_pure(&params),
        // Workbench M00–M04: capability probe + read-only view + locator.
        Some("capabilities") => Ok(crate::workbench::probe_capabilities(&params)),
        #[cfg(feature = "view")]
        Some("view.document") => view_document(&params),
        #[cfg(not(feature = "view"))]
        Some("view.document") => Err(err(
            "mlsp.unsupported",
            "view.document requires the view feature",
        )),
        #[cfg(feature = "view")]
        Some("resource.view") => resource_view(&params),
        #[cfg(not(feature = "view"))]
        Some("resource.view") => Err(err(
            "mlsp.unsupported",
            "resource.view requires the view feature",
        )),
        Some("view.reference_resolve") => crate::workbench::resolve_reference(&params),
        // `<TAB>` M9: occlusion label search (two-tier vocabulary).
        Some("labels.search") => labels_search(&params),
        // `<TAB>` M6: one-shot session summary (T1 staleness seen/fixed).
        Some("session.summary") => session_summary(),
        other => Err(err(
            "mlsp.unknown_method",
            format!("未知方法 {other:?}；可用：locate / syntax / validate / repair_targets / repair_apply / schema / artifact.find / artifact.inspect / link.resolve / artifact.verify / artifact.impact / artifact.bind / artifact.execute / artifact.reuse_execute / artifact.patch / artifact.extract_pure / document.extract / capabilities / view.document / resource.view / view.reference_resolve"),
        )),
    }
}

#[cfg(feature = "view")]
fn view_document(params: &Value) -> Result<Value, Value> {
    let req = crate::workbench::DocumentViewRequest::from_params(params)
        .map_err(|e| err("mlsp.bad_request", e))?;
    crate::workbench::build_document_view(&req)
}

#[cfg(feature = "view")]
fn resource_view(params: &Value) -> Result<Value, Value> {
    let req = crate::workbench::ResourceViewRequest::from_params(params)
        .map_err(|e| err("mlsp.bad_request", e))?;
    crate::workbench::build_resource_view(&req)
}

fn artifact_extract_pure(params: &Value) -> Result<Value, Value> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| err("mlsp.bad_request", "artifact.extract_pure 需要 name"))?;
    let expression = params
        .get("expression")
        .and_then(Value::as_str)
        .ok_or_else(|| err("mlsp.bad_request", "artifact.extract_pure 需要 expression"))?;
    match crate::sema::plan_pure_extract(name, expression) {
        Ok(plan) => Ok(json!({
            "schema_version": 1,
            "name": plan.name,
            "params": plan.params,
            "expression": plan.expression,
            "heading": plan.heading,
            "call": plan.call,
            "effects": ["pure"],
            "note": "预览补丁；写入请配合 artifact.patch / 人工审阅",
        })),
        Err(e) => Err(json!({"code": e.code, "message": e.message})),
    }
}

// ---------- Executable-knowledge artifact protocol ----------

/// All artifact methods compile the same semantic project as the CLI.  They
/// never fall back to EKC's lexical search when a binding decision is needed.
fn semantic_project(params: &Value) -> Result<crate::sema::Compilation, Value> {
    if params.get("source").is_some() {
        return Err(err("mlsp.overlay_requires_path", "artifact 方法需要 params.project/path；内存 overlay 尚未持久化到项目边界"));
    }
    let root = params.get("project").or_else(|| params.get("path"))
        .and_then(Value::as_str).unwrap_or(".");
    let root = PathBuf::from(root);
    let project_root = if root.is_file() {
        root.parent().unwrap_or_else(|| Path::new(".")).to_path_buf()
    } else { root.clone() };
    crate::sema::compile_project(&crate::sema::CompileOptions {
        project_root,
        entry: root.is_file().then_some(root),
        lock_mode: crate::sema::LockMode::Off,
    }).map_err(|e| err("mlsp.semantic_compile", format!("{e:#}")))
}

fn artifact_query<'a>(project: &'a crate::sema::SemanticProject, query: &str) -> Vec<&'a crate::sema::SemanticArtifact> {
    project.artifacts.iter().filter(|artifact| {
        artifact.artifact_id.0 == query || artifact.symbol_id.as_ref().is_some_and(|id| id.as_str() == query)
            || artifact.display_name == query || artifact.artifact_id.0.ends_with(query)
    }).collect()
}

fn document_extract(params: &Value) -> Result<Value, Value> {
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| err("mlsp.bad_request", "document.extract 需要 path"))?;
    let hint = params
        .get("format")
        .and_then(Value::as_str)
        .map(|s| s.to_string());
    let doc = crate::document_extract::extract_document(&crate::document_extract::DocumentSource {
        path: PathBuf::from(path),
        format_hint: hint,
    });
    Ok(doc.to_json())
}

fn artifact_find(params: &Value) -> Result<Value, Value> {
    let compilation = semantic_project(params)?;
    let query = params.get("query").and_then(Value::as_str).unwrap_or("").to_ascii_lowercase();
    let kind = params.get("kind").and_then(Value::as_str);
    let limit = params.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize;
    let mut matches: Vec<Value> = compilation.semantics.artifacts.iter().filter(|artifact| {
        kind.map(|kind| kind == artifact.kind).unwrap_or(true) && (query.is_empty()
            || artifact.display_name.to_ascii_lowercase().contains(&query)
            || artifact.artifact_id.0.to_ascii_lowercase().contains(&query))
    }).take(limit).map(|artifact| json!({
        "artifact_id": artifact.artifact_id.0, "symbol_id": artifact.symbol_id.as_ref().map(|id| id.as_str()),
        "display_name": artifact.display_name, "kind": artifact.kind,
        "effects": artifact.effects.values(), "source": {"file":artifact.source.file,"line":artifact.source.line},
    })).collect();
    matches.sort_by(|a, b| a["artifact_id"].as_str().cmp(&b["artifact_id"].as_str()));
    Ok(json!({"schema_version":1,"discovery_only":true,"candidates":matches}))
}

fn artifact_inspect(params: &Value) -> Result<Value, Value> {
    let compilation = semantic_project(params)?;
    let query = params.get("artifact_id").or_else(|| params.get("symbol_id")).or_else(|| params.get("id"))
        .and_then(Value::as_str).ok_or_else(|| err("mlsp.bad_request", "artifact.inspect 需要 artifact_id 或 symbol_id"))?;
    let candidates = artifact_query(&compilation.semantics, query);
    if candidates.is_empty() { return Err(err("artifact.unresolved", format!("没有制品 `{query}`"))); }
    if candidates.len() != 1 { return Err(json!({"code":"artifact.ambiguous","message":format!("制品 `{query}` 不唯一"),"candidates":candidates.iter().map(|a|&a.artifact_id.0).collect::<Vec<_>>() })); }
    let artifact = candidates[0];
    // Version contract: expected source/generation mismatch must not silently return current.
    check_artifact_version_params(params, &compilation, artifact)?;
    Ok(crate::sema::artifact_json(artifact))
}

/// Reject inspect when caller pinned an expected source revision or generation that
/// does not match the current project snapshot (`artifact.version_mismatch`).
fn check_artifact_version_params(
    params: &Value,
    compilation: &crate::sema::Compilation,
    artifact: &crate::sema::SemanticArtifact,
) -> Result<(), Value> {
    let expected_rev = params
        .get("expected_source_revision")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    if let Some(expected) = expected_rev {
        let project_root = params
            .get("project")
            .or_else(|| params.get("path"))
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let root = if project_root.is_file() {
            project_root
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        } else {
            project_root
        };
        let abs = root.join(&artifact.source.file);
        match std::fs::read(&abs) {
            Ok(raw) => {
                use sha2::{Digest, Sha256};
                let actual = format!("sha256:{:x}", Sha256::digest(&raw));
                if actual != expected {
                    return Err(json!({
                        "code": "artifact.version_mismatch",
                        "message": format!(
                            "expected source_revision {expected}, actual {actual}; refusing silent downgrade"
                        ),
                        "expected_source_revision": expected,
                        "actual_source_revision": actual,
                        "artifact_id": artifact.artifact_id.0,
                    }));
                }
            }
            Err(e) => {
                return Err(json!({
                    "code": "artifact.version_mismatch",
                    "message": format!(
                        "cannot verify source_revision for {}: {e}; refusing silent downgrade",
                        artifact.source.file
                    ),
                    "expected_source_revision": expected,
                    "artifact_id": artifact.artifact_id.0,
                }));
            }
        }
    }

    let expected_gen = params
        .get("expected_generation")
        .or_else(|| params.get("generation_id"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty());
    if let Some(expected) = expected_gen {
        let actual = semantic_generation_id(&compilation.semantics);
        if actual != expected {
            return Err(json!({
                "code": "artifact.version_mismatch",
                "message": format!(
                    "expected generation {expected}, current projection is {actual}; refusing silent downgrade"
                ),
                "expected_generation": expected,
                "actual_generation": actual,
                "artifact_id": artifact.artifact_id.0,
            }));
        }
    }
    Ok(())
}

/// Semantic generation token matching KnowledgePresentation / DocumentView.
fn semantic_generation_id(project: &crate::sema::SemanticProject) -> String {
    use sha2::{Digest, Sha256};
    let mut gh = Sha256::new();
    gh.update(project.project_id.as_bytes());
    for a in &project.artifacts {
        gh.update(a.hashes.artifact.0.as_bytes());
    }
    format!("{:x}", gh.finalize())[..16].to_string()
}

fn link_resolve(params: &Value) -> Result<Value, Value> {
    let compilation = semantic_project(params)?;
    let reference = params.get("payload").or_else(|| params.get("reference")).and_then(Value::as_str)
        .ok_or_else(|| err("mlsp.bad_request", "link.resolve 需要 payload/reference"))?;
    let candidates = artifact_query(&compilation.semantics, reference);
    let resolution = if candidates.is_empty() { "unresolved" } else if candidates.len() == 1 { "symbol" } else { "ambiguous" };
    Ok(json!({"schema_version":1,"reference":reference,"resolution":resolution,"candidates":candidates.iter().map(|a|json!({"artifact_id":a.artifact_id.0,"symbol_id":a.symbol_id.as_ref().map(|id|id.as_str()),"display_name":a.display_name,"effects":a.effects.values()})).collect::<Vec<_>>() }))
}

fn artifact_verify(params: &Value) -> Result<Value, Value> {
    let compilation = semantic_project(params)?;
    let query = params.get("artifact_id").or_else(|| params.get("symbol_id")).and_then(Value::as_str)
        .ok_or_else(|| err("mlsp.bad_request", "artifact.verify 需要 artifact_id 或 symbol_id"))?;
    let candidates = artifact_query(&compilation.semantics, query);
    if candidates.len() != 1 { return Err(err(if candidates.is_empty() { "artifact.unresolved" } else { "artifact.ambiguous" }, format!("无法唯一验证 `{query}`"))); }
    let artifact = candidates[0];
    let policy = params.get("policy").and_then(Value::as_str).unwrap_or("static");
    Ok(build_verification_report(artifact, policy))
}

/// CLI / product machine adjudication over semantic artifacts (M03).
pub fn adjudicate_project(project: &std::path::Path, artifact: Option<&str>, policy: &str) -> Result<Value, Value> {
    let compilation = semantic_project(&json!({"project": project}))?;
    let targets: Vec<&crate::sema::SemanticArtifact> = if let Some(q) = artifact {
        let c = artifact_query(&compilation.semantics, q);
        if c.is_empty() {
            return Err(err("artifact.unresolved", format!("没有制品 `{q}`")));
        }
        c
    } else {
        compilation
            .semantics
            .artifacts
            .iter()
            .filter(|a| a.kind == "function")
            .collect()
    };
    let reports: Vec<Value> = targets
        .iter()
        .map(|a| build_verification_report(a, policy))
        .collect();
    let any_denied = reports.iter().any(|r| r["may_bind"] == false);
    Ok(json!({
        "schema_version": 1,
        "policy": policy,
        "count": reports.len(),
        "may_bind_all": !any_denied,
        "reports": reports,
    }))
}

/// Known bind/verify policies (M03). Unknown strings are rejected.
fn parse_policy(policy: &str) -> Result<&str, Value> {
    match policy {
        "pure" | "business" | "static" => Ok(policy),
        other => Err(json!({
            "code": "artifact.unknown_policy",
            "message": format!("未知政策 `{other}`；允许 pure / business / static"),
            "policy": other,
        })),
    }
}

fn evidence_allows_business(status: &str) -> bool {
    matches!(status, "sourced" | "verified")
}

fn build_verification_report(artifact: &crate::sema::SemanticArtifact, policy: &str) -> Value {
    let policy = match parse_policy(policy) {
        Ok(p) => p,
        Err(e) => {
            return json!({
                "schema_version": 1,
                "artifact_id": artifact.artifact_id.0,
                "policy": policy,
                "checks": {},
                "blockers": [e],
                "may_bind": false,
                "may_publish": false,
                "verification": crate::sema::artifact_json(artifact)["verification"].clone(),
            });
        }
    };
    let region_status = if artifact.applicability.contains("region=") {
        "declared"
    } else {
        "unknown"
    };
    let checks = json!({
        "parse": {"status": artifact.verification.parse, "detail": "static_project_compile"},
        "resolve": {"status": artifact.verification.resolve, "detail": "symbol_bound"},
        "contract": {"status": artifact.verification.contract, "detail": "not_executed_by_verify"},
        "execution": {"status": artifact.verification.execution, "detail": "not_run"},
        "tests": {"status": artifact.verification.tests, "detail": "no_test_runner_invoked"},
        "evidence": {"status": artifact.verification.evidence, "detail": "compile_time_provenance"},
        "domain": {"status": artifact.verification.domain, "detail": "unreviewed"},
        "region": {"status": region_status, "detail": "input_domain_not_checked"},
        "input_domain": {"status": "unknown", "detail": "not_checked_by_static_verify"},
    });
    let mut blockers = Vec::new();
    // Never upgrade tests/execution to passed without a runner (M03).
    if matches!(
        artifact.verification.tests.as_str(),
        "passed" | "verified"
    ) && artifact
        .verification
        .methods
        .iter()
        .all(|m| m == "static_project_compile")
    {
        blockers.push(json!({
            "code": "artifact.tests_unproven",
            "check": "tests",
            "status": artifact.verification.tests,
            "detail": "static compile must not claim tests passed",
        }));
    }
    if policy == "business" && !evidence_allows_business(&artifact.verification.evidence) {
        blockers.push(json!({
            "code": "artifact.evidence_denied",
            "check": "evidence",
            "status": artifact.verification.evidence,
        }));
    }
    if policy == "pure"
        && artifact
            .effects
            .values()
            .iter()
            .any(|e| e != "pure")
    {
        blockers.push(json!({
            "code": "artifact.effect_denied",
            "check": "effects",
            "effects": artifact.effects.values(),
        }));
    }
    let may_bind = blockers.is_empty()
        && match policy {
            "business" => evidence_allows_business(&artifact.verification.evidence),
            "pure" => artifact.effects.values().iter().all(|e| e == "pure"),
            _ => true,
        };
    // Publish requires stronger evidence than bind; static verify never claims tests passed.
    let may_publish = may_bind
        && artifact.verification.tests != "unknown"
        && artifact.verification.tests != "not_run"
        && artifact.verification.execution != "not_run"
        && !matches!(
            artifact.verification.evidence.as_str(),
            "unverified" | "missing_source" | "anchor_missing" | "expired" | "invalid_date"
        );
    json!({
        "schema_version": 1,
        "artifact_id": artifact.artifact_id.0,
        "symbol_id": artifact.symbol_id.as_ref().map(|id| id.as_str()),
        "digest": artifact.hashes.artifact.0,
        "evidence_digest": artifact.hashes.evidence.0,
        "closure_digest": artifact.hashes.closure.0,
        "policy": policy,
        "checks": checks,
        "blockers": blockers,
        "may_bind": may_bind,
        "may_publish": may_publish,
        "verification": {
            "parse": artifact.verification.parse,
            "resolve": artifact.verification.resolve,
            "contract": artifact.verification.contract,
            "execution": artifact.verification.execution,
            "tests": artifact.verification.tests,
            "evidence": artifact.verification.evidence,
            "domain": artifact.verification.domain,
            "methods": artifact.verification.methods,
        },
    })
}

/// M04: bind then execute in one call (actual reuse, not discovery scores).
fn artifact_reuse_execute(params: &Value) -> Result<Value, Value> {
    let project = params
        .get("project")
        .or_else(|| params.get("path"))
        .cloned()
        .ok_or_else(|| err("mlsp.bad_request", "artifact.reuse_execute 需要 project"))?;
    let query = params
        .get("artifact_id")
        .or_else(|| params.get("symbol_id"))
        .or_else(|| params.get("query"))
        .and_then(Value::as_str)
        .ok_or_else(|| err("mlsp.bad_request", "artifact.reuse_execute 需要 artifact_id"))?;
    let policy = params.get("policy").and_then(Value::as_str).unwrap_or("pure");
    let inputs = params.get("inputs").cloned().unwrap_or(json!({}));
    let bound = artifact_bind(&json!({
        "project": project,
        "artifact_id": query,
        "policy": policy,
        "expected_digest": params.get("expected_digest"),
    }))?;
    let handle = bound
        .get("handle")
        .cloned()
        .ok_or_else(|| err("artifact.bind_failed", "bind 未返回 handle"))?;
    let executed = artifact_execute(&json!({
        "handle": handle,
        "inputs": inputs,
        "allow_effects": params.get("allow_effects"),
    }))?;
    Ok(json!({
        "schema_version": 1,
        "actual_reuse": true,
        "discovery_only": false,
        "artifact_id": executed.get("artifact_id"),
        "symbol_id": executed.get("symbol_id"),
        "digest": executed.get("digest"),
        "result": executed.get("result"),
        "stdout": executed.get("stdout"),
        "effects": executed.get("effects"),
        "trace": {
            "artifact_id": executed.get("artifact_id"),
            "symbol_id": executed.get("symbol_id"),
            "digest": executed.get("digest"),
            "inputs": inputs,
            "value": executed.get("result"),
            "stdout": executed.get("stdout"),
        },
    }))
}

fn artifact_impact(params: &Value) -> Result<Value, Value> {
    let compilation = semantic_project(params)?;
    let query = params.get("artifact_id").or_else(|| params.get("symbol_id")).or_else(|| params.get("path")).and_then(Value::as_str)
        .ok_or_else(|| err("mlsp.bad_request", "artifact.impact 需要 artifact_id、symbol_id 或 path"))?;
    Ok(crate::sema::semantic_impact(&compilation.semantics, query))
}

fn artifact_bind(params: &Value) -> Result<Value, Value> {
    let compilation = semantic_project(params)?;
    let query = params.get("artifact_id").or_else(|| params.get("symbol_id")).and_then(Value::as_str)
        .ok_or_else(|| err("mlsp.bad_request", "artifact.bind 需要 artifact_id 或 symbol_id"))?;
    let candidates = artifact_query(&compilation.semantics, query);
    if candidates.len() != 1 { return Err(err(if candidates.is_empty() { "artifact.unresolved" } else { "artifact.ambiguous" }, format!("无法唯一绑定 `{query}`"))); }
    let artifact = candidates[0];
    let expected = params.get("expected_digest").and_then(Value::as_str);
    if expected.is_some() && expected != Some(artifact.hashes.artifact.0.as_str()) {
        return Err(json!({"code":"artifact.digest_mismatch","message":"目标摘要已变化，拒绝绑定","expected":expected,"actual":artifact.hashes.artifact.0}));
    }
    let policy = parse_policy(
        params
            .get("policy")
            .and_then(Value::as_str)
            .unwrap_or("pure"),
    )?;
    let effects = artifact.effects.values();
    if policy == "pure"
        && effects.iter().any(|effect| effect != "pure")
    {
        return Err(json!({
            "code": "artifact.effect_denied",
            "message": "pure 策略下不能绑定含非纯/未知效应的制品",
            "policy": policy,
            "effects": effects,
        }));
    }
    if policy == "business" && !evidence_allows_business(&artifact.verification.evidence) {
        return Err(json!({
            "code": "artifact.evidence_denied",
            "message": "business 策略要求来源已关联且锚点有效、未过期；当前证据状态不足",
            "policy": policy,
            "evidence": artifact.verification.evidence,
        }));
    }
    let symbol_id = artifact
        .symbol_id
        .as_ref()
        .map(|id| id.as_str().to_string())
        .unwrap_or_else(|| artifact.artifact_id.0.clone());
    let valid_until = artifact
        .applicability
        .split(';')
        .find_map(|part| {
            part.trim()
                .strip_prefix("valid_until=")
                .map(|s| s.trim().to_string())
        });
    let provenance_digests: Vec<Value> = artifact
        .provenance
        .iter()
        .filter(|p| p.relation == "supports")
        .map(|p| {
            json!({
                "uri": p.uri,
                "locator": p.locator,
                "content_digest": p.content_digest.as_ref().map(|d| &d.0),
            })
        })
        .collect();
    Ok(json!({"schema_version":1,"handle":{
        "artifact_id": artifact.artifact_id.0,
        "symbol_id": symbol_id,
        "digest": artifact.hashes.artifact.0,
        "closure_digest": artifact.hashes.closure.0,
        "evidence_digest": artifact.hashes.evidence.0,
        "project": compilation.semantics.root,
        "policy": policy,
        "effects": effects,
        "evidence": artifact.verification.evidence,
        "valid_until": valid_until,
        "provenance": provenance_digests,
        "source": {"file": artifact.source.file, "line": artifact.source.line},
    }}))
}

/// Digest-guarded local patch (S3/G09). `intent: wording` refuses signature/effect drift.
fn artifact_patch(params: &Value) -> Result<Value, Value> {
    let compilation = semantic_project(params)?;
    let query = params
        .get("artifact_id")
        .or_else(|| params.get("symbol_id"))
        .and_then(Value::as_str)
        .ok_or_else(|| err("mlsp.bad_request", "artifact.patch 需要 artifact_id 或 symbol_id"))?;
    let candidates = artifact_query(&compilation.semantics, query);
    if candidates.len() != 1 {
        return Err(err(
            if candidates.is_empty() {
                "artifact.unresolved"
            } else {
                "artifact.ambiguous"
            },
            format!("无法唯一修补 `{query}`"),
        ));
    }
    let before = candidates[0];
    let expected = params
        .get("expected_digest")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            err(
                "mlsp.bad_request",
                "artifact.patch 需要 expected_digest（hashes.implementation）",
            )
        })?;
    if expected != before.hashes.implementation.0 {
        return Err(json!({
            "code": "artifact.digest_mismatch",
            "message": "源码实现摘要已变化，拒绝覆盖",
            "expected": expected,
            "actual": before.hashes.implementation.0,
        }));
    }
    let path = compilation.semantics.root.join(&before.source.file);
    let original = std::fs::read_to_string(&path)
        .map_err(|e| err("artifact.load", format!("读源失败: {e}")))?;
    let edits_v = params
        .get("edits")
        .and_then(Value::as_array)
        .ok_or_else(|| err("mlsp.bad_request", "artifact.patch 需要 edits"))?;
    let mut edits = Vec::new();
    for e in edits_v {
        let line = e
            .get("line")
            .and_then(Value::as_u64)
            .ok_or_else(|| err("mlsp.bad_request", "edit.line 必填"))? as u32;
        let kind = e.get("op").and_then(Value::as_str).unwrap_or("replace");
        match kind {
            "replace" => {
                let text = e
                    .get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| err("mlsp.bad_request", "replace 需要 text"))?
                    .to_string();
                edits.push(crate::repair::Edit::Replace { line, text });
            }
            "delete" => edits.push(crate::repair::Edit::Delete { line }),
            "insert_before" => {
                let text = e
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                edits.push(crate::repair::Edit::InsertBefore { line, text });
            }
            other => {
                return Err(err(
                    "mlsp.bad_request",
                    format!("未知 edit.op `{other}`"),
                ))
            }
        }
    }
    let start = before.source.line;
    let end = before.source.end_line.max(start);
    let ranges = [(start, end.max(start + 40))];
    let patched = crate::repair::apply_edits(&original, &ranges, &edits).map_err(|o| {
        json!({
            "code": "mlsp.repair_out_of_scope",
            "message": "补丁超出制品源码范围",
            "lines": o.lines,
            "ranges": o.ranges,
        })
    })?;
    std::fs::write(&path, &patched)
        .map_err(|e| err("artifact.write", format!("写源失败: {e}")))?;
    let after_comp = semantic_project(params)?;
    let after_cands = artifact_query(&after_comp.semantics, query);
    if after_cands.len() != 1 {
        // roll back
        let _ = std::fs::write(&path, &original);
        return Err(err(
            "artifact.patch_ambiguous",
            "补丁后制品无法唯一解析；已回滚",
        ));
    }
    let after = after_cands[0];
    let signature_changed = before.hashes.interface.0 != after.hashes.interface.0;
    let effects_changed = before.effects.values() != after.effects.values();
    let implementation_changed =
        before.hashes.implementation.0 != after.hashes.implementation.0;
    let intent = params.get("intent").and_then(Value::as_str).unwrap_or("edit");
    if intent == "wording" && (signature_changed || effects_changed) {
        let _ = std::fs::write(&path, &original);
        return Err(json!({
            "code": "artifact.wording_broke_contract",
            "message": "intent=wording 但签名或效应已变；已回滚",
            "signature_changed": signature_changed,
            "effects_changed": effects_changed,
        }));
    }
    Ok(json!({
        "schema_version": 1,
        "artifact_id": after.artifact_id.0,
        "symbol_id": after.symbol_id.as_ref().map(|id| id.as_str()),
        "applied": true,
        "diff": {
            "signature_changed": signature_changed,
            "effects_changed": effects_changed,
            "implementation_changed": implementation_changed,
            "before": {
                "interface": before.hashes.interface.0,
                "implementation": before.hashes.implementation.0,
                "effects": before.effects.values(),
            },
            "after": {
                "interface": after.hashes.interface.0,
                "implementation": after.hashes.implementation.0,
                "effects": after.effects.values(),
            },
        },
    }))
}

fn artifact_execute(params: &Value) -> Result<Value, Value> {
    let handle = params.get("handle").ok_or_else(|| err("mlsp.bad_request", "artifact.execute 需要 artifact.bind 返回的 handle"))?;
    let artifact_id = handle.get("artifact_id").and_then(Value::as_str).ok_or_else(|| err("mlsp.bad_handle", "handle 缺少 artifact_id"))?;
    let digest = handle.get("digest").and_then(Value::as_str).ok_or_else(|| err("mlsp.bad_handle", "handle 缺少 digest"))?;
    let project_path = handle.get("project").and_then(Value::as_str).or_else(|| params.get("project").and_then(Value::as_str)).ok_or_else(|| err("mlsp.bad_handle", "handle 缺少 project"))?;
    let compilation = semantic_project(&json!({"project":project_path}))?;
    let candidates = artifact_query(&compilation.semantics, artifact_id);
    if candidates.len() != 1 { return Err(err("artifact.stale_handle", "制品不再可唯一解析；请重新 bind")); }
    let artifact = candidates[0];
    if artifact.hashes.artifact.0 != digest { return Err(json!({"code":"artifact.stale_handle","message":"源码、契约或来源证据已改变；拒绝执行过期 handle","expected":digest,"actual":artifact.hashes.artifact.0})); }
    if let Some(bound_closure) = handle.get("closure_digest").and_then(Value::as_str) {
        if bound_closure != artifact.hashes.closure.0 {
            return Err(json!({
                "code": "artifact.stale_handle",
                "message": "依赖闭包已改变；拒绝执行过期 handle",
                "expected": bound_closure,
                "actual": artifact.hashes.closure.0,
            }));
        }
    }
    if let Some(bound_evidence) = handle.get("evidence_digest").and_then(Value::as_str) {
        if bound_evidence != artifact.hashes.evidence.0 {
            return Err(json!({
                "code": "artifact.stale_handle",
                "message": "来源证据摘要已改变；拒绝执行过期 handle",
                "expected": bound_evidence,
                "actual": artifact.hashes.evidence.0,
            }));
        }
    }
    if let Some(bound_symbol) = handle.get("symbol_id").and_then(Value::as_str) {
        let current = artifact
            .symbol_id
            .as_ref()
            .map(|id| id.as_str())
            .unwrap_or(artifact.artifact_id.0.as_str());
        if bound_symbol != current {
            return Err(json!({
                "code": "artifact.stale_handle",
                "message": "handle 绑定的 SymbolId 与当前制品不一致",
                "expected": bound_symbol,
                "actual": current,
            }));
        }
    }
    // Re-check policy at execute time (M02) — do not trust bind-time evidence string alone.
    let policy = handle.get("policy").and_then(Value::as_str).unwrap_or("pure");
    let _ = parse_policy(policy)?;
    if policy == "business" && !evidence_allows_business(&artifact.verification.evidence) {
        return Err(json!({
            "code": "artifact.evidence_denied",
            "message": "business 句柄在执行时证据状态不足（来源缺失、锚点失效或已过期）",
            "policy": policy,
            "evidence": artifact.verification.evidence,
        }));
    }
    if policy == "pure"
        && artifact
            .effects
            .values()
            .iter()
            .any(|effect| effect != "pure")
    {
        return Err(json!({
            "code": "artifact.effect_denied",
            "message": "pure 句柄在执行时出现非纯效应",
            "policy": policy,
            "effects": artifact.effects.values(),
        }));
    }
    if let Some(until) = handle.get("valid_until").and_then(Value::as_str) {
        if crate::sema::provenance::valid_until_expired(until) {
            return Err(json!({
                "code": "artifact.expired",
                "message": "句柄适用期已过",
                "valid_until": until,
            }));
        }
    }
    if artifact.kind != "function" || artifact.callable.as_deref() == Some("instance_method") { return Err(err("artifact.not_executable", "只有非实例函数制品可由 artifact.execute 直接执行")); }
    let allowed: Vec<&str> = params.get("allow_effects").and_then(Value::as_array).map(|items| items.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
    let denied: Vec<String> = artifact.effects.values().into_iter().filter(|effect| effect != "pure" && !allowed.contains(&effect.as_str())).collect();
    if !denied.is_empty() { return Err(json!({"code":"artifact.effect_denied","message":"效果策略拒绝执行","denied":denied,"effects":artifact.effects.values()})); }
    let path = compilation.semantics.root.join(&artifact.source.file);
    let module = load::load_module(&path).map_err(|e| diagnostic_err("artifact.load", &e))?;
    let input = params.get("inputs").cloned().unwrap_or_else(|| json!({}));
    let object = input.as_object().ok_or_else(|| err("mlsp.bad_request", "inputs 必须是 JSON 对象"))?;
    let mut args = Vec::new();
    for (name, value) in object { args.push((name.clone(), crate::host::json::json_to_value(value).map_err(|e| err("mlsp.bad_input", e))?)); }
    let caps = crate::host::HostCaps { fs_write: allowed.contains(&"fs.write"), exec: allowed.iter().any(|e| matches!(*e, "process.exec" | "plugin.call" | "llm.call")), net: allowed.iter().any(|e| matches!(*e, "network.read" | "network.write" | "llm.call")), plugin: allowed.contains(&"plugin.call") };
    let host = crate::host::HostContext::for_capture(Some(&path), caps);
    let mut interpreter = crate::interp::Interpreter::with_capture(Some(&path), false).with_host(host);
    // Bind by source line / declaration identity — never by bare display_name suffix (G07).
    let fun = find_function_at_line(&module.functions, artifact.source.line).ok_or_else(|| {
        err(
            "artifact.unresolved",
            format!(
                "无法在 {}:{} 定位绑定函数 {}",
                artifact.source.file, artifact.source.line, artifact.display_name
            ),
        )
    })?;
    let value = interpreter
        .invoke_bound_function(&module, fun, &args)
        .map_err(|e| diagnostic_err("artifact.execution_failed", &e))?;
    let stdout = std::mem::take(&mut interpreter.captured_stdout);
    Ok(json!({
        "schema_version": 1,
        "artifact_id": artifact.artifact_id.0,
        "symbol_id": artifact.symbol_id.as_ref().map(|id| id.as_str()),
        "digest": digest,
        "result": crate::value_as_json(&value).map_err(|e| err("artifact.serialization", e))?,
        "stdout": stdout,
        "effects": artifact.effects.values(),
    }))
}

fn find_function_at_line<'a>(
    functions: &'a [crate::ast::Function],
    line: u32,
) -> Option<&'a crate::ast::Function> {
    for fun in functions {
        if fun.span.line == line {
            return Some(fun);
        }
        if let Some(found) = find_function_at_line(&fun.children, line) {
            return Some(found);
        }
    }
    None
}

fn err(code: &str, message: impl Into<String>) -> Value {
    json!({"code": code, "message": message.into()})
}

// ---------- 构造卡片（语法唯一事实来源 = CORE_CONSTRUCTS） ----------

/// 检索别名（中/英关键字）——只做匹配，不定义语法。
const ALIASES: &[(&str, &[&str])] = &[
    ("narrative", &["叙述", "文字", "说明", "注释", "narrative", "prose"]),
    ("blank", &["空行", "blank"]),
    ("heading-unit", &["标题", "单元", "函数", "对象", "heading", "定义"]),
    ("call-line", &["调用", "call"]),
    ("branch-list", &["分支", "条件", "判断", "branch", "if"]),
    ("loop-list", &["循环", "遍历", "loop", "for", "while"]),
    ("param", &["参数", "形参", "param"]),
    ("table", &["表", "表格", "table", "集合", "数据"]),
    ("fence", &["代码块", "围栏", "fence", "公式", "code"]),
    ("hr", &["分隔线", "水平线", "hr"]),
    ("writeback", &["回写", "输出块", "writeback"]),
    ("ident", &["变量", "标识符", "ident", "名字"]),
    ("bold-code", &["粗体", "赋值", "语句", "bold", "执行"]),
    ("italic-return", &["返回", "return", "斜体"]),
    ("empty-return", &["空返回", "none", "结束", "副作用"]),
    ("link-index", &["取元", "索引", "index", "键", "行"]),
    ("bracket-call", &["修饰", "括号", "bracket", "开关参数"]),
    ("footnote-index", &["脚注", "footnote", "取列", "列"]),
    ("soft-emphasis", &["强调", "emphasis", "软强调"]),
];

/// core-surface.md 全文（与 `tests/core_surface.rs` 守卫同源）——卡片机械携带
/// 语义/戒律/样例**原文**，AI 一次查询即可自持，不必回读文档（three-problems.md §3.2「单构造戒律」）。
const SURFACE_MD: &str = include_str!("../doc/design/core-surface.md");

/// 机械提取 core-surface.md 清单行：`| \`id\` | 外形 | 语义 | 戒律 | 样例 |` →
///（语义, 戒律, 样例, 行原文）；只读不改，卡片内容与文档永不漂移。
fn surface_row(id: &str) -> Option<(String, String, String, String)> {
    for line in SURFACE_MD.lines() {
        let t = line.trim();
        if !t.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = t.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() != 5 {
            continue;
        }
        let rid = cells[0].trim_matches('`');
        if rid == id && !rid.is_empty() && !rid.contains(' ') && !rid.starts_with('-') {
            return Some((
                cells[2].to_string(),
                cells[3].to_string(),
                cells[4].to_string(),
                t.to_string(),
            ));
        }
    }
    None
}

fn card(id: &str) -> Value {
    let form = CORE_CONSTRUCTS
        .iter()
        .find(|(cid, _)| *cid == id)
        .map(|(_, f)| *f)
        .unwrap_or("?");
    let mut v = json!({
        "id": id,
        "form": form,
        "manual": "doc/design/core-surface.md",
    });
    if let Some((semantics, rule, sample, quote)) = surface_row(id) {
        v["semantics"] = semantics.into();
        v["rule"] = rule.into();
        v["sample"] = sample.into();
        v["doc_quote"] = quote.into();
    }
    v
}

fn match_constructs(query: &str) -> Vec<Value> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return ALIASES.iter().map(|(id, _)| card(id)).collect();
    }
    ALIASES
        .iter()
        .filter(|(_, keys)| {
            keys.iter()
                .any(|k| k.to_lowercase().contains(&q) || q.contains(&k.to_lowercase()))
        })
        .map(|(id, _)| card(id))
        .collect()
}

fn syntax(params: &Value) -> Result<Value, Value> {
    let query = params.get("query").and_then(|q| q.as_str()).unwrap_or("");
    let cards = match_constructs(query);
    if cards.is_empty() {
        return Err(err(
            "mlsp.no_match",
            format!("没有匹配「{query}」的语法构造；换关键字（如 返回/表/调用）或省略 query 列出全部"),
        ));
    }
    Ok(json!({"cards": cards}))
}

fn locate(params: &Value) -> Result<Value, Value> {
    if params.get("path").is_some() || params.get("source").is_some() {
        let (module, path) = module_from_params(params)?;
        let symbol = params
            .get("symbol")
            .and_then(|s| s.as_str())
            .ok_or_else(|| err("mlsp.bad_request", "locate 定位符号需要 params.symbol"))?;
        let fun = find_fn(&module, symbol)
            .ok_or_else(|| err("mlsp.unknown_unit", format!("没有名为 `{symbol}` 的单元")))?;
        return Ok(json!({
            "unit": fun.name,
            "level": fun.level,
            "span": span_json(path.as_deref(), fun.span),
            "params": fun.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>(),
            "contract": fun.contract.as_ref().map(|c| contract_json(c, path.as_deref())),
            "manual": "doc/design/core-surface.md",
        }));
    }
    Ok(json!({"cards": match_constructs(
        params.get("query").and_then(|q| q.as_str()).unwrap_or("")
    )}))
}

fn validate(params: &Value) -> Result<Value, Value> {
    let (module, path) = module_from_params(params)?;
    let mut diags = check_module(&module, path.as_deref());
    diags.sort_by_key(|d| (d.span.line, d.span.col));
    let jsons: Vec<Value> = diags.iter().map(|d| d.to_json()).collect();
    let ok = !diags
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostics::Severity::Error));
    Ok(json!({"ok": ok, "diagnostics": jsons}))
}

/// 修复靶点（T3.3 护栏）：锚点局部修复，**越界必拒**（禁止整文件重写/无关重构）。
fn repair_targets(params: &Value) -> Result<Value, Value> {
    let code = params.get("code").and_then(|c| c.as_str()).unwrap_or("");
    let anchor = params.get("doc_anchor").and_then(|a| a.as_str());
    let suggestion = params.get("suggestion").and_then(|s| s.as_str());
    if code.is_empty() {
        return Err(err("mlsp.bad_request", "repair_targets 需要 params.code"));
    }
    Ok(json!({
        "targets": [{
            "code": code,
            "doc_anchor": anchor,
            "span": params.get("span").cloned().unwrap_or(Value::Null),
        }],
        "repair": {
            "strategy": "anchored-local",
            "bounded": true,
            "max_attempts": 2,
            "scope": "doc_anchor 所锚定的契约表/语句行",
            "suggestion": suggestion,
            "on_violation": "abstain",
            "note": "只许改动锚定范围内的行；越界改动必须拒绝并报 abstain"
        }
    }))
}

/// 应用有界修复（T3.3）：行编辑全有或全无；越界 ⇒ `mlsp.repair_out_of_scope`（abstain），源零字节不改。
fn repair_apply(params: &Value) -> Result<Value, Value> {
    let source = source_text(params)?;
    let ranges = params
        .get("ranges")
        .and_then(|r| r.as_array())
        .ok_or_else(|| err("mlsp.bad_request", "repair_apply 需要 params.ranges（[[起,止],…]）"))?
        .iter()
        .map(|pair| {
            let a = pair.as_array()?;
            Some((a.first()?.as_u64()? as u32, a.get(1)?.as_u64()? as u32))
        })
        .collect::<Option<Vec<(u32, u32)>>>()
        .ok_or_else(|| err("mlsp.bad_request", "ranges 必须是 [[起,止],…]"))?;

    let mut edits = Vec::new();
    let edit_list = params
        .get("edits")
        .and_then(|e| e.as_array())
        .ok_or_else(|| err("mlsp.bad_request", "repair_apply 需要 params.edits"))?;
    for e in edit_list {
        let line = e
            .get("line")
            .and_then(|l| l.as_u64())
            .ok_or_else(|| err("mlsp.bad_request", "edit 缺 line"))? as u32;
        let text = e.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
        edits.push(match e.get("op").and_then(|o| o.as_str()) {
            Some("replace") => crate::repair::Edit::Replace { line, text },
            Some("delete") => crate::repair::Edit::Delete { line },
            Some("insert") => crate::repair::Edit::InsertBefore { line, text },
            other => {
                return Err(err(
                    "mlsp.bad_request",
                    format!("未知编辑 op {other:?}（replace / delete / insert）"),
                ))
            }
        });
    }

    let check_labels = match params.get("check_labels") {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => s != "off" && s != "false",
        _ => false,
    };

    match crate::repair::apply_edits(&source, &ranges, &edits) {
        Ok(new_source) => {
            if params.get("write").and_then(|w| w.as_bool()).unwrap_or(false) {
                let path = params
                    .get("path")
                    .and_then(|p| p.as_str())
                    .ok_or_else(|| err("mlsp.bad_request", "write=true 需要 params.path"))?;
                std::fs::write(path, &new_source)
                    .map_err(|e| err("mlsp.io_error", format!("写回 {path} 失败: {e}")))?;
            }
            // `<TAB>` M6 (T1): the edit is the perfect moment to ask "did the
            // description survive?" — before/after are both in hand, no VCS,
            // no ledger, and the answer is scoped to this very edit.
            let mut labels = Value::Null;
            if check_labels {
                if let (Ok(before), Ok(after)) = (
                    crate::load::load_module_from_source(&source),
                    crate::load::load_module_from_source(&new_source),
                ) {
                    let stale = crate::label::compare(&before, &after);
                    let newly = with_session(|s| s.record(&stale));
                    labels = json!({
                        "stale": stale
                            .iter()
                            .map(|s| json!({
                                "label": s.label,
                                "line": s.line,
                                "code_past": s.code_past,
                                "code_now": s.code_now,
                            }))
                            .collect::<Vec<_>>(),
                        "newly_seen": newly,
                    });
                }
            }
            Ok(json!({
                "source": new_source,
                "edits_applied": edits.len(),
                "labels": labels,
            }))
        }
        Err(oos) => Err(json!({
            "code": "mlsp.repair_out_of_scope",
            "message": format!(
                "修复越界（abstain）：触及行 {:?} 不在靶点范围 {:?} 内；已拒绝全部编辑，源未改动",
                oos.lines, oos.ranges
            ),
            "lines": oos.lines,
            "ranges": oos.ranges,
            "on_violation": "abstain",
        })),
    }
}

/// 取待修源码（source 直给 / path 读取）。
fn source_text(params: &Value) -> Result<String, Value> {
    if let Some(s) = params.get("source").and_then(|s| s.as_str()) {
        return Ok(s.to_string());
    }
    let path = params
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| err("mlsp.bad_request", "需要 params.source 或 params.path"))?;
    std::fs::read_to_string(path)
        .map_err(|e| err("mlsp.io_error", format!("读取 {path} 失败: {e}")))
}

/// 契约查询（T2.6）：单元名 → 形参/返回/字段契约。
fn schema(params: &Value) -> Result<Value, Value> {
    let (module, path) = module_from_params(params)?;
    let unit = params
        .get("unit")
        .and_then(|u| u.as_str())
        .ok_or_else(|| err("mlsp.bad_request", "schema 需要 params.unit"))?;
    let fun = find_fn(&module, unit)
        .ok_or_else(|| err("mlsp.unknown_unit", format!("没有名为 `{unit}` 的单元")))?;
    let mut vars = serde_json::Map::new();
    for (name, c) in &fun.var_contracts {
        vars.insert(name.clone(), contract_json(c, path.as_deref()));
    }
    Ok(json!({
        "unit": fun.name,
        "contract": fun.contract.as_ref().map(|c| contract_json(c, path.as_deref())),
        "vars": Value::Object(vars),
        "params_inferred": fun.params.iter().map(|p| json!({
            "name": p.name, "inferred": p.inferred
        })).collect::<Vec<_>>(),
    }))
}

// ---------- 公共小工具 ----------

/// Per-process T1 session state (`repair_apply` rounds accumulate here).
static SESSION: std::sync::Mutex<Option<crate::label::SessionSummary>> =
    std::sync::Mutex::new(None);

fn with_session<T>(f: impl FnOnce(&mut crate::label::SessionSummary) -> T) -> T {
    let mut guard = SESSION.lock().unwrap_or_else(|e| e.into_inner());
    let s = guard.get_or_insert_with(Default::default);
    f(s)
}

/// `session.summary` — report (and reset) the T1 counters. `<TAB>` M6
fn session_summary() -> Result<Value, Value> {
    let s = with_session(|s| {
        let snap = s.clone();
        *s = Default::default();
        snap
    });
    Ok(json!({
        "ok": true,
        "labels_stale_seen": s.stale_seen,
        "labels_fixed": s.fixed,
        "labels_outstanding": s.items.len(),
        "items": s.items,
    }))
}

/// `labels.search` — occlusion label query. `<TAB>` M9
///
/// params: `{ source | path, grep?: [..] | "a,b", also_builtin?: bool,
///           group?: "未完成,假数据" }`
fn labels_search(params: &Value) -> Result<Value, Value> {
    let (module, _path) = module_from_params(params)?;
    let mut words: Vec<String> = match params.get("grep") {
        Some(Value::String(s)) => crate::label::parse_words(s),
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect(),
        _ => Vec::new(),
    };
    let groups: Vec<String> = match params.get("group").and_then(|g| g.as_str()) {
        Some(spec) => crate::label::parse_words(spec),
        None => crate::label::PENDING_GROUPS
            .iter()
            .map(|s| s.to_string())
            .collect(),
    };
    let group_refs: Vec<&str> = groups.iter().map(|s| s.as_str()).collect();
    if words.is_empty() || params.get("also_builtin").and_then(|b| b.as_bool()) == Some(true) {
        for w in crate::label::group_words(&group_refs) {
            if !words.contains(&w) {
                words.push(w);
            }
        }
    }
    let hits: Vec<Value> = crate::label::search(&module, &words)
        .into_iter()
        .map(|h| {
            json!({
                "line": h.site.line,
                "unit": h.site.unit,
                "label": h.site.label,
                "words": h.words,
                "has_payload": h.site.has_payload,
            })
        })
        .collect();
    Ok(json!({"ok": true, "words": words, "count": hits.len(), "hits": hits}))
}

fn module_from_params(params: &Value) -> Result<(Module, Option<PathBuf>), Value> {
    if let Some(src) = params.get("source").and_then(|s| s.as_str()) {
        let module = load::load_module_from_source(src)
            .map_err(|e| diagnostic_err("mlsp.parse_error", &e))?;
        return Ok((module, None));
    }
    let path = params
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or_else(|| err("mlsp.bad_request", "需要 params.path 或 params.source"))?;
    let path = PathBuf::from(path);
    let module = load::load_module(&path).map_err(|e| diagnostic_err("mlsp.parse_error", &e))?;
    Ok((module, Some(path)))
}

/// 错误链里有结构化诊断就原样透出（AI/MLSP 面不许拍平）。
fn diagnostic_err(code: &str, e: &anyhow::Error) -> Value {
    match Diagnostic::find(e) {
        Some(d) => d.to_json(),
        None => err(code, format!("{e:#}")),
    }
}

fn find_fn<'a>(module: &'a Module, name: &str) -> Option<&'a Function> {
    fn walk<'a>(funs: &'a [Function], name: &str) -> Option<&'a Function> {
        funs.iter().find(|f| f.name == name).or_else(|| {
            funs.iter()
                .find_map(|f| walk(&f.children, name))
        })
    }
    walk(&module.functions, name)
}

fn span_json(path: Option<&Path>, span: crate::diagnostics::Span) -> Value {
    json!({
        "file": path.map(|p| p.display().to_string()),
        "line": span.line,
        "col": span.col,
    })
}

fn ty_json(ty: &Ty) -> Value {
    json!(ty.name())
}

fn contract_json(c: &Contract, path: Option<&Path>) -> Value {
    json!({
        "params": c.params.iter().map(|p| json!({
            "name": p.name, "type": ty_json(&p.ty), "desc": p.desc
        })).collect::<Vec<_>>(),
        "returns": c.returns.as_ref().map(|r| json!({
            "type": ty_json(&r.ty), "desc": r.desc
        })),
        "fields": c.fields.iter().map(|f| json!({
            "name": f.name, "type": ty_json(&f.ty), "nullable": f.nullable, "desc": f.desc
        })).collect::<Vec<_>>(),
        "doc_anchor": c.anchor(path),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn fixture() -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("marqdo-mlsp-artifact-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.mq.md"), "# main\n\n## add\n\n*`x` + 1*").unwrap();
        root
    }

    #[test]
    fn artifact_protocol_binds_then_executes_a_digest_checked_function() {
        let root = fixture();
        let project = root.display().to_string();
        let found = dispatch(&json!({"method":"artifact.find","params":{"project":project,"query":"add"}})).unwrap();
        let candidate = found["candidates"].as_array().unwrap().iter().find(|v| v["kind"] == "function").unwrap();
        let id = candidate["artifact_id"].as_str().unwrap();
        let bound = dispatch(&json!({"method":"artifact.bind","params":{"project":root,"artifact_id":id}})).unwrap();
        let handle = &bound["handle"];
        assert!(handle.get("symbol_id").and_then(|v| v.as_str()).is_some());
        assert!(handle.get("source").is_some());
        let result = dispatch(&json!({"method":"artifact.execute","params":{"handle":handle,"inputs":{"x":4}}})).unwrap();
        assert_eq!(result["result"], 5);
        assert_eq!(result["symbol_id"], handle["symbol_id"]);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn artifact_bind_rejects_non_pure_under_pure_policy() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("marqdo-mlsp-effect-{stamp}"));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("main.mq.md"),
            "# main\n\n## save\n\n> fs.write_text path=t.txt text=x\n\n*1*\n",
        )
        .unwrap();
        let found = dispatch(&json!({"method":"artifact.find","params":{"project":root,"query":"save"}})).unwrap();
        let id = found["candidates"].as_array().unwrap().iter().find(|v| v["kind"] == "function").unwrap()["artifact_id"]
            .as_str()
            .unwrap();
        let err = dispatch(&json!({"method":"artifact.bind","params":{"project":root,"artifact_id":id,"policy":"pure"}}))
            .unwrap_err();
        assert_eq!(err["code"], "artifact.effect_denied");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn link_resolution_never_silently_selects_a_nonexistent_target() {
        let root = fixture();
        let result = dispatch(&json!({"method":"link.resolve","params":{"project":root,"payload":"does_not_exist"}})).unwrap();
        assert_eq!(result["resolution"], "unresolved");
        fs::remove_dir_all(root).unwrap();
    }
}
