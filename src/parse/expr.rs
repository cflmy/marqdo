//! Expression parser (recursive descent over a single string).

use anyhow::{bail, Result};

use crate::ast::{BinaryOp, CallExpr, Expr, IndexKey, InterpPart, Literal, UnaryOp};

pub fn parse_expr(input: &str) -> Result<Expr> {
    parse_expr_mode(input, false)
}

/// Italic / bold value syntax: bare words are variables; text needs `"…"` / `'…'`.
pub fn parse_expr_prefer_var(input: &str) -> Result<Expr> {
    parse_expr_mode(input, true)
}

fn parse_expr_mode(input: &str, prefer_var: bool) -> Result<Expr> {
    parse_expr_opts(input, prefer_var, true)
}

/// Parse a full expression; `allow_paren_call` controls `name(args)` postfix calls.
fn parse_expr_opts(input: &str, prefer_var: bool, allow_paren_call: bool) -> Result<Expr> {
    let mut p = Parser::new_plain(input.trim(), prefer_var, allow_paren_call);
    let e = p.parse_or()?;
    p.skip_ws();
    if !p.rest().is_empty() {
        bail!("trailing input in expression: {:?}", p.rest());
    }
    Ok(e)
}

/// Argument / template RHS: prefer a full expression; else interpolated text.
/// Hyphenated prose (`none-falsy`) is not parsed as subtraction.
/// Quoted `"..."` always uses the expression parser (never the hyphen short-circuit).
pub fn parse_value_or_interp(input: &str) -> Result<Expr> {
    let s = input.trim();
    if s.is_empty() {
        return Ok(Expr::Literal(Literal::Text(String::new())));
    }
    if s.starts_with('"') || s.starts_with('\'') {
        return match parse_expr(s) {
            Ok(e) => Ok(e),
            Err(_) => Ok(parse_interp(s)),
        };
    }
    if has_hyphenated_prose(s) {
        return Ok(parse_interp(s));
    }
    match parse_expr(s) {
        Ok(e) => Ok(e),
        Err(_) => Ok(parse_interp(s)),
    }
}

/// Call-arg / param-default values: like [`parse_value_or_interp`], but also treat a chain of
/// unspaced `Text / Text` divisions as a single path string (`a/b`, `.marqdo/agent-kb`).
/// Real math (`1/2`, `` `n` / 2 ``) is unchanged.
pub fn parse_call_arg_value(input: &str) -> Result<Expr> {
    parse_call_arg_value_mode(input, false, false)
}

/// Shared call-argument value parsing.
///
/// `allow_call` is true only inside `callee(args)` (paren form), where arguments
/// are expression-like; classic space-separated values stay text-like so that
/// `source=print(2+2)` remains the text `print(2+2)`. `<TAB>` M1
fn parse_call_arg_value_mode(input: &str, prefer_var: bool, allow_call: bool) -> Result<Expr> {
    let s = input.trim();
    if s.is_empty() {
        return Ok(Expr::Literal(Literal::Text(String::new())));
    }
    if s.starts_with('"') || s.starts_with('\'') {
        return match parse_expr_opts(s, prefer_var, allow_call) {
            Ok(e) => Ok(e),
            Err(_) => Ok(parse_interp(s)),
        };
    }
    if has_hyphenated_prose(s) {
        return Ok(parse_interp(s));
    }
    match parse_expr_opts(s, prefer_var, allow_call) {
        Ok(e) => Ok(if prefer_var {
            e
        } else {
            collapse_text_div_path(e, s)
        }),
        Err(_) => {
            // ext/web paths: `nav.`nav``, `articles.`articles`.title` — ticks are path
            // punctuation, not interpolation. Plain `/about` is prose too.
            if prefer_var {
                if s.starts_with('/') || s.contains(".`") {
                    Ok(Expr::Literal(Literal::Text(s.to_string())))
                } else {
                    Ok(parse_interp(s))
                }
            } else if s.contains(".`") {
                Ok(Expr::Literal(Literal::Text(s.to_string())))
            } else {
                Ok(parse_interp(s))
            }
        }
    }
}

/// Like [`parse_call_arg_value`], but for call args inside `*…*` / `**…**`
/// value-expression segments: a bare word is a **variable** (not a text literal).
/// Quoted strings, hyphenated prose, and `/path` prose stay text; `` `x` `` still works.
pub fn parse_call_arg_value_prefer_var(input: &str) -> Result<Expr> {
    parse_call_arg_value_mode(input, true, false)
}

fn is_text_only_div_chain(e: &Expr) -> bool {
    match e {
        Expr::Literal(Literal::Text(_)) => true,
        Expr::Binary {
            op: BinaryOp::Div,
            left,
            right,
        } => is_text_only_div_chain(left) && matches!(right.as_ref(), Expr::Literal(Literal::Text(_))),
        _ => false,
    }
}

fn expr_has_div(e: &Expr) -> bool {
    match e {
        Expr::Binary {
            op: BinaryOp::Div, ..
        } => true,
        Expr::Binary { left, right, .. } => expr_has_div(left) || expr_has_div(right),
        Expr::Unary { expr, .. } => expr_has_div(expr),
        _ => false,
    }
}

fn collapse_text_div_path(e: Expr, raw: &str) -> Expr {
    if expr_has_div(&e) && is_text_only_div_chain(&e) {
        parse_interp(raw)
    } else {
        e
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
                let mut tail = &after[end + 1..];
                // `` `name`[^a][^b] `` → Index part (chained labels fold).
                if let Some(labels) = take_index_chain(&mut tail) {
                    parts.push(InterpPart::Index { base: name, labels });
                } else {
                    parts.push(InterpPart::Var(name));
                }
                rest = tail;
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
            InterpPart::Index { base, labels } => {
                let mut e = Expr::Var(base.clone());
                for label in labels {
                    e = Expr::Index {
                        base: Box::new(e),
                        label: IndexKey::Lit(label.clone()),
                    };
                }
                return e;
            }
        }
    }
    Expr::Interp(parts)
}

/// If `s` starts with one or more `[^label]` (no whitespace), consume them.
fn take_index_chain(s: &mut &str) -> Option<Vec<String>> {
    let mut labels = Vec::new();
    loop {
        if !s.starts_with("[^") {
            break;
        }
        let after_open = &s[2..];
        let Some(close) = after_open.find(']') else {
            break;
        };
        let label = after_open[..close].trim().to_string();
        if label.is_empty() {
            break;
        }
        labels.push(label);
        *s = &after_open[close + 1..];
    }
    if labels.is_empty() {
        None
    } else {
        Some(labels)
    }
}

struct Parser<'a> {
    src: &'a str,
    i: usize,
    /// When true (italic/bold), bare words are variables, not text literals.
    prefer_var: bool,
    /// `<TAB>` M1: allow `name(args)` postfix calls inside this expression.
    /// Classic (space-separated) call *values* stay text-like, so they turn
    /// this off to keep `source=print(2+2)` as text.
    allow_paren_call: bool,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str, prefer_var: bool) -> Self {
        Self {
            src,
            i: 0,
            prefer_var,
            allow_paren_call: true,
        }
    }

    fn new_plain(src: &'a str, prefer_var: bool, allow_paren_call: bool) -> Self {
        Self {
            src,
            i: 0,
            prefer_var,
            allow_paren_call,
        }
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

    /// Consume up to `max` hex digits; fewer (even zero) are allowed.
    /// Returns the parsed value, or an error if a digit exceeds the limit
    /// would overflow (`\xHH` requires exactly 2, `\u{...}` up to 6).
    fn take_hex_digits(&mut self, max: usize) -> Result<u64> {
        let mut value: u64 = 0;
        let mut count = 0;
        while count < max {
            let Some(c) = self.peek_char() else {
                break;
            };
            let Some(d) = c.to_digit(16) else {
                break;
            };
            value = value * 16 + u64::from(d);
            self.bump_char();
            count += 1;
        }
        Ok(value)
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
        // Link-shaped index `[key](collection)` vs bracket-marked call `[callee] …`
        // vs JSON-ish `[…]` (arg text fallback when expr parse fails).
        if self.starts_with("[") && !self.starts_with("[^") {
            if link_index_shape_ahead(self.rest()) {
                return self.parse_link_index();
            }
            if bracket_callee_shape_ahead(self.rest()) {
                let rest = self.rest();
                let prefer_var = self.prefer_var;
                let (call, consumed) = parse_call_tail(rest, prefer_var)?;
                self.i += consumed;
                return Ok(Expr::Call(call));
            }
            bail!("unexpected '[' in expression (not a link index or bracket call)");
        }
        if self.eat("\"") {
            return self.parse_quoted_string('"');
        }
        if self.eat("'") {
            return self.parse_quoted_string('\'');
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
            return self.parse_index_chain(Expr::Var(name));
        }
        // `礼貌 [问候] x` — after quotes/ticks so `[DONE]` inside strings is safe.
        if let Some((call, consumed)) = try_bracket_marked_call(self.rest(), self.prefer_var)? {
            self.i += consumed;
            return Ok(Expr::Call(call));
        }
        if self.eat(">") {
            // Inline call: `> name k=v`
            self.skip_ws();
            let prefer_var = self.prefer_var;
            return Ok(Expr::Call(parse_call_tail(self.rest(), prefer_var).map(
                |(c, consumed)| {
                    self.i += consumed;
                    c
                },
            )?));
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
        if self.peek_char().map(|c| c.is_ascii_digit()).unwrap_or(false)
            || (self.peek_char() == Some('.')
                && self
                    .rest()
                    .chars()
                    .nth(1)
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false))
        {
            let start = self.i;
            while self.peek_char().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                self.bump_char();
            }
            if self.peek_char() == Some('.') {
                self.bump_char();
                let frac_start = self.i;
                while self.peek_char().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    self.bump_char();
                }
                // Require at least one digit on one side of the dot.
                let int_part = &self.src[start..frac_start.saturating_sub(1)];
                let frac_part = &self.src[frac_start..self.i];
                if int_part.is_empty() && frac_part.is_empty() {
                    bail!("invalid number literal");
                }
                let n: f64 = self.src[start..self.i]
                    .parse()
                    .map_err(|e| anyhow::anyhow!("invalid float: {e}"))?;
                return Ok(Expr::Literal(Literal::Num(n)));
            }
            let n: i64 = self.src[start..self.i].parse()?;
            return Ok(Expr::Literal(Literal::Int(n)));
        }
        // Bare identifier / text word. If immediately followed by `[^…]`, treat as
        // variable + footnote index (exemption: ticks optional when `[^` marks the slot).
        if self.peek_char().map(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii()).unwrap_or(false)
        {
            let start = self.i;
            while let Some(c) = self.peek_char() {
                if c.is_whitespace()
                    || matches!(
                        c,
                        '=' | '+'
                            | '-'
                            | '*'
                            | '/'
                            | '<'
                            | '>'
                            | '('
                            | ')'
                            | '`'
                            | '|'
                            | ','
                            | '"'
                            | '\''
                            | '['
                            | ']'
                    )
                {
                    break;
                }
                self.bump_char();
            }
            let word = self.src[start..self.i].to_string();
            // `name(args)` postfix call: `(` must follow immediately. `<TAB>` M1
            if self.allow_paren_call && self.rest().starts_with('(') {
                let from_word = &self.src[start..];
                if let Some((call, used)) = try_paren_call(from_word, self.prefer_var)? {
                    self.i = start + used;
                    return Ok(Expr::Call(call));
                }
            }
            self.skip_ws();
            if self.starts_with("[^") {
                return self.parse_index_chain(Expr::Var(word));
            }
            if self.prefer_var {
                return Ok(Expr::Var(word));
            }
            return Ok(Expr::Literal(Literal::Text(word)));
        }
        bail!("unexpected expression input: {:?}", self.rest());
    }

    /// `` `xs`[^1] `` / `m[^key]` / chained `m[^k][^1]` (base already parsed).
    fn parse_index_chain(&mut self, mut expr: Expr) -> Result<Expr> {
        loop {
            self.skip_ws();
            if !self.starts_with("[^") {
                break;
            }
            let label = self.parse_footnote_label()?;
            expr = Expr::Index {
                base: Box::new(expr),
                label,
            };
        }
        Ok(expr)
    }

    /// `[key](collection)` — preferred collection get (GFM link shape).
    fn parse_link_index(&mut self) -> Result<Expr> {
        if !self.eat("[") {
            bail!("expected '[' for link index");
        }
        self.skip_ws();
        let label = if self.eat("`") {
            let start = self.i;
            while let Some(c) = self.peek_char() {
                if c == '`' {
                    break;
                }
                if c == '\n' || c == '\r' || c == ']' {
                    bail!("unterminated link-index var `[`…`](…)`");
                }
                self.bump_char();
            }
            let name = self.src[start..self.i].trim().to_string();
            if !self.eat("`") {
                bail!("unterminated link-index var `[`…`](…)`");
            }
            if name.is_empty() {
                bail!("empty link-index var `[`](…)`");
            }
            IndexKey::Var(name)
        } else {
            let start = self.i;
            while let Some(c) = self.peek_char() {
                if c == ']' {
                    break;
                }
                if c == '\n' || c == '\r' || c == '[' {
                    bail!("unterminated or nested link-index key `[…](…)`");
                }
                self.bump_char();
            }
            let key = self.src[start..self.i].trim().to_string();
            if key.is_empty() {
                bail!("empty link-index key `[](…)`");
            }
            IndexKey::Lit(key)
        };
        self.skip_ws();
        if !self.eat("]") {
            bail!("expected ']' after link-index key");
        }
        self.skip_ws();
        if !self.eat("(") {
            bail!("expected '(' collection after link-index `[key]`");
        }
        let base = self.parse_or()?;
        self.skip_ws();
        if !self.eat(")") {
            bail!("expected ')' after link-index collection");
        }
        let expr = Expr::Index {
            base: Box::new(base),
            label,
        };
        // Legacy: allow trailing footnote chain on the result.
        self.parse_index_chain(expr)
    }

    /// `"..."` or `'...'` with escapes and `` `var` `` interpolation.
    fn parse_quoted_string(&mut self, quote: char) -> Result<Expr> {
        use crate::ast::InterpPart;
        let mut parts: Vec<InterpPart> = Vec::new();
        let mut lit = String::new();
        loop {
            let Some(c) = self.peek_char() else {
                bail!("unterminated {quote} string");
            };
            if c == quote {
                self.bump_char();
                break;
            }
            if c == '\\' {
                self.bump_char();
                let esc = self
                    .peek_char()
                    .ok_or_else(|| anyhow::anyhow!("dangling \\ in string"))?;
                self.bump_char();
                match esc {
                    'n' => lit.push('\n'),
                    't' => lit.push('\t'),
                    'r' => lit.push('\r'),
                    '\\' => lit.push('\\'),
                    '"' => lit.push('"'),
                    '\'' => lit.push('\''),
                    'x' => {
                        let start = self.i;
                        let cp = self.take_hex_digits(2)?;
                        if self.i - start == 2 {
                            lit.push(
                                char::from_u32(cp as u32)
                                    .ok_or_else(|| anyhow::anyhow!("invalid \\x escape"))?,
                            );
                        } else {
                            // `\x` with fewer than two hex digits is not a valid
                            // escape: keep it verbatim.
                            self.i = start;
                            lit.push('\\');
                            lit.push('x');
                        }
                    }
                    'u' => {
                        if self.eat("{") {
                            let digits = self.take_hex_digits(6)?;
                            if !self.eat("}") {
                                bail!("unterminated \\u{{...}} escape in string");
                            }
                            lit.push(
                                char::from_u32(digits as u32)
                                    .ok_or_else(|| anyhow::anyhow!("invalid \\u escape"))?,
                            );
                        } else {
                            // `\u` without `{…}` is not a valid escape: keep it verbatim.
                            lit.push('\\');
                            lit.push('u');
                        }
                    }
                    // Unknown escapes are kept verbatim (`\q` → `\q`).
                    other => {
                        lit.push('\\');
                        lit.push(other);
                    }
                }
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
                // `` `name`[^a][^b] `` inside a quoted string → footnote index parts.
                if let Some(labels) = self.take_index_chain_in_string()? {
                    parts.push(InterpPart::Index { base: name, labels });
                } else {
                    parts.push(InterpPart::Var(name));
                }
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
                InterpPart::Index { base, labels } => {
                    let mut e = Expr::Var(base);
                    for label in labels {
                        e = Expr::Index {
                            base: Box::new(e),
                            label: IndexKey::Lit(label),
                        };
                    }
                    e
                }
            });
        }
        Ok(Expr::Interp(parts))
    }

    /// In quoted strings, consume `[^label]` chains following `` `var` ``.
    fn take_index_chain_in_string(&mut self) -> Result<Option<Vec<String>>> {
        let mut labels = Vec::new();
        loop {
            if !self.starts_with("[^") {
                break;
            }
            self.bump_char();
            self.bump_char(); // consume `[^`
            let start = self.i;
            while let Some(ch) = self.peek_char() {
                if ch == ']' {
                    break;
                }
                if ch == '\n' || ch == '\r' {
                    bail!("unterminated footnote index [^…] in string");
                }
                self.bump_char();
            }
            let label = self.src[start..self.i].trim().to_string();
            if !self.eat("]") {
                bail!("unterminated footnote index [^…] in string");
            }
            if label.is_empty() {
                bail!("empty footnote index `[^]` in string");
            }
            labels.push(label);
        }
        if labels.is_empty() {
            Ok(None)
        } else {
            Ok(Some(labels))
        }
    }

    /// Parse `[^label]` or `` [^`ident`] `` after a value.
    fn parse_footnote_label(&mut self) -> Result<IndexKey> {
        if !self.eat("[^") {
            bail!("expected [^ footnote index");
        }
        self.skip_ws();
        if self.eat("`") {
            let start = self.i;
            while let Some(c) = self.peek_char() {
                if c == '`' {
                    break;
                }
                if c == '\n' || c == '\r' || c == ']' {
                    bail!("unterminated footnote var `[^`…`]`");
                }
                self.bump_char();
            }
            let name = self.src[start..self.i].trim().to_string();
            if !self.eat("`") {
                bail!("unterminated footnote var `[^`…`]`");
            }
            self.skip_ws();
            if !self.eat("]") {
                bail!("unterminated footnote index [^…]");
            }
            if name.is_empty() {
                bail!("empty footnote var `[^`]`");
            }
            return Ok(IndexKey::Var(name));
        }
        let start = self.i;
        while let Some(c) = self.peek_char() {
            if c == ']' {
                break;
            }
            if c == '\n' || c == '\r' {
                bail!("unterminated footnote index [^…]");
            }
            self.bump_char();
        }
        let label = self.src[start..self.i].trim().to_string();
        if !self.eat("]") {
            bail!("unterminated footnote index [^…]");
        }
        if label.is_empty() {
            bail!("empty footnote index `[^]`");
        }
        Ok(IndexKey::Lit(label))
    }
}

fn is_word_end(s: &str, len: usize) -> bool {
    match s[len..].chars().next() {
        None => true,
        Some(c) => !(c.is_alphanumeric() || c == '_'),
    }
}

/// Parse `name key=val …` / positional args; returns call + bytes consumed from `s`
/// (after leading trim). Does **not** consume a trailing unmatched `)` so
/// `([callee] arg)` expression nesting works.
/// Callee may be a bare name, library path `time.parse`, or method `` `recv`.method ``.
/// Bracket-marked form: `mod… [callee] args…` (see bracket-call-modifiers.md).
/// When `prefer_var`, bare call-arg words parse as variables and bare dotted
/// callees are allowed to be method receivers.
pub fn parse_call_tail(s: &str, prefer_var: bool) -> Result<(CallExpr, usize)> {
    let s = s.trim_start();

    if let Some((call, consumed)) = try_bracket_marked_call(s, prefer_var)? {
        return Ok((call, consumed));
    }

    if let Some((call, consumed)) = try_paren_call(s, prefer_var)? {
        return Ok((call, consumed));
    }

    let callee_tok = {
        let mut parts = s.split_whitespace();
        parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("missing callee"))?
            .to_string()
    };

    let (receiver, callee, path) = split_callee(&callee_tok);

    let after_callee_offset = {
        let idx = s.find(&callee_tok).unwrap();
        idx + callee_tok.len()
    };
    let after = &s[after_callee_offset..];
    let args_src = after.trim_start();
    let ws = after.len() - args_src.len();
    let (args, args_consumed) = parse_call_args(args_src, prefer_var, false)?;

    Ok((
        CallExpr {
            callee,
            path,
            receiver,
            args,
            pre_modifiers: Vec::new(),
            bracket_marked: false,
        },
        after_callee_offset + ws + args_consumed,
    ))
}


/// Split a callee token into `(receiver, callee, lib path)`.
fn split_callee(callee_tok: &str) -> (Option<String>, String, Option<Vec<String>>) {
    if let Some((r, m)) = split_method_callee(callee_tok) {
        (Some(r), m, None)
    } else if let Some(segments) = split_lib_path_callee(callee_tok) {
        let last = segments.last().cloned().unwrap_or_default();
        (None, last, Some(segments))
    } else {
        (None, callee_tok.to_string(), None)
    }
}

/// `callee(args)` — callee is **everything before the first `(`** (spaces allowed).
///
/// Returns `None` when the shape is not a paren call (e.g. `print text=…`,
/// `text=(a)`), so callers can fall back to the classic token form. `<TAB>` M1.
fn try_paren_call(s: &str, prefer_var: bool) -> Result<Option<(CallExpr, usize)>> {
    let Some(open) = s.find('(') else {
        return Ok(None);
    };
    if open == 0 {
        return Ok(None);
    }
    let callee_tok = s[..open].trim();
    if callee_tok.is_empty() {
        return Ok(None);
    }
    // A named argument (`text=a(b)`) is not a paren call, and neither is a
    // bracket-marked argument (`trim [stdout](…)`): the callee side must not
    // contain `=`, quotes or brackets. Backticks stay legal so that the
    // backticked-receiver method form `` `obj`.方法(…) `` still reaches here.
    if callee_tok
        .chars()
        .any(|c| matches!(c, '=' | '[' | ']' | '(' | ')' | '"' | '\''))
    {
        return Ok(None);
    }
    let first = callee_tok.chars().next().unwrap_or(' ');
    if matches!(first, '[' | ']' | '"' | '\'') {
        return Ok(None);
    }
    let inside = &s[open + 1..];
    let (args, used) = parse_call_args(inside, prefer_var, true)?;
    let rest = &inside[used..];
    let rest_t = rest.trim_start();
    if !rest_t.starts_with(')') {
        return Ok(None);
    }
    let consumed = open + 1 + used + (rest.len() - rest_t.len()) + 1;
    let (receiver, callee, path) = split_callee(callee_tok);
    Ok(Some((
        CallExpr {
            callee,
            path,
            receiver,
            args,
            pre_modifiers: Vec::new(),
            bracket_marked: false,
        },
        consumed,
    )))
}

/// `mod… [callee] args` — `]` must not be immediately followed by `(` (that is link-index).
fn try_bracket_marked_call(s: &str, prefer_var: bool) -> Result<Option<(CallExpr, usize)>> {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'[' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'^' {
                i += 1;
                continue;
            }
            // Only treat as callee marker at start or after whitespace.
            if i > 0 && !s[..i].chars().last().map(|c| c.is_whitespace()).unwrap_or(false) {
                i += 1;
                continue;
            }
            let close = match s[i + 1..].find(']') {
                Some(rel) => i + 1 + rel,
                None => {
                    i += 1;
                    continue;
                }
            };
            let after = &s[close + 1..];
            if after.starts_with('(') {
                // `[key](coll)` link index — not a callee marker.
                i = close + 1;
                continue;
            }
            let pre = s[..i].trim();
            // Classic calls may embed `[…]` inside named/paren args
            // (`print text=(礼貌 [问候] x)`). Only treat as bracket-marked
            // when the prefix is pure modifiers (no `=` / `(` / quotes / `:`).
            if pre.contains('=')
                || pre.contains('(')
                || pre.contains('"')
                || pre.contains('\'')
                || pre.contains(':')
            {
                return Ok(None);
            }
            let inner = s[i + 1..close].trim();
            if inner.is_empty() {
                bail!("empty bracket callee `[]`");
            }
            if !is_bracket_callee_inner(inner) {
                i = close + 1;
                continue;
            }
            let pre_modifiers = match split_modifiers(pre) {
                Ok(ms) if ms.iter().all(|m| is_modifier_token(m)) => ms,
                _ => return Ok(None),
            };
            let callee_tok = if inner.starts_with('`') && inner.ends_with('`') && inner.len() >= 2 {
                inner.to_string()
            } else {
                inner.to_string()
            };
            let (receiver, callee, path) =
                if let Some((r, m)) = split_method_callee(&callee_tok) {
                    (Some(r), m, None)
                } else if let Some(segments) = split_lib_path_callee(&callee_tok) {
                    let last = segments.last().cloned().unwrap_or_default();
                    (None, last, Some(segments))
                } else {
                    let name = if callee_tok.starts_with('`')
                        && callee_tok.ends_with('`')
                        && callee_tok.len() >= 2
                    {
                        callee_tok[1..callee_tok.len() - 1].to_string()
                    } else {
                        callee_tok
                    };
                    (None, name, None)
                };
            let after_full = &s[close + 1..];
            let args_src = after_full.trim_start();
            let ws = after_full.len() - args_src.len();
            let (args, args_consumed) = parse_call_args(args_src, prefer_var, false)?;
            let consumed = close + 1 + ws + args_consumed;
            return Ok(Some((
                CallExpr {
                    callee,
                    path,
                    receiver,
                    args,
                    pre_modifiers,
                    bracket_marked: true,
                },
                consumed,
            )));
        }
        i += 1;
    }
    Ok(None)
}

fn split_modifiers(pre: &str) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let mut rest = pre.trim();
    while !rest.is_empty() {
        let (tok, next) = split_first_token(rest, false);
        if tok.is_empty() {
            break;
        }
        let name = if tok.starts_with('`') && tok.ends_with('`') && tok.len() >= 2 {
            tok[1..tok.len() - 1].to_string()
        } else {
            if tok.contains('=') {
                bail!("pre-bracket modifier cannot be a named arg (`{tok}`); put named args after `]`");
            }
            tok.to_string()
        };
        if name.is_empty() {
            bail!("empty pre-bracket modifier");
        }
        out.push(name);
        rest = next.trim_start();
    }
    Ok(out)
}

/// Parse call args; returns `(args, bytes_consumed)` from the start of `args_str`.
/// Stops before an unmatched `)` so parenthesized expression nesting works.
fn parse_call_args(
    args_str: &str,
    prefer_var: bool,
    paren_mode: bool,
) -> Result<(Vec<crate::ast::Arg>, usize)> {
    use crate::ast::Arg;
    let mut args = Vec::new();
    let mut seen_named = false;
    let mut rest = args_str;

    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() || rest.starts_with(')') {
            break;
        }
        // `<TAB>` M1: optional comma separator inside `callee(...)`.
        if paren_mode {
            while let Some(r) = rest.strip_prefix(',') {
                rest = r.trim_start();
            }
            if rest.is_empty() || rest.starts_with(')') {
                break;
            }
        }
        if let Some((key, after_eq)) = try_named_arg(rest) {
            seen_named = true;
            let val_end = find_next_arg_boundary(after_eq, paren_mode);
            let val_raw = after_eq[..val_end].trim_end();
            let value = if prefer_var {
                parse_call_arg_value_mode(val_raw, prefer_var, paren_mode)?
            } else {
                parse_call_arg_value_mode(val_raw, prefer_var, paren_mode)?
            };
            args.push(Arg::Named {
                name: key,
                value,
            });
            rest = after_eq[val_end..].trim_start();
        } else {
            if seen_named {
                bail!("positional argument after named argument is not allowed");
            }
            let (tok, next) = split_first_token(rest, paren_mode);
            if tok.is_empty() {
                break;
            }
            let value = if prefer_var {
                parse_call_arg_value_mode(tok, prefer_var, paren_mode)?
            } else {
                parse_call_arg_value_mode(tok, prefer_var, paren_mode)?
            };
            args.push(Arg::Positional(value));
            rest = next.trim_start();
        }
    }
    let consumed = args_str.len() - rest.len();
    Ok((args, consumed))
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

/// Split one call-argument token off `s`.
///
/// Stops at a **top-level** whitespace, `)`, and (when `stop_comma`) `,`.
/// Tracks `()` `[]` `{}` nesting plus quoted strings and backticks, so nested
/// calls (`f(g(1))`) and JSON-ish values stay one token. `<TAB>` M1.
fn split_first_token(s: &str, paren_mode: bool) -> (&str, &str) {
    let s = s.trim_start();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut in_bt = false;
    let mut chars = s.char_indices().peekable();
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
        match c {
            '`' => {
                in_bt = true;
                continue;
            }
            '"' => {
                in_str = true;
                continue;
            }
            '(' | '[' | '{' => {
                depth += 1;
                continue;
            }
            ')' | ']' | '}' => {
                if depth == 0 {
                    // Top-level closer terminates the token and is NOT consumed.
                    return (&s[..i], &s[i..]);
                }
                depth -= 1;
                continue;
            }
            ',' if paren_mode && depth == 0 => return (&s[..i], &s[i..]),
            c if depth == 0 && c.is_whitespace() => return (&s[..i], &s[i..]),
            _ => {}
        }
    }
    (s, "")
}

/// `[key](…)` with no whitespace between `]` and `(`.
fn link_index_shape_ahead(s: &str) -> bool {
    let s = s.trim_start();
    if !s.starts_with('[') || s.starts_with("[^") {
        return false;
    }
    let Some(close_rel) = s[1..].find(']') else {
        return false;
    };
    let after = &s[1 + close_rel + 1..];
    after.starts_with('(')
}

/// `[callee] …` (not link-index): inner must look like a function/method name, not JSON.
fn bracket_callee_shape_ahead(s: &str) -> bool {
    let s = s.trim_start();
    if !s.starts_with('[') || s.starts_with("[^") {
        return false;
    }
    let Some(close_rel) = s[1..].find(']') else {
        return false;
    };
    let after = &s[1 + close_rel + 1..];
    if after.starts_with('(') {
        return false;
    }
    is_bracket_callee_inner(&s[1..1 + close_rel])
}

fn is_bracket_callee_inner(inner: &str) -> bool {
    let t = inner.trim();
    if t.is_empty() {
        return false;
    }
    // JSON arrays / objects / strings are never callees.
    if t.contains(',')
        || t.contains(':')
        || t.contains('{')
        || t.contains('}')
        || t.contains('[')
        || t.contains(']')
        || t.starts_with('"')
        || t.starts_with('\'')
    {
        return false;
    }
    if t.starts_with('`') {
        return split_method_callee(t).is_some()
            || (t.ends_with('`') && t.len() >= 2 && !t[1..t.len() - 1].contains('`'));
    }
    if t.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return false;
    }
    // Bare name or `lib.path` callee. Internal spaces allowed: `[加一 号] 31`
    // is the bracket-marked form of a callee whose name contains spaces (D8).
    t.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '.' || c == ' ' || !c.is_ascii())
}

fn is_modifier_token(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii())
}

fn find_next_arg_boundary(after_eq: &str, paren_mode: bool) -> usize {
    // Look for ` <ident>=` pattern not inside backticks, quoted strings, or
    // JSON-ish `[…]` / `{…}` (GAP-07: `text=["a"] other=…`).
    let mut in_bt = false;
    let mut in_str = false;
    let mut bracket_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut paren_depth: i32 = 0;
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
        match c {
            '`' => {
                in_bt = true;
                continue;
            }
            '"' => {
                in_str = true;
                continue;
            }
            '[' => {
                bracket_depth += 1;
                continue;
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                continue;
            }
            '{' => {
                brace_depth += 1;
                continue;
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                continue;
            }
            '(' => {
                paren_depth += 1;
                continue;
            }
            ')' => {
                if paren_depth == 0 {
                    // `<TAB>` M1: `)` ends the enclosing call's argument list.
                    return i;
                }
                paren_depth -= 1;
                continue;
            }
            ',' if paren_mode && paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return i;
            }
            _ => {}
        }
        if bracket_depth > 0 || brace_depth > 0 || paren_depth > 0 {
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


/// Parse `> callee args` call expression/statement text (without leading `>`).
/// When `prefer_var` (inside `*…*` / `**…**`), bare call-arg words are variables
/// and a bare dotted callee may be a method receiver (resolved at eval/compile).
pub fn parse_call_after_gt(after_gt: &str, prefer_var: bool) -> Result<CallExpr> {
    let (call, _) = parse_call_tail(after_gt.trim(), prefer_var)?;
    Ok(call)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_var_footnote_index() {
        let e = parse_expr("ev[^result]").unwrap();
        match e {
            Expr::Index { base, label } => {
                assert!(matches!(base.as_ref(), Expr::Var(n) if n == "ev"));
                assert_eq!(label, IndexKey::Lit("result".into()));
            }
            other => panic!("expected Index, got {other:?}"),
        }
        let e = parse_expr("data[^choices][^1][^message]").unwrap();
        assert!(matches!(e, Expr::Index { .. }));
        // Without `[^`, bare word stays text literal.
        let e = parse_expr("ev").unwrap();
        assert!(matches!(e, Expr::Literal(Literal::Text(ref s)) if s == "ev"));
        // Ticked form still works.
        let e = parse_expr("`ev`[^result]").unwrap();
        assert!(matches!(e, Expr::Index { .. }));
    }

    #[test]
    fn prefer_var_bare_id_and_quotes() {
        let e = parse_expr_prefer_var("answer").unwrap();
        assert!(matches!(e, Expr::Var(ref n) if n == "answer"));
        let e = parse_expr_prefer_var("n + 1").unwrap();
        assert!(matches!(e, Expr::Binary { op: BinaryOp::Add, .. }));
        let e = parse_expr_prefer_var("\"ok\"").unwrap();
        assert!(matches!(e, Expr::Literal(Literal::Text(ref s)) if s == "ok"));
        let e = parse_expr_prefer_var("'hi'").unwrap();
        assert!(matches!(e, Expr::Literal(Literal::Text(ref s)) if s == "hi"));
        let e = parse_expr_prefer_var("`answer`").unwrap();
        assert!(matches!(e, Expr::Var(ref n) if n == "answer"));
    }

    #[test]
    fn call_arg_module_path_stays_text() {
        let e = parse_call_arg_value("nav.`nav`").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "nav.`nav`"
        ));
        let e = parse_call_arg_value("articles.`articles`.title").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "articles.`articles`.title"
        ));
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
    fn quoted_string_hex_unicode_escapes() {
        // `\x22` → `"` (hex escape)
        let e = parse_expr("\"a\\x22b\"").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "a\"b"
        ));
        // `\x1F` → control char
        let e = parse_expr("\"\\x1F\"").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "\u{1f}"
        ));
        // `\u{4e2d}` → 中
        let e = parse_expr("\"\\u{4e2d}\"").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "中"
        ));
        // `\u{1F600}` → 😀
        let e = parse_expr("\"\\u{1F600}\"").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "😀"
        ));
        // `\x22` inside a string that also interpolates
        let e = parse_expr("\"x=\\x22`v`\\x22\"").unwrap();
        assert!(matches!(e, Expr::Interp(_)));
        // `\x` without two hex digits is kept verbatim (`\x` → `\x`)
        let e = parse_expr("\"a\\xb\"").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "a\\xb"
        ));
        // `\u` without braces is kept verbatim (`\u` → `\u`)
        let e = parse_expr("\"a\\ub\"").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "a\\ub"
        ));
    }

    #[test]
    fn quoted_string_unknown_escape_kept_verbatim() {
        // Unknown escapes like `\q` are kept as-is rather than failing.
        let e = parse_expr("\"a\\qb\"").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "a\\qb"
        ));
    }

    #[test]
    fn call_arg_quoted_path_keeps_slashes() {
        let e = parse_call_arg_value("\".marqdo/agent-runs\"").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == ".marqdo/agent-runs"
        ));
        let e = parse_value_or_interp("\".marqdo/agent-kb\"").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == ".marqdo/agent-kb"
        ));
    }

    #[test]
    fn call_arg_bare_path_not_division() {
        let e = parse_call_arg_value("a/b/c").unwrap();
        assert!(matches!(
            e,
            Expr::Literal(Literal::Text(ref s)) if s == "a/b/c"
        ));
        let e = parse_call_arg_value("1/2").unwrap();
        assert!(matches!(
            e,
            Expr::Binary {
                op: BinaryOp::Div,
                ..
            }
        ));
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

    #[test]
    fn interp_footnote_index_after_var() {
        // `前缀`x`[^key]` → Interp([Lit, Index])
        let e = parse_interp("前缀`m`[^苹果]");
        match &e {
            Expr::Interp(parts) => {
                assert!(matches!(&parts[0], InterpPart::Lit(t) if t == "前缀"));
                assert!(matches!(
                    &parts[1],
                    InterpPart::Index { base, labels } if base == "m" && labels == &["苹果".to_string()]
                ));
            }
            other => panic!("expected Interp, got {other:?}"),
        }
        // Bare `m`[^key] with no prefix → Expr::Index
        let e = parse_interp("`m`[^苹果]");
        assert!(matches!(
            e,
            Expr::Index {
                label: IndexKey::Lit(ref s),
                ..
            } if s == "苹果"
        ));
        // Chained labels fold into one Index part with multiple labels.
        let e = parse_interp("`m`[^a][^b]");
        match &e {
            Expr::Index { base, label } => {
                assert!(matches!(
                    base.as_ref(),
                    Expr::Index {
                        label: IndexKey::Lit(ref l),
                        ..
                    } if l == "a"
                ));
                assert_eq!(label, &IndexKey::Lit("b".into()));
            }
            other => panic!("expected chained Index, got {other:?}"),
        }
        // `x`[^key] with plain Var text when no `[^` follows.
        let e = parse_interp("`m` 和 `n`");
        assert!(matches!(e, Expr::Interp(_)));
    }

    #[test]
    fn interp_footnote_index_keeps_text_between() {
        // `x` followed by whitespace then [^...] → [^...] stays literal text.
        let e = parse_interp("`m` [^苹果]");
        match &e {
            Expr::Interp(parts) => {
                assert!(matches!(&parts[0], InterpPart::Var(n) if n == "m"));
                assert!(matches!(&parts[1], InterpPart::Lit(t) if t == " [^苹果]"));
            }
            other => panic!("expected Interp, got {other:?}"),
        }
    }

    #[test]
    fn float_literal_and_dynamic_footnote_key() {
        let e = parse_expr("0.85").unwrap();
        assert!(matches!(e, Expr::Literal(Literal::Num(n)) if (n - 0.85).abs() < 1e-12));
        let e = parse_expr("28 * 0.85").unwrap();
        assert!(matches!(e, Expr::Binary { op: BinaryOp::Mul, .. }));
        let e = parse_expr_prefer_var("菜单[^`名`]").unwrap();
        match e {
            Expr::Index { base, label } => {
                assert!(matches!(base.as_ref(), Expr::Var(n) if n == "菜单"));
                assert_eq!(label, IndexKey::Var("名".into()));
            }
            other => panic!("expected dynamic Index, got {other:?}"),
        }
        let e = parse_expr("m[^拿铁]").unwrap();
        assert!(matches!(
            e,
            Expr::Index {
                label: IndexKey::Lit(ref s),
                ..
            } if s == "拿铁"
        ));
    }

    #[test]
    fn link_shaped_index() {
        let e = parse_expr_prefer_var("[拿铁](菜单)").unwrap();
        match e {
            Expr::Index { base, label } => {
                assert!(matches!(base.as_ref(), Expr::Var(n) if n == "菜单"));
                assert_eq!(label, IndexKey::Lit("拿铁".into()));
            }
            other => panic!("expected link Index, got {other:?}"),
        }
        let e = parse_expr_prefer_var("[`名`](菜单)").unwrap();
        match e {
            Expr::Index { base, label } => {
                assert!(matches!(base.as_ref(), Expr::Var(n) if n == "菜单"));
                assert_eq!(label, IndexKey::Var("名".into()));
            }
            other => panic!("expected dynamic link Index, got {other:?}"),
        }
        let e = parse_expr_prefer_var("[1](篮子)").unwrap();
        assert!(matches!(
            e,
            Expr::Index {
                label: IndexKey::Lit(ref s),
                ..
            } if s == "1"
        ));
        let e = parse_expr_prefer_var("[1]([苹果](货架))").unwrap();
        match e {
            Expr::Index { base, label } => {
                assert_eq!(label, IndexKey::Lit("1".into()));
                assert!(matches!(
                    base.as_ref(),
                    Expr::Index {
                        label: IndexKey::Lit(ref k),
                        ..
                    } if k == "苹果"
                ));
            }
            other => panic!("expected nested link Index, got {other:?}"),
        }
    }
}



#[cfg(test)]
mod bracket_call_tests {
    use super::*;
    use crate::ast::{Arg, Expr};

    #[test]
    fn bracket_with_modifier() {
        let (c, _) = parse_call_tail(r#"礼貌 [问候] "Marqdo""#, true).unwrap();
        assert_eq!(c.callee, "问候");
        assert_eq!(c.pre_modifiers, vec!["礼貌".to_string()]);
        assert!(c.bracket_marked);
        assert_eq!(c.args.len(), 1);
    }

    #[test]
    fn nested_paren_arg() {
        let (c, _) = parse_call_tail(r#"[问候] ([格式化] "x")"#, true).unwrap();
        assert_eq!(c.callee, "问候");
        assert!(c.bracket_marked);
        match &c.args[0] {
            Arg::Positional(Expr::Call(inner)) => {
                assert_eq!(inner.callee, "格式化");
                assert!(inner.bracket_marked);
            }
            other => panic!("expected nested call, got {:?}", other),
        }
    }

    #[test]
    fn paren_expr_bracket_call() {
        let e = parse_expr_prefer_var(r#"([文本] "hi")"#).unwrap();
        match e {
            Expr::Call(c) => assert_eq!(c.callee, "文本"),
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn modifier_in_paren_expr() {
        let e = parse_expr_prefer_var(r#"(礼貌 [问候] "Ada")"#).unwrap();
        match e {
            Expr::Call(c) => {
                assert_eq!(c.callee, "问候");
                assert_eq!(c.pre_modifiers, vec!["礼貌".to_string()]);
            }
            other => panic!("{:?}", other),
        }
    }

    #[test]
    fn link_index_not_bracket_call() {
        let e = parse_expr_prefer_var("[拿铁](菜单)").unwrap();
        assert!(matches!(e, Expr::Index { .. }));
    }

    // ---- M1: `()` 内调用语法（`)` 终止 / 逗号 / 嵌套 / 空格 callee）----

    fn call_of(e: Expr) -> crate::ast::CallExpr {
        match e {
            Expr::Call(c) => c,
            other => panic!("expected Call, got {other:?}"),
        }
    }

    #[test]
    fn paren_call_single_positional() {
        let (c, used) = parse_call_tail("加(1)", false).unwrap();
        assert_eq!(c.callee, "加");
        assert_eq!(used, "加(1)".len());
        assert_eq!(c.args.len(), 1);
    }

    #[test]
    fn paren_call_bare_number_stops_at_paren() {
        // 历史缺陷：末个位置实参会吞掉 `)`
        let (c, used) = parse_call_tail("加(31)", false).unwrap();
        assert_eq!(used, "加(31)".len(), "consumed must cover the closing paren");
        assert_eq!(c.args.len(), 1);
    }

    #[test]
    fn paren_call_comma_and_space_separated() {
        for src in ["加(1, 2)", "加(1 2)", "加(1,2)", "加( 1 , 2 )"] {
            let (c, _) = parse_call_tail(src, false).unwrap();
            assert_eq!(c.args.len(), 2, "src={src}");
        }
    }

    #[test]
    fn paren_call_named_args() {
        let (c, _) = parse_call_tail("计税(档位=2, 金额=100)", false).unwrap();
        assert_eq!(c.callee, "计税");
        assert_eq!(c.args.len(), 2);
        match &c.args[0] {
            crate::ast::Arg::Named { name, .. } => assert_eq!(name, "档位"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn paren_call_callee_may_contain_spaces() {
        let (c, used) = parse_call_tail("加一 号(31)", false).unwrap();
        assert_eq!(c.callee, "加一 号");
        assert_eq!(used, "加一 号(31)".len());
        assert_eq!(c.args.len(), 1);
    }

    #[test]
    fn paren_call_zero_args() {
        let (c, _) = parse_call_tail("刷新()", false).unwrap();
        assert_eq!(c.callee, "刷新");
        assert!(c.args.is_empty());
    }

    #[test]
    fn paren_call_lib_path_and_method() {
        let c = call_of(parse_expr("txt.str_trim(s)").unwrap());
        assert_eq!(c.callee, "str_trim");
        assert_eq!(c.path.as_deref(), Some(&["txt".to_string(), "str_trim".to_string()][..]));

        // 点号 callee 在 parse 期保存为 lib path；若 `obj` 是局部变量，
        // 运行期 eval_call 会把它当 method receiver（与 `> obj.方法` 一致）。
        let c = call_of(parse_expr("obj.方法(1)").unwrap());
        assert_eq!(c.callee, "方法");
        assert_eq!(
            c.path.as_deref(),
            Some(&["obj".to_string(), "方法".to_string()][..])
        );
        assert!(c.receiver.is_none());
    }

    #[test]
    fn paren_call_nested_in_expression() {
        let c = call_of(parse_expr("加(加(1 2), 3)").unwrap());
        assert_eq!(c.args.len(), 2);
        match &c.args[0] {
            crate::ast::Arg::Positional(Expr::Call(inner)) => {
                assert_eq!(inner.callee, "加");
                assert_eq!(inner.args.len(), 2);
            }
            other => panic!("expected nested call, got {other:?}"),
        }
    }

    #[test]
    fn paren_call_quoted_args_keep_commas() {
        // 逗号只在括号形里当分隔符；字符串内的逗号不受影响
        let (c, _) = parse_call_tail(r#"拼接("a,b", "c")"#, false).unwrap();
        assert_eq!(c.args.len(), 2);
    }

    #[test]
    fn whitespace_call_unaffected_by_commas() {
        // 旧式空格调用：`text=a,b` 的逗号仍是文本一部分
        let (c, _) = parse_call_tail("print text=a,b", false).unwrap();
        match &c.args[0] {
            crate::ast::Arg::Named { value, .. } => match value {
                Expr::Literal(Literal::Text(s)) => assert_eq!(s, "a,b"),
                other => panic!("expected text a,b, got {other:?}"),
            },
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn bracketed_call_still_works() {
        let (c, _) = parse_call_tail("[加一 号] 31", false).unwrap();
        assert_eq!(c.callee, "加一 号");
    }

    #[test]
    fn modifier_conflict_expands() {
        let (c, _) = parse_call_tail("礼貌 [问候] 礼貌=False", false).unwrap();
        let err = c.with_modifiers_expanded().unwrap_err();
        assert!(err.contains("conflicts"));
    }
}
