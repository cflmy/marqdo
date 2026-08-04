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
pub fn parse_value_or_interp(input: &str) -> Result<Expr> {
    let s = input.trim();
    if s.is_empty() {
        return Ok(Expr::Literal(Literal::Text(String::new())));
    }
    match parse_expr(s) {
        Ok(e) => Ok(e),
        Err(_) => Ok(parse_interp(s)),
    }
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

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        loop {
            self.skip_ws();
            if self.eat("or") {
                // ensure word boundary
                if self.peek_char().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
                    self.i -= 2;
                    break;
                }
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
            if self.eat("and") {
                if self.peek_char().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
                    self.i -= 3;
                    break;
                }
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
        if self.eat("not") {
            if self.peek_char().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
                self.i -= 3;
            } else {
                let expr = self.parse_unary()?;
                return Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                });
            }
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
        if self.starts_with("False") && is_word_end(self.rest(), 5) {
            self.i += 5;
            return Ok(Expr::Literal(Literal::Bool(false)));
        }
        if self.starts_with("None") && is_word_end(self.rest(), 4) {
            self.i += 4;
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
                if c.is_whitespace() || matches!(c, '=' | '+' | '-' | '*' | '/' | '<' | '>' | '(' | ')' | '`' | '|' | ',')
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
}

fn is_word_end(s: &str, len: usize) -> bool {
    match s[len..].chars().next() {
        None => true,
        Some(c) => !(c.is_alphanumeric() || c == '_'),
    }
}

/// Parse `name key=val key2=val2` from a string; returns call + bytes consumed.
pub fn parse_call_tail(s: &str) -> Result<(CallExpr, usize)> {
    let s = s.trim_start();
    let trimmed_lead = s.len(); // not used — work on s
    let _ = trimmed_lead;
    let original_len = s.len(); // we'll compute from full — caller passes rest after >

    let mut parts = s.split_whitespace();
    let callee = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing callee"))?
        .to_string();

    let after_callee_offset = {
        let idx = s.find(&callee).unwrap();
        idx + callee.len()
    };
    let mut args_str = s[after_callee_offset..].trim_start();
    let mut args = Vec::new();

    // Support one or more `key=value`; value may contain spaces until next ` key=` pattern.
    while !args_str.is_empty() {
        let eq = args_str
            .find('=')
            .ok_or_else(|| anyhow::anyhow!("expected key=value in call args, got {args_str:?}"))?;
        let key = args_str[..eq].trim().to_string();
        if key.is_empty() {
            bail!("empty argument name");
        }
        let after_eq = &args_str[eq + 1..];
        // Value runs until ` word=` where word is next key, or end.
        let val_end = find_next_arg_boundary(after_eq);
        let val_raw = after_eq[..val_end].trim_end();
        args.push((key, parse_value_or_interp(val_raw)?));
        args_str = after_eq[val_end..].trim_start();
    }

    let consumed = original_len; // entire rest consumed for stmt-form calls
    Ok((CallExpr { callee, args }, consumed))
}

fn find_next_arg_boundary(after_eq: &str) -> usize {
    // Look for ` <ident>=` pattern not inside backticks (UTF-8 safe).
    let mut in_bt = false;
    for (i, c) in after_eq.char_indices() {
        if c == '`' {
            in_bt = !in_bt;
            continue;
        }
        if !in_bt && c.is_whitespace() {
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
    fn interp_hello() {
        let e = parse_value_or_interp("Hello `谁`!").unwrap();
        assert!(matches!(e, Expr::Interp(_)));
    }
}
