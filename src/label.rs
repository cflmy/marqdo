//! `<TAB>` M2.5/M4: occlusion-label collection and diagnostics.
//!
//! A label is **inert prose**: it never runs, never binds, never types. This
//! module only *reads* it — to report structural problems (`label.missing`,
//! `label.empty_payload`) and, later, to check staleness (`label.ref_dangling`).
//!
//! Design: [occlusion-description-and-search.md](../../doc/design/occlusion-description-and-search.md)

use crate::ast::{Arg, BranchArm, Expr, Function, Module, Stmt};
use crate::diagnostics::{Diagnostic, Severity, Span};

/// One `[描述](实现)` site found in the module.
#[derive(Debug, Clone)]
pub struct OcclusionSite {
    /// Raw label text (already trimmed by the parser).
    pub label: String,
    /// `false` when written as `[描述]()` — the empty-payload stub.
    pub has_payload: bool,
    /// 1-based source line of the enclosing statement.
    pub line: u32,
    /// Enclosing function/object name (`# main` → `main`).
    pub unit: String,
    /// Identifiers the payload actually mentions (for L1 ref checks).
    pub payload_idents: std::collections::HashSet<String>,
    /// In-process fingerprint of the payload (T1 staleness comparison).
    pub payload_fp: String,
    /// Surface rendering of the payload (diagnostics: `code_past` / `code_now`).
    pub payload_src: String,
}

/// Collect every occlusion in `module` (declaration order).
pub fn collect(module: &Module) -> Vec<OcclusionSite> {
    let mut out = Vec::new();
    for fun in &module.functions {
        walk_fun(fun, &mut out);
    }
    out
}

fn walk_fun(fun: &Function, out: &mut Vec<OcclusionSite>) {
    for stmt in &fun.body {
        walk_stmt(stmt, &fun.name, out);
    }
    for child in &fun.children {
        walk_fun(child, out);
    }
}

fn walk_stmt(stmt: &Stmt, unit: &str, out: &mut Vec<OcclusionSite>) {
    match stmt {
        Stmt::Assign { value, span, .. }
        | Stmt::Return { value, span }
        | Stmt::Expr { value, span } => walk_expr(value, span.line, unit, out),
        Stmt::Call { call, span } => {
            for a in &call.args {
                match a {
                    Arg::Positional(e) | Arg::Named { value: e, .. } => {
                        walk_expr(e, span.line, unit, out)
                    }
                }
            }
        }
        Stmt::Branch { arms, span } => {
            for arm in arms {
                walk_arm(arm, span.line, unit, out);
            }
        }
        Stmt::While { condition, body, span } => {
            walk_expr(condition, span.line, unit, out);
            for s in body {
                walk_stmt(s, unit, out);
            }
        }
        Stmt::ForEach { body, span, .. } => {
            for s in body {
                walk_stmt(s, unit, out);
            }
            let _ = span;
        }
    }
}

fn walk_arm(arm: &BranchArm, line: u32, unit: &str, out: &mut Vec<OcclusionSite>) {
    // `condition: Option<Expr>` for the `*` else-arm.
    if let Some(cond) = &arm.condition {
        walk_expr(cond, line, unit, out);
    }
    for s in &arm.body {
        walk_stmt(s, unit, out);
    }
}

fn walk_expr(expr: &Expr, line: u32, unit: &str, out: &mut Vec<OcclusionSite>) {
    match expr {
        Expr::Occluded { label, payload } => {
            let mut idents = std::collections::HashSet::new();
            if let Some(inner) = payload {
                payload_idents(inner, &mut idents);
            }
            out.push(OcclusionSite {
                label: label.clone(),
                has_payload: payload.is_some(),
                line,
                unit: unit.to_string(),
                payload_idents: idents,
                payload_fp: payload_fingerprint(payload),
                payload_src: render_payload(payload),
            });
            if let Some(inner) = payload {
                walk_expr(inner, line, unit, out);
            }
        }
        Expr::Unary { expr, .. } => walk_expr(expr, line, unit, out),
        Expr::Binary { left, right, .. } => {
            walk_expr(left, line, unit, out);
            walk_expr(right, line, unit, out);
        }
        Expr::Call(call) => {
            for a in &call.args {
                match a {
                    Arg::Positional(e) | Arg::Named { value: e, .. } => walk_expr(e, line, unit, out),
                }
            }
        }
        Expr::List(items) => {
            for e in items {
                walk_expr(e, line, unit, out);
            }
        }
        Expr::Map(pairs) => {
            for (_, e) in pairs {
                walk_expr(e, line, unit, out);
            }
        }
        Expr::Index { base, .. } => walk_expr(base, line, unit, out),
        // Interpolation carries only variable *names* (strings), never nested
        // expressions, so there is nothing to walk into.
        Expr::Interp(_) => {}
        Expr::Literal(_) | Expr::Var(_) | Expr::Formula(_) | Expr::Code(_) => {}
    }
}

/// Backticked entity references inside a label: `` 按 `税率` 计算 `` → `["税率"]`.
///
/// Only explicit backticks count as claims — plain prose never is. `<TAB>` M4
pub fn refs(label: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = label;
    while let Some(open) = rest.find('`') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('`') else {
            break;
        };
        let name = after[..close].trim();
        if !name.is_empty() {
            out.push(name.to_string());
        }
        rest = &after[close + 1..];
    }
    out
}

/// Every identifier the payload actually mentions (variables, callees, path parts).
fn payload_idents(expr: &Expr, out: &mut std::collections::HashSet<String>) {
    match expr {
        Expr::Var(n) => {
            out.insert(n.clone());
        }
        Expr::Occluded { payload, .. } => {
            if let Some(inner) = payload {
                payload_idents(inner, out);
            }
        }
        Expr::Unary { expr, .. } => payload_idents(expr, out),
        Expr::Binary { left, right, .. } => {
            payload_idents(left, out);
            payload_idents(right, out);
        }
        Expr::Call(call) => {
            out.insert(call.callee.clone());
            if let Some(p) = &call.path {
                for seg in p {
                    out.insert(seg.clone());
                }
            }
            if let Some(r) = &call.receiver {
                out.insert(r.clone());
            }
            for a in &call.args {
                match a {
                    Arg::Positional(e) | Arg::Named { value: e, .. } => payload_idents(e, out),
                }
            }
        }
        Expr::List(items) => {
            for e in items {
                payload_idents(e, out);
            }
        }
        Expr::Map(pairs) => {
            for (_, e) in pairs {
                payload_idents(e, out);
            }
        }
        Expr::Index { base, label } => {
            payload_idents(base, out);
            if let crate::ast::IndexKey::Var(n) = label {
                out.insert(n.clone());
            }
        }
        Expr::Interp(parts) => {
            for p in parts {
                if let crate::ast::InterpPart::Var(n) = p {
                    out.insert(n.clone());
                }
            }
        }
        Expr::Literal(_) | Expr::Formula(_) | Expr::Code(_) => {}
    }
}

/// Names that exist in the document scope: functions/objects (recursive),
/// import aliases, frontmatter keys. Builtins are checked separately.
fn module_symbols(module: &Module) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    fn walk_fun(f: &Function, out: &mut std::collections::HashSet<String>) {
        out.insert(f.name.clone());
        for p in &f.params {
            out.insert(p.name.clone());
        }
        for c in &f.children {
            walk_fun(c, out);
        }
    }
    for f in &module.functions {
        walk_fun(f, &mut out);
    }
    for imp in &module.imports {
        out.insert(imp.bind.clone());
    }
    for u in &module.uses {
        out.insert(u.bind.clone());
    }
    for (k, _) in &module.metadata {
        out.insert(k.clone());
    }
    out
}

/// Diagnostics for label structure. `<TAB>` M2.5
///
/// - empty label → `label.missing` (**error**): you wrote a shape without prose
/// - empty payload → `label.empty_payload` (**warning**): say what the stub returns
pub fn check(module: &Module, path: Option<&std::path::Path>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let symbols = module_symbols(module);
    for site in collect(module) {
        let span = Span::new(site.line, 1);
        if site.label.is_empty() {
            out.push(
                Diagnostic::new(
                    path,
                    span,
                    format!(
                        "遮蔽缺少描述：`[]({})` —— 给这段实现写一句人话，或改用不带遮蔽的写法",
                        if site.has_payload { "…" } else { "" }
                    ),
                )
                .with_code("label.missing")
                .with_severity(Severity::Error)
                .with_suggestion("写成 `[这段实现做什么](实现)`；描述是自由文本，不参与执行"),
            );
            continue;
        }
        if !site.has_payload {
            out.push(
                Diagnostic::new(
                    path,
                    span,
                    format!("空载荷遮蔽：`[{}]()` —— 这里没有可执行内容", site.label),
                )
                .with_code("label.empty_payload")
                .with_severity(Severity::Warning)
                .with_doc_quote(site.label.clone())
                .with_suggestion("写清桩的返回值（如 `(None)` 或 `(0)`），或补上真实实现"),
            );
        }

        // `<TAB>` M4 (L1): every backticked ref in the label must resolve —
        // either the payload mentions it, or the document defines it, or it is
        // a builtin. This is the deterministic half of staleness detection.
        for r in refs(&site.label) {
            if site.payload_idents.contains(&r)
                || symbols.contains(&r)
                || crate::host::HostFn::from_name(&r).is_some()
            {
                continue;
            }
            out.push(
                Diagnostic::new(
                    path,
                    span,
                    format!(
                        "描述引用了不存在的实体：`{r}` —— 本段实现没有用到它，模块里也没有定义"
                    ),
                )
                .with_code("label.ref_dangling")
                .with_severity(Severity::Error)
                .with_doc_quote(site.label.clone())
                .with_suggestion(format!(
                    "检查是否改名/删除；若只是措辞，把 `{r}` 的反引号去掉或改写描述"
                )),
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::refs;

    #[test]
    fn refs_extracts_backticked_names() {
        assert_eq!(refs("按 `税率` 计算"), vec!["税率".to_string()]);
    }

    #[test]
    fn refs_multiple_and_skips_prose() {
        assert_eq!(
            refs("用 `a` 和 `b`，但 plain words 不算"),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn refs_none_and_unmatched_backtick() {
        assert!(refs("纯口语描述").is_empty());
        // Unmatched trailing backtick: ignore, do not panic.
        assert!(refs("半截 `名字").is_empty());
        // Empty ticks are not refs.
        assert!(refs("空 `` 引号").is_empty());
    }
}

// ---------------------------------------------------------------------------
// M9: 两级词表检索（built-in groups = 默认/范例；`--grep` = 调用方自定义）
// ---------------------------------------------------------------------------

/// One built-in keyword group. `<TAB>` M9
///
/// The table is **data**, not syntax: it seeds the default query
/// (`marqdo labels pending`) and gives LLMs a stable vocabulary; any project can
/// override or extend it at the call site with `--grep`.
pub struct WordGroup {
    pub name: &'static str,
    pub words: &'static [&'static str],
}

/// Built-in groups (v1). Extending this table never changes the grammar.
pub const BUILTIN_GROUPS: &[WordGroup] = &[
    WordGroup {
        name: "未完成",
        words: &[
            "待开发", "未开发", "开发中", "未实现", "待实现", "未接", "接口未接", "TODO",
        ],
    },
    WordGroup {
        name: "假数据",
        words: &["模拟", "假数据", "桩", "占位", "stub", "mock"],
    },
    WordGroup {
        name: "示例",
        words: &["示例", "样例", "demo"],
    },
    WordGroup {
        name: "实验",
        words: &["实验", "试验", "experimental"],
    },
    WordGroup {
        name: "废弃",
        words: &["废弃", "弃用", "deprecated"],
    },
    WordGroup {
        name: "待校验",
        words: &["待校验", "未验证", "待确认"],
    },
];

/// Groups that count as "pending" (default query for `labels pending`).
pub const PENDING_GROUPS: &[&str] = &["未完成", "假数据"];

/// Expand group names into their words; unknown names are ignored.
pub fn group_words(names: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    for g in BUILTIN_GROUPS {
        if names.iter().any(|n| *n == g.name) {
            out.extend(g.words.iter().map(|w| w.to_string()));
        }
    }
    out
}

/// Split a `--grep` value (`a,b` / `a，b`) into words.
pub fn parse_words(spec: &str) -> Vec<String> {
    spec.split([',', '，'])
        .map(|w| w.trim())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect()
}

/// Words that hit `label` (substring, ASCII case-insensitive). `<TAB>` M9
pub fn matched_words(label: &str, words: &[String]) -> Vec<String> {
    let hay = label.to_lowercase();
    let mut out = Vec::new();
    for w in words {
        if w.is_empty() {
            continue;
        }
        if hay.contains(&w.to_lowercase()) {
            out.push(w.clone());
        }
    }
    out
}

/// One search hit: where, what the label says, which words matched.
#[derive(Debug, Clone)]
pub struct Hit {
    pub site: OcclusionSite,
    pub words: Vec<String>,
}

/// Search every occlusion whose label matches any of `words`.
pub fn search(module: &Module, words: &[String]) -> Vec<Hit> {
    let mut out = Vec::new();
    for site in collect(module) {
        let words = matched_words(&site.label, words);
        if !words.is_empty() {
            out.push(Hit { site, words });
        }
    }
    out
}

// ---------------------------------------------------------------------------
// M6 (T1): session-scoped staleness — compare before/after edits in memory.
// ---------------------------------------------------------------------------

/// One occlusion whose payload changed while its label stayed the same.
#[derive(Debug, Clone)]
pub struct StaleItem {
    pub label: String,
    /// 1-based line in the **new** source.
    pub line: u32,
    /// Rendered payload before the edit.
    pub code_past: String,
    /// Rendered payload after the edit.
    pub code_now: String,
}

/// In-process fingerprint of a payload. Only used to compare two versions parsed
/// by the *same* process, so a stable cross-version format is unnecessary.
fn payload_fingerprint(payload: &Option<Box<Expr>>) -> String {
    match payload {
        Some(e) => format!("{e:?}"),
        None => "<empty>".to_string(),
    }
}

fn render_payload(payload: &Option<Box<Expr>>) -> String {
    match payload {
        Some(e) => crate::view::expr_display(e),
        None => String::new(),
    }
}

/// Compare two versions of a document and return occlusions whose **payload**
/// changed but whose **label** did not. Matching is by `(label, occurrence)`, so
/// moving code around does not produce false positives.
pub fn compare(before: &Module, after: &Module) -> Vec<StaleItem> {
    use std::collections::HashMap;

    let before_sites = collect(before);
    let after_sites = collect(after);

    // Group by label → ordered occurrences.
    let mut past: HashMap<&str, Vec<&OcclusionSite>> = HashMap::new();
    for s in &before_sites {
        past.entry(s.label.as_str()).or_default().push(s);
    }

    let mut seen: HashMap<&str, usize> = HashMap::new();
    let mut out = Vec::new();
    for site in &after_sites {
        let idx = seen.entry(site.label.as_str()).or_insert(0);
        let occurrence = *idx;
        *idx += 1;
        let Some(old) = past.get(site.label.as_str()).and_then(|v| v.get(occurrence)) else {
            continue; // brand-new label: no history, nothing to compare
        };
        if old.payload_fp == site.payload_fp {
            continue;
        }
        out.push(StaleItem {
            label: site.label.clone(),
            line: site.line,
            code_past: old.payload_src.clone(),
            code_now: site.payload_src.clone(),
        });
    }
    out
}

/// Session accumulator for `session.summary` (`<TAB>` M6).
#[derive(Debug, Default, Clone)]
pub struct SessionSummary {
    pub stale_seen: usize,
    pub fixed: usize,
    pub items: Vec<String>,
}

impl SessionSummary {
    /// Record one round's comparison; returns the number of *newly* seen labels.
    pub fn record(&mut self, stale: &[StaleItem]) -> usize {
        let now: Vec<String> = stale.iter().map(|s| s.label.clone()).collect();
        let mut newly = 0;
        for label in &now {
            if !self.items.contains(label) {
                self.items.push(label.clone());
                self.stale_seen += 1;
                newly += 1;
            }
        }
        // Labels that were outstanding and are no longer stale are considered fixed.
        let still: Vec<String> = self
            .items
            .iter()
            .filter(|l| now.contains(l))
            .cloned()
            .collect();
        self.fixed += self.items.len() - still.len();
        self.items = still;
        newly
    }
}

// ---------------------------------------------------------------------------
// M5: implicit occlusion — the narrative paragraph above a `>` call is its
// description. Only falsifiable claims (backticked refs) are checked, so
// ordinary prose never produces noise.
// ---------------------------------------------------------------------------

/// A `>` statement call with the identifiers it mentions.
#[derive(Debug, Clone)]
pub struct CallSite {
    pub line: u32,
    pub idents: std::collections::HashSet<String>,
}

/// Collect `>` call statements (recursively).
pub fn call_sites(module: &Module) -> Vec<CallSite> {
    fn walk_fun(f: &Function, out: &mut Vec<CallSite>) {
        for s in &f.body {
            walk_stmt(s, out);
        }
        for c in &f.children {
            walk_fun(c, out);
        }
    }
    fn walk_stmt(stmt: &Stmt, out: &mut Vec<CallSite>) {
        match stmt {
            Stmt::Call { call, span } => {
                let mut idents = std::collections::HashSet::new();
                idents.insert(call.callee.clone());
                if let Some(p) = &call.path {
                    for seg in p {
                        idents.insert(seg.clone());
                    }
                }
                if let Some(r) = &call.receiver {
                    idents.insert(r.clone());
                }
                for a in &call.args {
                    match a {
                        Arg::Positional(e) | Arg::Named { value: e, .. } => {
                            payload_idents(e, &mut idents)
                        }
                    }
                }
                out.push(CallSite {
                    line: span.line,
                    idents,
                });
            }
            Stmt::Branch { arms, .. } => {
                for arm in arms {
                    for s in &arm.body {
                        walk_stmt(s, out);
                    }
                }
            }
            Stmt::While { body, .. } | Stmt::ForEach { body, .. } => {
                for s in body {
                    walk_stmt(s, out);
                }
            }
            Stmt::Assign { value, .. } | Stmt::Return { value, .. } | Stmt::Expr { value, .. } => {
                // Nested calls inside expressions are not `>` statements.
                let _ = value;
            }
        }
    }
    let mut out = Vec::new();
    for f in &module.functions {
        walk_fun(f, &mut out);
    }
    out
}

/// The narrative paragraph immediately above `line` (1-based), if any.
///
/// A paragraph is a run of non-blank lines directly above; blank line ends it.
/// Structural lines (`#`, `**`, `>`, `-`, `|`, digits) are not prose.
fn paragraph_above(lines: &[&str], line: u32) -> Option<String> {
    let idx = (line as usize).checked_sub(2)?; // line-1 → 0-based index of the previous line
    let mut collected: Vec<&str> = Vec::new();
    let mut i = idx as isize;
    // People write `paragraph`, one blank line, then the `>` call (blockquote).
    // Allow exactly one blank separator; a second blank ends the description.
    if i >= 0 && lines[i as usize].trim().is_empty() {
        i -= 1;
    }
    while i >= 0 {
        let raw = lines[i as usize];
        let t = raw.trim();
        if t.is_empty() {
            break;
        }
        let first = t.chars().next().unwrap_or(' ');
        let structural = matches!(first, '#')
            || t.starts_with("**")
            || t.starts_with('>')
            || t.starts_with('-')
            || t.starts_with('|')
            || t.starts_with("```")
            || first.is_ascii_digit();
        if structural {
            break;
        }
        collected.push(raw);
        i -= 1;
    }
    if collected.is_empty() {
        None
    } else {
        collected.reverse();
        Some(collected.join("\n"))
    }
}

/// True when a backticked ref looks like an *entity name* rather than prose.
///
/// Prose often backticks shapes (`{ path, kind }`), flags (`source=`), or
/// headings (`## open`); those must not be treated as claims. `<TAB>` M5
fn is_entity_ref(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if first.is_ascii_digit() {
        return false;
    }
    let ok_first = first.is_ascii_alphabetic() || first == '_' || !first.is_ascii();
    let ok_rest = s.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '_' || c == '.' || (!c.is_ascii() && !c.is_whitespace())
    });
    ok_first && ok_rest
}

/// Implicit-occlusion diagnostics for `>` calls. `<TAB>` M5
///
/// The paragraph above a call is its description; backticked refs there must be
/// mentioned by the call (or resolve in the module). No paragraph / no refs →
/// silent, which keeps the check at zero noise on ordinary code.
pub fn check_implicit(
    module: &Module,
    source: &str,
    path: Option<&std::path::Path>,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let symbols = module_symbols(module);
    let lines: Vec<&str> = source.lines().collect();
    for site in call_sites(module) {
        let Some(para) = paragraph_above(&lines, site.line) else {
            continue;
        };
        for r in refs(&para).into_iter().filter(|r| is_entity_ref(r)) {
            if site.idents.contains(&r)
                || symbols.contains(&r)
                || crate::host::HostFn::from_name(&r).is_some()
            {
                continue;
            }
            out.push(
                Diagnostic::new(
                    path,
                    Span::new(site.line, 1),
                    format!("调用上方的描述提到 `{r}`，但这次调用没有用到它"),
                )
                .with_code("label.implicit_stale")
                .with_severity(Severity::Warning)
                .with_doc_quote(para.replace('\n', " "))
                .with_suggestion(format!(
                    "确认描述是否已过期：把 `{r}` 换成现在的参数名，或修改调用"
                )),
            );
        }
    }
    out
}
