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

/// 选项（§8.2；P0 生效的子集见字段注释）。
#[derive(Debug, Clone)]
pub struct ProveOptions {
    pub entry: Option<String>,
    pub only: Vec<String>,
    pub allow_opaque: Vec<String>,
    pub emit_dir: Option<PathBuf>,
    pub strict_types: bool,
    pub require_lean: bool,
    pub report_only: bool,
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
        }
    }
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
    /// 定理声明（id, premise, conclusion, tactic）。
    pub claims: Vec<(String, Option<String>, String, Option<String>)>,
}

/// 整体报告。
#[derive(Debug, Clone)]
pub struct Report {
    pub file: String,
    pub source_sha256: String,
    pub status: Verdict,
    pub functions: Vec<FunctionReport>,
    pub diagnostics: Vec<Diagnostic>,
    pub emitted: Vec<PathBuf>,
    pub duration_ms: u128,
}

impl Report {
    /// JSON 面（§3.5；`status` 枚举稳定，`lean` 恒 null 直到 P1）。
    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::json;
        json!({
            "schema": 1,
            "status": self.status.as_str(),
            "file": self.file,
            "source_sha256": self.source_sha256,
            "artifact_id": serde_json::Value::Null,
            "functions": self.functions.iter().map(|f| json!({
                "name": f.name,
                "status": f.verdict.as_str(),
                "tier": f.tier.as_str(),
                "claims": f.claims.iter().map(|(id, premise, conclusion, tactic)| json!({
                    "id": id,
                    "premise": premise,
                    "conclusion": conclusion,
                    "tactic": tactic,
                })).collect::<Vec<_>>(),
                "obligations": [],
                "ops": f.ops.iter().map(|(id, effect)| json!({
                    "id": id, "effect": effect,
                })).collect::<Vec<_>>(),
                "rejected": f.rejected.iter().map(|(code, message, line)| json!({
                    "code": code, "message": message, "line": line,
                })).collect::<Vec<_>>(),
            })).collect::<Vec<_>>(),
            "lean": serde_json::Value::Null,
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
        out.push_str(&format!(
            "  status: {}{}\n",
            self.status.as_str(),
            if self.status == Verdict::Unverified {
                "（P0：仅降级与生成，判定将在 P1 提供）"
            } else {
                ""
            }
        ));
        out.push_str(&format!(
            "  functions: {functions} · opaque ops: {ops} · rejected: {rejected}\n"
        ));
        for f in &self.functions {
            if f.verdict == Verdict::Rejected {
                let codes: Vec<&str> = f.rejected.iter().map(|(c, _, _)| c.as_str()).collect();
                out.push_str(&format!("  rejected: {} ({})\n", f.name, codes.join(", ")));
            }
        }
        for p in &self.emitted {
            out.push_str(&format!("  emitted: {}\n", p.display()));
        }
        out
    }
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
    }

    // 汇总：错误诊断 → error；有拒绝函数 → rejected；否则 unverified。
    let has_type_errors = diagnostics
        .iter()
        .any(|d| matches!(d.severity, crate::diagnostics::Severity::Error));
    let functions = summarize(&lowered, &opts.only);
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

    Ok(Report {
        file: file_display,
        source_sha256,
        status,
        functions,
        diagnostics,
        emitted,
        duration_ms: started.elapsed().as_millis(),
    })
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
                        .map(|c| {
                            (
                                c.id.clone(),
                                c.premise.clone(),
                                c.conclusion.clone(),
                                c.tactic.clone(),
                            )
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
