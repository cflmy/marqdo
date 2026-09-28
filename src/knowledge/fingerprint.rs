//! Symbol fingerprints + lexical similarity (char-bigram cosine).

use std::collections::HashMap;

use sha2::{Digest, Sha256};

use crate::ast::{Expr, Function, Stmt};
use crate::knowledge::ir::SymbolInfo;

pub fn char_bigram_tf(text: &str) -> HashMap<String, f64> {
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    let mut m = HashMap::new();
    if chars.is_empty() {
        return m;
    }
    if chars.len() == 1 {
        m.insert(chars[0].to_string(), 1.0);
        return m;
    }
    for w in chars.windows(2) {
        let key: String = w.iter().collect();
        *m.entry(key).or_insert(0.0) += 1.0;
    }
    m
}

pub fn cosine_tf(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    let mut na = 0.0;
    let mut nb = 0.0;
    for v in a.values() {
        na += v * v;
    }
    for v in b.values() {
        nb += v * v;
    }
    if na == 0.0 || nb == 0.0 {
        return 0.0;
    }
    let mut dot = 0.0;
    for (k, va) in a {
        if let Some(vb) = b.get(k) {
            dot += va * vb;
        }
    }
    dot / (na.sqrt() * nb.sqrt())
}

pub fn text_similarity(a: &str, b: &str) -> f64 {
    cosine_tf(&char_bigram_tf(a), &char_bigram_tf(b))
}

pub fn tokenize_ident(name: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in name.chars() {
        if c == '_' || c == '-' || c == '.' || c == '/' {
            if !cur.is_empty() {
                out.push(cur.to_ascii_lowercase());
                cur.clear();
            }
        } else if c.is_uppercase() && !cur.is_empty() && cur.chars().last().is_some_and(|x| x.is_lowercase())
        {
            out.push(cur.to_ascii_lowercase());
            cur.clear();
            cur.push(c);
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        out.push(cur.to_ascii_lowercase());
    }
    out
}

fn collect_calls_from_expr(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Call(c) => {
            let name = if let Some(path) = &c.path {
                path.join(".")
            } else if let Some(recv) = &c.receiver {
                format!("{recv}.{}", c.callee)
            } else {
                c.callee.clone()
            };
            out.push(name);
            for a in &c.args {
                match a {
                    crate::ast::Arg::Positional(e) | crate::ast::Arg::Named { value: e, .. } => {
                        collect_calls_from_expr(e, out);
                    }
                }
            }
        }
        Expr::Unary { expr, .. } => collect_calls_from_expr(expr, out),
        Expr::Binary { left, right, .. } => {
            collect_calls_from_expr(left, out);
            collect_calls_from_expr(right, out);
        }
        Expr::List(xs) => {
            for e in xs {
                collect_calls_from_expr(e, out);
            }
        }
        Expr::Map(pairs) => {
            for (_, e) in pairs {
                collect_calls_from_expr(e, out);
            }
        }
        Expr::Index { base, .. } => collect_calls_from_expr(base, out),
        Expr::Interp(parts) => {
            for p in parts {
                if let crate::ast::InterpPart::Index { .. } = p {
                    // no call
                }
            }
        }
        _ => {}
    }
}

fn collect_calls_from_stmts(stmts: &[Stmt], out: &mut Vec<String>) {
    for s in stmts {
        match s {
            Stmt::Assign { value, .. } | Stmt::Return { value, .. } | Stmt::Expr { value, .. } => {
                collect_calls_from_expr(value, out);
            }
            Stmt::Call { call, .. } => {
                let name = if let Some(path) = &call.path {
                    path.join(".")
                } else if let Some(recv) = &call.receiver {
                    format!("{recv}.{}", call.callee)
                } else {
                    call.callee.clone()
                };
                out.push(name);
                for a in &call.args {
                    match a {
                        crate::ast::Arg::Positional(e) | crate::ast::Arg::Named { value: e, .. } => {
                            collect_calls_from_expr(e, out);
                        }
                    }
                }
            }
            Stmt::Branch { arms, .. } => {
                for arm in arms {
                    if let Some(c) = &arm.condition {
                        collect_calls_from_expr(c, out);
                    }
                    collect_calls_from_stmts(&arm.body, out);
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                collect_calls_from_expr(condition, out);
                collect_calls_from_stmts(body, out);
            }
            Stmt::ForEach { body, .. } => collect_calls_from_stmts(body, out),
        }
    }
}

pub fn collect_calls(f: &Function) -> Vec<String> {
    let mut out = Vec::new();
    collect_calls_from_stmts(&f.body, &mut out);
    for child in &f.children {
        out.extend(collect_calls(child));
    }
    out.sort();
    out.dedup();
    out
}

pub fn ops_hint(f: &Function) -> Vec<String> {
    let mut ops = Vec::new();
    let name = f.name.to_ascii_lowercase();
    for verb in [
        "parse", "merge", "validate", "normalize", "resolve", "load", "save", "read", "write",
        "encode", "decode", "verify", "auth", "cache", "retry", "serialize", "render", "query",
        "create", "update", "delete", "check", "format", "convert",
    ] {
        if name.contains(verb) {
            ops.push(verb.to_string());
        }
    }
    for c in collect_calls(f) {
        let low = c.to_ascii_lowercase();
        for verb in ["parse", "merge", "validate", "json", "http", "hash", "auth"] {
            if low.contains(verb) && !ops.iter().any(|o| o == verb) {
                ops.push(verb.to_string());
            }
        }
    }
    // Body-size / structure hints keep near-duplicates discoverable
    let body_n = f.body.len();
    if body_n > 0 {
        ops.push(format!("stmts:{body_n}"));
    }
    if !f.params.is_empty() {
        ops.push(format!("arity:{}", f.params.len()));
    }
    ops
}

pub fn fingerprint_text(name: &str, params: &[String], calls: &[String], ops: &[String]) -> String {
    let mut parts = Vec::new();
    parts.push(format!("name:{}", tokenize_ident(name).join("_")));
    if !params.is_empty() {
        parts.push(format!("params:{}", params.join(",")));
    }
    if !calls.is_empty() {
        let mut c = calls.to_vec();
        c.sort();
        parts.push(format!("calls:{}", c.join(",")));
    }
    if !ops.is_empty() {
        parts.push(format!("ops:{}", ops.join(",")));
    }
    parts.join("|")
}

pub fn fingerprint_hash(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    format!("{:x}", h.finalize())[..16].to_string()
}

pub fn symbol_similarity(a: &SymbolInfo, b: &SymbolInfo) -> f64 {
    let mut score = text_similarity(&a.fingerprint_text, &b.fingerprint_text);
    let name_sim = text_similarity(&a.name, &b.name);
    // Token overlap on snake_case names (resolve_config vs normalize_settings share little;
    // merge_config vs merge_settings share "merge")
    let ta = tokenize_ident(&a.name);
    let tb = tokenize_ident(&b.name);
    let mut overlap = 0.0;
    if !ta.is_empty() && !tb.is_empty() {
        let shared = ta.iter().filter(|t| tb.contains(t)).count() as f64;
        overlap = shared / (ta.len().max(tb.len()) as f64);
    }
    let param_bonus = if !a.params.is_empty() && a.params == b.params {
        0.25
    } else if a.params.len() == b.params.len() && !a.params.is_empty() {
        0.1
    } else {
        0.0
    };
    score = (score * 0.25) + (name_sim * 0.15) + (overlap * 0.45) + param_bonus;
    score.min(1.0)
}
