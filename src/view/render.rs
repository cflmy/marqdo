//! Render Marqdo AST as HTML fragments for `view`.
//!
//! Comments are not in the AST; they are interleaved from classified source lines.
//! Expressions are shown in surface syntax, never Rust `Debug`.

use std::collections::HashMap;

use crate::ast::{
    Arg, BinaryOp, CallExpr, Expr, Function, InterpPart, Literal, Module, Stmt, UnaryOp,
};
use crate::host::writeback;
use crate::lex::{classify_source, ClassifiedLine, LineKind};
use crate::view::html::escape;
use pulldown_cmark::{html, Options, Parser};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureMode {
    /// Documentation browse: fun anchors, no breakpoint gutters
    Browse,
    /// Debugger: breakpoint gutters on statements
    Debug,
}

pub struct FileViewModel {
    #[allow(dead_code)]
    pub rel_path: String,
    pub source: String,
    pub structure_html: String,
    pub outline_html: String,
    pub stdout: String,
    pub stderr: String,
    pub ok: bool,
    /// Text shown in the live-view preset stdin box.
    pub preset_stdin: String,
    /// One entry per `input` / `输入` call (prompt text; may be empty).
    /// Empty vec → page does not use input; hide the preset form.
    pub input_prompts: Vec<String>,
    /// Live view: program uses `input` but no stdin yet — execution deferred until form submit.
    pub awaiting_input: bool,
    /// Live view: stdin present — skip blocking page-load run; client auto-starts Stream SSE.
    pub auto_stream: bool,
    /// Show the LLM/agent Stream panel (live view only; gated on ext llm/agent import).
    pub show_stream: bool,
    /// SVG plots from math lib (embedded in Execution).
    pub plots: Vec<String>,
    /// HTML for Variables panel (entry bindings after run).
    pub bindings_html: String,
}

/// Collect prompt strings from every `input` / `输入` call in the module (pre-order).
pub fn collect_input_prompts(module: &Module) -> Vec<String> {
    let mut out = Vec::new();
    for fun in &module.functions {
        collect_input_prompts_fun(fun, &mut out);
    }
    out
}

/// Stream panel is for LLM/agent token streaming — only when those ext packages are imported.
pub fn uses_stream_panel(module: &Module) -> bool {
    module.imports.iter().any(|imp| import_enables_stream(&imp.path))
}

fn import_enables_stream(path: &str) -> bool {
    let norm = path.replace('\\', "/");
    let lower = norm.to_ascii_lowercase();
    lower.contains("ext/ai/llm")
        || lower.contains("/llm.mq.md")
        || lower.ends_with("llm.mq.md")
        || lower.contains("ext/llm")
        || norm.contains("ext/ai/大模型")
        || norm.contains("大模型.mq.md")
        || lower.contains("ext/ai/agent")
        || lower.contains("/agent.mq.md")
        || lower.ends_with("agent.mq.md")
        || lower.contains("ext/agent")
        || norm.contains("ext/ai/智能体")
        || norm.contains("智能体.mq.md")
}

fn collect_input_prompts_fun(fun: &Function, out: &mut Vec<String>) {
    for stmt in &fun.body {
        collect_input_prompts_stmt(stmt, out);
    }
    for child in &fun.children {
        collect_input_prompts_fun(child, out);
    }
}

fn collect_input_prompts_stmt(stmt: &Stmt, out: &mut Vec<String>) {
    match stmt {
        Stmt::Assign { value, .. } | Stmt::Return { value, .. } => {
            collect_input_prompts_expr(value, out);
        }
        Stmt::Call { call, .. } => {
            push_input_prompt(call, out);
            for a in &call.args {
                match a {
                    Arg::Positional(e) | Arg::Named { value: e, .. } => {
                        collect_input_prompts_expr(e, out);
                    }
                }
            }
        }
        Stmt::Branch { arms, .. } => {
            for arm in arms {
                if let Some(c) = &arm.condition {
                    collect_input_prompts_expr(c, out);
                }
                for s in &arm.body {
                    collect_input_prompts_stmt(s, out);
                }
            }
        }
        Stmt::While {
            condition, body, ..
        } => {
            collect_input_prompts_expr(condition, out);
            for s in body {
                collect_input_prompts_stmt(s, out);
            }
        }
        Stmt::ForEach { body, .. } => {
            for s in body {
                collect_input_prompts_stmt(s, out);
            }
        }
    }
}

fn collect_input_prompts_expr(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Call(call) => {
            push_input_prompt(call, out);
            for a in &call.args {
                match a {
                    Arg::Positional(e) | Arg::Named { value: e, .. } => {
                        collect_input_prompts_expr(e, out);
                    }
                }
            }
        }
        Expr::Unary { expr, .. } => collect_input_prompts_expr(expr, out),
        Expr::Binary { left, right, .. } => {
            collect_input_prompts_expr(left, out);
            collect_input_prompts_expr(right, out);
        }
        Expr::List(items) => {
            for e in items {
                collect_input_prompts_expr(e, out);
            }
        }
        Expr::Map(pairs) => {
            for (_, e) in pairs {
                collect_input_prompts_expr(e, out);
            }
        }
        Expr::Index { base, .. } => collect_input_prompts_expr(base, out),
        Expr::Literal(_) | Expr::Var(_) | Expr::Interp(_) | Expr::Formula(_) | Expr::Code(_) => {}
    }
}

fn push_input_prompt(call: &CallExpr, out: &mut Vec<String>) {
    if crate::aliases::canonical_builtin(&call.callee) != Some("input") {
        return;
    }
    for a in &call.args {
        if let Arg::Named { name, value } = a {
            if crate::aliases::canonical_param("input", name) == "prompt" {
                out.push(prompt_expr_text(value));
                return;
            }
        }
    }
    if let Some(Arg::Positional(value)) = call.args.first() {
        out.push(prompt_expr_text(value));
        return;
    }
    out.push(String::new());
}

fn prompt_expr_text(expr: &Expr) -> String {
    match expr {
        Expr::Literal(Literal::Text(t)) => t.clone(),
        Expr::Literal(lit) => lit_display(lit),
        other => expr_display(other),
    }
}

pub fn render_module_structure(module: &Module, source: &str) -> String {
    render_module_structure_mode(module, source, StructureMode::Browse)
}

pub fn render_module_structure_mode(
    module: &Module,
    source: &str,
    mode: StructureMode,
) -> String {
    let lines = classify_source(source);
    let writebacks = writeback::writeback_map(source);
    let mut out = String::new();

    if !module.imports.is_empty() {
        out.push_str("<div class=\"card\"><span class=\"badge\">import</span>");
        for imp in &module.imports {
            let label = format!("{} as {}", imp.path, imp.bind);
            out.push_str(&format!("<span class=\"chip\">{}</span>", escape(&label)));
        }
        out.push_str("</div>");
    }
    if !module.uses.is_empty() {
        out.push_str("<div class=\"card\"><span class=\"badge\">use</span>");
        for u in &module.uses {
            let label = format!("{} as {}", u.path.join("."), u.bind);
            out.push_str(&format!("<span class=\"chip\">{}</span>", escape(&label)));
        }
        out.push_str("</div>");
    }

    let body_start = skip_frontmatter_end(&lines).unwrap_or(1);
    let mut cursor = body_start;

    if module.functions.is_empty() {
        out.push_str(&emit_comments(&lines, cursor, u32::MAX));
        if out.is_empty() {
            out.push_str("<p class=\"meta\">(empty module)</p>");
        }
        return out;
    }

    // Leading comments before first function
    let first = module.functions[0].span.line;
    out.push_str(&emit_comments(&lines, cursor, first));
    cursor = first;

    for (i, fun) in module.functions.iter().enumerate() {
        if i > 0 {
            out.push_str(&emit_comments(&lines, cursor, fun.span.line));
        }
        let (html, next) = render_fun(fun, &lines, 0, "", mode, &writebacks);
        out.push_str(&html);
        cursor = next;
    }

    // Trailing comments after last function tree
    out.push_str(&emit_comments(&lines, cursor, u32::MAX));
    out
}

pub fn render_function_outline(module: &Module) -> String {
    if module.functions.is_empty() {
        return String::new();
    }
    let mut out = String::from(
        r#"<div class="outline-panel" id="fn-outline">
<input type="search" id="fn-search" class="fn-search" placeholder="Search functions…" autocomplete="off" spellcheck="false"/>
<ul class="outline-tree">"#,
    );
    for fun in &module.functions {
        out.push_str(&outline_fun(fun, ""));
    }
    out.push_str("</ul></div>");
    out
}

/// Right rail: collapsible Functions + Variables (+ optional writeback HTML appended by caller).
pub fn render_right_rail(module: &Module, bindings_html: &str, writeback_html: &str) -> String {
    let fn_inner = if module.functions.is_empty() {
        r#"<p class="vars-empty">No functions</p>"#.to_string()
    } else {
        let mut tree = String::from(
            r#"<input type="search" id="fn-search" class="fn-search" placeholder="Search functions…" autocomplete="off" spellcheck="false"/>
<ul class="outline-tree">"#,
        );
        for fun in &module.functions {
            tree.push_str(&outline_fun(fun, ""));
        }
        tree.push_str("</ul>");
        tree
    };
    let vars = if bindings_html.is_empty() {
        r#"<p class="vars-empty">No bindings</p>"#
    } else {
        bindings_html
    };
    format!(
        r#"<div class="outline-rail" id="outline-rail">
<details class="outline-section" open>
<summary class="outline-summary">Functions</summary>
<div class="outline-panel" id="fn-outline">{fn_inner}</div>
</details>
<details class="outline-section" open>
<summary class="outline-summary">Variables</summary>
<div class="vars-panel" id="vars-panel">{vars}</div>
</details>
{wb}
</div>"#,
        fn_inner = fn_inner,
        vars = vars,
        wb = writeback_html,
    )
}

/// Render entry bindings as HTML tables (List / Map / records).
pub fn render_bindings_html(bindings: &HashMap<String, crate::value::Value>) -> String {
    if bindings.is_empty() {
        return String::new();
    }
    let mut names: Vec<&String> = bindings.keys().collect();
    names.sort();
    let mut out = String::from(r#"<ul class="vars-list">"#);
    for name in names {
        let Some(val) = bindings.get(name) else {
            continue;
        };
        out.push_str(&format!(
            r#"<li class="vars-item"><div class="vars-name">{}</div><div class="vars-value">{}</div></li>"#,
            escape(name),
            render_value_html(val, 0)
        ));
    }
    out.push_str("</ul>");
    out
}

const BIND_MAX_DEPTH: usize = 2;

fn render_value_html(v: &crate::value::Value, depth: usize) -> String {
    use crate::value::Value;
    match v {
        Value::List(xs) => {
            if let Some(keys) = record_keys(xs) {
                return render_records_table(xs, &keys, depth);
            }
            let mut s = String::from(r#"<table class="vars-table"><thead><tr><th>#</th><th>value</th></tr></thead><tbody>"#);
            for (i, item) in xs.iter().enumerate() {
                s.push_str(&format!(
                    "<tr><td class=\"vars-idx\">{}</td><td>{}</td></tr>",
                    i + 1,
                    cell_html(item, depth)
                ));
            }
            if xs.is_empty() {
                s.push_str(r#"<tr><td colspan="2" class="vars-empty">∅</td></tr>"#);
            }
            s.push_str("</tbody></table>");
            s
        }
        Value::Map(pairs) => {
            let mut s = String::from(
                r#"<table class="vars-table"><thead><tr><th>key</th><th>value</th></tr></thead><tbody>"#,
            );
            for (k, val) in pairs {
                s.push_str(&format!(
                    "<tr><td class=\"vars-key\">{}</td><td>{}</td></tr>",
                    escape(k),
                    cell_html(val, depth)
                ));
            }
            if pairs.is_empty() {
                s.push_str(r#"<tr><td colspan="2" class="vars-empty">∅</td></tr>"#);
            }
            s.push_str("</tbody></table>");
            s
        }
        other => format!(r#"<span class="vars-scalar">{}</span>"#, escape(&other.as_display())),
    }
}

fn cell_html(v: &crate::value::Value, depth: usize) -> String {
    use crate::value::Value;
    match v {
        Value::List(_) | Value::Map(_) if depth >= BIND_MAX_DEPTH => {
            format!(
                r#"<details class="vars-nested"><summary>{}</summary><span class="vars-scalar">{}</span></details>"#,
                escape(&brief_type(v)),
                escape(&v.as_display())
            )
        }
        Value::List(_) | Value::Map(_) => render_value_html(v, depth + 1),
        other => escape(&other.as_display()),
    }
}

fn brief_type(v: &crate::value::Value) -> String {
    use crate::value::Value;
    match v {
        Value::List(xs) => format!("List({})", xs.len()),
        Value::Map(xs) => format!("Map({})", xs.len()),
        _ => "…".into(),
    }
}

fn record_keys(xs: &[crate::value::Value]) -> Option<Vec<String>> {
    use crate::value::Value;
    if xs.is_empty() {
        return None;
    }
    let mut keys: Vec<String> = Vec::new();
    for (i, item) in xs.iter().enumerate() {
        let Value::Map(pairs) = item else {
            return None;
        };
        if i == 0 {
            keys = pairs.iter().map(|(k, _)| k.clone()).collect();
            if keys.is_empty() {
                return None;
            }
        }
    }
    Some(keys)
}

fn render_records_table(
    xs: &[crate::value::Value],
    keys: &[String],
    depth: usize,
) -> String {
    use crate::value::Value;
    let mut s = String::from(r#"<table class="vars-table"><thead><tr><th>#</th>"#);
    for k in keys {
        s.push_str(&format!("<th>{}</th>", escape(k)));
    }
    s.push_str("</tr></thead><tbody>");
    for (i, item) in xs.iter().enumerate() {
        let Value::Map(pairs) = item else {
            continue;
        };
        s.push_str(&format!(r#"<tr><td class="vars-idx">{}</td>"#, i + 1));
        for k in keys {
            let cell = pairs
                .iter()
                .find(|(kk, _)| kk == k)
                .map(|(_, v)| cell_html(v, depth))
                .unwrap_or_else(|| String::from(""));
            s.push_str(&format!("<td>{cell}</td>"));
        }
        s.push_str("</tr>");
    }
    s.push_str("</tbody></table>");
    s
}

fn outline_fun(fun: &Function, parent_path: &str) -> String {
    let fn_path = fn_path(parent_path, &fun.name);
    let mut s = format!(
        "<li class=\"outline-item\" data-fn=\"{}\" data-fn-path=\"{}\">\
         <a href=\"#fn-{}\">{}</a> <span class=\"ol-meta\">h{}</span>",
        escape(&fun.name),
        escape(&fn_path),
        fun.span.line,
        escape(&fun.name),
        fun.level,
    );
    if !fun.children.is_empty() {
        s.push_str("<ul>");
        for child in &fun.children {
            s.push_str(&outline_fun(child, &fn_path));
        }
        s.push_str("</ul>");
    }
    s.push_str("</li>");
    s
}

fn fn_path(parent_path: &str, name: &str) -> String {
    if parent_path.is_empty() {
        name.to_string()
    } else {
        format!("{parent_path}/{name}")
    }
}

/// Line after closing frontmatter `---`, or 1 if none.
fn skip_frontmatter_end(lines: &[ClassifiedLine]) -> Option<u32> {
    let first_code = lines.iter().find(|l| l.kind == LineKind::Code)?;
    if first_code.text.trim() != "---" {
        return Some(1);
    }
    for l in lines.iter().skip_while(|l| l.line_no <= first_code.line_no) {
        if l.kind == LineKind::Code && l.text.trim() == "---" {
            return Some(l.line_no + 1);
        }
    }
    Some(1)
}

fn emit_comments(lines: &[ClassifiedLine], from_line: u32, before_line: u32) -> String {
    let mut s = String::new();
    let mut paragraph: Vec<String> = Vec::new();

    let flush = |paragraph: &mut Vec<String>, s: &mut String| {
        if paragraph.is_empty() {
            return;
        }
        // Soft-wrapped source lines → one Markdown paragraph (spaces between).
        let text = paragraph.join(" ");
        paragraph.clear();
        s.push_str(&format!(
            "<div class=\"card comment-card\"><span class=\"badge\">comment</span><div class=\"comment-text\">{}</div></div>",
            comment_markdown_to_html(&text)
        ));
    };

    for l in lines {
        if l.line_no < from_line {
            continue;
        }
        if l.line_no >= before_line {
            break;
        }
        match l.kind {
            LineKind::Comment => {
                let text = l.text.trim();
                if !text.is_empty() {
                    paragraph.push(text.to_string());
                }
            }
            LineKind::Blank | LineKind::Writeback | LineKind::Code => flush(&mut paragraph, &mut s),
        }
    }
    flush(&mut paragraph, &mut s);
    s
}

/// Render narrative comments as Markdown (links, emphasis, code, lists, …).
fn comment_markdown_to_html(md: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(md, opts);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

fn render_fun(
    fun: &Function,
    lines: &[ClassifiedLine],
    depth: usize,
    parent_path: &str,
    mode: StructureMode,
    writebacks: &HashMap<u32, String>,
) -> (String, u32) {
    let fn_path = fn_path(parent_path, &fun.name);
    let mut s = String::new();
    let nest = if depth > 0 { " nested" } else { "" };
    s.push_str(&format!(
        "<div class=\"card fun-card{nest}\" id=\"fn-{}\" data-fn=\"{}\" data-fn-path=\"{}\">",
        fun.span.line,
        escape(&fun.name),
        escape(&fn_path),
    ));
    s.push_str(&format!(
        "<div><span class=\"badge\">fn · h{}</span><strong>{}</strong></div>",
        fun.level,
        escape(&fun.name)
    ));
    if !fun.params.is_empty() {
        s.push_str("<div class=\"params\">");
        for p in &fun.params {
            s.push_str(&format!("<span class=\"chip\">{}</span>", escape(&p.name)));
        }
        s.push_str("</div>");
    }

    let mut cursor = fun.span.line + 1;

    // Comments / gap before first body stmt or child
    let first_body = fun.body.first().map(stmt_start);
    let first_child = fun.children.first().map(|c| c.span.line);
    let first_content = match (first_body, first_child) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    if let Some(fc) = first_content {
        s.push_str(&emit_comments(lines, cursor, fc));
        cursor = fc;
    }

    // Interleave body stmts and nested functions by source line
    let mut bi = 0usize;
    let mut ci = 0usize;
    while bi < fun.body.len() || ci < fun.children.len() {
        let stmt_line = fun.body.get(bi).map(stmt_start);
        let child_line = fun.children.get(ci).map(|c| c.span.line);
        let take_stmt = match (stmt_line, child_line) {
            (Some(sl), Some(cl)) => sl <= cl,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => break,
        };
        if take_stmt {
            let stmt = &fun.body[bi];
            let start = stmt_start(stmt);
            s.push_str(&emit_comments(lines, cursor, start));
            let (html, end) = render_stmt(stmt, lines, mode, writebacks);
            s.push_str(&html);
            cursor = end;
            bi += 1;
        } else {
            let child = &fun.children[ci];
            s.push_str(&emit_comments(lines, cursor, child.span.line));
            let (html, end) = render_fun(child, lines, depth + 1, &fn_path, mode, writebacks);
            s.push_str(&format!("<div class=\"nested\">{html}</div>"));
            cursor = end;
            ci += 1;
        }
    }

    s.push_str("</div>");
    (s, cursor)
}

fn stmt_start(stmt: &Stmt) -> u32 {
    match stmt {
        Stmt::Assign { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::Call { span, .. }
        | Stmt::Branch { span, .. }
        | Stmt::While { span, .. }
        | Stmt::ForEach { span, .. } => span.line,
    }
}

fn stmt_end_line(stmt: &Stmt) -> u32 {
    match stmt {
        Stmt::Assign { end_line, .. } => end_line + 1,
        Stmt::Return { span, .. } | Stmt::Call { span, .. } => span.line + 1,
        Stmt::Branch { arms, span, .. } => {
            let mut end = span.line + 1;
            for arm in arms {
                for st in &arm.body {
                    end = end.max(stmt_end_line(st));
                }
            }
            end
        }
        Stmt::While { body, span, .. } | Stmt::ForEach { body, span, .. } => {
            let mut end = span.line + 1;
            for st in body {
                end = end.max(stmt_end_line(st));
            }
            end
        }
    }
}

fn output_card(line: u32, writebacks: &HashMap<u32, String>) -> String {
    match writebacks.get(&line) {
        Some(body) => {
            let badges = writeback_badges(body);
            format!(
                r#"<div class="card output-card" id="wb-{line}"><div class="wb-badges">{badges}</div><pre class="output-body">{}</pre></div>"#,
                escape(body)
            )
        }
        None => String::new(),
    }
}

fn writeback_badges(body: &str) -> String {
    let mut keys = Vec::new();
    for line in body.lines() {
        if let Some(rest) = line.strip_prefix('[') {
            if let Some(end) = rest.find(']') {
                let k = &rest[..end];
                if !k.is_empty() && !keys.iter().any(|x| x == k) {
                    keys.push(k.to_string());
                }
            }
        }
    }
    if keys.is_empty() {
        return r#"<span class="badge wb-badge">output</span>"#.into();
    }
    keys.into_iter()
        .map(|k| {
            let cls = match k.as_str() {
                "ok" => "wb-ok",
                "error" => "wb-error",
                "trace" => "wb-trace",
                _ => "wb-other",
            };
            format!(r#"<span class="badge wb-badge {cls}">{k}</span>"#, k = escape(&k))
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Outline links to writeback cards (`#wb-{line}`).
pub fn render_writeback_outline(source: &str) -> String {
    let map = writeback::writeback_map(source);
    if map.is_empty() {
        return String::new();
    }
    let mut lines: Vec<u32> = map.keys().copied().filter(|l| *l > 0).collect();
    lines.sort_unstable();
    if lines.is_empty() {
        return String::new();
    }
    let mut out = String::from(
        r#"<div class="wb-outline" id="wb-outline"><div class="wb-outline-title">Writebacks</div><ul class="outline-tree">"#,
    );
    for line in lines {
        let body = map.get(&line).map(|s| s.as_str()).unwrap_or("");
        let label = if body.contains("[error]") {
            "error"
        } else if body.contains("[ok]") {
            "ok"
        } else if body.contains("[trace]") {
            "trace"
        } else {
            "output"
        };
        out.push_str(&format!(
            r##"<li class="outline-item"><a href="#wb-{line}">L{line} · {label}</a></li>"##,
            line = line,
            label = label,
        ));
    }
    out.push_str("</ul></div>");
    out
}

fn render_stmt(
    stmt: &Stmt,
    lines: &[ClassifiedLine],
    mode: StructureMode,
    writebacks: &HashMap<u32, String>,
) -> (String, u32) {
    let end = stmt_end_line(stmt);
    let line = stmt_start(stmt);
    let inner = match stmt {
            Stmt::Assign {
                name,
                value: Expr::Formula(e),
                ..
            } => {
                let tex = escape(&formula_ascii_to_tex(&e.as_display()));
                format!(
                    "<div class=\"card formula-card\"><span class=\"badge\">formula</span> \
                     <code class=\"expr\">`{}` =</code> \
                     <div class=\"math-block\">$${}$$</div></div>",
                    escape(name),
                    tex
                )
            }
            Stmt::Assign {
                name,
                value: Expr::Code(c),
                ..
            } => {
                let placeholder = escape(&crate::foreign::default_cmd_display(&c.lang));
                let lang = escape(&c.lang);
                let src = escape(&c.source);
                format!(
                    r#"<div class="card code-card" data-lang="{lang}">
  <div class="code-head">
    <span class="badge">code</span>
    <code class="expr">`{name}` =</code>
    <span class="code-lang">{lang}</span>
    <div class="code-actions">
      <input type="text" class="code-cmd" placeholder="{placeholder}" spellcheck="false" autocomplete="off"/>
      <button type="button" class="code-run">Run</button>
    </div>
  </div>
  <pre class="code-body"><code class="language-{lang}">{src}</code></pre>
  <pre class="code-out" hidden></pre>
</div>"#,
                    lang = lang,
                    name = escape(name),
                    placeholder = placeholder,
                    src = src,
                )
            }
            Stmt::Assign { name, value, .. } => format!(
                "<div class=\"card\"><span class=\"badge\">bind</span> <code class=\"expr\">`{}` = {}</code></div>",
                escape(name),
                escape(&expr_display(value))
            ),
        Stmt::Return { value, .. } => format!(
            "<div class=\"card ret-card\"><span class=\"badge\">return</span><code class=\"expr\">{}</code></div>",
            escape(&expr_display(value))
        ),
        Stmt::Call { call, .. } => format!(
            "<div class=\"card call-card\"><span class=\"badge\">call</span><code class=\"expr\">{}</code></div>",
            escape(&call_display(call))
        ),
        Stmt::Branch { arms, .. } => {
            let mut body =
                String::from("<div class=\"card branch-card\"><span class=\"badge\">branch</span>");
            let mut arm_cursor = stmt_start(stmt) + 1;
            for arm in arms {
                let label = match &arm.condition {
                    None => "else".to_string(),
                    Some(c) => expr_display(c),
                };
                // comments before first stmt in arm (approx: between arms hard; use body starts)
                let arm_first = arm.body.first().map(stmt_start);
                body.push_str(&format!(
                    "<div class=\"arm\"><span class=\"badge\">arm</span><code class=\"expr\">{}</code>",
                    escape(&label)
                ));
                if let Some(af) = arm_first {
                    body.push_str(&emit_comments(lines, arm_cursor, af));
                    arm_cursor = af;
                }
                body.push_str("<div class=\"nested\">");
                for st in &arm.body {
                    let start = stmt_start(st);
                    body.push_str(&emit_comments(lines, arm_cursor, start));
                    let (h, e) = render_stmt(st, lines, mode, writebacks);
                    body.push_str(&h);
                    arm_cursor = e;
                }
                body.push_str("</div></div>");
            }
            body.push_str("</div>");
            body
        }
        Stmt::While {
            condition, body, ..
        } => {
            let mut body_html = format!(
                "<div class=\"card loop-card\"><span class=\"badge\">while</span><code class=\"expr\">{}</code><div class=\"nested\">",
                escape(&expr_display(condition))
            );
            let mut cursor = stmt_start(stmt) + 1;
            for st in body {
                let start = stmt_start(st);
                body_html.push_str(&emit_comments(lines, cursor, start));
                let (h, e) = render_stmt(st, lines, mode, writebacks);
                body_html.push_str(&h);
                cursor = e;
            }
            body_html.push_str("</div></div>");
            body_html
        }
        Stmt::ForEach {
            item,
            collection,
            body,
            ..
        } => {
            let mut body_html = format!(
                "<div class=\"card loop-card\"><span class=\"badge\">foreach</span><code class=\"expr\">[{}]({})</code><div class=\"nested\">",
                escape(item),
                escape(collection)
            );
            let mut cursor = stmt_start(stmt) + 1;
            for st in body {
                let start = stmt_start(st);
                body_html.push_str(&emit_comments(lines, cursor, start));
                let (h, e) = render_stmt(st, lines, mode, writebacks);
                body_html.push_str(&h);
                cursor = e;
            }
            body_html.push_str("</div></div>");
            body_html
        }
    };
    let inner = format!("{}{}", inner, output_card(line, writebacks));
    (stmt_shell(line, &inner, mode), end)
}

fn stmt_shell(line: u32, inner: &str, mode: StructureMode) -> String {
    match mode {
        StructureMode::Browse => format!(
            r#"<div class="stmt" data-line="{line}" id="stmt-{line}"><div class="stmt-body">{inner}</div></div>"#,
            line = line,
            inner = inner,
        ),
        StructureMode::Debug => format!(
            r#"<div class="stmt" data-line="{line}" id="stmt-{line}"><button type="button" class="bp-gutter" title="Toggle breakpoint" aria-label="Breakpoint line {line}"></button><div class="stmt-body">{inner}</div></div>"#,
            line = line,
            inner = inner,
        ),
    }
}

fn call_display(call: &CallExpr) -> String {
    let mut s = String::from("> ");
    if let Some(recv) = &call.receiver {
        s.push('`');
        s.push_str(recv);
        s.push('`');
        s.push('.');
        s.push_str(&call.callee);
    } else if let Some(path) = &call.path {
        s.push_str(&path.join("."));
    } else {
        s.push_str(&call.callee);
    }
    for a in &call.args {
        match a {
            Arg::Positional(e) => {
                s.push(' ');
                s.push_str(&expr_display(e));
            }
            Arg::Named { name, value } => {
                s.push(' ');
                s.push_str(name);
                s.push('=');
                s.push_str(&expr_display(value));
            }
        }
    }
    s
}

/// Surface-syntax preview for view (not Debug).
pub fn expr_display(expr: &Expr) -> String {
    expr_prec(expr, 0)
}

fn expr_prec(expr: &Expr, parent_prec: u8) -> String {
    match expr {
        Expr::Literal(lit) => lit_display(lit),
        Expr::Var(name) => format!("`{name}`"),
        Expr::Interp(parts) => {
            let mut s = String::new();
            for p in parts {
                match p {
                    InterpPart::Lit(t) => s.push_str(t),
                    InterpPart::Var(n) => {
                        s.push('`');
                        s.push_str(n);
                        s.push('`');
                    }
                }
            }
            s
        }
        Expr::Unary { op, expr } => {
            let prec = 5;
            let inner = match op {
                UnaryOp::Not => format!("not {}", expr_prec(expr, prec)),
                UnaryOp::Neg => format!("-{}", expr_prec(expr, prec)),
            };
            wrap(inner, prec, parent_prec)
        }
        Expr::Binary { op, left, right } => {
            let (sym, prec) = bin_sym_prec(*op);
            let inner = format!(
                "{} {} {}",
                expr_prec(left, prec),
                sym,
                expr_prec(right, prec + 1) // left-assoc bias
            );
            wrap(inner, prec, parent_prec)
        }
        Expr::Call(call) => call_display(call),
        Expr::List(items) => {
            let parts: Vec<String> = items.iter().map(|e| expr_prec(e, 0)).collect();
            format!("[{}]", parts.join(", "))
        }
        Expr::Map(pairs) => {
            let parts: Vec<String> = pairs
                .iter()
                .map(|(k, e)| format!("{}: {}", k, expr_prec(e, 0)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        Expr::Index { base, label } => {
            format!("{}[^{}]", expr_prec(base, 8), label)
        }
        Expr::Formula(e) => format!("$$ {} $$", e.as_display()),
        Expr::Code(c) => format!("```{} …```", c.lang),
    }
}

/// Best-effort ASCII formula → TeX for KaTeX in view Structure.
fn formula_ascii_to_tex(ascii: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = ascii.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '*' {
            // multiplication: a*b → a \cdot b
            out.push_str(r" \cdot ");
            i += 1;
            continue;
        }
        if c == '^' {
            out.push('^');
            i += 1;
            if i < chars.len() && chars[i] == '(' {
                out.push('{');
                i += 1;
                let start = i;
                let mut depth = 1i32;
                while i < chars.len() && depth > 0 {
                    match chars[i] {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        i += 1;
                    }
                }
                out.push_str(&chars[start..i].iter().collect::<String>());
                if i < chars.len() && chars[i] == ')' {
                    i += 1;
                }
                out.push('}');
            } else if i < chars.len() {
                out.push('{');
                out.push(chars[i]);
                out.push('}');
                i += 1;
            }
            continue;
        }
        // TeX specials that may appear in identifiers rarely
        match c {
            '{' | '}' | '%' | '&' | '#' | '$' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

fn wrap(inner: String, prec: u8, parent_prec: u8) -> String {
    if prec < parent_prec {
        format!("({inner})")
    } else {
        inner
    }
}

fn bin_sym_prec(op: BinaryOp) -> (&'static str, u8) {
    match op {
        BinaryOp::Or => ("or", 1),
        BinaryOp::And => ("and", 2),
        BinaryOp::Eq => ("==", 3),
        BinaryOp::Ne => ("!=", 3),
        BinaryOp::Lt => ("<", 3),
        BinaryOp::Le => ("<=", 3),
        BinaryOp::Gt => (">", 3),
        BinaryOp::Ge => (">=", 3),
        BinaryOp::Add => ("+", 4),
        BinaryOp::Sub => ("-", 4),
        BinaryOp::Mul => ("*", 5),
        BinaryOp::Div => ("/", 5),
    }
}

fn lit_display(lit: &Literal) -> String {
    match lit {
        Literal::None => "None".into(),
        Literal::Bool(true) => "True".into(),
        Literal::Bool(false) => "False".into(),
        Literal::Int(n) => n.to_string(),
        Literal::Text(t) => t.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Expr, Literal};

    #[test]
    fn expr_gt_is_surface() {
        let e = Expr::Binary {
            op: BinaryOp::Gt,
            left: Box::new(Expr::Var("x".into())),
            right: Box::new(Expr::Literal(Literal::Int(0))),
        };
        assert_eq!(expr_display(&e), "`x` > 0");
        assert!(!expr_display(&e).contains("Binary"));
    }

    #[test]
    fn structure_keeps_comments_and_surface_expr() {
        let src = include_str!("../../tests/structure/nested-call.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_structure(&module, src);
        assert!(html.contains("comment-card"));
        assert!(html.contains("print"));
        assert!(!html.contains("Binary {"));
        assert!(!html.contains("Literal(Int"));
    }

    #[test]
    fn comments_render_as_paragraphs() {
        let src = "段首说明\n续行仍属同段\n\n下一段\n\n# main\n\n> print text=x\n";
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_structure(&module, src);
        let cards = html.matches("comment-card").count();
        assert_eq!(cards, 2, "blank line should split comment paragraphs: {html}");
        assert!(html.contains("段首说明 续行仍属同段") || html.contains("段首说明</p>"));
        assert!(html.contains("下一段"));
    }

    #[test]
    fn comments_render_markdown_links() {
        let src = "见[仓库](https://github.com/cflmy/marqdo)与 `code`\n\n# main\n\n> print text=x\n";
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_structure(&module, src);
        assert!(
            html.contains("<a href=\"https://github.com/cflmy/marqdo\">仓库</a>"),
            "{html}"
        );
        assert!(html.contains("<code>code</code>"), "{html}");
        assert!(!html.contains("[仓库](https://github.com/cflmy/marqdo)"));
    }

    #[test]
    fn collect_input_prompts_from_fixture() {
        let src = include_str!("../../tests/keywords/input.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let prompts = collect_input_prompts(&module);
        assert_eq!(prompts, vec!["Name:".to_string()]);
    }

    #[test]
    fn collect_input_prompts_absent_without_input() {
        let src = include_str!("../../tests/structure/hello.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        assert!(collect_input_prompts(&module).is_empty());
    }

    #[test]
    fn structure_omits_imported_lib_bodies() {
        let src = "---\ntitle: t\n> lib/text.mq.md\n---\n\n# main\n\n> print text=hi\n";
        let local = crate::parse::parse_source(src).unwrap();
        let html = render_module_structure(&local, src);
        assert!(html.contains("badge\">import"), "{html}");
        assert!(html.contains("lib/text.mq.md"), "{html}");
        assert!(html.contains("main"), "{html}");
        assert!(!html.contains("trim"), "{html}");
        assert!(html.contains("class=\"stmt\""), "{html}");
        assert!(html.contains("data-line="), "{html}");
        assert_eq!(local.functions.len(), 1);
        assert_eq!(local.functions[0].name, "main");
    }

    #[test]
    fn structure_formula_assign_not_comment() {
        let src = include_str!("../../tests/lib/math-formula.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_structure(&module, src);
        assert!(html.contains("badge\">formula"), "{html}");
        assert!(html.contains("math-block"), "{html}");
        assert!(html.contains("$$"), "{html}");
        assert!(
            !html.contains("comment-text\">x^2 - 2")
                && !html.contains("comment-text\"><p>x^2 - 2"),
            "formula fence leaked into comments: {html}"
        );
    }

    #[test]
    fn structure_code_card() {
        let src = include_str!("../../tests/lib/foreign-python.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let html = render_module_structure(&module, src);
        assert!(html.contains("code-card"), "{html}");
        assert!(html.contains("code-run"), "{html}");
        assert!(html.contains("hello-from-python"), "{html}");
        assert!(!html.contains("python name=hi"), "{html}");
    }

    #[test]
    fn formula_ascii_to_tex_pow_and_mul() {
        assert_eq!(formula_ascii_to_tex("x^2 - 2"), "x^{2} - 2");
        assert_eq!(formula_ascii_to_tex("2*x"), r"2 \cdot x");
    }

    #[test]
    fn structure_mode_browse_vs_debug() {
        let src = include_str!("../../tests/structure/nested-call.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let browse = render_module_structure_mode(&module, src, StructureMode::Browse);
        let debug = render_module_structure_mode(&module, src, StructureMode::Debug);
        assert!(browse.contains("id=\"fn-"), "{browse}");
        assert!(!browse.contains("bp-gutter"), "{browse}");
        assert!(debug.contains("bp-gutter"), "{debug}");
        assert!(debug.contains("id=\"stmt-"), "{debug}");
    }

    #[test]
    fn function_outline_lists_nested_names() {
        let src = include_str!("../../tests/structure/nested-call.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let outline = render_function_outline(&module);
        assert!(outline.contains("outline-item"), "{outline}");
        assert!(outline.contains("data-fn=\"main\""), "{outline}");
        assert!(outline.contains("data-fn=\"问候\""), "{outline}");
        assert!(outline.contains("data-fn-path=\"main/问候\""), "{outline}");
        assert!(outline.contains("href=\"#fn-"), "{outline}");
        assert!(outline.contains("ol-meta\">h"), "{outline}");
    }

    #[test]
    fn right_rail_has_functions_and_variables() {
        let src = include_str!("../../tests/structure/hello.mq.md");
        let module = crate::parse::parse_source(src).unwrap();
        let mut bind = HashMap::new();
        bind.insert(
            "xs".into(),
            crate::value::Value::List(vec![
                crate::value::Value::Text("a".into()),
                crate::value::Value::Text("b".into()),
            ]),
        );
        let html = render_right_rail(&module, &render_bindings_html(&bind), "");
        assert!(html.contains("outline-rail"), "{html}");
        assert!(html.contains("Functions"), "{html}");
        assert!(html.contains("Variables"), "{html}");
        assert!(html.contains("vars-panel"), "{html}");
        assert!(html.contains("vars-table"), "{html}");
        assert!(html.contains(">xs<") || html.contains("vars-name\">xs"), "{html}");
    }

    #[test]
    fn stream_panel_gated_on_llm_or_agent_import() {
        let hello = crate::parse::parse_source(include_str!("../../tests/structure/hello.mq.md"))
            .unwrap();
        assert!(!uses_stream_panel(&hello));
        let llm = crate::parse::parse_source(include_str!("../../tests/ext/llm-import.mq.md")).unwrap();
        assert!(uses_stream_panel(&llm));
    }
}
