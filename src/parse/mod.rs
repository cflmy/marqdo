//! Line-oriented recursive descent → AST (no Bison).

mod expr;

pub use expr::{
    parse_call_after_gt, parse_call_arg_value, parse_expr, parse_expr_prefer_var,
    parse_value_or_interp,
};

use anyhow::{bail, Result};

use crate::ast::{
    BranchArm, Expr, Function, Import, Literal, Module, Stmt, Use,
};
use crate::diagnostics::{bail_at, Diagnostic, Span};
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
    let (imports, uses) = cur.parse_frontmatter()?;
    let mut functions = Vec::new();
    while cur.skip_noise() {
        let Some(line) = cur.peek() else { break };
        let trimmed = line.text.trim();
        if is_heading(trimmed) {
            functions.push(cur.parse_function(1)?);
        } else {
            bail!(
                "{}:1: expected heading (`#` object or `##` function), got: {trimmed}",
                line.line_no
            );
        }
    }
    for u in &uses {
        if imports.iter().any(|i| i.bind == u.bind) {
            bail!("import bind `{}` conflicts with file import library name", u.bind);
        }
        if functions.iter().any(|f| f.name == u.bind) {
            bail!("import bind `{}` conflicts with top-level name", u.bind);
        }
    }
    Ok(Module {
        imports,
        uses,
        functions,
        import_modules: std::collections::HashMap::new(),
    })
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
                LineKind::Blank | LineKind::Comment | LineKind::Writeback => {
                    self.i += 1;
                }
                LineKind::Code => return true,
            }
        }
        false
    }

    fn parse_frontmatter(&mut self) -> Result<(Vec<Import>, Vec<Use>)> {
        let mut imports = Vec::new();
        let mut uses = Vec::new();
        if !self.skip_noise() {
            return Ok((imports, uses));
        }
        let Some(first) = self.peek() else {
            return Ok((imports, uses));
        };
        if first.text.trim() != "---" {
            return Ok((imports, uses));
        }
        self.bump();
        while let Some(line) = self.peek() {
            let t = line.text.trim();
            if t == "---" {
                self.bump();
                break;
            }
            let span = Span::new(line.line_no, 1);
            if let Some(msg) = legacy_frontmatter_import_hint(t) {
                return Err(bail_at(None, span, msg));
            }
            match parse_import_line(t) {
                Ok(Some(ImportLine::File(imp))) => {
                    if imports.iter().any(|i| i.bind == imp.bind)
                        || uses.iter().any(|u| u.bind == imp.bind)
                    {
                        return Err(bail_at(
                            None,
                            span,
                            format!("duplicate import bind `{}`", imp.bind),
                        ));
                    }
                    imports.push(imp);
                }
                Ok(Some(ImportLine::Member(u))) => {
                    if imports.iter().any(|i| i.bind == u.bind)
                        || uses.iter().any(|x| x.bind == u.bind)
                    {
                        return Err(bail_at(
                            None,
                            span,
                            format!("duplicate import bind `{}`", u.bind),
                        ));
                    }
                    uses.push(u);
                }
                Ok(None) => {}
                Err(e) => return Err(bail_at(None, span, e.to_string())),
            }
            self.bump();
        }
        Ok((imports, uses))
    }

    fn parse_function(&mut self, min_level: u8) -> Result<Function> {
        let line = self.bump().unwrap();
        let (level, raw_name) = parse_heading(line.text.trim())
            .ok_or_else(|| anyhow::anyhow!("{}:1: expected heading", line.line_no))?;
        if level < min_level {
            bail!("{}:1: heading level {level} < expected {min_level}", line.line_no);
        }
        let span = Span {
            line: line.line_no,
            col: 1,
        };
        let (name, base) = parse_object_decl(level, raw_name, span)?;

        let mut fun = Function {
            name,
            level,
            span,
            params: Vec::new(),
            body: Vec::new(),
            children: Vec::new(),
            base,
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
            if is_ordered_branch(trimmed) {
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
                parse_expr_prefer_var(inner)?
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
            bail!("{span}: italic statement must be an assignment (name = …)");
        };
        if crate::aliases::is_reserved_keyword(&name) {
            bail!("{span}: `{name}` is a reserved keyword");
        }
        let (value, end_line) = self.resolve_assign_rhs(&rhs, span, true)?;
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
        let (value, end_line) = self.resolve_assign_rhs(&rhs, span, false)?;
        Ok(Some(Stmt::Assign {
            name,
            value,
            span,
            end_line,
        }))
    }

    /// RHS on the same line, or empty RHS → following `$$` / ```lang fence or table.
    /// Returns `(expr, inclusive end_line)`.
    /// `prefer_var`: italic RHS uses variable-preferring expression syntax.
    fn resolve_assign_rhs(
        &mut self,
        rhs: &str,
        span: Span,
        prefer_var: bool,
    ) -> Result<(Expr, u32)> {
        let rhs = rhs.trim();
        if rhs.is_empty() {
            self.skip_blanks();
            if self.peek_is_formula_fence() {
                return self.consume_formula_fence(span);
            }
            if self.peek_is_code_fence() {
                return self.consume_code_fence(span);
            }
            let start_i = self.i;
            let table = self.consume_table()?;
            let end_line = if self.i > start_i {
                self.lines[self.i - 1].line_no
            } else {
                span.line
            };
            return Ok((table, end_line));
        }
        if let Some(expr) = try_parse_inline_formula(rhs)? {
            return Ok((expr, span.line));
        }
        if rhs.starts_with('>') {
            return Ok((Expr::Call(call_after_gt(rhs[1..].trim())?), span.line));
        }
        let value = if prefer_var {
            parse_expr_prefer_var(rhs)?
        } else {
            parse_value_or_interp(rhs)?
        };
        Ok((value, span.line))
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

    fn peek_is_code_fence(&self) -> bool {
        self.peek()
            .map(|l| crate::foreign::is_fence_opener(l.text.trim()))
            .unwrap_or(false)
    }

    fn consume_code_fence(&mut self, span: Span) -> Result<(Expr, u32)> {
        let open = self
            .bump()
            .ok_or_else(|| anyhow::anyhow!("{span}: expected ```lang code fence"))?;
        let open_line = open.line_no;
        let trimmed = open.text.trim();
        let lang = crate::foreign::fence_lang(trimmed).ok_or_else(|| {
            anyhow::anyhow!("{open_line}:1: expected ```lang fence after empty assignment")
        })?;
        if trimmed.contains("name=") {
            bail!(
                "{open_line}:1: `name=` on fences was removed; \
                 use `name` = then a ```{lang} fence (same shape as formulas)"
            );
        }
        let mut body_lines: Vec<String> = Vec::new();
        let mut closed = false;
        let mut end_line = open_line;
        while let Some(l) = self.peek() {
            if crate::foreign::is_fence_closer(l.text.trim()) {
                end_line = l.line_no;
                self.bump();
                closed = true;
                break;
            }
            body_lines.push(l.text.clone());
            self.bump();
        }
        if !closed {
            bail!("{open_line}:1: unclosed code fence ```{lang}");
        }
        let source = body_lines.join("\n");
        Ok((
            Expr::Code(crate::value::CodeBlock { lang, source }),
            end_line,
        ))
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
            return Ok((
                formula_fence_to_expr(body)
                    .map_err(|e| anyhow::anyhow!("{open_line}:1: formula: {e}"))?,
                open_line,
            ));
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
        let expr = formula_fence_to_expr(&text)
            .map_err(|e| anyhow::anyhow!("{open_line}:1: formula: {e}"))?;
        Ok((expr, end_line))
    }

    /// GFM table after empty RHS:
    /// - 1-col → List
    /// - first header `@` / `行` / `row` → List of row Maps (marker col excluded)
    /// - else ≥2-col → Map (column-oriented)
    ///
    /// Data cells are expressions ([`parse_call_arg_value`]); headers stay key text.
    fn consume_table(&mut self) -> Result<Expr> {
        self.skip_noise();
        let mut header: Vec<String> = Vec::new();
        let mut header_line: u32 = 1;
        let mut data_rows: Vec<(u32, Vec<String>)> = Vec::new();
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
            let line_no = l.line_no;
            self.bump();
            let cells = split_table_row(t);
            if !header_done {
                header = cells;
                header_line = line_no;
                header_done = true;
                continue;
            }
            if !sep_done
                && cells
                    .iter()
                    .all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch.is_whitespace()))
            {
                sep_done = true;
                continue;
            }
            sep_done = true;
            data_rows.push((line_no, cells));
        }
        if !header_done {
            bail!("expected GFM table after empty assignment");
        }
        let row_oriented = header
            .first()
            .map(|h| is_row_table_marker(h))
            .unwrap_or(false);
        if row_oriented {
            return build_row_oriented_table(header, header_line, data_rows);
        }
        if header.len() <= 1 {
            let mut rows = Vec::new();
            for (line_no, cells) in data_rows {
                if let Some(cell) = cells.first() {
                    rows.push(parse_table_cell(cell, line_no)?);
                }
            }
            return Ok(Expr::List(rows));
        }
        // Duplicate header keys are errors (column identity must be unique).
        let mut seen = std::collections::HashSet::new();
        for key in &header {
            if !seen.insert(key.clone()) {
                return Err(Diagnostic::new(
                    None,
                    Span {
                        line: header_line,
                        col: 1,
                    },
                    format!("duplicate table header `{key}`"),
                )
                .into());
            }
        }
        let n_rows = data_rows.len();
        let mut pairs = Vec::with_capacity(header.len());
        for (col, key) in header.iter().enumerate() {
            if n_rows == 1 {
                let (line_no, cells) = &data_rows[0];
                let raw = cells.get(col).map(|s| s.as_str()).unwrap_or("");
                pairs.push((key.clone(), parse_table_cell(raw, *line_no)?));
            } else {
                // 0 rows → empty lists; ≥2 rows → column lists.
                let mut col_vals = Vec::with_capacity(n_rows);
                for (line_no, cells) in &data_rows {
                    let raw = cells.get(col).map(|s| s.as_str()).unwrap_or("");
                    col_vals.push(parse_table_cell(raw, *line_no)?);
                }
                pairs.push((key.clone(), Expr::List(col_vals)));
            }
        }
        Ok(Expr::Map(pairs))
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
            let Some(num) = ordered_item_number(trimmed) else {
                break;
            };
            if ind != base_indent {
                break;
            }
            // Same-indent restart at `1.` begins a new Branch statement.
            if !arms.is_empty() && num == 1 {
                break;
            }
            let cond_src = strip_ordered(trimmed).unwrap();
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
            if is_ordered_branch(trimmed) {
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
    if let Some(rest) = inner.strip_prefix('`') {
        let end = rest.find('`')?;
        let name = rest[..end].to_string();
        if name.is_empty() {
            return None;
        }
        let after = rest[end + 1..].trim_start();
        let rhs = after.strip_prefix('=')?.trim().to_string();
        return Some((name, rhs));
    }
    // Bare LHS: `answer = …` (italic exemption).
    let eq = inner.find('=')?;
    let name = inner[..eq].trim();
    if !is_bare_assign_ident(name) {
        return None;
    }
    let rhs = inner[eq + 1..].trim().to_string();
    Some((name.to_string(), rhs))
}

fn is_bare_assign_ident(s: &str) -> bool {
    if s.is_empty() || s.contains('`') || s.chars().any(char::is_whitespace) {
        return false;
    }
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_alphanumeric() || first == '_' || !first.is_ascii()) {
        return false;
    }
    true
}

/// Same-line `$$body$$` → Formula or numeric matrix List; otherwise None.
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
        let expr = formula_fence_to_expr(body).map_err(|e| anyhow::anyhow!("formula: {e}"))?;
        return Ok(Some(expr));
    }
    bail!("incomplete inline formula (expected `$$…$$` on one line)");
}

fn formula_fence_to_expr(body: &str) -> Result<Expr, String> {
    if crate::formula::looks_like_matrix(body) {
        let rows = crate::formula::parse_eval_matrix(body)?;
        return Ok(Expr::Formula(crate::formula::Expr::Matrix {
            source: body.trim().to_string(),
            rows,
        }));
    }
    let expr = crate::formula::parse(body)?;
    Ok(Expr::Formula(expr))
}

fn parse_backtick_ident(s: &str) -> Option<String> {
    let s = s.trim();
    if !s.starts_with('`') {
        return None;
    }
    let inner = &s[1..];
    let end = inner.find('`')?;
    let name = &inner[..end];
    if name.is_empty() || s.len() != end + 2 {
        return None;
    }
    Some(name.to_string())
}

fn parse_param_line(trimmed: &str) -> Option<crate::ast::Param> {
    use crate::ast::Param;
    let rest = trimmed.strip_prefix('+')?.trim();
    if rest.is_empty() || !rest.starts_with('`') {
        return None;
    }
    let end_tick = rest[1..].find('`')?;
    let name = rest[1..end_tick + 1].to_string();
    if name.is_empty() {
        return None;
    }
    let after = rest[end_tick + 2..].trim();
    let default = if let Some(def_s) = after.strip_prefix('=') {
        let def_s = def_s.trim();
        if def_s.is_empty() {
            return None;
        }
        Some(parse_call_arg_value(def_s).ok()?)
    } else if after.is_empty() {
        None
    } else {
        return None;
    };
    Some(Param { name, default })
}

fn parse_foreach_ident(s: &str) -> Option<String> {
    let s = s.trim();
    if s.starts_with('`') {
        return parse_backtick_ident(s);
    }
    if s.is_empty() {
        return None;
    }
    // Bare name exemption: `[item](list)` without ticks (foreach only).
    if s.chars().any(|c| c.is_whitespace() || c == '`' || c == '[' || c == ']') {
        return None;
    }
    let mut chars = s.chars();
    let first = chars.next()?;
    if !(first.is_alphanumeric() || first == '_' || !first.is_ascii()) {
        return None;
    }
    Some(s.to_string())
}

fn parse_foreach_header(rest: &str) -> Option<(String, String)> {
    // [`item`](`collection`) or bare [item](collection); sides may mix.
    let rest = rest.trim();
    if !rest.starts_with('[') {
        return None;
    }
    let end_item = rest.find(']')?;
    let item = parse_foreach_ident(&rest[1..end_item])?;
    let after = rest[end_item + 1..].trim_start();
    if !after.starts_with('(') || !after.ends_with(')') {
        return None;
    }
    let coll = parse_foreach_ident(&after[1..after.len() - 1])?;
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

/// `# Child = > Parent` → `(Child, Some(Parent))`; plain `# Child` → `(Child, None)`.
fn parse_object_decl(level: u8, raw: &str, span: Span) -> Result<(String, Option<String>)> {
    let raw = raw.trim();
    if let Some(eq) = raw.find('=') {
        if level != 1 {
            return Err(Diagnostic::new(
                None,
                span,
                "inheritance (`= > Base`) is only allowed on `#` object headings",
            )
            .into());
        }
        let name = raw[..eq].trim();
        let rhs = raw[eq + 1..].trim();
        if name.is_empty() {
            return Err(Diagnostic::new(None, span, "object name missing before `=`").into());
        }
        if !is_simple_type_name(name) {
            return Err(Diagnostic::new(
                None,
                span,
                format!("invalid object name `{name}`"),
            )
            .into());
        }
        let Some(after_gt) = rhs.strip_prefix('>') else {
            return Err(Diagnostic::new(
                None,
                span,
                "object inheritance requires `# Name = > Base` (base must follow `>`)",
            )
            .into());
        };
        let base = after_gt.trim();
        if base.is_empty() {
            return Err(Diagnostic::new(None, span, "missing base type after `>`").into());
        }
        if base.contains('.') {
            return Err(Diagnostic::new(
                None,
                span,
                format!("base type must be a simple name, not a path (`{base}`)"),
            )
            .into());
        }
        if !is_simple_type_name(base) {
            return Err(Diagnostic::new(
                None,
                span,
                format!("invalid base type name `{base}`"),
            )
            .into());
        }
        return Ok((name.to_string(), Some(base.to_string())));
    }
    Ok((raw.to_string(), None))
}

fn is_simple_type_name(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_alphabetic() || first == '_' || !first.is_ascii()) {
        return false;
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii())
}

/// Result of parsing one frontmatter `import` / `导入` line.
#[derive(Debug, Clone)]
pub enum ImportLine {
    File(Import),
    Member(Use),
}

fn legacy_frontmatter_import_hint(trimmed: &str) -> Option<String> {
    let t = trimmed.trim();
    if !t.starts_with('>') {
        return None;
    }
    let rest = t[1..].trim();
    let looks_file = rest.contains(".mq.md");
    let looks_use = rest.starts_with("use")
        || rest.starts_with("使用")
        || rest.starts_with("use ")
        || rest.starts_with("使用 ");
    if looks_file || looks_use {
        return Some(
            "legacy frontmatter import `> …` is removed; use `import bind:path.mq.md` \
             or `import bind:lib.member` (Chinese: `导入`)"
                .into(),
        );
    }
    None
}

/// Parse `import json:lib/json.mq.md` / `导入 fmt:time.format`.
/// Returns `Ok(None)` for non-import frontmatter lines (e.g. `title:`).
pub fn parse_import_line(trimmed: &str) -> Result<Option<ImportLine>> {
    let t = trimmed.trim();
    let after = if t.starts_with("import")
        && t.get(6..)
            .is_some_and(|s| s.starts_with(|c: char| c.is_whitespace()))
    {
        t[6..].trim()
    } else if let Some(a) = t.strip_prefix("导入") {
        let a = a.trim();
        if a.is_empty() {
            bail!("expected `导入 bind:target`");
        }
        a
    } else {
        return Ok(None);
    };

    let Some((bind_raw, target_raw)) = after.split_once(':') else {
        bail!("expected `import bind:target` (file `.mq.md` or `lib.member`)");
    };
    let bind = bind_raw.trim();
    let target = target_raw.trim();
    if bind.is_empty() || !is_simple_bind_name(bind) {
        bail!("invalid import bind `{bind}`");
    }
    if target.is_empty() || target.contains('`') || target.contains(char::is_whitespace) {
        bail!("invalid import target `{target}`");
    }

    if target.ends_with(".mq.md") {
        return Ok(Some(ImportLine::File(Import {
            path: target.to_string(),
            bind: bind.to_string(),
        })));
    }

    let path: Vec<String> = target
        .split('.')
        .map(|s| s.trim().to_string())
        .collect();
    if path.len() < 2 || path.iter().any(|p| p.is_empty() || !is_simple_bind_name(p)) {
        bail!(
            "import target `{target}` must be a `.mq.md` path or dotted member path (e.g. `time.format`)"
        );
    }
    Ok(Some(ImportLine::Member(Use {
        path,
        bind: bind.to_string(),
    })))
}

fn is_simple_bind_name(s: &str) -> bool {
    is_simple_type_name(s)
}

/// Parse a file import line body for catalog / tooling (`bind:path.mq.md` after keyword stripped),
/// or the full `import bind:path` / legacy-free form via [`parse_import_line`].
pub fn parse_import_spec(line: &str) -> Option<Import> {
    match parse_import_line(line).ok()? {
        Some(ImportLine::File(imp)) => Some(imp),
        _ => None,
    }
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

/// Leading `N` in an ordered-list item `N. …` (1-based Markdown marker).
fn ordered_item_number(trimmed: &str) -> Option<u32> {
    let mut chars = trimmed.chars();
    let mut n: u32 = 0;
    let mut saw_digit = false;
    while let Some(c) = chars.next() {
        if let Some(d) = c.to_digit(10) {
            saw_digit = true;
            n = n.saturating_mul(10).saturating_add(d);
            continue;
        }
        if c == '.' && saw_digit {
            return Some(n);
        }
        return None;
    }
    None
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

fn parse_table_cell(raw: &str, line: u32) -> Result<Expr> {
    parse_call_arg_value(raw).map_err(|e| {
        Diagnostic::new(
            None,
            Span { line, col: 1 },
            format!("table cell: {e}"),
        )
        .into()
    })
}

/// First-column header that opts into row-oriented records (`List` of `Map`).
fn is_row_table_marker(header: &str) -> bool {
    matches!(header.trim(), "@" | "行" | "row")
}

fn build_row_oriented_table(
    header: Vec<String>,
    header_line: u32,
    data_rows: Vec<(u32, Vec<String>)>,
) -> Result<Expr> {
    if header.len() < 2 {
        return Err(Diagnostic::new(
            None,
            Span {
                line: header_line,
                col: 1,
            },
            "row-oriented table needs field columns after `@` / `行` / `row`",
        )
        .into());
    }
    let fields: Vec<String> = header[1..].to_vec();
    let mut seen = std::collections::HashSet::new();
    for key in &fields {
        if !seen.insert(key.clone()) {
            return Err(Diagnostic::new(
                None,
                Span {
                    line: header_line,
                    col: 1,
                },
                format!("duplicate table header `{key}`"),
            )
            .into());
        }
    }
    let mut rows = Vec::with_capacity(data_rows.len());
    for (line_no, cells) in data_rows {
        let mut pairs = Vec::with_capacity(fields.len());
        for (i, key) in fields.iter().enumerate() {
            // Skip marker column (index 0); fields start at column 1.
            let raw = cells.get(i + 1).map(|s| s.as_str()).unwrap_or("");
            pairs.push((key.clone(), parse_table_cell(raw, line_no)?));
        }
        rows.push(Expr::Map(pairs));
    }
    Ok(Expr::List(rows))
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
        assert_eq!(main.children[0].params[0].name, "谁");
    }

    #[test]
    fn parse_optional_param_default() {
        let src = "# main\n\n## f\n    + `a`\n    + `b`=Hi\n\n> print text=`a`\n";
        let m = parse_source(src).unwrap();
        let f = &m.functions[0].children[0];
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.params[0].name, "a");
        assert_eq!(f.params[1].name, "b");
        assert!(matches!(
            f.params[1].default,
            Some(Expr::Literal(Literal::Text(ref s))) if s == "Hi"
        ));
    }

    #[test]
    fn parse_foreach_backtick_ids() {
        let src = "# main\n\n- [`x`](`xs`)\n  > print text=`x`\n";
        let m = parse_source(src).unwrap();
        match &m.functions[0].body[0] {
            Stmt::ForEach { item, collection, .. } => {
                assert_eq!(item, "x");
                assert_eq!(collection, "xs");
            }
            other => panic!("expected foreach, got {other:?}"),
        }
    }

    #[test]
    fn parse_foreach_bare_ids() {
        let src = "# main\n\n- [x](xs)\n  > print text=`x`\n";
        let m = parse_source(src).unwrap();
        match &m.functions[0].body[0] {
            Stmt::ForEach { item, collection, .. } => {
                assert_eq!(item, "x");
                assert_eq!(collection, "xs");
            }
            other => panic!("expected foreach, got {other:?}"),
        }
    }

    #[test]
    fn parse_foreach_mixed_ticks() {
        let src = "# main\n\n- [果](`篮子`)\n  > print text=`果`\n";
        let m = parse_source(src).unwrap();
        match &m.functions[0].body[0] {
            Stmt::ForEach { item, collection, .. } => {
                assert_eq!(item, "果");
                assert_eq!(collection, "篮子");
            }
            other => panic!("expected foreach, got {other:?}"),
        }
    }

    #[test]
    fn parse_italic_bare_lhs_prefer_var() {
        let src = "# main\n\n*answer = 1*\n**answer**\n";
        let m = parse_source(src).unwrap();
        match &m.functions[0].body[0] {
            Stmt::Assign { name, value, .. } => {
                assert_eq!(name, "answer");
                assert!(matches!(value, Expr::Literal(Literal::Int(1))));
            }
            other => panic!("expected assign, got {other:?}"),
        }
        match &m.functions[0].body[1] {
            Stmt::Return { value, .. } => {
                assert!(matches!(value, Expr::Var(n) if n == "answer"));
            }
            other => panic!("expected return, got {other:?}"),
        }
    }

    #[test]
    fn parse_branch_rejects_plus_arm() {
        let src = "# main\n\n+ `x` > 0\n  > print text=ok\n";
        let err = parse_source(src).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("unrecognized") || msg.contains("empty branch"),
            "{msg}"
        );
    }

    #[test]
    fn parse_branch() {
        let src = include_str!("../../tests/structure/branch.mq.md");
        let m = parse_source(src).unwrap();
        let main = &m.functions[0];
        assert!(main.body.iter().any(|s| matches!(s, Stmt::Branch { .. })));
    }

    #[test]
    fn consecutive_branches_restart_at_one() {
        let m = parse_source(include_str!(
            "../../tests/structure/branch-consecutive.mq.md"
        ))
        .unwrap();
        let branches: Vec<_> = m.functions[0]
            .body
            .iter()
            .filter(|s| matches!(s, Stmt::Branch { .. }))
            .collect();
        assert_eq!(
            branches.len(),
            3,
            "expected three Branch stmts, got {:?}",
            m.functions[0].body
        );
        for b in &branches {
            match b {
                Stmt::Branch { arms, .. } => assert_eq!(arms.len(), 2),
                _ => unreachable!(),
            }
        }
        // Arm bodies must hold the prints (not leaked as sibling Calls).
        match branches[0] {
            Stmt::Branch { arms, .. } => {
                assert!(
                    matches!(arms[0].body[0], Stmt::Call { .. }),
                    "first arm body={:?}",
                    arms[0].body
                );
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn consecutive_branches_without_blank_line() {
        let src = "# main\n\n1. True\n  > print text=A\n2. *\n  > print text=skip\n1. True\n  > print text=B\n";
        let m = parse_source(src).unwrap();
        let branches: Vec<_> = m.functions[0]
            .body
            .iter()
            .filter(|s| matches!(s, Stmt::Branch { .. }))
            .collect();
        assert_eq!(branches.len(), 2, "got {:?}", m.functions[0].body);
        match branches[0] {
            Stmt::Branch { arms, .. } => assert_eq!(arms.len(), 2),
            _ => unreachable!(),
        }
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

    #[test]
    fn parse_code_assign_fence() {
        let src = "# main\n\n`hi` =\n```python\nprint(1)\n```\n\n> print text=ok\n";
        let m = parse_source(src).unwrap();
        match &m.functions[0].body[0] {
            Stmt::Assign {
                name,
                value: Expr::Code(c),
                ..
            } => {
                assert_eq!(name, "hi");
                assert_eq!(c.lang, "python");
                assert_eq!(c.source, "print(1)");
            }
            other => panic!("expected code assign, got {other:?}"),
        }
    }
}
