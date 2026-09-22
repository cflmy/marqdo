//! 有界修复（T3.3 护栏的**机械执行面**）：锚点局部修复，越界必拒（abstain），全有或全无。
//!
//! 修复循环三步：诊断（run/check）→ `mlsp repair_targets`（靶点 + 范围）→ `mlsp repair_apply`（改行）。
//! 护栏不是提示词愿望，而是这里代码强制的：
//!
//! - 只允许三种行编辑：`Replace` / `Delete` / `InsertBefore`；
//! - 被触碰的每一行都必须落在靶点范围内（`InsertBefore` 额外允许紧跟范围末尾追加），
//!   否则**整体拒绝**并报越界行清单——一个字节都不改（全有或全无）；
//! - 应用结果交给调用方复验（`validate` / `run`）。
//!
//! 目标（[three-problems.md](../../doc/roadmap/three-problems.md) §5.3）：易锚定错误
//! ≥80% 修复率的同时，**越权范围的改动必拒**、失败后 abstain 而非乱改。

use std::collections::{HashMap, HashSet};

/// 一行编辑（1-based 行号）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// 把 `line` 整行换成 `text`。
    Replace { line: u32, text: String },
    /// 删除 `line`。
    Delete { line: u32 },
    /// 在 `line` **之前**插入 `text`（允许 `line == 范围末行 + 1`，即紧跟块尾追加）。
    InsertBefore { line: u32, text: String },
}

impl Edit {
    /// 被触碰的行（范围检查用）。
    pub fn touched(&self) -> u32 {
        match self {
            Edit::Replace { line, .. } | Edit::Delete { line } | Edit::InsertBefore { line, .. } => {
                *line
            }
        }
    }
}

/// 越界拒绝（abstain）：报告越界行与允许范围，源保持不变。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutOfScope {
    pub lines: Vec<u32>,
    pub ranges: Vec<(u32, u32)>,
}

fn in_ranges(ranges: &[(u32, u32)], line: u32) -> bool {
    ranges.iter().any(|(s, e)| line >= *s && line <= *e)
}

fn at_tail(ranges: &[(u32, u32)], line: u32) -> bool {
    ranges.iter().any(|(_, e)| line == e + 1)
}

/// 全有或全无地应用行编辑。
///
/// - 任何被触碰行越出 `ranges`（或行号非法）⇒ [`Err`]（越界清单），**源不变**；
/// - 同一行被多个编辑触碰时 `Delete` 优先于 `Replace`，同种后者覆盖前者。
pub fn apply_edits(
    source: &str,
    ranges: &[(u32, u32)],
    edits: &[Edit],
) -> Result<String, OutOfScope> {
    let lines: Vec<&str> = source.lines().collect();
    let n_lines = lines.len() as u32;

    // ---- 范围检查（先全体验完再动源） ----
    let mut bad = Vec::new();
    for e in edits {
        let line = e.touched();
        let legal = match e {
            Edit::InsertBefore { .. } => {
                (line >= 1 && line <= n_lines + 1)
                    && (in_ranges(ranges, line) || at_tail(ranges, line))
            }
            _ => line >= 1 && line <= n_lines && in_ranges(ranges, line),
        };
        if !legal {
            bad.push(line);
        }
    }
    bad.sort_unstable();
    bad.dedup();
    if !bad.is_empty() {
        return Err(OutOfScope {
            lines: bad,
            ranges: ranges.to_vec(),
        });
    }

    // ---- 应用 ----
    let mut replaced: HashMap<u32, String> = HashMap::new();
    let mut deleted: HashSet<u32> = HashSet::new();
    let mut inserts: HashMap<u32, Vec<String>> = HashMap::new();
    for e in edits {
        match e {
            Edit::Replace { line, text } => {
                replaced.insert(*line, text.clone());
            }
            Edit::Delete { line } => {
                deleted.insert(*line);
                replaced.remove(line); // Delete 优先
            }
            Edit::InsertBefore { line, text } => {
                inserts.entry(*line).or_default().push(text.clone());
            }
        }
    }

    let trailing_newline = source.ends_with('\n');
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + inserts.len());
    for (i, l) in lines.iter().enumerate() {
        let n = (i + 1) as u32;
        if let Some(texts) = inserts.get(&n) {
            out.extend(texts.iter().cloned());
        }
        if deleted.contains(&n) {
            continue;
        }
        out.push(replaced.get(&n).cloned().unwrap_or_else(|| l.to_string()));
    }
    if let Some(texts) = inserts.get(&(n_lines + 1)) {
        out.extend(texts.iter().cloned());
    }

    let mut s = out.join("\n");
    if trailing_newline {
        s.push('\n');
    }
    Ok(s)
}
