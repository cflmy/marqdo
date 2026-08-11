//! Minimal HTML shell for assembled pages / parts.

use serde_json::{Map, Value};

use crate::db;
use crate::table::{as_bind, normalize_slot, project_rows};

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

fn resolve_links(raw: Option<&Value>, db_url: Option<&str>) -> Vec<(String, String)> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let binds = as_bind(raw);
    let arr = binds.as_array().cloned().unwrap_or_default();
    if arr.is_empty() {
        return Vec::new();
    }
    let has_db = arr.iter().any(|b| {
        b.get("back")
            .and_then(|v| v.as_str())
            .is_some_and(|s| s.contains('.'))
    });
    if has_db {
        let Some(url) = db_url else {
            return Vec::new();
        };
        let table = crate::table::bind_table_name(&arr);
        let Some(table) = table else {
            return Vec::new();
        };
        let Ok(data) = db::select(url, &table, 200) else {
            return Vec::new();
        };
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
    // static: front=label back=href
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

fn resolve_main(args: &Value, db_url: Option<&str>) -> (String, Vec<Map<String, Value>>) {
    let intro = args
        .get("intro")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let Some(main) = args.get("main") else {
        return (intro, Vec::new());
    };
    let binds = as_bind(main);
    let arr = binds.as_array().cloned().unwrap_or_default();
    if arr.is_empty() {
        return (intro, Vec::new());
    }
    let Some(url) = db_url else {
        return (intro, Vec::new());
    };
    let Some(table) = crate::table::bind_table_name(&arr) else {
        return (intro, Vec::new());
    };
    let Ok(data) = db::select(url, &table, 200) else {
        return (intro, Vec::new());
    };
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
    (intro, items)
}

fn field_css(obj: &Map<String, Value>, field: &str) -> String {
    obj.get("_css")
        .and_then(|v| v.as_object())
        .and_then(|m| m.get(field))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn slot_attrs(slot: &str, parts: &Map<String, Value>) -> String {
    let mut a = format!(" data-slot=\"{}\"", esc(slot));
    for (id, cfg) in parts {
        let s = cfg
            .get("slot")
            .or_else(|| cfg.get("fragment"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if s == slot || (slot == "sidebar" && id == "side") || (slot == "main" && id == "index") {
            a.push_str(&format!(" data-slot-src=\"/_part/{}\"", esc(id)));
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

const SHELL_CSS: &str = r#"
:root { --ink:#1c1917; --muted:#57534e; --paper:#fafaf9; --line:#e7e5e4; --accent:#0f766e; }
body { margin:0; font-family: "IBM Plex Sans", "Noto Sans SC", sans-serif; background:var(--paper); color:var(--ink); display:grid; min-height:100vh; grid-template-rows:auto 1fr auto; }
body.has-sidebar { grid-template-columns:14rem 1fr; grid-template-areas:"top top" "side main" "foot foot"; }
body.no-sidebar { grid-template-areas:"top" "main" "foot"; }
header.topnav { grid-area:top; border-bottom:1px solid var(--line); padding:.85rem 1.25rem; background:#fff; }
aside.side { grid-area:side; border-right:1px solid var(--line); padding:1.25rem 1rem; background:#f5f5f4; }
main.main { grid-area:main; padding:1.5rem 1.25rem 2rem; }
footer.foot { grid-area:foot; border-top:1px solid var(--line); padding:.85rem 1.25rem; color:var(--muted); background:#fff; }
ul.nav, ul.side-nav, ul.foot-nav { list-style:none; margin:0; padding:0; display:flex; flex-wrap:wrap; gap:.35rem 1rem; }
ul.side-nav { flex-direction:column; }
a { color:var(--ink); text-decoration:none; }
a:hover { color:var(--accent); }
.content.cards { display:grid; gap:1rem; margin-top:1.25rem; grid-template-columns:repeat(auto-fill,minmax(16rem,1fr)); }
.content.cards article { background:#fff; border:1px solid var(--line); border-radius:6px; padding:1rem; }
.main-intro h1 { margin:0 0 .75rem; font-size:2rem; }
.main-intro p { color:var(--muted); }
"#;

pub fn render_page(args: &Value, db_url: Option<&str>) -> String {
    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Marqdo Web");
    let extra = args
        .get("styles_css")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let parts = args
        .get("parts")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let nav = resolve_links(args.get("nav"), db_url);
    let side = resolve_links(args.get("sidebar"), db_url);
    let foot = resolve_links(args.get("footer"), db_url);
    let (intro, items) = resolve_main(args, db_url);

    let has_side = args.get("sidebar").is_some() || !side.is_empty();
    let body_class = if has_side {
        "has-sidebar"
    } else {
        "no-sidebar"
    };

    let mut main_html = String::new();
    if !intro.is_empty() {
        main_html.push_str(&format!("<div class=\"main-intro\">{intro}</div>"));
    }
    if !items.is_empty() {
        main_html.push_str("<section class=\"content cards\">");
        for it in &items {
            let title = it
                .get("title")
                .map(text)
                .unwrap_or_else(|| "item".into());
            let body = it.get("body").map(text).unwrap_or_default();
            let tc = class_attr(&field_css(it, "title"));
            let bc = class_attr(&field_css(it, "body"));
            main_html.push_str("<article>");
            main_html.push_str(&format!("<h2{tc}>{}</h2>", esc(&title)));
            if !body.is_empty() {
                main_html.push_str(&format!("<p{bc}>{}</p>", esc(&body)));
            }
            main_html.push_str("</article>");
        }
        main_html.push_str("</section>");
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{title}</title>
<style>{SHELL_CSS}{extra}</style>
</head>
<body class="{body_class}">
<header class="{nav_class}"{nav_attrs}>{nav_ul}</header>
{side_html}
<main class="{main_class}"{main_attrs}>{main_html}</main>
<footer class="{foot_class}"{foot_attrs}>{foot_ul}</footer>
</body></html>"#,
        title = esc(title),
        nav_class = slot_class(args, "nav", "topnav"),
        nav_attrs = slot_attrs("nav", &parts),
        nav_ul = render_ul(&nav, "nav"),
        side_html = if has_side {
            format!(
                "<aside class=\"{}\"{}><span class=\"side-label\">侧栏</span>{}</aside>",
                slot_class(args, "sidebar", "side"),
                slot_attrs("sidebar", &parts),
                render_ul(&side, "side-nav")
            )
        } else {
            String::new()
        },
        main_class = slot_class(args, "main", "main"),
        main_attrs = slot_attrs("main", &parts),
        foot_class = slot_class(args, "footer", "foot"),
        foot_attrs = slot_attrs("footer", &parts),
        foot_ul = render_ul(&foot, "foot-nav"),
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
            let (intro, items) = resolve_main(args, db_url);
            let mut body = String::new();
            if !intro.is_empty() {
                body.push_str(&format!("<div class=\"main-intro\">{intro}</div>"));
            }
            if !items.is_empty() {
                body.push_str("<section class=\"content cards\">");
                for it in &items {
                    let title = it.get("title").map(text).unwrap_or_default();
                    let b = it.get("body").map(text).unwrap_or_default();
                    let tc = class_attr(&field_css(it, "title"));
                    let bc = class_attr(&field_css(it, "body"));
                    body.push_str(&format!(
                        "<article><h2{tc}>{}</h2><p{bc}>{}</p></article>",
                        esc(&title),
                        esc(&b)
                    ));
                }
                body.push_str("</section>");
            }
            format!("<main class=\"main\" data-slot=\"main\">{body}</main>")
        }
    }
}
