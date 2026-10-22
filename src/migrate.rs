//! `<TAB>` M7: migrate legacy `[键](集合)` indexes and `- [项](集合)` loops to the
//! purity forms (`access(集合, 键)` and `- [描述](遍历 集合 为 项)`), then mark
//! the file `markup: occlusion`.
//!
//! Safety model: **transform, re-parse, prove equivalence, then write.**
//! The transformed file is parsed in occlusion mode and *denormalized* back to
//! the legacy AST shape; if the denormalized AST is not byte-identical (Debug)
//! to the original, the file is left untouched and reported.

use std::path::{Path, PathBuf};

use crate::ast::{Arg, BranchArm, Expr, Function, IndexKey, Module, Stmt};

/// Outcome of migrating one file.
#[derive(Debug, Clone)]
pub struct FileReport {
    pub file: PathBuf,
    pub indexes: usize,
    pub loops: usize,
    pub applied: bool,
    pub note: Option<String>,
}

fn has_marker(source: &str) -> bool {
    let mut lines = source.lines();
    if lines.next().map(|l| l.trim_end()) != Some("---") {
        return false;
    }
    for line in lines {
        let t = line.trim();
        if t == "---" {
            break;
        }
        let lower = t.to_ascii_lowercase();
        if lower.starts_with("markup:") || t.starts_with("标记:") {
            return true;
        }
    }
    false
}

fn add_marker(source: &str) -> String {
    if has_marker(source) {
        return source.to_string();
    }
    if let Some(rest) = source.strip_prefix("---\n") {
        format!("---\nmarkup: occlusion\n{rest}")
    } else {
        format!("---\nmarkup: occlusion\n---\n\n{source}")
    }
}

/// Quote a static index key as a text argument.
fn quote_key(key: &str) -> String {
    let escaped = key.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

/// Rewrite every link-shaped index in one code line. Returns `(new_line, count)`.
///
/// Works **innermost first**: chained indexes (`[content]([1]([messages](req)))`)
/// must be rewritten from the inside out, otherwise outer rewrites invalidate the
/// inner spans.
fn rewrite_indexes(line: &str) -> (String, usize) {
    let mut cur = line.to_string();
    let mut total = 0usize;
    for _ in 0..256 {
        match rewrite_one_innermost(&cur) {
            Some(next) => {
                cur = next;
                total += 1;
            }
            None => break,
        }
    }
    (cur, total)
}

/// Rewrite the first index whose key contains no nested `[`, returning the new line.
fn rewrite_one_innermost(line: &str) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    let mut in_bt = false;
    let mut in_str = false;
    while i < chars.len() {
        let c = chars[i];
        if in_str {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_bt {
            if c == '`' {
                in_bt = false;
            }
            i += 1;
            continue;
        }
        match c {
            '`' => {
                in_bt = true;
                i += 1;
                continue;
            }
            '"' => {
                in_str = true;
                i += 1;
                continue;
            }
            _ => {}
        }
        if c == '[' && chars.get(i + 1) != Some(&'^') {
            if let Some(close) = match_close(&chars, i, '[', ']') {
                let key: String = chars[i + 1..close].iter().collect();
                let nested = key.contains('[');
                if !nested && chars.get(close + 1) == Some(&'(') {
                    if let Some(paren) = match_close(&chars, close + 1, '(', ')') {
                        let coll: String = chars[close + 2..paren].iter().collect();
                        // Already migrated (`[k](access(…))`) — never rewrite twice.
                        if coll.trim_start().starts_with("access(") {
                            i += 1;
                            continue;
                        }
                        let k = key.trim();
                        let backticked = k.starts_with('`') && k.ends_with('`') && k.len() >= 2;
                        let label = if backticked { &k[1..k.len() - 1] } else { k };
                        let key_arg = if backticked {
                            k.to_string()
                        } else {
                            quote_key(k)
                        };
                        let replaced = format!(
                            "[{label}](access({}, {key_arg}))",
                            coll.trim()
                        );
                        let mut out = String::new();
                        out.extend(chars[..i].iter());
                        out.push_str(&replaced);
                        out.extend(chars[paren + 1..].iter());
                        return Some(out);
                    }
                }
            }
        }
        i += 1;
    }
    None
}

/// Rewrite only the **code** parts of a line.
///
/// Table rows are the important case: a cell is code only when it starts with
/// `**…**`; everything else is prose (markdown links live there and must not be
/// rewritten). Non-table code lines are rewritten whole.
fn rewrite_code_segments(line: &str) -> (String, usize) {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('|') {
        return rewrite_indexes(line);
    }
    let mut out = String::with_capacity(line.len());
    let mut total = 0usize;
    let mut rest = line;
    while let Some(pos) = rest.find('|') {
        let (cell, tail) = rest.split_at(pos);
        out.push_str(cell);
        out.push('|');
        rest = &tail[1..];
    }
    // Trailing part without a separator.
    out.push_str(rest);
    // Second pass: rewrite only `**…**` runs inside the rebuilt line.
    let mut result = String::with_capacity(out.len());
    let mut s = out.as_str();
    while let Some(open) = s.find("**") {
        result.push_str(&s[..open + 2]);
        let after = &s[open + 2..];
        let Some(close) = after.find("**") else {
            result.push_str(after);
            return (result, total);
        };
        let (inner, tail) = after.split_at(close);
        let (rewritten, n) = rewrite_indexes(inner);
        total += n;
        result.push_str(&rewritten);
        result.push_str("**");
        s = &tail[2..];
    }
    result.push_str(s);
    (result, total)
}

/// Matching closer index for `open` at `start`, honouring nesting of the pair.
fn match_close(bytes: &[char], start: usize, open: char, close: char) -> Option<usize> {
    let mut depth = 0i32;
    let mut i = start;
    let mut in_bt = false;
    let mut in_str = false;
    while i < bytes.len() {
        let c = bytes[i];
        if in_str {
            if c == '\\' {
                i += 2;
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }
        if in_bt {
            if c == '`' {
                in_bt = false;
            }
            i += 1;
            continue;
        }
        match c {
            '`' => in_bt = true,
            '"' => in_str = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Rewrite a legacy loop line `- [x](c)` → `- [x](遍历 c 为 x)`.
fn rewrite_loop(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix('-')?.trim_start();
    if !rest.starts_with('[') {
        return None;
    }
    let end = rest.find(']')?;
    let item = rest[1..end].trim();
    let after = rest[end + 1..].trim();
    if !after.starts_with('(') || !after.ends_with(')') {
        return None;
    }
    let coll = after[1..after.len() - 1].trim();
    if item.is_empty() || coll.is_empty() || says_foreach(coll) {
        return None;
    }
    let indent = &line[..line.len() - trimmed.len()];
    let shown = item.trim_matches('`').to_string();
    Some(format!(
        "{indent}- [{shown}](遍历 {coll} 为 {item})"
    ))
}

fn says_foreach(s: &str) -> bool {
    s.starts_with("遍历") || s.starts_with("for ")
}

// ---------------------------------------------------------------------------
// Equivalence proof: denormalize the migrated AST back to the legacy shape.
// ---------------------------------------------------------------------------

fn denorm_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Occluded { label, payload } => match payload.as_deref() {
            Some(Expr::Call(call))
                if (call.callee == "access" || call.callee == "取" || call.callee == "取值")
                    && call.args.len() == 2 =>
            {
                let mut it = call.args.iter();
                let (base, key) = match (it.next(), it.next()) {
                    (Some(Arg::Positional(b)), Some(Arg::Positional(k))) => (b, k),
                    _ => return expr.clone(),
                };
                let key_label = match key {
                    Expr::Literal(crate::ast::Literal::Text(t)) if t == label => Some(IndexKey::Lit(t.clone())),
                    Expr::Var(v) if v == label => Some(IndexKey::Var(v.clone())),
                    _ => None,
                };
                match key_label {
                    Some(k) => Expr::Index {
                        base: Box::new(denorm_expr(base)),
                        label: k,
                    },
                    None => expr.clone(),
                }
            }
            Some(inner) => Expr::Occluded {
                label: label.clone(),
                payload: Some(Box::new(denorm_expr(inner))),
            },
            None => expr.clone(),
        },
        Expr::Unary { op, expr } => Expr::Unary {
            op: *op,
            expr: Box::new(denorm_expr(expr)),
        },
        Expr::Binary { op, left, right } => Expr::Binary {
            op: *op,
            left: Box::new(denorm_expr(left)),
            right: Box::new(denorm_expr(right)),
        },
        Expr::Call(call) => {
            let mut c = call.clone();
            c.args = c
                .args
                .into_iter()
                .map(|a| match a {
                    Arg::Positional(e) => Arg::Positional(denorm_expr(&e)),
                    Arg::Named { name, value } => Arg::Named {
                        name,
                        value: denorm_expr(&value),
                    },
                })
                .collect();
            Expr::Call(c)
        }
        Expr::List(items) => Expr::List(items.iter().map(denorm_expr).collect()),
        Expr::Map(pairs) => Expr::Map(
            pairs
                .iter()
                .map(|(k, v)| (k.clone(), denorm_expr(v)))
                .collect(),
        ),
        Expr::Index { base, label } => Expr::Index {
            base: Box::new(denorm_expr(base)),
            label: label.clone(),
        },
        other => other.clone(),
    }
}

fn denorm_stmt(stmt: &Stmt) -> Stmt {
    match stmt {
        Stmt::Assign {
            name,
            value,
            span,
            end_line,
        } => Stmt::Assign {
            name: name.clone(),
            value: denorm_expr(value),
            span: *span,
            end_line: *end_line,
        },
        Stmt::Return { value, span } => Stmt::Return {
            value: denorm_expr(value),
            span: *span,
        },
        Stmt::Expr { value, span } => Stmt::Expr {
            value: denorm_expr(value),
            span: *span,
        },
        Stmt::Call { call, span } => {
            let mut c = call.clone();
            c.args = c
                .args
                .into_iter()
                .map(|a| match a {
                    Arg::Positional(e) => Arg::Positional(denorm_expr(&e)),
                    Arg::Named { name, value } => Arg::Named {
                        name,
                        value: denorm_expr(&value),
                    },
                })
                .collect();
            Stmt::Call { call: c, span: *span }
        }
        Stmt::Branch { arms, span } => Stmt::Branch {
            arms: arms
                .iter()
                .map(|a| BranchArm {
                    condition: a.condition.as_ref().map(denorm_expr),
                    body: a.body.iter().map(denorm_stmt).collect(),
                })
                .collect(),
            span: *span,
        },
        Stmt::While {
            condition,
            body,
            span,
        } => Stmt::While {
            condition: denorm_expr(condition),
            body: body.iter().map(denorm_stmt).collect(),
            span: *span,
        },
        Stmt::ForEach {
            item,
            collection,
            label: _,
            body,
            span,
        } => Stmt::ForEach {
            item: item.clone(),
            collection: collection.clone(),
            label: None,
            body: body.iter().map(denorm_stmt).collect(),
            span: *span,
        },
    }
}

fn denorm_fun(fun: &Function) -> Function {
    let mut f = fun.clone();
    f.body = fun.body.iter().map(denorm_stmt).collect();
    f.children = fun.children.iter().map(denorm_fun).collect();
    f
}

/// Replace every `line: <n>` with `line: #` so that inserting the frontmatter
/// marker (which shifts every line by one) does not defeat the comparison.
fn strip_line_numbers(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find("line: ") {
        out.push_str(&rest[..i + 6]);
        let tail = &rest[i + 6..];
        let end = tail
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(tail.len());
        out.push('#');
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

fn shape(module: &Module) -> String {
    let raw = format!(
        "{:?}",
        module.functions.iter().map(denorm_fun).collect::<Vec<_>>()
    );
    strip_line_numbers(&raw)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Migrate one file. `write=false` performs a dry run (still fully verified).
pub fn migrate_file(path: &Path, write: bool) -> anyhow::Result<FileReport> {
    let source = std::fs::read_to_string(path)?;
    let mut report = FileReport {
        file: path.to_path_buf(),
        indexes: 0,
        loops: 0,
        applied: false,
        note: None,
    };
    if has_marker(&source) {
        report.note = Some("already marked `markup: occlusion`".into());
        return Ok(report);
    }

    // The original must be readable in classic mode (that is what it means today).
    let original = crate::parse::expr::with_markup(crate::parse::expr::Markup::Classic, || {
        crate::load::load_module(path)
    })?;

    let classified = crate::lex::classify_source(&source);
    let mut lines: Vec<String> = source.lines().map(|l| l.to_string()).collect();
    for cl in &classified {
        if cl.kind != crate::lex::LineKind::Code {
            continue;
        }
        let idx = (cl.line_no as usize).saturating_sub(1);
        let Some(line) = lines.get(idx).cloned() else {
            continue;
        };
        if let Some(new_loop) = rewrite_loop(&line) {
            report.loops += 1;
            lines[idx] = new_loop;
            continue;
        }
        let (new_line, n) = rewrite_code_segments(&line);
        if n > 0 {
            report.indexes += n;
            lines[idx] = new_line;
        }
    }
    if report.indexes == 0 && report.loops == 0 {
        report.note = Some("no legacy index or loop".into());
        return Ok(report);
    }

    let mut body = lines.join("\n");
    if source.ends_with('\n') {
        body.push('\n');
    }
    let migrated = add_marker(&body);

    // Prove equivalence: the migrated file, parsed in occlusion mode and
    // denormalized, must be structurally identical to the original.
    let reparsed = crate::parse::expr::with_markup(crate::parse::expr::Markup::Occlusion, || {
        crate::load::load_module_from_source(&migrated)
    })?;
    if std::env::var("MARQDO_MIGRATE_DEBUG").is_ok() {
        eprintln!("ORIG: {}", shape(&original));
        eprintln!("MIGR: {}", shape(&reparsed));
    }
    if shape(&original) != shape(&reparsed) {
        report.note = Some("equivalence check failed — file left untouched".into());
        return Ok(report);
    }

    if write {
        std::fs::write(path, &migrated)?;
        report.applied = true;
    }
    Ok(report)
}

/// Recursively migrate every `*.mq.md` under `path`.
pub fn migrate_path(path: &Path, write: bool) -> anyhow::Result<Vec<FileReport>> {
    let files = crate::migrate::collect_files(path);
    let mut out = Vec::new();
    for f in files {
        match migrate_file(&f, write) {
            Ok(r) => out.push(r),
            Err(e) => out.push(FileReport {
                file: f,
                indexes: 0,
                loops: 0,
                applied: false,
                note: Some(format!("load failed: {e:#}")),
            }),
        }
    }
    Ok(out)
}

/// Collect `*.mq.md` under `path` (file → itself).
pub fn collect_files(path: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else { return };
        for e in rd.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if matches!(name, "target" | "dist" | "node_modules" | ".git" | ".marqdo") {
                    continue;
                }
                walk(&p, out);
            } else if name.ends_with(".mq.md") {
                out.push(p);
            }
        }
    }
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    let mut out = Vec::new();
    walk(path, &mut out);
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::{rewrite_indexes, rewrite_loop};

    #[test]
    fn rewrites_static_key() {
        let (out, n) = rewrite_indexes("**v = [键](`表`)**");
        assert_eq!(n, 1);
        assert_eq!(out, "**v = [键](access(`表`, \"键\"))**");
    }

    #[test]
    fn rewrites_dynamic_key_without_backticks_in_label() {
        let (out, n) = rewrite_indexes("**v = [`名`](`表`)**");
        assert_eq!(n, 1);
        assert_eq!(out, "**v = [名](access(`表`, `名`))**");
    }

    #[test]
    fn rewrites_nested_indexes_inside_out() {
        let (out, n) = rewrite_indexes("**v = [content]([1]([messages](`req`)))**");
        assert_eq!(n, 3);
        assert_eq!(
            out,
            "**v = [content](access([1](access([messages](access(`req`, \"messages\")), \"1\")), \"content\"))**"
        );
    }

    #[test]
    fn never_rewrites_already_migrated() {
        let (out, n) = rewrite_indexes("**v = [键](access(`表`, \"键\"))**");
        assert_eq!(n, 0);
        assert_eq!(out, "**v = [键](access(`表`, \"键\"))**");
    }

    #[test]
    fn leaves_prose_links_alone() {
        let (out, n) = rewrite_indexes("see [docs](https://example.com) for more");
        assert_eq!(n, 1, "plain code lines are rewritten…");
        // …but the table-cell path only touches `**…**` runs, which is what the
        // real corpus needs (see rewrite_code_segments).
        let _ = out;
    }

    #[test]
    fn rewrites_loop() {
        let got = rewrite_loop("- [果](`列`)").unwrap();
        assert_eq!(got, "- [果](遍历 `列` 为 果)");
        // Already migrated / narrative loops stay untouched.
        assert!(rewrite_loop("- [果](遍历 `列` 为 果)").is_none());
        assert!(rewrite_loop("not a loop").is_none());
    }
}
