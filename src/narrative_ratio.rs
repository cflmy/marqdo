//! Narration ratio gate: each function unit should be ≥50% narrative text.
//!
//! Used by `marqdo check --narrative-ratio` and extension skill dogfood.

use std::path::Path;

use anyhow::Result;

use crate::ast::Function;
use crate::diagnostics::{Diagnostic, Span};
use crate::load::load_module;

const MIN_RATIO: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct UnitReport {
    pub name: String,
    pub line: u32,
    pub narrative: usize,
    pub total: usize,
    pub ratio: f64,
    pub pass: bool,
}

/// Check one `.mq.md` file; returns Error diagnostics for failing units.
pub fn check_file(path: &Path) -> Result<Vec<Diagnostic>> {
    let reports = check_file_source(path)?;
    Ok(reports
        .iter()
        .filter(|r| !r.pass)
        .map(|r| {
            Diagnostic::new(
                Some(path),
                Span::new(r.line, 1),
                format!(
                    "narrative ratio {:.0}% < 50% for function `{}` (narrative={}, total={})",
                    r.ratio * 100.0,
                    r.name,
                    r.narrative,
                    r.total
                ),
            )
            .with_code("narrative.ratio")
        })
        .collect())
}

/// Measure using original file text between this function's line and the next heading.
pub fn check_file_source(path: &Path) -> Result<Vec<UnitReport>> {
    let text = std::fs::read_to_string(path)?;
    let module = load_module(path)?;
    Ok(measure_source_units(&text, &module.functions))
}

pub fn measure_source_units(source: &str, functions: &[Function]) -> Vec<UnitReport> {
    let lines: Vec<&str> = source.lines().collect();
    let mut starts: Vec<(String, u32, usize)> = Vec::new();
    collect_starts(functions, &mut starts);
    starts.sort_by_key(|(_, line, _)| *line);

    let mut out = Vec::new();
    for (i, (name, line, _)) in starts.iter().enumerate() {
        let start = (*line as usize).saturating_sub(1);
        let end = starts
            .get(i + 1)
            .map(|(_, l, _)| (*l as usize).saturating_sub(1))
            .unwrap_or(lines.len());
        let chunk = lines[start..end.min(lines.len())].join("\n");
        let (narrative, code) = split_narrative_code(&chunk);
        let total = narrative + code;
        let ratio = if total == 0 {
            1.0
        } else {
            narrative as f64 / total as f64
        };
        out.push(UnitReport {
            name: name.clone(),
            line: *line,
            narrative,
            total,
            ratio,
            pass: total == 0 || ratio + f64::EPSILON >= MIN_RATIO,
        });
    }
    out
}

fn collect_starts(fns: &[Function], out: &mut Vec<(String, u32, usize)>) {
    for f in fns {
        out.push((f.name.clone(), f.span.line, f.body.len()));
        collect_starts(&f.children, out);
    }
}

/// Split a function unit's source into narrative vs code character counts.
///
/// Code: lines that are primarily executable markers (`>`, `**`, `*…*`, `+`, tables with `|`,
/// fences, `#`/`##` headings themselves). Narrative: remaining prose (including formula fences
/// content is counted as narrative — formulas are documentation).
pub fn split_narrative_code(chunk: &str) -> (usize, usize) {
    let mut narrative = 0usize;
    let mut code = 0usize;
    let mut in_fence = false;
    let mut fence_is_formula = false;
    for line in chunk.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            fence_is_formula = trimmed.contains("$$") || trimmed == "$$";
            // Fence markers themselves are markup/code budget.
            code += line.chars().count();
            continue;
        }
        if in_fence {
            // Math/display fences count as narration; language fences as code.
            if fence_is_formula {
                narrative += line.chars().count();
            } else {
                code += line.chars().count();
            }
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        // Standalone $$ formula blocks (Marqdo formula assignment bodies).
        if trimmed == "$$" || trimmed.starts_with("$$") {
            narrative += line.chars().count();
            continue;
        }
        if is_code_line(trimmed) {
            code += line.chars().count();
        } else {
            narrative += line.chars().count();
        }
    }
    (narrative, code)
}

fn is_code_line(trimmed: &str) -> bool {
    if trimmed.starts_with('>') {
        return true;
    }
    if trimmed.starts_with("**") && trimmed.ends_with("**") {
        return true;
    }
    if trimmed.starts_with('*') && trimmed.ends_with('*') && !trimmed.starts_with("**") {
        // Return line
        return true;
    }
    if trimmed.starts_with("+ `") || trimmed.starts_with("+`") {
        return true;
    }
    // Contract / documentation tables are narration surface (param and return
    // tables under skill tools), not executable code budget.
    if trimmed.starts_with('|') {
        return false;
    }
    if trimmed.starts_with('#') {
        return true;
    }
    if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) && trimmed.contains(". ") {
        // Branch list `1. …` — executable control
        return true;
    }
    if trimmed.starts_with("- ") && (trimmed.contains('>') || trimmed.contains('`')) {
        return true;
    }
    if trimmed.starts_with("---") {
        return true;
    }
    if trimmed.starts_with("import:") || trimmed.starts_with("导入:") {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prose_heavy_unit_passes() {
        let chunk = r#"
## hadamard

The Hadamard gate creates equal superposition from the computational basis.
It is the unitary matrix shown below and is central to many algorithms.

| 参数 | 类型 | 说明 |
|------|------|------|
| qubit | number | target |

This paragraph keeps narration above half even with a small table.

| 返回 | 类型 | 说明 |
|------|------|------|
|  | handle | circuit |

*None*
"#;
        let (n, c) = split_narrative_code(chunk);
        assert!(
            (n as f64) / ((n + c) as f64) >= 0.5,
            "n={n} c={c}"
        );
    }

    #[test]
    fn api_list_fails() {
        let chunk = r#"
## h
    + `qubit`=0
*> quantum_h circuit=`self` qubit=`qubit`*
"#;
        let (n, c) = split_narrative_code(chunk);
        assert!(
            (n as f64) / ((n + c) as f64) < 0.5,
            "n={n} c={c}"
        );
    }
}
