//! Expression parser (recursive descent over a single string).

use anyhow::{bail, Result};

use crate::ast::{BinaryOp, CallExpr, Expr, InterpPart, Literal, UnaryOp};

pub fn parse_expr(input: &str) -> Result<Expr> {
    parse_expr_mode(input, false)
}

/// Italic / bold value syntax: bare words are variables; text needs `"…"` / `'…'`.
pub fn parse_expr_prefer_var(input: &str) -> Result<Expr> {
    parse_expr_mode(input, true)
}

fn parse_expr_mode(input: &str, prefer_var: bool) -> Result<Expr> {
    let mut p = Parser::new(input.trim(), prefer_var);
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
        Ok(e) => Ok(collapse_text_div_path(e, s)),
        Err(_) => {
            // ext/web paths: `nav.`nav``, `articles.`articles`.title` — ticks are path
            // punctuation, not interpolation.
            if s.contains(".`") {
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
    match parse_expr_prefer_var(s) {
        Ok(e) => Ok(e),
        Err(_) => {
            // `/about`, `.marqdo/x`, `nav.`nav`` etc. → prose text, not variables.
            if s.starts_with('/') || s.contains(".`") {
                Ok(Expr::Literal(Literal::Text(s.to_string())))
            } else {
                Ok(parse_interp(s))
            }
        }
    }
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
                        label: label.clone(),
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
}

impl<'a> Parser<'a> {
    fn new(src: &'a str, prefer_var: bool) -> Self {
        Self {
            src,
            i: 0,
            prefer_var,
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
        if self.peek_char().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            let start = self.i;
            while self.peek_char().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                self.bump_char();
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
                            label,
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

    /// Parse `[^label]` after a value; label is text until `]`.
    fn parse_footnote_label(&mut self) -> Result<String> {
        if !self.eat("[^") {
            bail!("expected [^ footnote index");
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
        Ok(label)
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
/// When `prefer_var`, bare call-arg words parse as variables and bare dotted
/// callees are allowed to be method receivers.
pub fn parse_call_tail(s: &str, prefer_var: bool) -> Result<(CallExpr, usize)> {
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
            let value = if prefer_var {
                parse_call_arg_value_prefer_var(val_raw)?
            } else {
                parse_call_arg_value(val_raw)?
            };
            args.push(Arg::Named {
                name: key,
                value,
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
            let value = if prefer_var {
                parse_call_arg_value_prefer_var(tok)?
            } else {
                parse_call_arg_value(tok)?
            };
            args.push(Arg::Positional(value));
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
                assert_eq!(label, "result");
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
        assert!(matches!(e, Expr::Index { label, .. } if label == "苹果"));
        // Chained labels fold into one Index part with multiple labels.
        let e = parse_interp("`m`[^a][^b]");
        match &e {
            Expr::Index { base, label } => {
                assert!(matches!(base.as_ref(), Expr::Index { label: l, .. } if l == "a"));
                assert_eq!(label, "b");
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
}


