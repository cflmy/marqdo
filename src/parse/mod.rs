//! Line-oriented recursive descent → AST (no Bison).

mod expr;

pub use expr::{parse_call_after_gt, parse_expr, parse_value_or_interp};

use anyhow::{bail, Result};

use crate::ast::{
    BranchArm, Expr, Function, Literal, Module, Stmt,
};
use crate::diagnostics::{Diagnostic, Span};
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

        // Body until next heading at level <= ours, or explicit end (--- / *** / empty **).
        while self.skip_noise() {
            let Some(l) = self.peek() else { break };
            let trimmed = l.text.trim();

            // Empty bold return ends the function body.
            if is_empty_bold_return(trimmed) {
                let span = Span {
                    line: l.line_no,
                    col: 1,
                };
                self.bump();
                fun.body.push(Stmt::Return {
                    value: Expr::Literal(Literal::None),
                    span,
                });
                break;
            }

            // --- / *** at function body top level ends the function (no return).
            if is_frame(trimmed) {
                self.bump();
                break;
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

        if trimmed.starts_with("$$") {
            bail!(
                "{span}: formula fence must follow an empty assignment (`name` =); \
                 `$$:name` binding was removed"
            );
        }

        // Bold return **…** (empty inner → None)
        if trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() >= 4 {
            let inner = &trimmed[2..trimmed.len() - 2];
            let value = if inner.trim().is_empty() {
                Expr::Literal(Literal::None)
            } else {
                parse_expr_str(inner)?
            };
            return Ok(Stmt::Return { value, span });
        }

        // Italic statement *…*
        if trimmed.starts_with('*') && trimmed.ends_with('*') && !trimmed.starts_with("**") {
            let inner = &trimmed[1..trimmed.len() - 1];
            return self.parse_italic_assign(inner, span);
        }

        // Call >
        if trimmed.starts_with('>') {
            let call = call_after_gt(trimmed[1..].trim())?;
            return Ok(Stmt::Call { call, span });
        }

        // Bare assign `` `x` = … `` possibly followed by formula fence or table
        if trimmed.starts_with('`') {
            if let Some(stmt) = self.parse_backtick_assign(trimmed, span.clone())? {
                return Ok(stmt);
            }
        }

        Err(Diagnostic::new(None, span, format!("unrecognized statement: {trimmed}")).into())
    }

    fn parse_italic_assign(&mut self, inner: &str, span: Span) -> Result<Stmt> {
        let Some((name, rhs)) = split_assign_inner(inner) else {
            bail!("{span}: italic statement must be an assignment (`name` = …)");
        };
        if crate::aliases::is_reserved_keyword(&name) {
            bail!("{span}: `{name}` is a reserved keyword");
        }
        let (value, end_line) = self.resolve_assign_rhs(&rhs, span)?;
        Ok(Stmt::Assign {
            name,
            value,
            span,
            end_line,
        })
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
        if crate::aliases::is_reserved_keyword(&name) {
            bail!("{span}: `{name}` is a reserved keyword");
        }
        let after = rest[end + 1..].trim_start();
        if !after.starts_with('=') {
            return Ok(None);
        }
        let rhs = after[1..].trim().to_string();
        let (value, end_line) = self.resolve_assign_rhs(&rhs, span)?;
        Ok(Some(Stmt::Assign {
            name,
            value,
            span,
            end_line,
        }))
    }

    /// RHS on the same line, or empty RHS → following `$$` fence or table.
    /// Returns `(expr, inclusive end_line)`.
    fn resolve_assign_rhs(&mut self, rhs: &str, span: Span) -> Result<(Expr, u32)> {
        let rhs = rhs.trim();
        if rhs.is_empty() {
            self.skip_blanks();
            if self.peek_is_formula_fence() {
                return self.consume_formula_fence(span);
            }
            let start_i = self.i;
            let list = self.consume_table()?;
            let end_line = if self.i > start_i {
                self.lines[self.i - 1].line_no
            } else {
                span.line
            };
            return Ok((
                Expr::List(list.into_iter().map(Expr::Literal).collect()),
                end_line,
            ));
        }
        if let Some(expr) = try_parse_inline_formula(rhs)? {
            return Ok((expr, span.line));
        }
        if rhs.starts_with('>') {
            return Ok((Expr::Call(call_after_gt(rhs[1..].trim())?), span.line));
        }
        Ok((parse_value_or_interp(rhs)?, span.line))
    }

    fn skip_blanks(&mut self) {
        while let Some(l) = self.peek() {
            if l.kind == LineKind::Blank {
                self.i += 1;
            } else {
                break;
            }
        }
    }

    fn peek_is_formula_fence(&self) -> bool {
        self.peek()
            .map(|l| l.text.trim().starts_with("$$"))
            .unwrap_or(false)
    }

    fn consume_formula_fence(&mut self, span: Span) -> Result<(Expr, u32)> {
        let open = self
            .bump()
            .ok_or_else(|| anyhow::anyhow!("{span}: expected `$$` formula fence"))?;
        let open_line = open.line_no;
        let trimmed = open.text.trim();
        if trimmed.starts_with("$$:") || trimmed.starts_with("$$ name=") {
            bail!(
                "{open_line}:1: `$$:name` / `$$ name=` binding was removed; \
                 use `name` = then a `$$…$$` fence"
            );
        }
        // Single-line: $$body$$
        if trimmed.starts_with("$$") && trimmed.ends_with("$$") && trimmed.len() > 4 {
            let body = trimmed[2..trimmed.len() - 2].trim();
            if body.is_empty() {
                bail!("{open_line}:1: empty formula");
            }
            let expr = crate::formula::parse(body)
                .map_err(|e| anyhow::anyhow!("{open_line}:1: formula: {e}"))?;
            return Ok((Expr::Formula(expr), open_line));
        }
        if trimmed != "$$" {
            bail!(
                "{open_line}:1: expected `$$` fence opener after empty assignment, got: {trimmed}"
            );
        }
        let mut body_lines: Vec<String> = Vec::new();
        let mut closed = false;
        let mut end_line = open_line;
        while let Some(l) = self.peek() {
            if l.text.trim() == "$$" {
                end_line = l.line_no;
                self.bump();
                closed = true;
                break;
            }
            body_lines.push(l.text.clone());
            self.bump();
        }
        if !closed {
            bail!("{open_line}:1: unclosed formula fence `$$`");
        }
        let text = body_lines.join("\n").trim().to_string();
        if text.is_empty() {
            bail!("{open_line}:1: empty formula");
        }
        let expr = crate::formula::parse(&text)
            .map_err(|e| anyhow::anyhow!("{open_line}:1: formula: {e}"))?;
        Ok((Expr::Formula(expr), end_line))
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

fn split_assign_inner(inner: &str) -> Option<(String, String)> {
    let inner = inner.trim();
    let rest = inner.strip_prefix('`')?;
    let end = rest.find('`')?;
    let name = rest[..end].to_string();
    let after = rest[end + 1..].trim_start();
    let rhs = after.strip_prefix('=')?.trim().to_string();
    Some((name, rhs))
}

/// Same-line `$$body$$` → Formula; otherwise None (caller parses normally).
fn try_parse_inline_formula(rhs: &str) -> Result<Option<Expr>> {
    let rhs = rhs.trim();
    if !rhs.starts_with("$$") {
        return Ok(None);
    }
    if rhs.starts_with("$$:") || rhs.starts_with("$$ name=") {
        bail!(
            "`$$:name` / `$$ name=` binding was removed; use `name` = then a `$$…$$` fence"
        );
    }
    if rhs.ends_with("$$") && rhs.len() > 4 {
        let body = rhs[2..rhs.len() - 2].trim();
        if body.is_empty() {
            bail!("empty formula");
        }
        let expr = crate::formula::parse(body).map_err(|e| anyhow::anyhow!("formula: {e}"))?;
        return Ok(Some(Expr::Formula(expr)));
    }
    bail!("incomplete inline formula (expected `$$…$$` on one line)");
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
    // Empty bold return `****` is handled separately; frames are --- or *** (etc.).
    if is_empty_bold_return(trimmed) {
        return false;
    }
    trimmed.chars().all(|c| c == '-') || trimmed.chars().all(|c| c == '*')
}

/// `****` or `**` + whitespace only + `**`.
fn is_empty_bold_return(trimmed: &str) -> bool {
    if !(trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() >= 4) {
        return false;
    }
    trimmed[2..trimmed.len() - 2].trim().is_empty()
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
        let src = include_str!("../../tests/structure/hello.mq.md");
        let m = parse_source(src).unwrap();
        assert_eq!(m.functions.len(), 1);
        assert_eq!(m.functions[0].name, "Hello World");
        assert_eq!(m.functions[0].body.len(), 1);
        assert!(matches!(m.functions[0].body[0], Stmt::Call { .. }));
    }

    #[test]
    fn parse_index() {
        let src = include_str!("../../tests/structure/nested-call.mq.md");
        let m = parse_source(src).unwrap();
        assert!(m.imports.is_empty());
        let main = m.functions.iter().find(|f| f.name == "main").unwrap();
        assert_eq!(main.children.len(), 1);
        assert_eq!(main.children[0].name, "问候");
        assert_eq!(main.children[0].params, vec!["谁".to_string()]);
    }

    #[test]
    fn parse_branch() {
        let src = include_str!("../../tests/structure/branch.mq.md");
        let m = parse_source(src).unwrap();
        let main = &m.functions[0];
        assert!(main.body.iter().any(|s| matches!(s, Stmt::Branch { .. })));
    }

    #[test]
    fn parse_formula_assign_fence() {
        let src = "# main\n\n`f` =\n$$\nx^2 - 2\n$$\n\n> print text=`f`\n";
        let m = parse_source(src).unwrap();
        let body = &m.functions[0].body;
        match &body[0] {
            Stmt::Assign {
                name,
                value: Expr::Formula(e),
                end_line,
                ..
            } => {
                assert_eq!(name, "f");
                assert_eq!(e.as_display(), "x^2 - 2");
                assert!(*end_line > 3, "end_line should cover the fence, got {end_line}");
            }
            other => panic!("expected formula assign, got {other:?}"),
        }
    }
}
