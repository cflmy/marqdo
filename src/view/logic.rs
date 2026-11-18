//! Read-only presentation of link-shaped descriptions. No evaluation or fetching.
use crate::ast::{Arg, CallExpr, Expr, Literal, Module};
use crate::view::{html::escape, render::expr_display};
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use std::collections::HashMap;

pub(crate) struct Span<'a> {
    pub start: usize,
    pub end: usize,
    pub label: &'a str,
    pub payload: &'a str,
}

/// Preserve quoted source data in executable spans and inline code in narration.
pub(crate) fn spans(text: &str, code: bool) -> Vec<Span<'_>> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' { i = (i + 2).min(bytes.len()); continue; }
        if bytes[i] == b'\x60' || code && matches!(bytes[i], b'"' | b'\'') {
            let quote = bytes[i];
            let run = if quote == b'\x60' { bytes[i..].iter().take_while(|b| **b == quote).count() } else { 1 };
            i += run;
            while i < bytes.len() {
                if bytes[i] == b'\\' { i = (i + 2).min(bytes.len()); }
                else if bytes[i..].starts_with(&vec![quote; run]) { i += run; break; }
                else { i += 1; }
            }
            continue;
        }
        if bytes[i] != b'[' || bytes.get(i + 1) == Some(&b'^')
            || i > 0 && bytes[i - 1] == b'!' {
            i += 1; continue;
        }
        let start = i;
        let mut j = i + 1;
        let mut depth = 1;
        while j < bytes.len() {
            match bytes[j] {
                b'\\' => { j += 2; continue; }
                b'[' => depth += 1,
                b']' => { depth -= 1; if depth == 0 { break; } }
                _ => {}
            }
            j += 1;
        }
        if depth != 0 || bytes.get(j + 1) != Some(&b'(') { i += 1; continue; }
        let label_end = j;
        j += 2;
        let payload_start = j;
        depth = 1;
        while j < bytes.len() {
            match bytes[j] {
                b'\\' => { j += 2; continue; }
                b'"' | b'\'' | b'\x60' if code || j == payload_start
                    || bytes.get(j.wrapping_sub(1)).is_some_and(|b| b" (=,[{:\t".contains(b)) => {
                    let quote = bytes[j]; j += 1;
                    while j < bytes.len() {
                        if bytes[j] == b'\\' { j += 2; }
                        else if bytes[j] == quote { j += 1; break; }
                        else { j += 1; }
                    }
                    continue;
                }
                b'(' => depth += 1,
                b')' => { depth -= 1; if depth == 0 { break; } }
                _ => {}
            }
            j += 1;
        }
        if depth == 0 {
            found.push(Span { start, end: j + 1, label: &text[start + 1..label_end], payload: text[payload_start..j].trim() });
            i = j + 1;
        } else { i += 1; }
    }
    found
}

pub(crate) fn id(line: u32, at: usize) -> String { format!("logic-{line}-{at}") }

fn markdown(text: &str) -> String {
    let parser = Parser::new_ext(text, Options::ENABLE_STRIKETHROUGH).map(|event| match event {
        Event::Html(value) | Event::InlineHtml(value) => Event::Text(value),
        Event::Start(Tag::Image { .. }) => Event::Html("<span class=\"logic-image-label\">".into()),
        Event::End(pulldown_cmark::TagEnd::Image) => Event::Html("</span>".into()),
        Event::Start(Tag::Link { dest_url, .. }) => Event::Html(format!("<span title=\"{}\">", escape(&dest_url)).into()),
        Event::End(pulldown_cmark::TagEnd::Link) => Event::Html("</span>".into()),
        other => other,
    });
    let mut rendered = String::new();
    html::push_html(&mut rendered, parser);
    rendered
}
fn label_html(label: &str) -> String {
    let label = if label.is_empty() { "未命名内容" } else { label };
    let value = markdown(label);
    value.strip_prefix("<p>").and_then(|v| v.strip_suffix("</p>\n")).unwrap_or(&value).to_string()
}
fn safe_href(target: &str) -> bool {
    let normalized: String = target.chars().filter(|c| !c.is_ascii_whitespace() && !c.is_control()).collect();
    let lower = normalized.to_ascii_lowercase();
    !lower.starts_with("javascript:") && !lower.starts_with("vbscript:")
        && !lower.starts_with("data:") && !lower.starts_with("file:")
}
fn destination(payload: &str) -> Option<String> {
    let candidate = format!("[target]({payload})");
    let target = Parser::new(&candidate).find_map(|event| match event {
        Event::Start(Tag::Link { dest_url, .. }) => Some(dest_url.into_string()),
        _ => None,
    }).unwrap_or_else(|| payload.trim_matches(['<', '>']).to_string());
    let scheme = target.split_once(':').is_some_and(|(scheme, _)|
        scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')));
    let relative_path = target.contains('/') && !target.contains(['(', ')'])
        && !target.chars().any(char::is_whitespace)
        && target.starts_with(|c: char| c.is_alphabetic());
    let looks_like_link = scheme || relative_path || target.starts_with('#') || target.starts_with('/')
        || target.starts_with("./") || target.starts_with("../")
        || target.starts_with("mailto:") || target.starts_with("tel:")
        || target.to_ascii_lowercase().starts_with("javascript:")
        || target.to_ascii_lowercase().starts_with("data:")
        || target.to_ascii_lowercase().starts_with("vbscript:")
        || [".mq.md", ".md", ".html", ".pdf", ".txt", ".json", ".svg", ".png"]
            .iter().any(|ext| target.split(['?', '#']).next().unwrap_or("").ends_with(ext));
    looks_like_link.then_some(target)
}

struct Payload { kind: &'static str, heading: &'static str, body: String, href: Option<String> }
fn payload(text: &str) -> Payload {
    if text.trim().is_empty() {
        return Payload { kind:"empty", heading:"待补充", body:"<p class=\"logic-empty\">待补充实际逻辑</p>".into(), href:None };
    }
    let expression = crate::parse::expr::parse_expr_prefer_var(text).ok();
    if let Some(target) = destination(text).filter(|_| !matches!(expression, Some(Expr::List(_) | Expr::Map(_) | Expr::Literal(_)))) {
        let body = if safe_href(&target) {
            let external = target.starts_with("https://") || target.starts_with("http://") || target.starts_with("//");
            let attributes = if external { " target=\"_blank\" rel=\"noopener noreferrer\"" } else { "" };
            format!("<a class=\"logic-link-target\" href=\"{}\" data-document-href=\"{}\"{attributes}><span class=\"logic-link-address\">{}</span><span class=\"logic-link-open\">打开链接 ↗</span></a>", escape(&target), escape(&target), escape(&target))
        } else {
            format!("<p class=\"logic-link-address\">{}</p><p class=\"logic-hint\">此目标仅作为文字显示，不启用跳转。</p>", escape(&target))
        };
        return Payload { kind:"link", heading:"跳转链接", body, href:safe_href(&target).then_some(target) };
    }
    if text.starts_with(['[', '{']) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
            return Payload { kind:"value", heading:"内容与值", body:json_html(&value, 0), href:None };
        }
    }
    if let Some(expression) = expression {
        let (kind, heading) = match expression {
            Expr::Call(_) => ("call", "函数调用"),
            Expr::List(_) | Expr::Map(_) | Expr::Literal(_) => ("value", "内容与值"),
            Expr::Var(_) => ("reference", "引用"),
            _ => ("expression", "表达式"),
        };
        return Payload { kind, heading, body:expression_html(&expression, 0), href:None };
    }
    Payload { kind:"content", heading:"内容说明", body:format!("<div class=\"logic-content\">{}</div>", markdown(text)), href:None }
}
/// JSON is document content here, never passed to the evaluator as an expression.
fn json_html(value: &serde_json::Value, depth: usize) -> String {
    if depth > 5 { return format!("<code class=\"logic-expression\">{}</code>", escape(&value.to_string())); }
    match value {
        serde_json::Value::Array(values) => {
            if values.is_empty() { return "<span class=\"logic-empty-value\">空列表</span>".into(); }
            format!("<ol class=\"logic-values\">{}</ol>", values.iter()
                .map(|v| format!("<li>{}</li>", json_html(v, depth + 1))).collect::<String>())
        }
        serde_json::Value::Object(values) => {
            if values.is_empty() { return "<span class=\"logic-empty-value\">空记录</span>".into(); }
            format!("<dl class=\"logic-arguments\">{}</dl>", values.iter()
                .map(|(key, v)| format!("<div class=\"logic-argument\"><dt>{}</dt><dd>{}</dd></div>", escape(key), json_html(v, depth + 1))).collect::<String>())
        }
        serde_json::Value::String(value) => format!("<span class=\"logic-string\">{}</span>", escape(value)),
        _ => format!("<code class=\"logic-expression\">{}</code>", escape(&value.to_string())),
    }
}
fn call_name(call: &CallExpr) -> String {
    if let Some(receiver) = &call.receiver { format!("{receiver}.{}", call.callee) }
    else if let Some(path) = &call.path { path.join(".") }
    else { call.callee.clone() }
}
fn expression_html(expression: &Expr, depth: usize) -> String {
    if depth > 5 { return format!("<code class=\"logic-expression\">{}</code>", escape(&expr_display(expression))); }
    match expression {
        Expr::Call(call) => {
            let name = call_name(call);
            let mut out = format!("<div class=\"logic-call\"><code class=\"logic-call-name\" data-call-name=\"{}\">{}</code>", escape(&name), escape(&name));
            if !call.pre_modifiers.is_empty() { out.push_str(&format!("<small class=\"logic-hint\">{}</small>", escape(&call.pre_modifiers.join(" ")))); }
            if call.args.is_empty() { out.push_str("<p class=\"logic-no-args\">无需传入参数</p>"); }
            else {
                out.push_str("<dl class=\"logic-arguments\">");
                for (index, argument) in call.args.iter().enumerate() {
                    let (name, value) = match argument {
                        Arg::Named { name, value } => (name.clone(), value),
                        Arg::Positional(value) => (format!("参数 {}", index + 1), value),
                    };
                    out.push_str(&format!("<div class=\"logic-argument\"><dt>{}</dt><dd>{}</dd></div>", escape(&name), expression_html(value, depth + 1)));
                }
                out.push_str("</dl>");
            }
            out.push_str("</div>"); out
        }
        Expr::Literal(Literal::Text(value)) =>
            format!("<span class=\"logic-string\">{}</span>", escape(value)),
        Expr::Literal(_) | Expr::Var(_) =>
            format!("<code class=\"logic-expression\">{}</code>", escape(&expr_display(expression))),
        Expr::List(values) => {
            if values.is_empty() { return "<span class=\"logic-empty-value\">空列表</span>".into(); }
            let rows = values.iter().map(|v| format!("<li>{}</li>", expression_html(v, depth + 1))).collect::<String>();
            format!("<ol class=\"logic-values\">{rows}</ol>")
        }
        Expr::Map(values) => {
            if values.is_empty() { return "<span class=\"logic-empty-value\">空记录</span>".into(); }
            let rows = values.iter().map(|(key, value)| format!("<div class=\"logic-argument\"><dt>{}</dt><dd>{}</dd></div>", escape(key), expression_html(value, depth + 1))).collect::<String>();
            format!("<dl class=\"logic-arguments\">{rows}</dl>")
        }
        Expr::Occluded { label, payload } => format!("<div class=\"logic-nested\"><span class=\"logic-nested-label\">{}</span>{}</div>", escape(label), payload.as_deref().map(|v|expression_html(v,depth+1)).unwrap_or_else(||"<span class=\"logic-empty-value\">待补充实际逻辑</span>".into())),
        _ => format!("<code class=\"logic-expression\">{}</code>", escape(&expr_display(expression))),
    }
}

pub(crate) fn source(line: u32, span: &Span<'_>, base: usize) -> String {
    let id = id(line, base + span.start);
    let item = payload(span.payload);
    let attrs = format!("id=\"{id}-source\" data-logic-id=\"{id}\" data-line=\"{line}\" data-byte-start=\"{}\" data-byte-end=\"{}\" aria-describedby=\"{id}-note\"", base+span.start,base+span.end);
    let label = label_html(span.label);
    let mut out = if let Some(href) = item.href {
        format!("<a href=\"{}\" data-document-href=\"{}\" class=\"occl-label logic-link-label mq-anchor\" {attrs}>{label}</a>", escape(&href), escape(&href))
    } else {
        format!("<button type=\"button\" class=\"occl-label mq-anchor\" {attrs} aria-controls=\"{id}-note\" title=\"查看{}（阅读不会执行）\">{label}</button>", item.heading)
    };
    if item.kind == "empty" { out.push_str("<span class=\"occl-stub\">待补充实际逻辑</span>"); }
    out
}
pub(crate) fn note(line: u32, span: &Span<'_>, base: usize, executable: bool) -> String {
    let id = id(line, base+span.start);
    let item = payload(span.payload);
    let hint = if executable { "源码说明 · 不是运行结果" } else { "文档引用 · 阅读不会执行" };
    let original = if span.payload.is_empty() && executable { "待补充实际逻辑；当前空遮蔽返回 None。" } else { span.payload };
    format!(r#"<details class="annot-chip annot-implementation" id="{id}-note" data-annot-id="{id}" data-annot-kind="implementation" data-logic-kind="{kind}" data-anchor-line="{line}" data-anchor-byte-start="{start}" data-anchor-byte-end="{end}"><summary class="annot-label"><span class="logic-kind">{heading}</span><span class="logic-note-label">{label}</span></summary><div class="annot-value logic-body">{body}<details class="logic-source"><summary>原始写法</summary><pre><code class="occl-code">{original}</code></pre></details></div><small class="logic-hint">{hint}</small></details>"#,
        kind=item.kind, start=base+span.start, end=base+span.end, heading=item.heading,
        label=label_html(span.label), body=item.body, original=escape(original))
}

pub(crate) struct Narrative {
    pub markdown: String,
    pub replacements: Vec<(String,String)>,
    pub notes: String,
}
pub(crate) fn narrative(line: u32, text: &str, base: usize) -> Narrative {
    let mut result = Narrative { markdown:String::new(), replacements:Vec::new(), notes:String::new() };
    let mut prefix = format!("MQVIEWLINK{line}X{base}Q");
    while text.contains(&prefix) { prefix.push('Q'); }
    let mut cursor = 0;
    for (index, span) in spans(text, false).into_iter().enumerate() {
        let token = format!("{prefix}{index}END");
        result.markdown.push_str(&text[cursor..span.start]);
        result.markdown.push_str(&token);
        result.replacements.push((token,source(line,&span,base)));
        result.notes.push_str(&note(line,&span,base,false));
        cursor=span.end;
    }
    result.markdown.push_str(&text[cursor..]);
    result
}
pub(crate) fn replace(mut html: String, replacements: &[(String,String)]) -> String {
    for (token,value) in replacements { html=html.replace(token,value); }
    html
}

/// Link only unambiguous definitions in this document.
pub(crate) fn link_definitions(mut html: String, module: &Module) -> String {
    fn walk(functions: &[crate::ast::Function], prefix:&str, names:&mut HashMap<String,Vec<u32>>) {
        for function in functions {
            let path=if prefix.is_empty(){function.name.clone()}else{format!("{prefix}.{}",function.name)};
            names.entry(path.clone()).or_default().push(function.span.line);
            if !prefix.is_empty(){names.entry(function.name.clone()).or_default().push(function.span.line);}
            walk(&function.children,&path,names);
        }
    }
    let mut names=HashMap::new();walk(&module.functions,"",&mut names);
    for (name,lines) in names {
        if lines.len()!=1 {continue;}
        let escaped=escape(&name);
        html=html.replace(&format!("<code class=\"logic-call-name\" data-call-name=\"{escaped}\">{escaped}</code>"),
            &format!("<a class=\"logic-definition\" href=\"#fn-{}\" title=\"查看本页函数定义\"><code class=\"logic-call-name\">{escaped}</code><span aria-hidden=\"true\"> ↗</span></a>",lines[0]));
    }
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn calls_and_collections_containing_urls_remain_structured_values() {
        let call = payload("fetch(url=\"https://example.com/file.mq.md\", options={retry: 2})");
        assert_eq!(call.kind, "call");
        assert!(call.body.contains("logic-call-name") && call.body.contains("<dt>url</dt>"));
        assert!(call.body.contains("<dt>options</dt>"));
        assert!(!call.body.contains("logic-link-target"));
        for text in ["[\"https://example.com\", 2]", "{\"url\": \"https://example.com\"}", "\"guide.mq.md\""] {
            assert_eq!(payload(text).kind, "value", "{text}");
        }
    }
    #[test]
    fn balanced_scanning_preserves_inline_code_quotes_and_byte_anchors() {
        let code = "\"[data](untouched)\" [操作](f(value=\")\", nested=g(2))) [空]()";
        let found = spans(code, true);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].payload, "f(value=\")\", nested=g(2))");
        assert_eq!(&code[found[0].start..found[0].end], "[操作](f(value=\")\", nested=g(2)))");
        let narrative = "don't hide [链接](https://example.com/a(b)) or `[代码](opaque)`";
        let found = spans(narrative, false);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].label, "链接");
    }
    #[test]
    fn unsafe_targets_and_html_are_displayed_without_navigation_or_execution() {
        for text in ["javascript:alert(1)", "data:text/html,<svg/onload=alert(1)>", "vbscript:evil"] {
            let item = payload(text);
            assert_eq!(item.kind, "link");
            assert!(item.href.is_none());
            assert!(!item.body.contains("<a "));
        }
        let item = payload("<script>alert(1)</script> beautiful prose");
        assert!(!item.body.contains("<script>"));
        assert!(item.body.contains("&lt;script&gt;"));
    }
    #[test]
    fn markdown_link_titles_and_nested_parentheses_keep_the_destination() {
        for (text, target) in [("https://example.com/a(b)", "https://example.com/a(b)"),
            ("<guide.mq.md> \"说明\"", "guide.mq.md"), ("#fn-7", "#fn-7")] {
            assert_eq!(payload(text).href.as_deref(), Some(target));
        }
    }
}
