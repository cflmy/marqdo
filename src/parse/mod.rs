//! Line-oriented recursive descent → AST (no Bison).

mod expr;

pub use expr::{parse_call_after_gt, parse_expr, parse_value_or_interp};

use anyhow::{bail, Result};

use crate::ast::{
    BranchArm, Expr, Function, Literal, Module, Stmt,
};
use crate::diagnostics::Span;
use crate::lex::{classify_source, ClassifiedLine, LineKind};
use crate::parse::expr::parse_call_after_gt as call_after_gt;
use crate::parse::expr::parse_expr as parse_expr_str;

/// Parse full source (with optional frontmatter) into a module AST.
pub fn parse_source(source: &str) -> Result<Module> {
    let lines = classify_source(source);
    parse_classified(&lines)
}

pub fn parse_classified(lines: &[ClassifiedLine]) -> Result<Module> {
    let mut cur = Cursor::new(lines);
    let imports = cur.parse_frontmatter()?;
    let mut functions = Vec::new();
    while cur.skip_noise() {
        let Some(line) = cur.peek() else { break };
        let trimmed = line.text.trim();
        if is_heading(trimmed) {
            functions.push(cur.parse_function(1)?);
        } else {
            bail!(
                "{}:1: expected function heading `# …`, got: {trimmed}",
                line.line_no
            );
        }
    }
    Ok(Module { imports, functions })
}

struct Cursor<'a> {
    lines: &'a [ClassifiedLine],
    i: usize,
}

impl<'a> Cursor<'a> {
    fn new(lines: &'a [ClassifiedLine]) -> Self {
        Self { lines, i: 0 }
    }

    fn peek(&self) -> Option<&'a ClassifiedLine> {
        self.lines.get(self.i)
    }

    fn bump(&mut self) -> Option<&'a ClassifiedLine> {
        let l = self.lines.get(self.i)?;
        self.i += 1;
        Some(l)
    }

    /// Skip blank + comment; return true if a line remains.
    fn skip_noise(&mut self) -> bool {
        while let Some(l) = self.peek() {
            match l.kind {
                LineKind::Blank | LineKind::Comment => {
                    self.i += 1;
                }
                LineKind::Code => return true,
            }
        }
        false
    }

    fn parse_frontmatter(&mut self) -> Result<Vec<String>> {
        let mut imports = Vec::new();
        if !self.skip_noise() {
            return Ok(imports);
        }
        let Some(first) = self.peek() else {
            return Ok(imports);
        };
        if first.text.trim() != "---" {
            return Ok(imports);
        }
        self.bump();
        while let Some(line) = self.peek() {
            let t = line.text.trim();
            if t == "---" {
                self.bump();
                break;
            }
            if t.starts_with('>') {
                let path = t[1..].trim();
                if path.ends_with(".mq.md") {
                    imports.push(path.to_string());
                }
            }
            self.bump();
        }
        Ok(imports)
    }

    fn parse_function(&mut self, min_level: u8) -> Result<Function> {
        let line = self.bump().unwrap();
        let (level, name) = parse_heading(line.text.trim())
            .ok_or_else(|| anyhow::anyhow!("{}:1: expected heading", line.line_no))?;
        if level < min_level {
            bail!("{}:1: heading level {level} < expected {min_level}", line.line_no);
        }

        let mut fun = Function {
            name: name.to_string(),
            level,
            span: Span {
                line: line.line_no,
                col: 1,
            },
            params: Vec::new(),
            body: Vec::new(),
            children: Vec::new(),
        };

        // Parameter zone: indented `- name` pure identifiers.
        while self.skip_noise() {
            let Some(l) = self.peek() else { break };
            let trimmed = l.text.trim();
            if is_heading(trimmed) {
                let (nl, _) = parse_heading(trimmed).unwrap();
                if nl > level {
                    fun.children.push(self.parse_function(level + 1)?);
                    continue;
                }
                break;
            }
            if let Some(param) = parse_param_line(trimmed) {
                // Only accept as param before any body stmt (params first).
                if fun.body.is_empty() {
                    fun.params.push(param);
                    self.bump();
                    continue;
                }
            }
            break;
        }

        // Body until next heading at level <= ours.
        while self.skip_noise() {
            let Some(l) = self.peek() else { break };
            let trimmed = l.text.trim();

            if is_frame(trimmed) {
                self.bump();
                continue;
            }

            if is_heading(trimmed) {
                let (nl, _) = parse_heading(trimmed).unwrap();
                if nl > level {
                    fun.children.push(self.parse_function(level + 1)?);
                    continue;
                }
                break;
            }

            // Nested params mid-body shouldn't happen; treat `-` as control flow.
            if trimmed.starts_with('+') || is_ordered_branch(trimmed) {
                fun.body.push(self.parse_branch()?);
                continue;
            }
            if trimmed.starts_with('-') {
                fun.body.push(self.parse_loop_or_err()?);
                continue;
            }

            fun.body.push(self.parse_simple_stmt()?);
        }

        Ok(fun)
    }

    fn parse_simple_stmt(&mut self) -> Result<Stmt> {
        let line = self.bump().unwrap();
        let span = Span {
            line: line.line_no,
            col: 1,
        };
        let trimmed = line.text.trim();

        if is_frame(trimmed) {
            bail!("{span}: unexpected frame line as statement");
        }

        // Bold return **…**
        if trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() >= 4 {
            let inner = &trimmed[2..trimmed.len() - 2];
            let value = parse_expr_str(inner)?;
            return Ok(Stmt::Return { value, span });
        }

        // Italic statement *…*
        if trimmed.starts_with('*') && trimmed.ends_with('*') && !trimmed.starts_with("**") {
            let inner = &trimmed[1..trimmed.len() - 1];
            return parse_assign_or_expr_stmt(inner, span);
        }

        // Call >
        if trimmed.starts_with('>') {
            let call = call_after_gt(trimmed[1..].trim())?;
            return Ok(Stmt::Call { call, span });
        }

        // Bare assign `` `x` = … `` possibly followed by table
        if trimmed.starts_with('`') {
            if let Some(stmt) = self.parse_backtick_assign(trimmed, span.clone())? {
                return Ok(stmt);
            }
        }

        bail!("{span}: unrecognized statement: {trimmed}")
    }

    fn parse_backtick_assign(&mut self, trimmed: &str, span: Span) -> Result<Option<Stmt>> {
        // `name` = [rhs]
        let Some(rest) = trimmed.strip_prefix('`') else {
            return Ok(None);
        };
        let Some(end) = rest.find('`') else {
            return Ok(None);
        };
        let name = rest[..end].to_string();
        let after = rest[end + 1..].trim_start();
        if !after.starts_with('=') {
            return Ok(None);
        }
        let rhs = after[1..].trim();
        if rhs.is_empty() {
            // Consume following table if present.
            let list = self.consume_table()?;
            return Ok(Some(Stmt::Assign {
                name,
                value: Expr::List(list.into_iter().map(Expr::Literal).collect()),
                span,
            }));
        }
        Ok(Some(Stmt::Assign {
            name,
            value: parse_value_or_interp(rhs)?,
            span,
        }))
    }

    fn consume_table(&mut self) -> Result<Vec<Literal>> {
        self.skip_noise();
        let mut rows = Vec::new();
        let mut header_done = false;
        let mut sep_done = false;
        while let Some(l) = self.peek() {
            if l.kind != LineKind::Code {
                break;
            }
            let t = l.text.trim();
            if !t.starts_with('|') {
                break;
            }
            self.bump();
            let cells = split_table_row(t);
            if !header_done {
                header_done = true;
                continue;
            }
            if !sep_done && cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch.is_whitespace() || ch == '|'))
            {
                // separator row like |----|
                sep_done = true;
                continue;
            }
            // data row — single column examples: take first cell
            if let Some(cell) = cells.first() {
                rows.push(Literal::Text(cell.clone()));
            }
            sep_done = true;
        }
        if !header_done {
            bail!("expected GFM table after empty assignment");
        }
        Ok(rows)
    }

    fn parse_branch(&mut self) -> Result<Stmt> {
        let first = self.peek().unwrap();
        let span = Span {
            line: first.line_no,
            col: 1,
        };
        let base_indent = indent_of(&first.text);
        let mut arms = Vec::new();

        while self.skip_noise() {
            let Some(l) = self.peek() else { break };
            let ind = indent_of(&l.text);
            if ind < base_indent {
                break;
            }
            let trimmed = l.text.trim();
            let cond_src = if let Some(rest) = trimmed.strip_prefix('+') {
                rest.trim()
            } else if let Some(rest) = strip_ordered(trimmed) {
                rest
            } else {
                break;
            };
            if ind != base_indent {
                break;
            }
            self.bump();

            let condition = if cond_src == "*" || cond_src.is_empty() {
                None
            } else {
                Some(parse_expr_str(cond_src)?)
            };

            let body = self.parse_indented_body(base_indent)?;
            arms.push(BranchArm { condition, body });
        }

        if arms.is_empty() {
            bail!("{span}: empty branch");
        }
        Ok(Stmt::Branch { arms, span })
    }

    fn parse_loop_or_err(&mut self) -> Result<Stmt> {
        let line = self.bump().unwrap();
        let span = Span {
            line: line.line_no,
            col: 1,
        };
        let base_indent = indent_of(&line.text);
        let trimmed = line.text.trim();
        let rest = trimmed.strip_prefix('-').unwrap().trim();

        // foreach: [item](collection)
        if let Some((item, coll)) = parse_foreach_header(rest) {
            let body = self.parse_indented_body(base_indent)?;
            return Ok(Stmt::ForEach {
                item,
                collection: coll,
                body,
                span,
            });
        }

        // while: expression
        let condition = parse_expr_str(rest)?;
        let body = self.parse_indented_body(base_indent)?;
        Ok(Stmt::While {
            condition,
            body,
            span,
        })
    }

    fn parse_indented_body(&mut self, parent_indent: usize) -> Result<Vec<Stmt>> {
        let mut body = Vec::new();
        while self.skip_noise() {
            let Some(l) = self.peek() else { break };
            let ind = indent_of(&l.text);
            if ind <= parent_indent {
                // Same-indent `+`/`-` ends body (next arm / sibling).
                break;
            }
            let trimmed = l.text.trim();
            if is_frame(trimmed) {
                self.bump();
                continue;
            }
            if is_heading(trimmed) {
                break;
            }
            if trimmed.starts_with('+') || is_ordered_branch(trimmed) {
                // Nested branch inside body
                body.push(self.parse_branch()?);
                continue;
            }
            if trimmed.starts_with('-') {
                body.push(self.parse_loop_or_err()?);
                continue;
            }
            body.push(self.parse_simple_stmt()?);
        }
        Ok(body)
    }
}

fn parse_assign_or_expr_stmt(inner: &str, span: Span) -> Result<Stmt> {
    let inner = inner.trim();
    // `` `name` = … ``
    if let Some(rest) = inner.strip_prefix('`') {
        if let Some(end) = rest.find('`') {
            let name = rest[..end].to_string();
            let after = rest[end + 1..].trim_start();
            if let Some(rhs) = after.strip_prefix('=') {
                let rhs = rhs.trim();
                // Inline call assign: `> fn …`
                let value = if rhs.starts_with('>') {
                    Expr::Call(call_after_gt(rhs[1..].trim())?)
                } else {
                    parse_value_or_interp(rhs)?
                };
                return Ok(Stmt::Assign {
                    name,
                    value,
                    span,
                });
            }
        }
    }
    bail!("{span}: italic statement must be an assignment (`name` = …)")
}

fn parse_param_line(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix('-')?.trim();
    if rest.is_empty() || rest.contains('`') || rest.contains('[') || rest.contains('=') {
        return None;
    }
    if rest.split_whitespace().count() != 1 {
        return None;
    }
    // Reject if looks like comparison / operator expr
    if rest.contains('>') || rest.contains('<') || rest.contains('+') || rest.contains('*') {
        return None;
    }
    Some(rest.to_string())
}

fn parse_foreach_header(rest: &str) -> Option<(String, String)> {
    // [item](collection)
    let rest = rest.trim();
    if !rest.starts_with('[') {
        return None;
    }
    let end_item = rest.find(']')?;
    let item = rest[1..end_item].to_string();
    let after = rest[end_item + 1..].trim_start();
    if !after.starts_with('(') || !after.ends_with(')') {
        return None;
    }
    let coll = after[1..after.len() - 1].to_string();
    Some((item, coll))
}

fn parse_heading(trimmed: &str) -> Option<(u8, &str)> {
    if !trimmed.starts_with('#') {
        return None;
    }
    let mut level = 0u8;
    for c in trimmed.chars() {
        if c == '#' {
            level = level.saturating_add(1);
        } else {
            break;
        }
    }
    if level == 0 || level > 6 {
        return None;
    }
    let name = trimmed[level as usize..].trim();
    if name.is_empty() {
        return None;
    }
    Some((level, name))
}

fn is_heading(trimmed: &str) -> bool {
    parse_heading(trimmed).is_some()
}

fn is_frame(trimmed: &str) -> bool {
    if trimmed.len() < 3 {
        return false;
    }
    trimmed.chars().all(|c| c == '-') || trimmed.chars().all(|c| c == '*')
}

fn is_ordered_branch(trimmed: &str) -> bool {
    strip_ordered(trimmed).is_some()
}

fn strip_ordered(trimmed: &str) -> Option<&str> {
    let mut chars = trimmed.chars();
    let mut saw_digit = false;
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            saw_digit = true;
            continue;
        }
        if c == '.' && saw_digit {
            return Some(chars.as_str().trim_start());
        }
        return None;
    }
    None
}

fn indent_of(text: &str) -> usize {
    text.chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .map(|c| if c == '\t' { 4 } else { 1 })
        .sum()
}

fn split_table_row(line: &str) -> Vec<String> {
    let t = line.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').unwrap_or(t);
    t.split('|').map(|c| c.trim().to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Stmt;

    #[test]
    fn parse_hello() {
        let src = include_str!("../../examples/hello.mq.md");
        let m = parse_source(src).unwrap();
        assert_eq!(m.functions.len(), 1);
        assert_eq!(m.functions[0].name, "Hello World");
        assert_eq!(m.functions[0].body.len(), 1);
        assert!(matches!(m.functions[0].body[0], Stmt::Call { .. }));
    }

    #[test]
    fn parse_index() {
        let src = include_str!("../../examples/index.mq.md");
        let m = parse_source(src).unwrap();
        assert!(m.imports.is_empty());
        let main = m.functions.iter().find(|f| f.name == "main").unwrap();
        assert_eq!(main.children.len(), 1);
        assert_eq!(main.children[0].name, "问候");
        assert_eq!(main.children[0].params, vec!["谁".to_string()]);
    }

    #[test]
    fn parse_branch() {
        let src = include_str!("../../examples/branch.mq.md");
        let m = parse_source(src).unwrap();
        let main = &m.functions[0];
        assert!(main.body.iter().any(|s| matches!(s, Stmt::Branch { .. })));
    }
}
