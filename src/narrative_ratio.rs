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
/// Count executable inline payloads separately from the sentences containing them.
/// Occlusion labels and unbound contract/documentation tables are narration;
/// bound data tables, expression payloads, code fences and structure markers are code.
/// Display formulas retain the existing documentation treatment.
pub fn split_narrative_code(chunk: &str) -> (usize, usize) {
    use crate::parse::{scan_md_links_in_line, scan_prose_line, ProseBit};
    let mut narrative = 0usize;
    let mut code = 0usize;
    let mut in_fence = false;
    let mut fence_is_formula = false;
    let mut pending_table = false;
    let mut data_table = false;
    for line in chunk.lines() {
        let trimmed = line.trim();
        let count = line.chars().count();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            fence_is_formula = trimmed.contains("$$") || trimmed == "$$";
            code += count;
            continue;
        }
        if in_fence {
            if fence_is_formula {
                narrative += count;
            } else {
                code += count;
            }
            continue;
        }
        if trimmed.is_empty() {
            data_table = false;
            continue;
        }
        if trimmed.starts_with('|') {
            if pending_table {
                data_table = true;
                pending_table = false;
            }
            if data_table {
                code += count;
            } else {
                narrative += count;
            }
            continue;
        }
        data_table = false;
        pending_table = (trimmed.starts_with('`') && trimmed.ends_with('='))
            || (trimmed.starts_with("**")
                && trimmed.ends_with("**")
                && trimmed[2..trimmed.len() - 2].trim_end().ends_with('='));
        if trimmed.starts_with("$$") {
            narrative += count;
            continue;
        }
        let mut inline_code = 0;
        for bit in scan_prose_line(line) {
            let (inner, markers) = match bit {
                ProseBit::BoldCode { inner, .. } => (inner, 4),
                ProseBit::ItalicReturn { inner, .. } => (inner, 2),
                ProseBit::Decl { .. } => continue,
            };
            let labels: usize = scan_md_links_in_line(&inner, 0)
                .iter()
                .map(|link| link.label.chars().count())
                .sum();
            inline_code += (inner.chars().count() + markers).saturating_sub(labels);
        }
        if inline_code > 0 {
            let actual = inline_code.min(count);
            code += actual;
            narrative += count - actual;
        } else if is_code_line(trimmed) || pending_table {
            let labels: usize = scan_md_links_in_line(line, 0)
                .iter()
                .map(|link| link.label.chars().count())
                .sum();
            narrative += labels.min(count);
            code += count.saturating_sub(labels);
        } else {
            narrative += count;
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
        assert!((n as f64) / ((n + c) as f64) >= 0.5, "n={n} c={c}");
    }

    #[test]
    fn api_list_fails() {
        let chunk = r#"
## h
    + `qubit`=0
*> quantum_h circuit=`self` qubit=`qubit`*
"#;
        let (n, c) = split_narrative_code(chunk);
        assert!((n as f64) / ((n + c) as f64) < 0.5, "n={n} c={c}");
    }
}

#[cfg(test)]
mod inline_measurement_tests {
    use super::*;
    #[test]
    fn inline_expressions_are_not_all_prose() {
        let (n, c) =
            split_narrative_code("Now **out = [Read the value](lookup(key=\"selected\"))**.");
        assert_eq!(
            n,
            "Now .".chars().count() + "Read the value".chars().count()
        );
        assert!(c > n);
    }
    #[test]
    fn bound_data_is_code_but_contracts_are_narration() {
        let table = "| field | value |\n|---|---|\n| example | item |";
        let (n, c) = split_narrative_code(table);
        assert!(n > 0 && c == 0);
        let (n, c) = split_narrative_code(&format!("`record` =\n\n{table}"));
        assert_eq!(n, 0);
        assert!(c > 0);
    }
}
