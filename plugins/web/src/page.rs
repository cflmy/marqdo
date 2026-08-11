//! HTML page shell + content layouts (prose / cards / table / list).
//! Regions resolve from unified bind tables (`front` / `back` / `css`).

use serde_json::{json, Map, Value};

use crate::db;
use crate::table_util::{
    as_bind, bind_has_db, bind_table_name, binds_as_static_links, project_rows, qualify_binds,
};

const DEFAULT_THEME_CSS: &str = include_str!("theme.css");

fn link_label(item: &Value) -> String {
    item.get("label")
        .or_else(|| item.get("标签"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn link_href(item: &Value) -> String {
    item.get("href")
        .or_else(|| item.get("链接"))
        .or_else(|| item.get("路径"))
        .and_then(|v| v.as_str())
        .unwrap_or("#")
        .to_string()
}

fn link_css(item: &Value) -> String {
    item.get("css")
        .or_else(|| item.get("class"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn class_attr(css: &str) -> String {
    let css = css.trim();
    if css.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", html_escape(css))
    }
}

fn render_links(items: &[Value], class: &str) -> String {
    let mut out = format!("<ul class=\"{class}\">");
    for it in items {
        let label = html_escape(&link_label(it));
        let href = html_escape(&link_href(it));
        let css = class_attr(&link_css(it));
        out.push_str(&format!("<li><a href=\"{href}\"{css}>{label}</a></li>"));
    }
    out.push_str("</ul>");
    out
}

pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn items_from(v: Option<&Value>) -> Vec<Value> {
    match v {
        Some(Value::Array(a)) => a.clone(),
        Some(Value::Object(m)) => {
            if let Some(Value::Array(rows)) = m.get("rows") {
                return rows.clone();
            }
            Vec::new()
        }
        _ => Vec::new(),
    }
}

fn looks_like_bind(v: &Value) -> bool {
    match v {
        Value::Array(a) => a.first().and_then(|x| x.as_object()).is_some_and(|m| {
            m.contains_key("front")
                || m.contains_key("前端变量")
                || m.contains_key("页面导航")
                || m.contains_key("label")
        }),
        Value::Object(m) => {
            m.contains_key("前端变量")
                || m.contains_key("front")
                || m.contains_key("页面导航")
                || m.contains_key("对应路由")
        }
        _ => false,
    }
}

fn cell_text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn record_title(obj: &Map<String, Value>) -> String {
    for k in ["title", "标题", "name", "名", "label", "标签"] {
        if let Some(v) = obj.get(k) {
            let s = cell_text(v);
            if !s.is_empty() {
                return s;
            }
        }
    }
    obj.iter()
        .find(|(k, _)| *k != "_css" && *k != "id")
        .map(|(_, v)| cell_text(v))
        .unwrap_or_else(|| "item".into())
}

fn record_body(obj: &Map<String, Value>) -> String {
    for k in ["body", "正文", "content", "内容", "description", "描述", "summary"] {
        if let Some(v) = obj.get(k) {
            let s = cell_text(v);
            if !s.is_empty() {
                return s;
            }
        }
    }
    String::new()
}

fn field_css(obj: &Map<String, Value>, field: &str) -> String {
    obj.get("_css")
        .and_then(|v| v.as_object())
        .and_then(|m| m.get(field))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn render_cards(items: &[Value]) -> String {
    let mut out = String::from("<section class=\"content cards\">");
    for it in items {
        if let Some(obj) = it.as_object() {
            let title = html_escape(&record_title(obj));
            let body = record_body(obj);
            let title_css = class_attr(&field_css(obj, "title"));
            let title_css = if title_css.is_empty() {
                class_attr(&field_css(obj, "标题"))
            } else {
                title_css
            };
            let body_css = class_attr(&field_css(obj, "body"));
            let body_css = if body_css.is_empty() {
                class_attr(&field_css(obj, "正文"))
            } else {
                body_css
            };
            out.push_str("<article>");
            out.push_str(&format!("<h2{title_css}>{title}</h2>"));
            if !body.is_empty() {
                out.push_str(&format!("<p{body_css}>{}</p>", html_escape(&body)));
            }
            // Extra bound fronts (not title/body/id/_css)
            for (k, v) in obj {
                if matches!(
                    k.as_str(),
                    "title"
                        | "标题"
                        | "body"
                        | "正文"
                        | "content"
                        | "内容"
                        | "id"
                        | "_css"
                        | "name"
                        | "名"
                        | "label"
                        | "标签"
                ) {
                    continue;
                }
                let css = class_attr(&field_css(obj, k));
                out.push_str(&format!(
                    "<div data-field=\"{}\"{css}>{}</div>",
                    html_escape(k),
                    html_escape(&cell_text(v))
                ));
            }
            if let Some(id) = obj.get("id") {
                out.push_str(&format!(
                    "<div class=\"meta\">#{}</div>",
                    html_escape(&cell_text(id))
                ));
            }
            out.push_str("</article>");
        } else if let Some(s) = it.as_str() {
            out.push_str(&format!(
                "<article><h2>{}</h2></article>",
                html_escape(s)
            ));
        }
    }
    out.push_str("</section>");
    out
}

fn render_table(items: &[Value]) -> String {
    let Some(first) = items.first().and_then(|v| v.as_object()) else {
        return String::from("<p class=\"main-intro\">No rows.</p>");
    };
    let cols: Vec<String> = first
        .keys()
        .filter(|k| *k != "_css")
        .cloned()
        .collect();
    let mut out = String::from("<div class=\"content table-wrap\"><table><thead><tr>");
    for c in &cols {
        out.push_str(&format!("<th>{}</th>", html_escape(c)));
    }
    out.push_str("</tr></thead><tbody>");
    for it in items {
        let Some(obj) = it.as_object() else {
            continue;
        };
        out.push_str("<tr>");
        for c in &cols {
            let cell = obj.get(c).map(cell_text).unwrap_or_default();
            let css = class_attr(&field_css(obj, c));
            out.push_str(&format!("<td{css}>{}</td>", html_escape(&cell)));
        }
        out.push_str("</tr>");
    }
    out.push_str("</tbody></table></div>");
    out
}

fn render_list(items: &[Value]) -> String {
    let mut out = String::from("<ul class=\"content list\">");
    for it in items {
        if let Some(obj) = it.as_object() {
            let title = html_escape(&record_title(obj));
            let body = record_body(obj);
            let title_css = class_attr(&field_css(obj, "title"));
            if body.is_empty() {
                out.push_str(&format!("<li{title_css}>{title}</li>"));
            } else {
                out.push_str(&format!(
                    "<li><strong{title_css}>{title}</strong> — {}</li>",
                    html_escape(&body)
                ));
            }
        } else if let Some(s) = it.as_str() {
            out.push_str(&format!("<li>{}</li>", html_escape(s)));
        }
    }
    out.push_str("</ul>");
    out
}

fn render_main_block(intro: &str, items: Option<&Vec<Value>>, layout: &str) -> String {
    let mut out = String::new();
    if !intro.is_empty() {
        out.push_str(&format!("<div class=\"main-intro\">{intro}</div>"));
    }
    let Some(items) = items.filter(|v| !v.is_empty()) else {
        return out;
    };
    match layout {
        "table" | "表格" => out.push_str(&render_table(items)),
        "list" | "列表" => out.push_str(&render_list(items)),
        "prose" | "正文" => {
            out.push_str("<section class=\"content prose\">");
            for it in items {
                if let Some(obj) = it.as_object() {
                    let title = html_escape(&record_title(obj));
                    let body = record_body(obj);
                    let title_css = class_attr(&field_css(obj, "title"));
                    let body_css = class_attr(&field_css(obj, "body"));
                    out.push_str(&format!("<h2{title_css}>{title}</h2>"));
                    if !body.is_empty() {
                        out.push_str(&format!("<p{body_css}>{}</p>", html_escape(&body)));
                    }
                }
            }
            out.push_str("</section>");
        }
        _ => out.push_str(&render_cards(items)),
    }
    out
}

/// Resolve a region value into link items (static or DB-looped).
fn resolve_link_region(
    raw: Option<&Value>,
    db_url: Option<&str>,
    default_table: Option<&str>,
) -> Vec<Value> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    // Already resolved link list?
    if let Value::Array(a) = raw {
        if a.first().and_then(|x| x.as_object()).is_some_and(|m| {
            m.contains_key("label") || m.contains_key("href")
        }) && !a
            .first()
            .and_then(|x| x.as_object())
            .is_some_and(|m| m.contains_key("front"))
        {
            return a.clone();
        }
    }
    if !looks_like_bind(raw) {
        return items_from(Some(raw));
    }
    let binds = as_bind(raw);
    let arr = qualify_binds(binds.as_array().map(|v| v.as_slice()).unwrap_or(&[]), default_table);
    if bind_has_db(&arr) {
        let Some(url) = db_url else {
            return Vec::new();
        };
        let Some(table) = bind_table_name(&arr) else {
            return Vec::new();
        };
        let Ok(data) = db::query_all(url, &table, 200) else {
            return Vec::new();
        };
        let rows = data
            .get("rows")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let projected = project_rows(&arr, &rows);
        let mut links = Vec::new();
        for it in projected.as_array().cloned().unwrap_or_default() {
            let obj = it.as_object().cloned().unwrap_or_default();
            let label = obj
                .get("label")
                .or_else(|| obj.get("标签"))
                .or_else(|| obj.get("title"))
                .or_else(|| obj.get("前端变量"))
                .map(cell_text)
                .filter(|s| !s.is_empty())
                .or_else(|| {
                    obj.iter()
                        .find(|(k, _)| *k != "_css" && *k != "id" && *k != "href" && *k != "路径")
                        .map(|(_, v)| cell_text(v))
                })
                .unwrap_or_default();
            let href = obj
                .get("href")
                .or_else(|| obj.get("路径"))
                .or_else(|| obj.get("对应路由"))
                .map(cell_text)
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "#".into());
            let css = obj
                .get("_css")
                .and_then(|v| v.as_object())
                .and_then(|m| {
                    m.get("label")
                        .or_else(|| m.get("title"))
                        .or_else(|| m.values().next())
                })
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            links.push(json!({ "label": label, "href": href, "css": css }));
        }
        links
    } else {
        binds_as_static_links(&arr)
            .as_array()
            .cloned()
            .unwrap_or_default()
    }
}

/// Resolve main: bind table → DB loop, or legacy live list / HTML intro.
fn resolve_main(
    args: &Value,
    db_url: Option<&str>,
    default_table: Option<&str>,
) -> (String, Option<Vec<Value>>) {
    let intro = args
        .get("intro")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if let Some(items) = args.get("main_items").and_then(|v| v.as_array()) {
        return (intro, Some(items.clone()));
    }

    let main = args.get("main");
    if let Some(raw) = main {
        if looks_like_bind(raw) {
            let binds = as_bind(raw);
            let arr = qualify_binds(binds.as_array().map(|v| v.as_slice()).unwrap_or(&[]), default_table);
            if bind_has_db(&arr) {
                if let (Some(url), Some(table)) = (db_url, bind_table_name(&arr)) {
                    if let Ok(data) = db::query_all(url, &table, 200) {
                        let rows = data
                            .get("rows")
                            .and_then(|v| v.as_array())
                            .cloned()
                            .unwrap_or_default();
                        let projected = project_rows(&arr, &rows);
                        return (
                            intro,
                            Some(projected.as_array().cloned().unwrap_or_default()),
                        );
                    }
                }
                return (intro, Some(Vec::new()));
            }
        }
        if let Value::Array(a) = raw {
            return (intro, Some(a.clone()));
        }
        if let Some(s) = raw.as_str() {
            if intro.is_empty() {
                return (s.to_string(), None);
            }
        }
    }

    if !intro.is_empty() {
        return (intro, None);
    }
    if let Some(s) = main.and_then(|v| v.as_str()) {
        return (s.to_string(), None);
    }
    ("<p>Welcome</p>".into(), None)
}

/// Render a four-region page shell; optional `db_url` resolves live binds.
pub fn render_page(args: &Value) -> String {
    let db_url = args
        .get("db_url")
        .or_else(|| args.get("url"))
        .and_then(|v| v.as_str());
    render_page_with_db(args, db_url)
}

pub fn render_page_with_db(args: &Value, db_url: Option<&str>) -> String {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Marqdo Web");
    let theme_css = args.get("css").and_then(|v| v.as_str()).unwrap_or("");
    let layout = args
        .get("layout")
        .or_else(|| args.get("主体样式"))
        .and_then(|v| v.as_str())
        .unwrap_or("cards");
    let default_table = args
        .get("table")
        .or_else(|| args.get("数据表"))
        .and_then(|v| v.as_str());

    let nav = resolve_link_region(args.get("nav"), db_url, default_table);
    let sidebar = resolve_link_region(args.get("sidebar"), db_url, default_table);
    let footer = resolve_link_region(args.get("footer"), db_url, default_table);
    let (intro, main_items) = resolve_main(args, db_url, default_table);

    let body_main = if main_items.is_some() {
        render_main_block(&intro, main_items.as_ref(), layout)
    } else if !intro.is_empty() {
        format!("<div class=\"main-intro\">{intro}</div>")
    } else {
        String::from("<div class=\"main-intro\"><p>Welcome</p></div>")
    };

    let has_side = !sidebar.is_empty();
    let layout_class = if has_side {
        "has-sidebar"
    } else {
        "no-sidebar"
    };
    let extra_css = if theme_css.is_empty() {
        String::new()
    } else {
        format!(
            "<link rel=\"stylesheet\" href=\"{}\"/>",
            html_escape(theme_css)
        )
    };
    let side = if has_side {
        format!(
            "<aside class=\"side\"><span class=\"side-label\">侧栏</span>{}</aside>",
            render_links(&sidebar, "side-nav")
        )
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{title}</title>
<style>
{builtin}
</style>
{extra_css}
</head>
<body class="{layout_class}">
<header class="topnav">{nav}</header>
{side}
<main class="main">{main}</main>
<footer class="foot">{foot}</footer>
</body>
</html>"#,
        title = html_escape(title),
        builtin = DEFAULT_THEME_CSS,
        extra_css = extra_css,
        layout_class = layout_class,
        nav = render_links(&nav, "nav"),
        side = side,
        main = body_main,
        foot = render_links(&footer, "foot-nav"),
    )
}

pub fn default_hello_html() -> String {
    render_page(&json!({
        "title": "Marqdo Web",
        "nav": [{"label": "Home", "href": "/"}],
        "intro": "<h1>Marqdo Web</h1><p>Hello from ext/web.</p>",
        "footer": [{"label": "Marqdo", "href": "https://github.com/cflmy/marqdo"}],
    }))
}
