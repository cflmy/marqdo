//! Expression parser (recursive descent over a single string).

use anyhow::{bail, Result};

use crate::ast::{BinaryOp, CallExpr, Expr, InterpPart, Literal, UnaryOp};

pub fn parse_expr(input: &str) -> Result<Expr> {
    let mut p = Parser::new(input.trim());
    let e = p.parse_or()?;
    p.skip_ws();
    if !p.rest().is_empty() {
        bail!("trailing input in expression: {:?}", p.rest());
    }
    Ok(e)
}

/// Argument / template RHS: prefer a full expression; else interpolated text.
/// Hyphenated prose (`none-falsy`) is not parsed as subtraction.
pub fn parse_value_or_interp(input: &str) -> Result<Expr> {
    let s = input.trim();
    if s.is_empty() {
        return Ok(Expr::Literal(Literal::Text(String::new())));
    }
    if has_hyphenated_prose(s) {
        return Ok(parse_interp(s));
    }
    match parse_expr(s) {
        Ok(e) => Ok(e),
        Err(_) => Ok(parse_interp(s)),
    }
}

fn has_hyphenated_prose(s: &str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    for i in 1..chars.len().saturating_sub(1) {
        if chars[i] != '-' {
            continue;
        }
        let left = chars[i - 1];
        let right = chars[i + 1];
        if left.is_whitespace() || right.is_whitespace() || left == '`' || right == '`' {
            continue;
        }
        // `none-falsy` / `Hello-World` — not `` `n` - 1 ``
        if left.is_alphanumeric() || right.is_alphanumeric() || !left.is_ascii() || !right.is_ascii()
        {
            return true;
        }
    }
    false
}

pub fn parse_interp(s: &str) -> Expr {
    let mut parts = Vec::new();
    let mut rest = s;
    while !rest.is_empty() {
        if let Some(start) = rest.find('`') {
            if start > 0 {
                parts.push(InterpPart::Lit(rest[..start].to_string()));
            }
            let after = &rest[start + 1..];
            if let Some(end) = after.find('`') {
                let name = after[..end].to_string();
                parts.push(InterpPart::Var(name));
                rest = &after[end + 1..];
            } else {
                parts.push(InterpPart::Lit(format!("`{after}")));
                break;
            }
        } else {
            parts.push(InterpPart::Lit(rest.to_string()));
            break;
        }
    }
    if parts.len() == 1 {
        match &parts[0] {
            InterpPart::Lit(t) => return Expr::Literal(Literal::Text(t.clone())),
            InterpPart::Var(n) => return Expr::Var(n.clone()),
        }
    }
    Expr::Interp(parts)
}

struct Parser<'a> {
    src: &'a str,
    i: usize,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str) -> Self {
        Self { src, i: 0 }
    }

    fn rest(&self) -> &str {
        &self.src[self.i..]
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.bump_char();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let mut chars = self.rest().chars();
        let c = chars.next()?;
        self.i += c.len_utf8();
        Some(c)
    }

    fn starts_with(&self, s: &str) -> bool {
        self.rest().starts_with(s)
    }

    fn eat(&mut self, s: &str) -> bool {
        if self.starts_with(s) {
            self.i += s.len();
            true
        } else {
            false
        }
    }

    /// Eat English or Chinese logic keyword with word-boundary check.
    fn eat_logic_kw(&mut self, en: &str, zh: &str) -> bool {
        if self.starts_with(en) && is_word_end(self.rest(), en.len()) {
            self.i += en.len();
            return true;
        }
        if self.starts_with(zh) && is_word_end(self.rest(), zh.len()) {
            self.i += zh.len();
            return true;
        }
        false
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        loop {
            self.skip_ws();
            if self.eat_logic_kw("or", "或") {
                let right = self.parse_and()?;
                left = Expr::Binary {
                    op: BinaryOp::Or,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_cmp()?;
        loop {
            self.skip_ws();
            if self.eat_logic_kw("and", "且") {
                let right = self.parse_cmp()?;
                left = Expr::Binary {
                    op: BinaryOp::And,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_cmp(&mut self) -> Result<Expr> {
        let mut left = self.parse_add()?;
        self.skip_ws();
        let op = if self.eat("==") {
            Some(BinaryOp::Eq)
        } else if self.eat("!=") {
            Some(BinaryOp::Ne)
        } else if self.eat("<=") {
            Some(BinaryOp::Le)
        } else if self.eat(">=") {
            Some(BinaryOp::Ge)
        } else if self.eat("<") {
            Some(BinaryOp::Lt)
        } else if self.eat(">") {
            Some(BinaryOp::Gt)
        } else {
            None
        };
        if let Some(op) = op {
            let right = self.parse_add()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_add(&mut self) -> Result<Expr> {
        let mut left = self.parse_mul()?;
        loop {
            self.skip_ws();
            if self.eat("+") {
                let right = self.parse_mul()?;
                left = Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.eat("-") {
                let right = self.parse_mul()?;
                left = Expr::Binary {
                    op: BinaryOp::Sub,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_mul(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            self.skip_ws();
            if self.eat("*") {
                let right = self.parse_unary()?;
                left = Expr::Binary {
                    op: BinaryOp::Mul,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else if self.eat("/") {
                let right = self.parse_unary()?;
                left = Expr::Binary {
                    op: BinaryOp::Div,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        self.skip_ws();
        if self.eat_logic_kw("not", "非") {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
            });
        }
        if self.eat("-") {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(expr),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        self.skip_ws();
        if self.eat("(") {
            let e = self.parse_or()?;
            self.skip_ws();
            if !self.eat(")") {
                bail!("expected ')'");
            }
            return Ok(e);
        }
        if self.eat("\"") {
            return self.parse_quoted_string();
        }
        if self.eat("`") {
            let start = self.i;
            while let Some(c) = self.peek_char() {
                if c == '`' {
                    break;
                }
                self.bump_char();
            }
            let name = self.src[start..self.i].to_string();
            if !self.eat("`") {
                bail!("unterminated `name`");
            }
            return Ok(Expr::Var(name));
        }
        if self.eat(">") {
            // Inline call: `> name k=v`
            self.skip_ws();
            return Ok(Expr::Call(parse_call_tail(self.rest()).map(|(c, consumed)| {
                self.i += consumed;
                c
            })?));
        }
        if self.starts_with("True") && is_word_end(self.rest(), 4) {
            self.i += 4;
            return Ok(Expr::Literal(Literal::Bool(true)));
        }
        if self.starts_with("真") && is_word_end(self.rest(), '真'.len_utf8()) {
            self.i += '真'.len_utf8();
            return Ok(Expr::Literal(Literal::Bool(true)));
        }
        if self.starts_with("False") && is_word_end(self.rest(), 5) {
            self.i += 5;
            return Ok(Expr::Literal(Literal::Bool(false)));
        }
        if self.starts_with("假") && is_word_end(self.rest(), '假'.len_utf8()) {
            self.i += '假'.len_utf8();
            return Ok(Expr::Literal(Literal::Bool(false)));
        }
        if self.starts_with("None") && is_word_end(self.rest(), 4) {
            self.i += 4;
            return Ok(Expr::Literal(Literal::None));
        }
        if self.starts_with("空") && is_word_end(self.rest(), '空'.len_utf8()) {
            self.i += '空'.len_utf8();
            return Ok(Expr::Literal(Literal::None));
        }
        if self.peek_char().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            let start = self.i;
            while self.peek_char().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                self.bump_char();
            }
            let n: i64 = self.src[start..self.i].parse()?;
            return Ok(Expr::Literal(Literal::Int(n)));
        }
        // Bare identifier / text word
        if self.peek_char().map(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii()).unwrap_or(false)
        {
            let start = self.i;
            while let Some(c) = self.peek_char() {
                if c.is_whitespace() || matches!(c, '=' | '+' | '-' | '*' | '/' | '<' | '>' | '(' | ')' | '`' | '|' | ',' | '"')
                {
                    break;
                }
                self.bump_char();
            }
            let word = &self.src[start..self.i];
            return Ok(Expr::Literal(Literal::Text(word.to_string())));
        }
        bail!("unexpected expression input: {:?}", self.rest());
    }

    /// `"..."` string with `\n` `\t` `\r` `\\` `\"` and `` `var` `` interpolation.
    fn parse_quoted_string(&mut self) -> Result<Expr> {
        use crate::ast::InterpPart;
        let mut parts: Vec<InterpPart> = Vec::new();
        let mut lit = String::new();
        loop {
            let Some(c) = self.peek_char() else {
                bail!("unterminated \" string");
            };
            if c == '"' {
                self.bump_char();
                break;
            }
            if c == '\\' {
                self.bump_char();
                let esc = self.peek_char().ok_or_else(|| anyhow::anyhow!("dangling \\ in string"))?;
                self.bump_char();
                lit.push(match esc {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    other => bail!("unknown escape \\{other} in string"),
                });
                continue;
            }
            if c == '`' {
                if !lit.is_empty() {
                    parts.push(InterpPart::Lit(std::mem::take(&mut lit)));
                }
                self.bump_char();
                let start = self.i;
                while let Some(ch) = self.peek_char() {
                    if ch == '`' {
                        break;
                    }
                    self.bump_char();
                }
                let name = self.src[start..self.i].to_string();
                if !self.eat("`") {
                    bail!("unterminated `name` in string");
                }
                parts.push(InterpPart::Var(name));
                continue;
            }
            lit.push(c);
            self.bump_char();
        }
        if !lit.is_empty() {
            parts.push(InterpPart::Lit(lit));
        }
        if parts.is_empty() {
            return Ok(Expr::Literal(Literal::Text(String::new())));
        }
        if parts.len() == 1 {
            return Ok(match parts.remove(0) {
                InterpPart::Lit(t) => Expr::Literal(Literal::Text(t)),
                InterpPart::Var(n) => Expr::Var(n),
            });
        }
        Ok(Expr::Interp(parts))
    }
}

fn is_word_end(s: &str, len: usize) -> bool {
    match s[len..].chars().next() {
        None => true,
        Some(c) => !(c.is_alphanumeric() || c == '_'),
    }
}

/// Parse `name key=val …` / positional args; returns call + bytes consumed.
/// Callee may be a bare name, library path `time.parse`, or method `` `recv`.method ``.
pub fn parse_call_tail(s: &str) -> Result<(CallExpr, usize)> {
    use crate::ast::Arg;

    let s = s.trim_start();
    let original_len = s.len();

    let callee_tok = {
        let mut parts = s.split_whitespace();
        parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing callee"))?
            .to_string()
    };

    let (receiver, callee, path) = if let Some((r, m)) = split_method_callee(&callee_tok) {
        (Some(r), m, None)
    } else if let Some(segments) = split_lib_path_callee(&callee_tok) {
        let last = segments.last().cloned().unwrap_or_default();
        (None, last, Some(segments))
    } else {
        (None, callee_tok.clone(), None)
    };

    let after_callee_offset = {
        let idx = s.find(&callee_tok).unwrap();
        idx + callee_tok.len()
    };
    let mut args_str = s[after_callee_offset..].trim_start();
    let mut args = Vec::new();
    let mut seen_named = false;

    while !args_str.is_empty() {
        if let Some((key, after_eq)) = try_named_arg(args_str) {
            seen_named = true;
            let val_end = find_next_arg_boundary(after_eq);
            let val_raw = after_eq[..val_end].trim_end();
            args.push(Arg::Named {
                name: key,
                value: parse_value_or_interp(val_raw)?,
            });
            args_str = after_eq[val_end..].trim_start();
        } else {
            if seen_named {
                bail!("positional argument after named argument is not allowed");
            }
            let (tok, rest) = split_first_token(args_str);
            if tok.is_empty() {
                break;
            }
            args.push(Arg::Positional(parse_value_or_interp(tok)?));
            args_str = rest.trim_start();
        }
    }

    Ok((
        CallExpr {
            callee,
            path,
            receiver,
            args,
        },
        original_len,
    ))
}

/// `` `recv`.method `` → (recv, method).
fn split_method_callee(tok: &str) -> Option<(String, String)> {
    if !tok.starts_with('`') {
        return None;
    }
    let rest = &tok[1..];
    let end = rest.find('`')?;
    let recv = rest[..end].to_string();
    if recv.is_empty() {
        return None;
    }
    let after = &rest[end + 1..];
    let method = after.strip_prefix('.')?;
    if method.is_empty() || method.contains('`') || method.contains('.') {
        return None;
    }
    Some((recv, method.to_string()))
}

/// Bare `time.parse` / `agent.agent` → path segments (len ≥ 2).
fn split_lib_path_callee(tok: &str) -> Option<Vec<String>> {
    if tok.starts_with('`') || !tok.contains('.') {
        return None;
    }
    let parts: Vec<String> = tok.split('.').map(|s| s.to_string()).collect();
    if parts.len() < 2 {
        return None;
    }
    if parts.iter().any(|p| p.is_empty() || p.contains('`')) {
        return None;
    }
    Some(parts)
}

/// If `s` begins with `ident=`, return (ident, rest_after_eq).
fn try_named_arg(s: &str) -> Option<(String, &str)> {
    let s = s.trim_start();
    for (i, c) in s.char_indices() {
        if c == '=' {
            if i == 0 {
                return None;
            }
            let key = &s[..i];
            if key.chars().any(|ch| ch.is_whitespace()) {
                return None;
            }
            return Some((key.to_string(), &s[i + 1..]));
        }
        if c.is_whitespace() {
            return None;
        }
    }
    None
}

fn split_first_token(s: &str) -> (&str, &str) {
    let s = s.trim_start();
    if s.starts_with('"') {
        if let Some(end) = end_of_quoted_string(s) {
            return (&s[..end], &s[end..]);
        }
    }
    if s.starts_with('`') {
        // `` `name` `` as one token
        if let Some(end) = s[1..].find('`') {
            let end = end + 2; // include closing `
            return (&s[..end], &s[end..]);
        }
    }
    match s.find(char::is_whitespace) {
        Some(i) => (&s[..i], &s[i..]),
        None => (s, ""),
    }
}

fn find_next_arg_boundary(after_eq: &str) -> usize {
    // Look for ` <ident>=` pattern not inside backticks or quoted strings.
    let mut in_bt = false;
    let mut in_str = false;
    let mut chars = after_eq.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if in_str {
            if c == '\\' {
                chars.next();
                continue;
            }
            if c == '"' {
                in_str = false;
            }
            continue;
        }
        if in_bt {
            if c == '`' {
                in_bt = false;
            }
            continue;
        }
        if c == '`' {
            in_bt = true;
            continue;
        }
        if c == '"' {
            in_str = true;
            continue;
        }
        if c.is_whitespace() {
            let rest = &after_eq[i..];
            let trimmed = rest.trim_start();
            let skipped = rest.len() - trimmed.len();
            if let Some(eq_at) = trimmed.find('=') {
                let key = &trimmed[..eq_at];
                if !key.is_empty()
                    && !key.contains(char::is_whitespace)
                    && key
                        .chars()
                        .all(|ch| ch.is_alphanumeric() || ch == '_' || !ch.is_ascii())
                {
                    return i + skipped;
                }
            }
        }
    }
    after_eq.len()
}

/// Byte index after a leading `"..."` token (opening quote included in slice).
fn end_of_quoted_string(s: &str) -> Option<usize> {
    if !s.starts_with('"') {
        return None;
    }
    let mut escaped = false;
    for (i, c) in s.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == '"' {
            return Some(i + c.len_utf8());
        }
    }
    None
}

/// Parse `> callee args` call expression/statement text (without leading `>`).
pub fn parse_call_after_gt(after_gt: &str) -> Result<CallExpr> {
    let (call, _) = parse_call_tail(after_gt.trim())?;
    Ok(call)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmp_and_arith() {
        let e = parse_expr("`x` > 0").unwrap();
        assert!(matches!(e, Expr::Binary { op: BinaryOp::Gt, .. }));
        let e = parse_expr("`n` + 1").unwrap();
        assert!(matches!(e, Expr::Binary { op: BinaryOp::Add, .. }));
    }

    #[test]
    fn quoted_string_escapes() {
        let e = parse_expr("\"a\\nb\"").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "a\nb"
        ));
        let e = parse_expr("\"hi `x`!\"").unwrap();
        assert!(matches!(e, Expr::Interp(_)));
    }

    #[test]
    fn bare_token_no_escape() {
        let e = parse_expr("a\\nb").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "a\\nb"
        ));
    }

    #[test]
    fn chinese_logic_and_literals() {
        let e = parse_expr("`a` 且 非 `b`").unwrap();
        assert!(matches!(e, Expr::Binary { op: BinaryOp::And, .. }));
        assert!(matches!(
            parse_expr("真").unwrap(),
            Expr::Literal(Literal::Bool(true))
        ));
        assert!(matches!(
            parse_expr("空").unwrap(),
            Expr::Literal(Literal::None)
        ));
    }
}
