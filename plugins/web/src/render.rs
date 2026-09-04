//! Minimal HTML shell for assembled pages / parts.

use serde_json::{json, Map, Value};

use crate::db;
use crate::markdown;
use crate::table::{as_bind, normalize_ref, normalize_slot, parse_site_path, project_rows, SitePath};

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn class_attr(css: &str) -> String {
    if css.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", esc(css))
    }
}

fn is_simple_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || !c.is_ascii())
}

/// True when `back` is a DB field path (`table.col` / `mod.table.col`), not a URL or site path.
fn is_db_bind_back(back: &str) -> bool {
    let s = normalize_ref(back);
    if s.is_empty()
        || s.contains("://")
        || s.starts_with('/')
        || s.starts_with('#')
        || s.starts_with('?')
    {
        return false;
    }
    match parse_site_path(&s) {
        SitePath::DbField { .. } => true,
        SitePath::LibMember { lib, member } => is_simple_ident(&lib) && is_simple_ident(&member),
        SitePath::Plain(p) => {
            let parts: Vec<&str> = p.split('.').filter(|x| !x.is_empty()).collect();
            parts.len() == 2 && parts.iter().all(|x| is_simple_ident(x))
        }
    }
}

fn resolve_links(raw: Option<&Value>, db_url: Option<&str>) -> Vec<(String, String)> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let binds = as_bind(raw);
    let arr = binds.as_array().cloned().unwrap_or_default();
    if arr.is_empty() {
        return Vec::new();
    }
    let has_db = arr
        .iter()
        .any(|b| b.get("back").and_then(|v| v.as_str()).is_some_and(is_db_bind_back));
    if has_db {
        let Some(url) = db_url else {
            return Vec::new();
        };
        let table = crate::table::bind_table_name(&arr);
        let Some(table) = table else {
            return Vec::new();
        };
        let data = select_page_data(url, &table, raw, 200, 0);
        let rows = data
            .get("rows")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let projected = project_rows(&arr, &rows);
        let mut out = Vec::new();
        for it in projected.as_array().cloned().unwrap_or_default() {
            let m = it.as_object().cloned().unwrap_or_default();
            let label = m
                .get("title")
                .or_else(|| m.get("label"))
                .map(text)
                .unwrap_or_default();
            let href = m
                .get("href")
                .or_else(|| m.get("path"))
                .map(text)
                .unwrap_or_else(|| "#".into());
            if !label.is_empty() {
                out.push((label, href));
            }
        }
        return out;
    }
    // static: front=label back=href (URLs with `.` must stay here)
    arr.into_iter()
        .map(|b| {
            let label = b.get("front").map(text).unwrap_or_default();
            let href = b.get("back").map(text).unwrap_or_else(|| "#".into());
            (label, href)
        })
        .filter(|(l, _)| !l.is_empty())
        .collect()
}

fn render_ul(links: &[(String, String)], class: &str) -> String {
    let mut s = format!("<ul class=\"{}\">", esc(class));
    for (label, href) in links {
        s.push_str(&format!(
            "<li><a href=\"{}\">{}</a></li>",
            esc(href),
            esc(label)
        ));
    }
    s.push_str("</ul>");
    s
}

fn resolve_main(args: &Value, db_url: Option<&str>) -> (String, Vec<Map<String, Value>>, Option<i64>) {
    let intro = args
        .get("intro")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let Some(main) = args.get("main") else {
        return (intro, Vec::new(), None);
    };
    let binds = as_bind(main);
    let arr = binds.as_array().cloned().unwrap_or_default();
    if arr.is_empty() {
        return (intro, Vec::new(), None);
    }
    let Some(url) = db_url else {
        return (intro, Vec::new(), None);
    };
    let Some(table) = crate::table::bind_table_name(&arr) else {
        return (intro, Vec::new(), None);
    };
    let limit = args
        .get("paginate")
        .and_then(|p| p.get("limit"))
        .and_then(|v| v.as_i64())
        .unwrap_or(200);
    let offset = args
        .get("paginate")
        .and_then(|p| p.get("offset"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let data = select_page_data(url, &table, args, limit, offset);
    let total = data.get("total").and_then(|v| v.as_i64());
    let rows = data
        .get("rows")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let projected = project_rows(&arr, &rows);
    let items = projected
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_object().cloned())
        .collect();
    (intro, items, total)
}

/// Run the page's DB query: `where` (with `{param}` placeholders resolved from
/// `params`) and optional `order`, then project through the bind table.
fn select_page_data(
    url: &str,
    table: &str,
    args: &Value,
    limit: i64,
    offset: i64,
) -> Value {
    let mut where_v = args.get("query").cloned();
    if let Some(m) = where_v.as_ref().and_then(|v| v.as_object()) {
        if m.iter().any(|(_, v)| v.as_str().is_some_and(|s| s.contains('{'))) {
            let mut resolved = Map::new();
            for (k, v) in m {
                let v = match v {
                    Value::String(s) => Value::String(resolve_params(s.as_str(), args)),
                    other => other.clone(),
                };
                resolved.insert(k.clone(), v);
            }
            where_v = Some(Value::Object(resolved));
        }
    }
    let order = args.get("order").and_then(|v| v.as_str());
    let paginate = args.get("paginate").is_some();
    db::select_order(
        url,
        table,
        limit,
        where_v.as_ref(),
        order.filter(|s| !s.trim().is_empty()),
        if paginate || offset > 0 {
            Some(offset)
        } else {
            None
        },
        None,
    )
    .unwrap_or(json!({ "rows": [] }))
}

/// Replace `{param}` placeholders with values from `args["params"]` (a map).
fn resolve_params(s: &str, args: &Value) -> String {
    let params = args
        .get("params")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('}') {
            let key = &rest[..end];
            let val = params
                .get(key)
                .map(text)
                .unwrap_or_else(|| format!("{{{key}}}"));
            out.push_str(&val);
            rest = &rest[end + 1..];
        } else {
            out.push('{');
            break;
        }
    }
    out.push_str(rest);
    out
}

fn field_css(obj: &Map<String, Value>, field: &str) -> String {
    obj.get("_css")
        .and_then(|v| v.as_object())
        .and_then(|m| m.get(field))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn part_src_prefix(args: &Value) -> String {
    match args.get("_route").and_then(|v| v.as_str()) {
        Some(r) if !r.is_empty() && r != "/" => {
            let mut p = r.to_string();
            while p.len() > 1 && p.ends_with('/') {
                p.pop();
            }
            if p.starts_with('/') {
                p
            } else {
                format!("/{p}")
            }
        }
        _ => String::new(),
    }
}

fn slot_attrs(slot: &str, parts: &Map<String, Value>, page: &Value) -> String {
    let prefix = part_src_prefix(page);
    let mut a = format!(" data-slot=\"{}\"", esc(slot));
    for (id, cfg) in parts {
        let s = cfg
            .get("slot")
            .or_else(|| cfg.get("fragment"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if s == slot || (slot == "sidebar" && id == "side") || (slot == "main" && id == "index") {
            a.push_str(&format!(
                " data-slot-src=\"{}/_part/{}\"",
                esc(&prefix),
                esc(id)
            ));
            break;
        }
    }
    a
}

fn slot_class(args: &Value, slot: &str, base: &str) -> String {
    let extra = args
        .get("slot_class")
        .and_then(|v| v.as_object())
        .and_then(|m| m.get(slot))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if extra.is_empty() {
        base.to_string()
    } else {
        format!("{base} {extra}")
    }
}

const SHELL_VARS: &str = r#"
:root { --ink:#1c1917; --muted:#57534e; --paper:#fafaf9; --line:#e7e5e4; --accent:#0f766e; }
body { margin:0; font-family: "IBM Plex Sans", "Noto Sans SC", sans-serif; background:var(--paper); color:var(--ink); }
a { color:var(--ink); text-decoration:none; }
a:hover { color:var(--accent); }
"#;

const SHELL_LAYOUT: &str = r#"
body { display:grid; min-height:100vh; grid-template-rows:auto 1fr auto; }
body.has-sidebar { grid-template-columns:14rem 1fr; grid-template-areas:"top top" "side main" "foot foot"; }
body.no-sidebar { grid-template-areas:"top" "main" "foot"; }
@media (max-width:720px) {
  body.has-sidebar { grid-template-columns:1fr; grid-template-areas:"top" "main" "foot" "side"; }
  body.has-sidebar aside.side { border-right:0; border-top:1px solid var(--line); }
}
body.layout-stacked { display:flex; flex-direction:column; min-height:100vh; }
body.layout-stacked header.topnav, body.layout-stacked aside.side, body.layout-stacked main.main, body.layout-stacked footer.foot { width:100%; }
body.layout-stacked aside.side { border-right:0; border-bottom:1px solid var(--line); }
body.layout-bare { display:block; min-height:100vh; }
body.layout-bare main.main { padding:1.5rem 1.25rem 2rem; }
header.topnav { grid-area:top; border-bottom:1px solid var(--line); padding:.85rem 1.25rem; background:#fff; }
aside.side { grid-area:side; border-right:1px solid var(--line); padding:1.25rem 1rem; background:#f5f5f4; }
main.main { grid-area:main; padding:1.5rem 1.25rem 2rem; }
footer.foot { grid-area:foot; border-top:1px solid var(--line); padding:.85rem 1.25rem; color:var(--muted); background:#fff; }
ul.nav, ul.side-nav, ul.foot-nav { list-style:none; margin:0; padding:0; display:flex; flex-wrap:wrap; gap:.35rem 1rem; }
ul.side-nav { flex-direction:column; }
"#;

const SHELL_WIDGETS: &str = r#"
.content.cards { display:grid; gap:1rem; margin-top:1.25rem; grid-template-columns:repeat(auto-fill,minmax(16rem,1fr)); }
.content.cards article { background:#fff; border:1px solid var(--line); border-radius:6px; padding:1rem; }
.main-intro h1 { margin:0 0 .75rem; font-size:2rem; }
.main-intro p { color:var(--muted); }
.site-form { margin-top:1.25rem; max-width:28rem; }
.site-form form { display:grid; gap:.85rem; }
.site-form label { display:grid; gap:.25rem; font-size:.9rem; }
.site-form input, .site-form textarea { padding:.5rem .6rem; border:1px solid var(--line); border-radius:4px; font:inherit; background:#fff; }
.site-form input[readonly] { background:#f5f5f4; color:var(--muted); }
.site-form .err { color:#b91c1c; font-size:.85rem; }
.site-form .actions { display:flex; gap:.75rem; align-items:center; flex-wrap:wrap; }
.site-form button { background:var(--accent); color:#fff; border:0; padding:.55rem 1rem; border-radius:4px; cursor:pointer; }
.site-form .meta { color:var(--muted); font-size:.9rem; }
.article { background:#fff; border:1px solid var(--line); border-radius:8px; padding:2rem; margin-top:1.5rem; }
.article-meta { color:var(--muted); font-size:.85rem; }
.article-title { margin:0 0 .75rem; font-size:2rem; }
.article-tags { margin-bottom:1rem; }
.article-body { line-height:1.75; color:var(--ink); }
.article-p { margin:0 0 1rem; white-space:pre-line; }
.article-h2 { margin:1.5rem 0 .75rem; }
.article-body.md { line-height: 1.75; }
.article-body.md pre { background:#f5f5f4; border:1px solid var(--line); border-radius:6px; padding:1rem; overflow-x:auto; }
.article-body.md code { font-size:.9em; }
.pagination { display:flex; gap:1rem; align-items:center; margin:1.5rem 0; font-size:.95rem; }
.pagination a { color:var(--accent); text-decoration:none; }
.pagination a:hover { text-decoration:underline; }
.pagination .page-status { color:var(--muted); }
"#;

fn normalize_shell_css(raw: &str) -> &'static str {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "full" | "default" => "full",
        "minimal" | "min" | "vars" => "minimal",
        "off" | "none" | "false" | "0" => "off",
        _ => "full",
    }
}

fn shell_css_for(mode: &str) -> String {
    match normalize_shell_css(mode) {
        "off" => String::new(),
        "minimal" => SHELL_VARS.to_string(),
        _ => format!("{SHELL_VARS}{SHELL_LAYOUT}{SHELL_WIDGETS}"),
    }
}

fn normalize_layout(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "" | "auto" | "default" => String::new(),
        "sidebar" | "side" => "sidebar".into(),
        "stacked" | "stack" | "column" => "stacked".into(),
        "bare" | "main" | "none" => "bare".into(),
        "rail" => "rail".into(),
        other => other.to_string(),
    }
}

fn resolve_shell_css(args: &Value) -> String {
    let raw = args
        .get("shell_css")
        .or_else(|| args.get("壳样式"))
        .and_then(|v| v.as_str())
        .unwrap_or("full");
    shell_css_for(raw)
}

fn resolve_layout(args: &Value) -> String {
    let raw = args
        .get("layout")
        .or_else(|| args.get("布局"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    normalize_layout(raw)
}

fn head_html(args: &Value, default_title: &str) -> String {
    let meta = args.get("meta").and_then(|v| v.as_object());
    let page_title = meta
        .and_then(|m| m.get("title"))
        .map(text)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| default_title.to_string());
    let mut s = format!("<title>{}</title>", esc(&page_title));
    if let Some(m) = meta {
        for (k, v) in m {
            if k == "title" {
                continue;
            }
            let val = text(v);
            if val.is_empty() {
                continue;
            }
            match k.as_str() {
                "description" => {
                    s.push_str(&format!(
                        "<meta name=\"description\" content=\"{}\"/>",
                        esc(&val)
                    ));
                }
                "canonical" => {
                    s.push_str(&format!("<link rel=\"canonical\" href=\"{}\"/>", esc(&val)));
                }
                "icon" | "favicon" => {
                    s.push_str(&format!(
                        "<link rel=\"icon\" href=\"{}\"/>",
                        esc(&val)
                    ));
                }
                "apple-touch-icon" | "apple_touch_icon" => {
                    s.push_str(&format!(
                        "<link rel=\"apple-touch-icon\" href=\"{}\"/>",
                        esc(&val)
                    ));
                }
                k if k.starts_with("og:") => {
                    s.push_str(&format!(
                        "<meta property=\"{}\" content=\"{}\"/>",
                        esc(k),
                        esc(&val)
                    ));
                }
                other => {
                    s.push_str(&format!(
                        "<meta name=\"{}\" content=\"{}\"/>",
                        esc(other),
                        esc(&val)
                    ));
                }
            }
        }
    }
    if let Some(head) = args.get("head") {
        let links = crate::assets::head_links_from_json(head);
        s.push_str(&crate::assets::render_head_links(&links));
    }
    s
}

fn render_pagination(args: &Value, total: i64, item_count: usize) -> String {
    let Some(p) = args.get("paginate") else {
        return String::new();
    };
    let offset = p.get("offset").and_then(|v| v.as_i64()).unwrap_or(0);
    let limit = p.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
    if limit <= 0 {
        return String::new();
    }
    let path = p
        .get("path")
        .and_then(|v| v.as_str())
        .unwrap_or("/")
        .to_string();
    let mut s = String::from("<nav class=\"pagination\" aria-label=\"Pagination\">");
    if offset > 0 {
        let prev = offset.saturating_sub(limit);
        s.push_str(&format!(
            "<a class=\"page-prev\" href=\"{}?offset={}\">← Previous</a> ",
            esc(&path),
            prev
        ));
    }
    let end = offset + item_count as i64;
    s.push_str(&format!(
        "<span class=\"page-status\">{}–{} of {}</span> ",
        offset + 1,
        end.min(total),
        total
    ));
    if end < total {
        s.push_str(&format!(
            "<a class=\"page-next\" href=\"{}?offset={}\">Next →</a>",
            esc(&path),
            end
        ));
    }
    s.push_str("</nav>");
    s
}

pub fn render_page(args: &Value, db_url: Option<&str>, csrf: Option<&str>) -> String {
    render_page_ex(args, db_url, None, None, csrf)
}

/// Like `render_page`, optionally replaying form POST data/errors into the embedded form.
pub fn render_page_ex(
    args: &Value,
    db_url: Option<&str>,
    form_data: Option<&Value>,
    form_errors: Option<&Value>,
    csrf: Option<&str>,
) -> String {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Marqdo Web");
    let extra = args
        .get("styles_css")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let shell = resolve_shell_css(args);
    let layout = resolve_layout(args);
    let parts = args
        .get("parts")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let nav = resolve_links(args.get("nav"), db_url);
    let side = resolve_links(args.get("sidebar"), db_url);
    let foot = resolve_links(args.get("footer"), db_url);
    let (intro, items, total) = resolve_main(args, db_url);

    let has_side = args.get("sidebar").is_some() || !side.is_empty();
    let bare = layout == "bare";
    let body_class = match layout.as_str() {
        "bare" => "layout-bare".to_string(),
        "stacked" => "layout-stacked".to_string(),
        "rail" => {
            if has_side {
                "has-rail has-sidebar".into()
            } else {
                "has-rail no-sidebar".into()
            }
        }
        "sidebar" => {
            if has_side {
                "has-sidebar".into()
            } else {
                "no-sidebar".into()
            }
        }
        "" => {
            if has_side {
                "has-sidebar".into()
            } else {
                "no-sidebar".into()
            }
        }
        other => other.to_string(),
    };

    let mut main_html = String::new();
    if let Some(images) = args.get("images_html").and_then(|v| v.as_str()) {
        if !images.is_empty() {
            main_html.push_str(images);
        }
    }
    if !intro.is_empty() {
        main_html.push_str(&format!("<div class=\"main-intro\">{intro}</div>"));
    }
    if let Some(form) = args.get("form") {
        let form_id = args
            .get("form_id")
            .and_then(|v| v.as_str())
            .unwrap_or("form");
        main_html.push_str(&crate::form::render_body(
            form,
            form_id,
            form_data,
            form_errors,
            csrf,
        ));
    }
    if !items.is_empty() {
        let is_detail = args
            .get("detail")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if is_detail {
            main_html.push_str(&render_article(&items[0]));
        } else {
            main_html.push_str("<section class=\"content cards\">");
            for it in &items {
                main_html.push_str(&render_card(args, it));
            }
            main_html.push_str("</section>");
        }
        if let Some(t) = total {
            main_html.push_str(&render_pagination(args, t, items.len()));
        }
    }

    let style_block = if shell.is_empty() && extra.is_empty() {
        String::new()
    } else {
        format!("<style>{shell}{extra}</style>")
    };

    let show_chrome = !bare;
    let side_html = if show_chrome && has_side {
        format!(
            "<aside class=\"{}\"{}><span class=\"side-label\">侧栏</span>{}</aside>",
            slot_class(args, "sidebar", "side"),
            slot_attrs("sidebar", &parts, args),
            render_ul(&side, "side-nav")
        )
    } else {
        String::new()
    };
    let header_html = if show_chrome {
        format!(
            "<header class=\"{}\"{}>{}</header>",
            slot_class(args, "nav", "topnav"),
            slot_attrs("nav", &parts, args),
            render_ul(&nav, "nav")
        )
    } else {
        String::new()
    };
    let footer_html = if show_chrome {
        format!(
            "<footer class=\"{}\"{}>{}</footer>",
            slot_class(args, "footer", "foot"),
            slot_attrs("footer", &parts, args),
            render_ul(&foot, "foot-nav")
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
{head}
{style_block}
</head>
<body class="{body_class}">
{header_html}
{side_html}
<main class="{main_class}"{main_attrs}>{main_html}</main>
{footer_html}
</body></html>"#,
        head = head_html(args, title),
        main_class = slot_class(args, "main", "main"),
        main_attrs = slot_attrs("main", &parts, args),
    )
}

pub fn render_fragment(args: &Value, db_url: Option<&str>) -> String {
    let slot = args
        .get("fragment")
        .or_else(|| args.get("slot"))
        .and_then(|v| v.as_str())
        .map(normalize_slot)
        .unwrap_or_else(|| "main".into());
    match slot.as_str() {
        "nav" => {
            let links = resolve_links(args.get("nav"), db_url);
            format!(
                "<header class=\"topnav\" data-slot=\"nav\">{}</header>",
                render_ul(&links, "nav")
            )
        }
        "sidebar" => {
            let links = resolve_links(args.get("sidebar"), db_url);
            format!(
                "<aside class=\"side\" data-slot=\"sidebar\"><span class=\"side-label\">侧栏</span>{}</aside>",
                render_ul(&links, "side-nav")
            )
        }
        "footer" => {
            let links = resolve_links(args.get("footer"), db_url);
            format!(
                "<footer class=\"foot\" data-slot=\"footer\">{}</footer>",
                render_ul(&links, "foot-nav")
            )
        }
        _ => {
            let (intro, items, _total) = resolve_main(args, db_url);
            let mut body = String::new();
            if let Some(images) = args.get("images_html").and_then(|v| v.as_str()) {
                if !images.is_empty() {
                    body.push_str(images);
                }
            }
            if !intro.is_empty() {
                body.push_str(&format!("<div class=\"main-intro\">{intro}</div>"));
            }
            if let Some(form) = args.get("form") {
                let form_id = args
                    .get("form_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("form");
                body.push_str(&crate::form::render_body(form, form_id, None, None, None));
            }
            if !items.is_empty() {
                let is_detail = args
                    .get("detail")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if is_detail {
                    body.push_str(&render_article(&items[0]));
                } else {
                    body.push_str("<section class=\"content cards\">");
                    for it in &items {
                        body.push_str(&render_card(args, it));
                    }
                    body.push_str("</section>");
                }
            }
            format!("<main class=\"main\" data-slot=\"main\">{body}</main>")
        }
    }
}

/// Render a single list card from a projected DB row.
fn render_card(args: &Value, it: &Map<String, Value>) -> String {
    let title = it.get("title").map(text).unwrap_or_default();
    let body = it.get("body").map(text).unwrap_or_default();
    let href = it.get("href").map(text).unwrap_or_default();
    let meta = it.get("meta").map(text).unwrap_or_default();
    let tag = it.get("tag").map(text).unwrap_or_default();
    let tc = class_attr(&field_css(it, "title"));
    let bc = class_attr(&field_css(it, "body"));
    let mut card = String::from("<article class=\"card\">");
    if !href.is_empty() {
        let prefix = args
            .get("link_prefix")
            .and_then(|v| v.as_str())
            .unwrap_or("/post/");
        card.push_str(&format!(
            "<a class=\"card-link\" href=\"{}\">",
            esc(&format!("{prefix}{href}"))
        ));
    }
    if !meta.is_empty() {
        card.push_str(&format!(
            "<div class=\"card-meta\">{}</div>",
            esc(&meta)
        ));
    }
    card.push_str(&format!("<h2{tc}>{}</h2>", esc(&title)));
    if !tag.is_empty() {
        card.push_str(&format!(
            "<div class=\"card-tag\">{}</div>",
            esc(&tag)
        ));
    }
    if !body.is_empty() {
        card.push_str(&format!("<p{bc}>{}</p>", esc(&body)));
    }
    if !href.is_empty() {
        card.push_str("</a>");
    }
    card.push_str("</article>");
    card
}

/// Render a single article (detail page) from a projected DB row.
fn render_article(it: &Map<String, Value>) -> String {
    let title = it.get("title").map(text).unwrap_or_default();
    let body = it.get("body").map(text).unwrap_or_default();
    let meta = it.get("meta").map(text).unwrap_or_default();
    let tag = it.get("tag").map(text).unwrap_or_default();
    let mut s = String::from("<article class=\"article\">");
    if !meta.is_empty() {
        s.push_str(&format!(
            "<div class=\"article-meta\">{}</div>",
            esc(&meta)
        ));
    }
    s.push_str(&format!("<h1 class=\"article-title\">{}</h1>", esc(&title)));
    if !tag.is_empty() {
        s.push_str(&format!(
            "<div class=\"article-tags\">{}</div>",
            esc(&tag)
        ));
    }
    if !body.is_empty() {
        s.push_str(&markdown::to_html(&body));
    }
    s.push_str("</article>");
    s
}

#[cfg(test)]
mod shell_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn shell_off_omits_sidebar_grid() {
        let page = json!({
            "title": "t",
            "shell_css": "off",
            "layout": "stacked",
            "sidebar": [{"label": "A", "href": "/a"}],
        });
        let html = render_page(&page, None, None);
        assert!(html.contains("layout-stacked"));
        assert!(!html.contains("has-sidebar"));
        assert!(!html.contains("grid-template-columns:14rem"));
    }

    #[test]
    fn shell_minimal_keeps_vars_not_cards() {
        let page = json!({ "title": "t", "shell_css": "minimal" });
        let html = render_page(&page, None, None);
        assert!(html.contains("--ink:"));
        assert!(!html.contains(".content.cards article"));
        assert!(!html.contains("grid-template-columns:14rem"));
    }

    #[test]
    fn layout_bare_skips_aside() {
        let page = json!({
            "title": "t",
            "layout": "bare",
            "sidebar": [{"label": "A", "href": "/a"}],
            "nav": [{"label": "Home", "href": "/"}],
        });
        let html = render_page(&page, None, None);
        assert!(html.contains("layout-bare"));
        assert!(!html.contains("<aside"));
        assert!(!html.contains("<header"));
    }

    #[test]
    fn default_full_keeps_sidebar_grid() {
        let page = json!({
            "title": "t",
            "sidebar": [{"label": "A", "href": "/a"}],
        });
        let html = render_page(&page, None, None);
        assert!(html.contains("has-sidebar"));
        assert!(html.contains("grid-template-columns:14rem"));
    }
}
