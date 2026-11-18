//! Lean 驱动（设计 §8.4 / roadmap P1）：工具链发现、语义底座构建、
//! `lean` 调用（超时）、输出解析、`-- mq:` span 映射。
//!
//! 与设计的偏差（§12 记录）：驱动直接用 `lean -o` 构建 `MarqdoProof.olean`，
//! 不经 `lake`——运行时只依赖 `lean` 一个二进制，库源内嵌于本二进制
//! （`include_str!`），按内容哈希落盘缓存；lakefile 仍保留给人/CI 使用。

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::prove::Verdict;

/// 内嵌语义底座源（发布二进制自带；构建期校验见 scripts/check-proof-library.sh）。
pub const LIBRARY_SRC: &str = include_str!("../../doc/verification/skill-proof/MarqdoProof.lean");

/// 单条定理的判定结果。
#[derive(Debug, Clone)]
pub struct ClaimOutcome {
    /// 定理名（生成文件里的 `«函数.定理id»`）。
    pub theorem: String,
    /// 归属函数显示名。
    pub function: String,
    /// 定理 id（定理表里的 id）。
    pub claim: String,
    pub status: Verdict,
    /// 映射回 `.mq.md` 的行号（定理表所在行附近；找不到为 None）。
    pub source_line: Option<u32>,
    /// Lean 原始报错（unproved/lean-error 时）。
    pub detail: String,
}

/// 一次模块级 Lean 判定。
#[derive(Debug, Clone)]
pub struct LeanOutcome {
    pub status: Verdict,
    pub claims: Vec<ClaimOutcome>,
    /// 原始 stdout+stderr（诊断证据）。
    pub log: String,
    /// lean 行 → mq 行 的映射表（进证书）。
    pub span_map: Vec<(u32, u32)>,
}

/// 已发现的工具链。
#[derive(Debug, Clone)]
pub struct Toolchain {
    pub lean: PathBuf,
    pub version: String,
}

impl Toolchain {
    /// 发现顺序（§8.4）：显式路径 → `MARQDO_PROOF_LEAN` → `PATH`。
    /// 显式给出但不可用 → Err（**不**回退：避免"以为在跑 Lean 其实没有"）。
    pub fn discover(explicit: Option<&Path>) -> Result<Option<Toolchain>> {
        if let Some(path) = explicit {
            let t = Self::from_path(path)
                .with_context(|| format!("proof.lean_missing：{}", path.display()))?;
            return Ok(Some(t));
        }
        if let Ok(env_path) = std::env::var("MARQDO_PROOF_LEAN") {
            if !env_path.trim().is_empty() {
                let t = Self::from_path(Path::new(&env_path)).with_context(|| {
                    format!("proof.lean_missing：MARQDO_PROOF_LEAN={env_path}")
                })?;
                return Ok(Some(t));
            }
        }
        for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
            let candidate = dir.join("lean");
            if candidate.is_file() {
                return Self::from_path(&candidate).map(Some);
            }
        }
        Ok(None)
    }

    fn from_path(path: &Path) -> Result<Toolchain> {
        let out = Command::new(path)
            .arg("--version")
            .output()
            .with_context(|| format!("执行 {}", path.display()))?;
        if !out.status.success() {
            anyhow::bail!("{} --version 失败", path.display());
        }
        let version = String::from_utf8_lossy(&out.stdout).trim().to_string();
        Ok(Toolchain {
            lean: path.to_path_buf(),
            version,
        })
    }
}

/// 语义底座目录（按内容哈希缓存），返回（目录，库 sha256）。
pub fn ensure_library(cache_root: &Path, lean: &Toolchain) -> Result<(PathBuf, String)> {
    let lib_sha = crate::prove::cache::sha256_hex(LIBRARY_SRC.as_bytes());
    let dir = cache_root.join(format!("library-{}", &lib_sha[..8]));
    let src = dir.join("MarqdoProof.lean");
    let olean = dir.join("MarqdoProof.olean");
    if !olean.exists() {
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("创建 {}", dir.display()))?;
        std::fs::write(&src, LIBRARY_SRC)
            .with_context(|| format!("写入 {}", src.display()))?;
        let out = Command::new(&lean.lean)
            .arg("-o")
            .arg(&olean)
            .arg(&src)
            .output()
            .with_context(|| format!("构建 MarqdoProof.olean"))?;
        if !out.status.success() || !olean.exists() {
            anyhow::bail!(
                "MarqdoProof 库构建失败：\n{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
    Ok((dir, lib_sha))
}

/// 判定一个生成的 Lean 模块。
/// Compile a self-contained Lean module for a reusable, independent caller
/// proof. Source and output must both be inside the immutable build directory.
pub fn compile_import_module(lean:&Toolchain,src:&Path,olean:&Path,
    import_root:&Path,timeout:Duration)->Result<()> {
    let original=std::fs::read(src)?;
    let mut command=Command::new(&lean.lean);
    command.env("LEAN_PATH",import_root)
        .arg("-o").arg(olean).arg(src);
    let (log,status,timed_out)=run_with_timeout(&mut command,timeout)?;
    anyhow::ensure!(!timed_out,"proof.compose_timeout: {}",src.display());
    anyhow::ensure!(status==Some(0)&&olean.is_file(),
        "proof.compose_lean_failed: {log}");
    anyhow::ensure!(std::fs::read(src)?==original,
        "proof.compose_source_race: generated Lean changed during verification");
    Ok(())
}

/// Re-check a generated, closed proof-obligation module with the Lean kernel.
/// No local JSON certificate or Rust-side decision may replace this check.
pub fn check_closed_goals(
    toolchain:&Toolchain, source:&Path, timeout:Duration
)->Result<()>{
    let source_before=std::fs::read(source)?;
    let mut command=Command::new(&toolchain.lean);
    command.arg(source);
    let (log,code,timed_out)=run_with_timeout(&mut command,timeout)?;
    anyhow::ensure!(std::fs::read(source)?==source_before,
        "proof.modular_source_race: Lean goal module changed during verification");
    anyhow::ensure!(!timed_out,
        "proof.modular_timeout: Lean did not finish within {}s",timeout.as_secs());
    anyhow::ensure!(code==Some(0),
        "proof.modular_unproved: Lean rejected callsite goals: {log}");
    Ok(())
}

pub fn check_module(
    lean: &Toolchain,
    lib_dir: &Path,
    generated: &Path,
    generated_src: &str,
    timeout: Duration,
    max_heartbeats: Option<u64>,
) -> Result<LeanOutcome> {
    let span_map = span_map_of(generated_src);
    let mut cmd = Command::new(&lean.lean);
    cmd.env("LEAN_PATH", lib_dir);
    if let Some(hb) = max_heartbeats {
        cmd.arg(format!("-DmaxHeartbeats={hb}"));
    }
    cmd.arg(generated);
    let (log, code, timed_out) = run_with_timeout(&mut cmd, timeout)?;
    if timed_out {
        return Ok(LeanOutcome {
            status: Verdict::Error,
            claims: Vec::new(),
            log: format!("timeout after {}s\n{log}", timeout.as_secs()),
            span_map,
        });
    }

    // 归因：错误行 → 所在定理；每条定理都给结论（无错 → proved / sorry → unverified）。
    let theorems = theorem_spans(generated_src);
    let mut claims: Vec<ClaimOutcome> = Vec::new();
    let mut any_unproved = false;
    let mut any_error = false;
    for (line_no, kind, message) in parse_lean_errors(&log) {
        let info = theorems
            .iter()
            .find(|t| line_no >= t.start && line_no <= t.end);
        let (theorem, claim, source_line) = match info {
            Some(t) => (
                t.name.clone(),
                t.claim.clone(),
                t.source_line.or_else(|| map_to_source(line_no, &span_map)),
            ),
            // 错误在函数体/模块级（如 def 编译失败）：claim 留空，走整体 Error。
            None => (
                String::new(),
                String::new(),
                map_to_source(line_no, &span_map),
            ),
        };
        let function = if theorem.is_empty() {
            String::new()
        } else {
            function_of_theorem(&theorem, &claim)
        };
        let status = match kind {
            ErrorKind::Unsolved => {
                any_unproved = true;
                Verdict::Unproved
            }
            ErrorKind::Other => {
                any_error = true;
                Verdict::Error
            }
        };
        claims.push(ClaimOutcome {
            theorem,
            function,
            claim,
            status,
            source_line,
            detail: message,
        });
    }

    for t in &theorems {
        if claims.iter().any(|c| c.theorem == t.name) {
            continue;
        }
        let (status, detail) = if t.has_sorry {
            (
                Verdict::Unverified,
                "策略 none：定理生成但不证明".to_string(),
            )
        } else {
            (Verdict::Proved, String::new())
        };
        claims.push(ClaimOutcome {
            theorem: t.name.clone(),
            function: function_of_theorem(&t.name, &t.claim),
            claim: t.claim.clone(),
            status,
            source_line: t.source_line,
            detail,
        });
    }
    claims.sort_by(|a, b| a.theorem.cmp(&b.theorem));

    let status = if any_error || code.map(|c| c != 0).unwrap_or(false) && !any_unproved {
        // 解析到其它编译错误，或退出码非 0 且没有可归属的未证目标 → 判定层错误。
        Verdict::Error
    } else if any_unproved {
        Verdict::Unproved
    } else {
        Verdict::Proved
    };
    Ok(LeanOutcome {
        status,
        claims,
        log,
        span_map,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ErrorKind {
    Unsolved,
    Other,
}

/// 带超时的进程运行（无外部依赖：spawn + try_wait 轮询）。
fn run_with_timeout(
    cmd: &mut Command,
    timeout: Duration,
) -> Result<(String, Option<i32>, bool)> {
    use std::io::Read;
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("spawn lean")?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut s) = child.stdout.take() {
                let _ = s.read_to_end(&mut stdout);
            }
            if let Some(mut s) = child.stderr.take() {
                let _ = s.read_to_end(&mut stderr);
            }
            let mut log = String::from_utf8_lossy(&stdout).to_string();
            log.push_str(&String::from_utf8_lossy(&stderr));
            return Ok((log, Some(status.code().unwrap_or(-1)), false));
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Ok((String::new(), None, true));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// 解析 lean 输出里的 `<path>:LINE:COL: error…`；`unsolved goals` 单独归类。
/// lean 的报错头独占一行（`Generated/x.lean:41:0: error: unsolved goals`），
/// 后随目标上下文行；这里只取头行。
fn parse_lean_errors(log: &str) -> Vec<(u32, ErrorKind, String)> {
    let mut out = Vec::new();
    for line in log.lines() {
        // 头行形如 `<file>:LINE:COL: error: ...`（COL 可能省略）。
        let Some(col_idx) = line.find(": error") else {
            continue;
        };
        let pos = &line[..col_idx];
        let message = line[col_idx..].trim_start_matches(": ").to_string();
        // pos 形如 `<file>:LINE:COL`（COL 可省略）；取**倒数第二个**数字段。
        let fields: Vec<&str> = pos.rsplit(':').collect();
        let line_no: u32 = if let Ok(col) = fields[0].parse::<u32>() {
            fields.get(1).and_then(|s| s.parse().ok()).unwrap_or(col)
        } else if let Ok(n) = fields[0].parse::<u32>() {
            n
        } else {
            continue;
        };
        let kind = if message.contains("unsolved goals") {
            ErrorKind::Unsolved
        } else {
            ErrorKind::Other
        };
        out.push((line_no, kind, message));
    }
    out
}

/// 生成文件里的一条定理。
#[derive(Debug, Clone)]
pub struct TheoremInfo {
    pub name: String,
    pub start: u32,
    pub end: u32,
    /// 头注释里的定理 id：`-- 定理 <id>（策略 …；mq: N）`。
    pub claim: String,
    /// 头注释 `；mq: N` 的 N —— 定理表在 `.mq.md` 里的行。
    pub source_line: Option<u32>,
    pub has_sorry: bool,
}

/// 扫描生成文件：`-- 定理` 头注释 + `theorem` 行区间 + sorry 标记。
pub fn theorem_spans(src: &str) -> Vec<TheoremInfo> {
    let lines: Vec<&str> = src.lines().collect();
    let mut out: Vec<TheoremInfo> = Vec::new();
    // (name, start, claim, mq_line)
    let mut current: Option<(String, u32, String, Option<u32>)> = None;
    // 头注释暂存（claim id, mq 行）——紧随其后的 `theorem` 行使用。
    let mut pending: Option<(String, Option<u32>)> = None;
    for (idx, line) in lines.iter().enumerate() {
        let line_no = (idx + 1) as u32;
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("-- 定理 ") {
            let head = rest.split('（').next().unwrap_or(rest).trim().to_string();
            let mq = rest
                .rsplit_once("mq: ")
                .and_then(|(_, n)| n.trim_end_matches('）').trim().parse().ok());
            pending = Some((head, mq));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("theorem ") {
            if let Some((name, start, claim, mq)) = current.take() {
                out.push(theorem_info(src, name, start, line_no - 1, claim, mq));
            }
            if let Some(name) = rest.split_whitespace().next() {
                let (claim, mq) = pending.take().unwrap_or_default();
                current = Some((name.to_string(), line_no, claim, mq));
            }
        } else if current.is_some()
            && !trimmed.is_empty()
            && !line.starts_with(' ')
            && !line.starts_with('\t')
            && !trimmed.starts_with("--")
        {
            let (name, start, claim, mq) = current.take().unwrap();
            out.push(theorem_info(src, name, start, line_no.saturating_sub(1), claim, mq));
        }
    }
    if let Some((name, start, claim, mq)) = current {
        out.push(theorem_info(src, name, start, lines.len() as u32, claim, mq));
    }
    out
}

fn theorem_info(
    src: &str,
    name: String,
    start: u32,
    end: u32,
    claim: String,
    source_line: Option<u32>,
) -> TheoremInfo {
    let has_sorry = src
        .lines()
        .skip(start.saturating_sub(1) as usize)
        .take(end.saturating_sub(start) as usize + 1)
        .any(|l| l.contains("sorry"));
    TheoremInfo {
        name,
        start,
        end,
        claim,
        source_line,
        has_sorry,
    }
}

/// 定理名 → 归属函数：claim id 作后缀剥掉（函数名可含点，如 `台账.记录`）。
fn function_of_theorem(theorem: &str, claim: &str) -> String {
    let inner = theorem
        .trim_start_matches('«')
        .trim_end_matches('»')
        .to_string();
    if !claim.is_empty() {
        let suffix = format!(".{claim}");
        if let Some(f) = inner.strip_suffix(&suffix) {
            return f.to_string();
        }
    }
    // 兜底：第一段（单段函数名）。
    inner
        .split('.')
        .next()
        .unwrap_or(&inner)
        .to_string()
}

/// 生成文件的 `-- mq: LINE:COL` → (lean 行, mq 行)。
pub fn span_map_of(src: &str) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    for (idx, line) in src.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("-- mq: ") {
            if let Some((l, _)) = rest.split_once(':') {
                if let Ok(mq_line) = l.trim().parse::<u32>() {
                    out.push(((idx + 1) as u32, mq_line));
                }
            }
        }
    }
    out
}

/// lean 行号 → 最近的 `-- mq:` 注释对应的源行。
fn map_to_source(lean_line: u32, span_map: &[(u32, u32)]) -> Option<u32> {
    span_map
        .iter()
        .filter(|(l, _)| *l <= lean_line)
        .map(|(_, m)| *m)
        .next_back()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lean_errors_distinguishes_line_from_col() {
        // `<file>:47:62: error: …` — 行 47，列 62；曾把列当行导致定理错配。
        let log = "Generated/x.lean:47:62: error: unsolved goals\n  «结果» = …\n";
        let errs = parse_lean_errors(log);
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].0, 47, "必须取行号而非列号");
        assert_eq!(errs[0].1, ErrorKind::Unsolved);

        let log2 = "Generated/x.lean:12:5: error: unknown identifier «Foo»\n";
        let errs2 = parse_lean_errors(log2);
        assert_eq!(errs2[0].0, 12);
        assert_eq!(errs2[0].1, ErrorKind::Other);
    }

    #[test]
    fn theorem_spans_reads_header_comment_claim_and_line() {
        let src = "\
-- mq: 14:1
def «台账.记录» (env : Env) («金额» : Val) : Val × List Op :=
  («金额», [])
-- 定理 记过即记（策略 none；mq: 10）
theorem «台账.记录.记过即记» (env : Env) («金额» : Val) :
    True := by
  trivial
-- 定理 另一条（策略 auto；mq: 11）
theorem «台账.记录.另一条» (env : Env) : True := by
  sorry
";
        let ts = theorem_spans(src);
        assert_eq!(ts.len(), 2);
        assert_eq!(ts[0].name, "«台账.记录.记过即记»");
        assert_eq!(ts[0].claim, "记过即记");
        assert_eq!(ts[0].source_line, Some(10));
        assert!(!ts[0].has_sorry);
        assert_eq!(ts[1].claim, "另一条");
        assert_eq!(ts[1].source_line, Some(11));
        assert!(ts[1].has_sorry, "sorry 必须被识别为 unverified");
    }

    #[test]
    fn function_of_theorem_strips_claim_suffix_not_first_segment() {
        // 函数名可含点：台账.记录.记过即记 → 函数 = 台账.记录。
        assert_eq!(
            function_of_theorem("«台账.记录.记过即记»", "记过即记"),
            "台账.记录"
        );
        assert_eq!(
            function_of_theorem("«审批单.大额必经人工»", "大额必经人工"),
            "审批单"
        );
        assert_eq!(function_of_theorem("«孤定理»", ""), "孤定理");
    }

    #[test]
    fn span_map_pairs_lean_and_mq_lines() {
        let src = "-- mq: 14:1\nlet x := 1\n-- mq: 16:1\nlet y := 2\n";
        assert_eq!(span_map_of(src), vec![(1, 14), (3, 16)]);
    }

    #[test]
    fn map_to_source_finds_nearest_preceding_comment() {
        let map = vec![(21, 14), (24, 16), (40, 33)];
        assert_eq!(map_to_source(21, &map), Some(14));
        assert_eq!(map_to_source(47, &map), Some(33));
        assert_eq!(map_to_source(1, &map), None);
    }

    #[test]
    fn toolchain_discovery_rejects_explicit_missing_path() {
        let err = Toolchain::discover(Some(Path::new("/nonexistent/lean-绝无仅有")))
            .expect_err("显式坏路径必须报错（不回退）");
        assert!(format!("{err:#}").contains("proof.lean_missing"));
    }
}
