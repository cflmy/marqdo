//! Artifact Metadata Binding (`${ns.name}`) — Phase 1: frontmatter only.
//!
//! Spec: `doc/design/binding.md` · ADR 0006.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::diagnostics::{bail_at, Span};
use crate::value::Value;

/// Unresolved frontmatter value (literal, sole binding, or interpolated text).
#[derive(Debug, Clone)]
pub enum MetaValue {
    /// Already typed literal (no `${}`).
    Literal(Value),
    /// Whole value is one `${…}` expression.
    Binding(BindingExpr),
    /// Quoted/bare string containing `${…}` interpolations → Text after resolve.
    Interpolated(String),
}

/// Parsed binding expression (inside `${…}`).
#[derive(Debug, Clone)]
pub struct BindingExpr {
    pub cast: Option<CastKind>,
    pub primary: BindingRef,
    pub default: Option<DefaultVal>,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastKind {
    Int,
    Float,
    Bool,
}

#[derive(Debug, Clone)]
pub struct BindingRef {
    pub ns: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum DefaultVal {
    Text(String),
    /// Nested `${…}` not supported in Phase 1 defaults; only quoted/bare literals.
    Binding(Box<BindingExpr>),
}

/// Sources available while resolving metadata at load time.
#[derive(Debug, Clone, Default)]
pub struct BindingContext {
    /// `--bind KEY=VALUE` → `arg.KEY` and post-resolve overlay.
    pub args: HashMap<String, String>,
    pub cwd: PathBuf,
}

impl BindingContext {
    pub fn from_binds(binds: &[(String, String)], cwd: PathBuf) -> Self {
        let mut args = HashMap::new();
        for (k, v) in binds {
            args.insert(k.clone(), v.clone());
        }
        Self { args, cwd }
    }
}

/// Parse a frontmatter RHS into a [`MetaValue`].
pub fn parse_meta_value(raw: &str) -> Result<MetaValue> {
    let t = raw.trim();
    if t.is_empty() {
        return Ok(MetaValue::Literal(Value::Text(String::new())));
    }

    // Sole binding: entire value is `${…}`
    if let Some(inner) = sole_binding(t) {
        let expr = parse_binding_expr(inner)?;
        return Ok(MetaValue::Binding(expr));
    }

    // Quoted string (may interpolate)
    if (t.starts_with('"') && t.ends_with('"') && t.len() >= 2)
        || (t.starts_with('\'') && t.ends_with('\'') && t.len() >= 2)
    {
        let inner = &t[1..t.len() - 1];
        if inner.contains("${") {
            return Ok(MetaValue::Interpolated(unquote_escapes(inner)));
        }
        return Ok(MetaValue::Literal(Value::Text(unquote_escapes(inner))));
    }

    // Bare with interpolation
    if t.contains("${") {
        return Ok(MetaValue::Interpolated(t.to_string()));
    }

    Ok(MetaValue::Literal(parse_scalar_literal(t)))
}

fn sole_binding(t: &str) -> Option<&str> {
    let t = t.trim();
    if !t.starts_with("${") || !t.ends_with('}') {
        return None;
    }
    // Ensure matching: first `${` closes at last `}` and nothing outside
    let inner = &t[2..t.len() - 1];
    // Reject if another `${` appears that would mean concatenation (not sole)
    // Allow nested parens inside cast: ${int(env.X ?? "1")}
    Some(inner)
}

fn unquote_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('\'') => out.push('\''),
                Some(o) => {
                    out.push('\\');
                    out.push(o);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn parse_scalar_literal(t: &str) -> Value {
    match t {
        "true" | "True" | "TRUE" => Value::Bool(true),
        "false" | "False" | "FALSE" => Value::Bool(false),
        "null" | "Null" | "None" | "~" => Value::None,
        _ => {
            if let Ok(n) = t.parse::<i64>() {
                return Value::Int(n);
            }
            if let Ok(n) = t.parse::<f64>() {
                if !n.is_nan() && !n.is_infinite() {
                    return Value::Num(n);
                }
            }
            Value::Text(t.to_string())
        }
    }
}

/// Parse inside `${…}` (without braces).
pub fn parse_binding_expr(inner: &str) -> Result<BindingExpr> {
    let s = inner.trim();
    // cast: int(…) / float(…) / bool(…)
    if let Some((cast, rest)) = parse_cast_wrap(s)? {
        let mut expr = parse_binding_expr(rest)?;
        if expr.cast.is_some() {
            bail!("nested casts are not allowed in binding expressions");
        }
        expr.cast = Some(cast);
        return Ok(expr);
    }

    let (body, required) = if let Some(b) = s.strip_suffix('!') {
        (b.trim(), true)
    } else {
        (s, false)
    };

    let (primary_s, default) = if let Some((left, right)) = split_default(body) {
        (left.trim(), Some(parse_default_val(right.trim())?))
    } else {
        (body, None)
    };

    if required && default.is_some() {
        bail!("binding cannot combine `!` with `??` default");
    }

    let primary = parse_binding_ref(primary_s)?;
    Ok(BindingExpr {
        cast: None,
        primary,
        default,
        required,
    })
}

fn parse_cast_wrap(s: &str) -> Result<Option<(CastKind, &str)>> {
    for (name, kind) in [
        ("int", CastKind::Int),
        ("float", CastKind::Float),
        ("bool", CastKind::Bool),
    ] {
        let prefix = format!("{name}(");
        if let Some(rest) = s.strip_prefix(&prefix) {
            if let Some(inner) = strip_trailing_paren(rest) {
                return Ok(Some((kind, inner)));
            }
            bail!("unclosed `{name}(` in binding expression");
        }
    }
    Ok(None)
}

fn strip_trailing_paren(s: &str) -> Option<&str> {
    let s = s.trim_end();
    if !s.ends_with(')') {
        return None;
    }
    Some(s[..s.len() - 1].trim())
}

fn split_default(s: &str) -> Option<(&str, &str)> {
    // Split on top-level `??`
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut depth = 0i32;
    while i + 1 < bytes.len() {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'?' if depth == 0 && bytes[i + 1] == b'?' => {
                return Some((&s[..i], &s[i + 2..]));
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn parse_default_val(s: &str) -> Result<DefaultVal> {
    let s = s.trim();
    if let Some(inner) = sole_binding(s) {
        return Ok(DefaultVal::Binding(Box::new(parse_binding_expr(inner)?)));
    }
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        return Ok(DefaultVal::Text(unquote_escapes(&s[1..s.len() - 1])));
    }
    // bare default token
    Ok(DefaultVal::Text(s.to_string()))
}

fn parse_binding_ref(s: &str) -> Result<BindingRef> {
    let s = s.trim();
    let Some((ns, name)) = s.split_once('.') else {
        bail!("binding path must be `namespace.name` (got `{s}`)");
    };
    let ns = ns.trim();
    let name = name.trim();
    if ns.is_empty() || name.is_empty() {
        bail!("binding path must be `namespace.name` (got `{s}`)");
    }
    match ns {
        "env" | "arg" | "sys" | "secret" => {}
        "config" | "file" => {
            bail!("binding namespace `{ns}` is not available in Phase 1 (Metadata-only)")
        }
        other => bail!("unknown binding namespace `{other}`"),
    }
    if name.contains('.') {
        // Phase 1: single segment name only
        bail!("binding name must be a single segment (got `{s}`)");
    }
    Ok(BindingRef {
        ns: ns.to_string(),
        name: name.to_string(),
    })
}

/// Resolve raw metadata entries; then overlay `--bind` keys.
pub fn resolve_metadata(
    raw: &[(String, MetaValue)],
    ctx: &BindingContext,
    span: Span,
) -> Result<Vec<(String, Value)>> {
    let mut out: Vec<(String, Value)> = Vec::with_capacity(raw.len());
    for (k, mv) in raw {
        let v = resolve_meta_value(mv, ctx, span)
            .map_err(|e| bail_at(None, span, e.to_string()))?;
        out.push((k.clone(), v));
    }
    // CLI overlay
    for (k, v) in &ctx.args {
        if let Some(slot) = out.iter_mut().find(|(mk, _)| mk == k) {
            slot.1 = Value::Text(v.clone());
        } else {
            out.push((k.clone(), Value::Text(v.clone())));
        }
    }
    Ok(out)
}

fn resolve_meta_value(mv: &MetaValue, ctx: &BindingContext, span: Span) -> Result<Value> {
    match mv {
        MetaValue::Literal(v) => Ok(v.clone()),
        MetaValue::Binding(expr) => resolve_expr(expr, ctx, span),
        MetaValue::Interpolated(s) => {
            let text = interpolate(s, ctx, span)?;
            Ok(Value::Text(text))
        }
    }
}

fn interpolate(s: &str, ctx: &BindingContext, span: Span) -> Result<String> {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = find_binding_close(after).ok_or_else(|| {
            anyhow::anyhow!("{}:{}: unclosed `${{` in metadata interpolation", span.line, span.col)
        })?;
        let inner = &after[..end];
        let expr = parse_binding_expr(inner)?;
        let v = resolve_expr(&expr, ctx, span)?;
        out.push_str(&v.as_display());
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn find_binding_close(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            '}' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

fn resolve_expr(expr: &BindingExpr, ctx: &BindingContext, span: Span) -> Result<Value> {
    let got = lookup(&expr.primary, ctx)?;
    let value = match got {
        Some(v) => v,
        None if expr.required => {
            bail!(
                "required binding `{}.{}` is missing",
                expr.primary.ns,
                expr.primary.name
            )
        }
        None => match &expr.default {
            Some(DefaultVal::Text(t)) => Value::Text(t.clone()),
            Some(DefaultVal::Binding(inner)) => resolve_expr(inner, ctx, span)?,
            None => Value::None,
        },
    };
    apply_cast(expr.cast, value)
}

fn lookup(r: &BindingRef, ctx: &BindingContext) -> Result<Option<Value>> {
    match r.ns.as_str() {
        "env" => Ok(std::env::var(&r.name).ok().map(Value::Text)),
        "arg" => Ok(ctx.args.get(&r.name).cloned().map(Value::Text)),
        "secret" => Ok(std::env::var(&r.name).ok().map(Value::Secret)),
        "sys" => Ok(lookup_sys(&r.name, ctx)),
        other => bail!("unknown binding namespace `{other}`"),
    }
}

fn lookup_sys(name: &str, ctx: &BindingContext) -> Option<Value> {
    match name {
        "cwd" => Some(Value::Text(ctx.cwd.display().to_string())),
        "platform" => Some(Value::Text(std::env::consts::OS.to_string())),
        _ => None,
    }
}

fn apply_cast(cast: Option<CastKind>, v: Value) -> Result<Value> {
    let Some(c) = cast else {
        return Ok(v);
    };
    let text = match &v {
        Value::None => {
            bail!("cannot cast None to {:?}", c);
        }
        Value::Text(s) => s.clone(),
        Value::Secret(s) => s.clone(),
        Value::Int(n) => n.to_string(),
        Value::Num(n) => n.to_string(),
        Value::Bool(b) => {
            if *b {
                "true".into()
            } else {
                "false".into()
            }
        }
        other => bail!("cannot cast {} to {:?}", other.as_display(), c),
    };
    match c {
        CastKind::Int => {
            let n: i64 = text
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("cannot cast `{text}` to int"))?;
            Ok(Value::Int(n))
        }
        CastKind::Float => {
            let n: f64 = text
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("cannot cast `{text}` to float"))?;
            Ok(Value::Num(n))
        }
        CastKind::Bool => {
            let t = text.trim().to_ascii_lowercase();
            match t.as_str() {
                "1" | "true" | "yes" | "on" => Ok(Value::Bool(true)),
                "0" | "false" | "no" | "off" => Ok(Value::Bool(false)),
                _ => bail!("cannot cast `{text}` to bool"),
            }
        }
    }
}

/// Split `KEY=VALUE` from CLI `--bind`.
pub fn parse_bind_kv(s: &str) -> Result<(String, String)> {
    let Some((k, v)) = s.split_once('=') else {
        bail!("--bind expects KEY=VALUE (got `{s}`)");
    };
    let k = k.trim();
    if k.is_empty() {
        bail!("--bind key must be non-empty");
    }
    Ok((k.to_string(), v.to_string()))
}

pub fn cwd_for_path(path: Option<&Path>) -> PathBuf {
    path.and_then(|p| p.parent())
        .map(|p| {
            if p.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                p.to_path_buf()
            }
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sole_env_default() {
        let mv = parse_meta_value(r#"${env.NO_SUCH_MARQDO_VAR_XYZ ?? "gpt"}"#).unwrap();
        let ctx = BindingContext::default();
        let v = resolve_meta_value(&mv, &ctx, Span::new(1, 1)).unwrap();
        assert_eq!(v, Value::Text("gpt".into()));
    }

    #[test]
    fn required_missing_errors() {
        let mv = parse_meta_value("${env.NO_SUCH_MARQDO_REQ_XYZ!}").unwrap();
        let ctx = BindingContext::default();
        let err = resolve_meta_value(&mv, &ctx, Span::new(1, 1)).unwrap_err();
        assert!(err.to_string().contains("required binding"));
    }

    #[test]
    fn secret_masks_display() {
        std::env::set_var("MARQDO_TEST_SECRET_BIND", "s3cr3t");
        let mv = parse_meta_value("${secret.MARQDO_TEST_SECRET_BIND}").unwrap();
        let ctx = BindingContext::default();
        let v = resolve_meta_value(&mv, &ctx, Span::new(1, 1)).unwrap();
        assert_eq!(v.as_display(), "<secret>");
        match v {
            Value::Secret(s) => assert_eq!(s, "s3cr3t"),
            other => panic!("expected Secret, got {other:?}"),
        }
        std::env::remove_var("MARQDO_TEST_SECRET_BIND");
    }

    #[test]
    fn literal_not_overridden_by_env() {
        std::env::set_var("OPENAI_MODEL", "from-env");
        let raw = vec![(
            "model".into(),
            MetaValue::Literal(Value::Text("gpt-literal".into())),
        )];
        let ctx = BindingContext::default();
        let meta = resolve_metadata(&raw, &ctx, Span::new(1, 1)).unwrap();
        assert_eq!(meta[0].1, Value::Text("gpt-literal".into()));
        std::env::remove_var("OPENAI_MODEL");
    }

    #[test]
    fn bind_overlay_wins() {
        let raw = vec![(
            "model".into(),
            MetaValue::Literal(Value::Text("gpt-literal".into())),
        )];
        let ctx = BindingContext::from_binds(&[("model".into(), "from-cli".into())], PathBuf::from("."));
        let meta = resolve_metadata(&raw, &ctx, Span::new(1, 1)).unwrap();
        assert_eq!(meta[0].1, Value::Text("from-cli".into()));
    }

    #[test]
    fn int_cast() {
        std::env::set_var("MARQDO_TEST_MAX_TOKENS", "2048");
        let mv = parse_meta_value("${int(env.MARQDO_TEST_MAX_TOKENS)}").unwrap();
        let v = resolve_meta_value(&mv, &BindingContext::default(), Span::new(1, 1)).unwrap();
        assert_eq!(v, Value::Int(2048));
        std::env::remove_var("MARQDO_TEST_MAX_TOKENS");
    }

    #[test]
    fn interpolate_string() {
        std::env::set_var("MARQDO_TEST_USER", "Ada");
        let mv = parse_meta_value(r#""Hello ${env.MARQDO_TEST_USER}""#).unwrap();
        let v = resolve_meta_value(&mv, &BindingContext::default(), Span::new(1, 1)).unwrap();
        assert_eq!(v, Value::Text("Hello Ada".into()));
        std::env::remove_var("MARQDO_TEST_USER");
    }
}
