//! `marqdo prove`：SKILL 控制平面 → Lean 4（`doc/design/prove-cli.md`）。
//!
//! 三条契约（设计 §1.3）：
//! 1. **诚实**——只报实际做过的判定；`checked` 永不冒充 Lean 结论；
//! 2. **单调**——离线判定严格弱于 Lean（P2 落地护栏）；
//! 3. **不执行**——本模块只读源码，从不运行目标程序。
//!
//! P0 范围（roadmap/prove.md §2）：装载 → 降级 → 定型 → 确定性生成 + audit。
//! 判定（离线 ⑤ / Lean ⑥）在 P1/P2 接入；状态只到 `unverified` / `rejected` /
//! `error`。

pub mod cache;
pub mod driver;
pub mod emit;
pub mod ir;
pub mod lower;
pub mod proof_check;
pub mod types;

use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result};

use crate::diagnostics::Diagnostic;
use crate::prove::lower::Lowered;

/// 判定层级（P0 恒 `none`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Lean,
    Offline,
    None,
}

impl Tier {
    pub fn as_str(self) -> &'static str {
        match self {
            Tier::Lean => "lean",
            Tier::Offline => "offline",
            Tier::None => "none",
        }
    }
}

/// 状态词（§1.2；P0 只会产生 `unverified` / `rejected` / `error`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Proved,
    Checked,
    Unverified,
    Unproved,
    Rejected,
    Error,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Proved => "proved",
            Verdict::Checked => "checked",
            Verdict::Unverified => "unverified",
            Verdict::Unproved => "unproved",
            Verdict::Rejected => "rejected",
            Verdict::Error => "error",
        }
    }

    /// 聚合优先级（差者胜）：error > unproved > rejected > unverified > checked > proved。
    pub fn rank(self) -> u8 {
        match self {
            Verdict::Proved => 0,
            Verdict::Checked => 1,
            Verdict::Unverified => 2,
            Verdict::Rejected => 3,
            Verdict::Unproved => 4,
            Verdict::Error => 5,
        }
    }

    pub fn worst(a: Verdict, b: Verdict) -> Verdict {
        if a.rank() >= b.rank() {
            a
        } else {
            b
        }
    }
}

/// 退出码（§3.3）。
impl Verdict {
    pub fn exit_code(self, require_lean: bool) -> i32 {
        match self {
            Verdict::Error => 1,
            Verdict::Unproved => 1,
            Verdict::Rejected | Verdict::Unverified | Verdict::Checked if require_lean => 2,
            _ => 0,
        }
    }
}

/// 选项（§8.2）。
#[derive(Debug, Clone)]
pub struct ProveOptions {
    pub entry: Option<String>,
    pub only: Vec<String>,
    pub allow_opaque: Vec<String>,
    pub emit_dir: Option<PathBuf>,
    pub strict_types: bool,
    pub require_lean: bool,
    pub report_only: bool,
    /// 绕过证书缓存（`--no-cache` / `MARQDO_PROOF_NO_CACHE`）。
    pub no_cache: bool,
    /// 显式 lean 路径（`--lean` / `MARQDO_PROOF_LEAN`）。
    pub lean: Option<PathBuf>,
    /// 单次 lean 调用超时（默认 120s）。
    pub timeout: std::time::Duration,
    /// 是否执行判定（`prove emit`/`audit` 子命令 = false，只生成不证）。
    pub judge: bool,
    /// `maxHeartbeats`（默认随 lean）。
    pub max_heartbeats: Option<u64>,
}

impl Default for ProveOptions {
    fn default() -> Self {
        Self {
            entry: None,
            only: Vec::new(),
            allow_opaque: Vec::new(),
            emit_dir: None,
            strict_types: true,
            require_lean: false,
            report_only: false,
            no_cache: false,
            lean: None,
            timeout: std::time::Duration::from_secs(120),
            judge: true,
            max_heartbeats: None,
        }
    }
}

/// 单条定理的判定面（§3.5 claims）。
#[derive(Debug, Clone)]
pub struct ClaimReport {
    pub id: String,
    pub premise: Option<String>,
    pub conclusion: String,
    pub tactic: Option<String>,
    /// `proved` / `unproved` / `unverified`（判定层给出前与函数一致）。
    pub status: Verdict,
    /// `trace`（P2 离线）/ `lean` / 空（未判定）。
    pub decided_by: String,
    /// 映射回 `.mq.md` 的行号（失败证据）。
    pub source_line: Option<u32>,
    /// Lean 原始报错片段（unproved 时）。
    pub evidence: Option<String>,
}

/// 单函数报告。
#[derive(Debug, Clone)]
pub struct FunctionReport {
    pub name: String,
    pub verdict: Verdict,
    pub tier: Tier,
    /// 不透明操作（id, effect）。
    pub ops: Vec<(String, String)>,
    /// 拒绝原因（code, message, line）。
    pub rejected: Vec<(String, String, u32)>,
    /// 定理声明 + 判定。
    pub claims: Vec<ClaimReport>,
}

/// Lean 判定信息（`lean: null` 表示没跑）。
#[derive(Debug, Clone)]
pub struct LeanInfo {
    pub toolchain: String,
    pub library_sha256: String,
    pub module_sha256: String,
    /// `hit` / `miss`。
    pub cache: &'static str,
}

/// 整体报告。
#[derive(Debug, Clone)]
pub struct Report {
    pub file: String,
    pub source_sha256: String,
    /// 产物目录 id（`.marqdo/prove/<artifact_id>`）。
    pub artifact_id: Option<String>,
    pub status: Verdict,
    pub functions: Vec<FunctionReport>,
    pub diagnostics: Vec<Diagnostic>,
    pub emitted: Vec<PathBuf>,
    pub lean: Option<LeanInfo>,
    pub duration_ms: u128,
}

impl Report {
    /// JSON 面（§3.5；`status` 枚举稳定）。
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "schema": 1,
            "status": self.status.as_str(),
            "file": self.file,
            "source_sha256": self.source_sha256,
            "artifact_id": match &self.artifact_id {
                Some(id) => json!(format!("mq://prove/{id}")),
                None => serde_json::Value::Null,
            },
            "functions": self.functions.iter().map(|f| json!({
                "name": f.name,
                "status": f.verdict.as_str(),
                "tier": f.tier.as_str(),
                "claims": f.claims.iter().map(|c| {
                    let mut j = json!({
                        "id": c.id,
                        "premise": c.premise,
                        "conclusion": c.conclusion,
                        "tactic": c.tactic,
                        "status": c.status.as_str(),
                    });
                    if !c.decided_by.is_empty() {
                        j["decided_by"] = json!(c.decided_by);
                    }
                    if let Some(line) = c.source_line {
                        j["line"] = json!(line);
                    }
                    if let Some(ev) = &c.evidence {
                        j["evidence"] = json!(ev);
                    }
                    j
                }).collect::<Vec<_>>(),
                "obligations": [],
                "ops": f.ops.iter().map(|(id, effect)| json!({
                    "id": id, "effect": effect,
                })).collect::<Vec<_>>(),
                "rejected": f.rejected.iter().map(|(code, message, line)| json!({
                    "code": code, "message": message, "line": line,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "lean": match &self.lean {
                Some(info) => json!({
                    "toolchain": info.toolchain,
                    "library_sha256": info.library_sha256,
                    "module_sha256": info.module_sha256,
                    "cache": info.cache,
                }),
                None => serde_json::Value::Null,
            },
            "assumptions": [],
            "diagnostics": self.diagnostics.iter().map(|d| d.to_json()).collect::<Vec<_>>(),
            "emitted": self.emitted.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "duration_ms": self.duration_ms as u64,
        })
    }

    /// 文本面（§3.4 风格；诊断走 stderr 由 main 负责）。
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        let functions = self.functions.len();
        let rejected = self
            .functions
            .iter()
            .filter(|f| f.verdict == Verdict::Rejected)
            .count();
        let ops: usize = self.functions.iter().map(|f| f.ops.len()).sum();
        out.push_str(&format!("prove: {}\n", self.file));
        let cache_note = match &self.lean {
            Some(info) => format!("（lean · cache: {}）", info.cache),
            None => String::new(),
        };
        out.push_str(&format!(
            "  status: {}{}\n",
            self.status.as_str(),
            cache_note
        ));
        out.push_str(&format!(
            "  functions: {functions} · opaque ops: {ops} · rejected: {rejected}\n"
        ));
        for f in &self.functions {
            if f.verdict == Verdict::Rejected {
                let codes: Vec<&str> = f.rejected.iter().map(|(c, _, _)| c.as_str()).collect();
                out.push_str(&format!("  rejected: {} ({})\n", f.name, codes.join(", ")));
            }
            for c in &f.claims {
                match (c.status, &c.evidence) {
                    (Verdict::Proved, _) => {
                        out.push_str(&format!("  proved: {}.{}\n", f.name, c.id));
                    }
                    (Verdict::Unproved, Some(ev)) => {
                        let line = c
                            .source_line
                            .map(|l| format!(":{l}"))
                            .unwrap_or_default();
                        out.push_str(&format!(
                            "  unproved: {}.{line}{} — {}\n",
                            f.name,
                            c.id,
                            first_line(ev)
                        ));
                    }
                    (Verdict::Unverified, _) => {
                        out.push_str(&format!("  unverified: {}.{}\n", f.name, c.id));
                    }
                    _ => {}
                }
            }
        }
        for p in &self.emitted {
            out.push_str(&format!("  emitted: {}\n", p.display()));
        }
        out
    }
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").to_string()
}

/// P0 主入口：装载 → 降级 → 定型 → 生成。
pub fn prove_path(path: &Path, opts: &ProveOptions) -> Result<Report> {
    let started = Instant::now();
    let module = crate::load::load_module(path)
        .with_context(|| format!("装载 {}", path.display()))?;
    let source_sha256 = module
        .source
        .sha256
        .clone()
        .unwrap_or_default();
    let file_display = display_path(path);

    // ① 装载完成；② 降级（效应表来自 sema；失败时保守 unknown）。
    let effects = lower::effect_map(path);
    let (plan, lowered) = lower::build_plan(
        &module,
        &source_sha256,
        &file_display,
        effects,
        &opts.allow_opaque,
    )?;

    // ③ 定型（分支合流等；错误码 proof.type_mismatch）。
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    diagnostics.extend(types::typecheck_plan(&module, &lowered, opts.strict_types));

    // ④ 生成（--report-only 时跳过写盘）。
    let mut emitted: Vec<PathBuf> = Vec::new();
    let mut artifact_dir: Option<PathBuf> = None;
    let mut generated_lean: Option<(PathBuf, String)> = None;
    if !opts.report_only {
        let dir = match &opts.emit_dir {
            Some(dir) => dir.clone(),
            None => default_emit_dir(&source_sha256),
        };
        let generated = dir.join("Generated");
        std::fs::create_dir_all(&generated)
            .with_context(|| format!("创建 {}", generated.display()))?;
        let (file_name, lean) = emit::emit_module(&plan, &emit::stem_of(&file_display));
        let lean_path = generated.join(&file_name);
        std::fs::write(&lean_path, &lean)
            .with_context(|| format!("写入 {}", lean_path.display()))?;
        let plan_path = dir.join("plan.json");
        std::fs::write(&plan_path, plan.to_stable_json())
            .with_context(|| format!("写入 {}", plan_path.display()))?;
        emitted.push(relative_cwd(&lean_path));
        emitted.push(relative_cwd(&plan_path));
        artifact_dir = Some(dir.clone());
        generated_lean = Some((lean_path, lean));
    }

    // 汇总：错误诊断 → error；有拒绝函数 → rejected；否则 unverified。
    let has_type_errors = diagnostics
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostics::Severity::Error));
    let mut functions = summarize(&lowered, &opts.only);
    let mut status = Verdict::Unverified;
    if !functions.is_empty() {
        status = functions
            .iter()
            .map(|f| f.verdict)
            .fold(Verdict::Proved, Verdict::worst);
    }
    if has_type_errors {
        status = Verdict::Error;
    }

    // ⑤ Lean 判定（P1）：有定理、有产物、非 report-only 时进行。
    let mut lean_info: Option<LeanInfo> = None;
    let claims_present = plan.functions.iter().any(|f| !f.claims.is_empty());
    if claims_present && opts.judge && !opts.report_only {
        match judge_with_lean(
            &plan,
            &source_sha256,
            artifact_dir.as_deref(),
            &generated_lean,
            opts,
            &mut functions,
            &mut diagnostics,
        ) {
            Ok((verdict, info)) => {
                lean_info = info;
                if let Some(v) = verdict {
                    status = v;
                }
            }
            Err(err) => {
                diagnostics.push(
                    Diagnostic::new(None, crate::diagnostics::Span::new(0, 0), format!("{err:#}"))
                        .with_code("proof.lean_error"),
                );
                status = Verdict::Error;
            }
        }
    } else if claims_present && opts.report_only {
        // report-only：不判定、不写盘；未证状态如实保留（恒返回 0 由 exit_code 处理）。
    }

    // 函数级状态聚合（判定后重算）：文件状态只看**有定理**的函数
    // 与被拒绝的函数——无定理函数没有义务，不该拖低整体（但它们
    // 自己的行如实保留 unverified / tier none）。
    {
        let relevant = functions.iter().filter(|f| {
            !f.claims.is_empty() || f.verdict == Verdict::Rejected
        });
        for f in relevant {
            status = Verdict::worst(status, f.verdict);
        }
    }

    Ok(Report {
        file: file_display,
        source_sha256,
        artifact_id: artifact_dir
            .as_ref()
            .map(|d| d.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or(None),
        status,
        functions,
        diagnostics,
        emitted,
        lean: lean_info,
        duration_ms: started.elapsed().as_millis(),
    })
}

/// 单一判定来源：缓存证书 → Lean 现算。返回（整体判定，lean 信息）。
#[allow(clippy::too_many_arguments)]
fn judge_with_lean(
    plan: &crate::prove::ir::Plan,
    source_sha256: &str,
    artifact_dir: Option<&Path>,
    generated_lean: &Option<(PathBuf, String)>,
    opts: &ProveOptions,
    functions: &mut [FunctionReport],
    diagnostics: &mut Vec<Diagnostic>,
) -> Result<(Option<Verdict>, Option<LeanInfo>)> {
    use crate::diagnostics::Span;
    let Some((lean_path, lean_src)) = generated_lean else {
        return Ok((None, None));
    };

    // 缓存键素材：工具链版本先探测（没有工具链就谈不上判定）。
    let toolchain = match driver::Toolchain::discover(opts.lean.as_deref()) {
        Ok(t) => t,
        Err(err) => {
            if opts.require_lean {
                return Err(err);
            }
            diagnostics.push(
                Diagnostic::new(None, Span::new(0, 0), format!("{err:#}"))
                    .with_code("proof.lean_missing"),
            );
            return Ok((None, None));
        }
    };
    let Some(toolchain) = toolchain else {
        // 没找到工具链：保持 unverified（诚实：没跑就是没跑）。
        // --require-lean 时退出码由 Verdict::exit_code 升到 2（§3.3 F3）。
        if opts.require_lean {
            diagnostics.push(
                Diagnostic::new(
                    None,
                    Span::new(0, 0),
                    "--require-lean：未找到 Lean 工具链（PATH/MARQDO_PROOF_LEAN/--lean）",
                )
                .with_code("proof.lean_missing"),
            );
        }
        return Ok((None, None));
    };

    // 库（内嵌源 → olean，按内容哈希缓存）。
    let cache_root = Path::new(".marqdo/prove");
    let (lib_dir, library_sha256) = driver::ensure_library(cache_root, &toolchain)?;
    let module_sha256 = cache::sha256_hex(lean_src.as_bytes());
    let plan_sha = cache::sha256_hex(plan.to_stable_json().as_bytes());
    let key = cache::cache_key(
        source_sha256,
        &plan_sha,
        &library_sha256,
        &toolchain.version,
        env!("CARGO_PKG_VERSION"),
    );

    let mut artifact_fallback: Option<PathBuf> = None;
    let artifact: &Path = match artifact_dir {
        Some(d) => d,
        None => {
            let d = default_emit_dir(source_sha256);
            // 极少见：report-only 之外无产物目录（理论上不可达）——现建。
            let _ = std::fs::create_dir_all(&d);
            // 借用问题：把目录放成一个本地 owned 值，再取引用。
            artifact_fallback.get_or_insert(d).as_path()
        }
    };

    // ① 证书命中 → 跳过判定。
    if !opts.no_cache {
        if let Some(cert) = cache::load(artifact, &key) {
            apply_claims(functions, &cert.claims, &cert.span_map, diagnostics);
            let status = cert.status;
            return Ok((
                Some(status),
                Some(LeanInfo {
                    toolchain: toolchain.version,
                    library_sha256,
                    module_sha256,
                    cache: "hit",
                }),
            ));
        }
    }

    // ② 现算：lake env lean 等价的直接 lean 调用（LEAN_PATH 指向底座 olean）。
    let outcome = driver::check_module(
        &toolchain,
        &lib_dir,
        lean_path,
        lean_src,
        opts.timeout,
        opts.max_heartbeats,
    )?;

    // ③ 证据 → 诊断（proof.claim_failed，指回源码行）。
    let claim_records: Vec<cache::ClaimRecord> = outcome
        .claims
        .iter()
        .map(|c| cache::ClaimRecord {
            id: c.claim.clone(),
            function: c.function.clone(),
            status: c.status,
            source_line: c.source_line,
        })
        .collect();
    apply_outcome(functions, &outcome.claims, diagnostics);

    let cert = cache::Certificate {
        key: key.clone(),
        status: outcome.status,
        tier: "lean",
        source_sha256: source_sha256.to_string(),
        library_sha256: library_sha256.clone(),
        toolchain: toolchain.version.clone(),
        module_sha256: module_sha256.clone(),
        claims: claim_records,
        span_map: outcome.span_map.clone(),
        marqdo: env!("CARGO_PKG_VERSION").to_string(),
    };
    if let Err(err) = cache::store(artifact, &cert) {
        diagnostics.push(
            Diagnostic::new(
                None,
                Span::new(0, 0),
                format!("证书写入失败（不阻塞判定）：{err:#}"),
            )
            .with_code("proof.cache_write"),
        );
    }

    Ok((
        Some(outcome.status),
        Some(LeanInfo {
            toolchain: toolchain.version,
            library_sha256,
            module_sha256,
            cache: "miss",
        }),
    ))
}

/// 把判定结果按 (函数, 定理 id) 贴回函数报告。
fn apply_outcome(
    functions: &mut [FunctionReport],
    claims: &[driver::ClaimOutcome],
    diagnostics: &mut Vec<Diagnostic>,
) {
    use crate::diagnostics::Span;
    for c in claims {
        let matches_fn = |f: &FunctionReport| {
            f.name == c.function
                || emit::lean_ident(&f.name)
                    .trim_matches('«')
                    .trim_matches('»')
                    == c.function
        };
        let Some(pos) = functions.iter().position(matches_fn) else {
            continue;
        };
        let f = &mut functions[pos];
        if let Some(claim) = f.claims.iter_mut().find(|x| x.id == c.claim) {
            claim.status = c.status;
            claim.decided_by = "lean".into();
            claim.source_line = c.source_line;
            claim.evidence = (!c.detail.is_empty()).then(|| c.detail.clone());
            if c.status == Verdict::Unproved {
                diagnostics.push(
                    Diagnostic::new(
                        None,
                        Span::new(c.source_line.unwrap_or(0), 0),
                        format!(
                            "定理 {}.{} 未证：{}",
                            c.function,
                            c.claim,
                            first_line(&c.detail)
                        ),
                    )
                    .with_code("proof.claim_failed"),
                );
            }
            f.verdict = f
                .claims
                .iter()
                .map(|x| x.status)
                .fold(Verdict::Proved, Verdict::worst);
            f.tier = Tier::Lean;
        }
    }
}

/// 证书命中的同款贴回。
fn apply_claims(
    functions: &mut [FunctionReport],
    claims: &[cache::ClaimRecord],
    span_map: &[(u32, u32)],
    diagnostics: &mut Vec<Diagnostic>,
) {
    use crate::diagnostics::Span;
    for c in claims {
        let source_line = c.source_line.or_else(|| {
            span_map
                .first()
                .map(|(_, m)| *m)
        });
        if let Some(f) = functions
            .iter_mut()
            .find(|f| f.name == c.function)
        {
            if let Some(claim) = f.claims.iter_mut().find(|x| x.id == c.id) {
                claim.status = c.status;
                claim.decided_by = "lean".into();
                claim.source_line = source_line;
                if c.status == Verdict::Unproved {
                    diagnostics.push(
                        Diagnostic::new(
                            None,
                            Span::new(source_line.unwrap_or(0), 0),
                            format!("定理 {}.{} 未证（缓存）", f.name, c.id),
                        )
                        .with_code("proof.claim_failed"),
                    );
                }
                f.verdict = f
                    .claims
                    .iter()
                    .map(|x| x.status)
                    .fold(Verdict::Proved, Verdict::worst);
                f.tier = Tier::Lean;
            }
        }
    }
}

fn summarize(lowered: &[Lowered], only: &[String]) -> Vec<FunctionReport> {
    lowered
        .iter()
        .filter(|l| !l.is_object)
        .filter(|l| only.is_empty() || only.iter().any(|name| name == &l.display))
        .map(|l| {
            let (verdict, tier, ops, rejected, claims) = match &l.plan {
                Some(plan) => (
                    Verdict::Unverified,
                    Tier::None,
                    plan.ops
                        .iter()
                        .map(|op| (op.id.clone(), op.effect.clone()))
                        .collect(),
                    Vec::new(),
                    plan.claims
                        .iter()
                        .map(|c| ClaimReport {
                            id: c.id.clone(),
                            premise: c.premise.clone(),
                            conclusion: c.conclusion.clone(),
                            tactic: c.tactic.clone(),
                            status: Verdict::Unverified,
                            decided_by: String::new(),
                            source_line: None,
                            evidence: None,
                        })
                        .collect(),
                ),
                None => (
                    Verdict::Rejected,
                    Tier::None,
                    Vec::new(),
                    l.rejected
                        .iter()
                        .map(|d| {
                            (
                                d.code.clone(),
                                d.message.clone(),
                                d.span.line,
                            )
                        })
                        .collect(),
                    Vec::new(),
                ),
            };
            FunctionReport {
                name: l.display.clone(),
                verdict,
                tier,
                ops,
                rejected,
                claims,
            }
        })
        .collect()
}

/// 默认产物目录：`.marqdo/prove/<key8>/`（key = 源 sha256 前 8 位）。
pub fn default_emit_dir(source_sha256: &str) -> PathBuf {
    PathBuf::from(".marqdo/prove").join(&source_sha256[..source_sha256.len().min(8)])
}

/// `prove clean`：删除 `.marqdo/prove`（--emit 指定目录不动）。
pub fn clean() -> Result<usize> {
    let root = Path::new(".marqdo/prove");
    if !root.exists() {
        return Ok(0);
    }
    let count = std::fs::read_dir(root)?.count();
    std::fs::remove_dir_all(root)
        .with_context(|| format!("删除 {}", root.display()))?;
    Ok(count)
}

fn display_path(path: &Path) -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    path.strip_prefix(&cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn relative_cwd(path: &Path) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    path.strip_prefix(&cwd).unwrap_or(path).to_path_buf()
}
