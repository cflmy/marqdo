//! Site icons, `<head>` link/script assembly, and image HTML assembly.
//! Tables stay data; assembly is a function — 配置即数据、装配即函数.

use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

use crate::table::normalize_ref;

#[derive(Clone, Debug)]
pub struct HeadLink {
    pub rel: String,
    pub href: String,
    pub type_: String,
    pub sizes: String,
    pub media: String,
    pub as_: String,
    pub crossorigin: String,
}

#[derive(Clone, Debug)]
pub struct IconRoute {
    pub url: String,
    pub path: PathBuf,
    pub content_type: String,
}

fn cell_str(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn pick<'a>(m: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|k| m.get(*k))
}

fn row_str(m: &Map<String, Value>, keys: &[&str]) -> String {
    pick(m, keys)
        .map(|v| normalize_ref(&cell_str(v)))
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Infer Content-Type from file extension.
pub fn mime_for_path(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".ico") {
        "image/x-icon"
    } else if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webmanifest") || lower.ends_with(".json") {
        "application/manifest+json"
    } else if lower.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if lower.ends_with(".js") || lower.ends_with(".mjs") {
        "text/javascript; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

fn rows_of(table: &Value) -> Vec<Map<String, Value>> {
    match table {
        Value::Array(a) => a
            .iter()
            .filter_map(|v| v.as_object().cloned())
            .collect(),
        Value::Object(m) => {
            // Columnar GFM: { 关系: […], 地址: […] }
            let keys: Vec<String> = m.keys().cloned().collect();
            if keys.is_empty() {
                return Vec::new();
            }
            let len = m
                .values()
                .filter_map(|v| v.as_array().map(|a| a.len()))
                .max()
                .unwrap_or(0);
            if len == 0 {
                return vec![m.clone()];
            }
            let mut out = Vec::with_capacity(len);
            for i in 0..len {
                let mut row = Map::new();
                for k in &keys {
                    if let Some(Value::Array(a)) = m.get(k) {
                        row.insert(k.clone(), a.get(i).cloned().unwrap_or(Value::Null));
                    } else if i == 0 {
                        row.insert(k.clone(), m.get(k).cloned().unwrap_or(Value::Null));
                    }
                }
                out.push(row);
            }
            out
        }
        _ => Vec::new(),
    }
}

/// Normalize a Head resource table → list of HeadLink.
pub fn as_head_links(table: &Value) -> Vec<HeadLink> {
    let mut out = Vec::new();
    for m in rows_of(table) {
        let rel = row_str(&m, &["关系", "rel", "Rel"]);
        let href = row_str(
            &m,
            &["地址", "href", "src", "url", "Href", "Src", "URL"],
        );
        if href.is_empty() {
            continue;
        }
        let rel = if rel.is_empty() {
            "stylesheet".into()
        } else {
            rel
        };
        out.push(HeadLink {
            rel,
            href,
            type_: row_str(&m, &["类型", "type", "Type"]),
            sizes: row_str(&m, &["尺寸", "sizes", "Sizes"]),
            media: row_str(&m, &["媒体", "media", "Media"]),
            as_: row_str(&m, &["作为", "as", "As"]),
            crossorigin: row_str(&m, &["跨域", "crossorigin", "crossOrigin"]),
        });
    }
    out
}

pub fn head_links_to_json(links: &[HeadLink]) -> Value {
    Value::Array(
        links
            .iter()
            .map(|l| {
                json!({
                    "rel": l.rel,
                    "href": l.href,
                    "type": l.type_,
                    "sizes": l.sizes,
                    "media": l.media,
                    "as": l.as_,
                    "crossorigin": l.crossorigin,
                })
            })
            .collect(),
    )
}

pub fn head_links_from_json(v: &Value) -> Vec<HeadLink> {
    let Some(arr) = v.as_array() else {
        return as_head_links(v);
    };
    let mut out = Vec::new();
    for item in arr {
        let Some(m) = item.as_object() else {
            continue;
        };
        let href = row_str(m, &["href", "src", "url", "地址"]);
        if href.is_empty() {
            continue;
        }
        out.push(HeadLink {
            rel: row_str(m, &["rel", "关系"]),
            href,
            type_: row_str(m, &["type", "类型"]),
            sizes: row_str(m, &["sizes", "尺寸"]),
            media: row_str(m, &["media", "媒体"]),
            as_: row_str(m, &["as", "作为"]),
            crossorigin: row_str(m, &["crossorigin", "跨域"]),
        });
    }
    out
}

/// Render HeadLink list to HTML (link / script tags).
pub fn render_head_links(links: &[HeadLink]) -> String {
    let mut s = String::new();
    for l in links {
        let rel = l.rel.trim().to_ascii_lowercase();
        if rel == "script" || rel == "module" {
            let mut tag = String::from("<script");
            if rel == "module" || l.type_ == "module" {
                tag.push_str(" type=\"module\"");
            } else if !l.type_.is_empty() {
                tag.push_str(&format!(" type=\"{}\"", esc(&l.type_)));
            }
            if !l.crossorigin.is_empty() {
                tag.push_str(&format!(" crossorigin=\"{}\"", esc(&l.crossorigin)));
            }
            tag.push_str(&format!(" src=\"{}\"></script>", esc(&l.href)));
            s.push_str(&tag);
            continue;
        }
        let mut tag = format!("<link rel=\"{}\" href=\"{}\"", esc(&l.rel), esc(&l.href));
        if !l.type_.is_empty() {
            tag.push_str(&format!(" type=\"{}\"", esc(&l.type_)));
        }
        if !l.sizes.is_empty() {
            tag.push_str(&format!(" sizes=\"{}\"", esc(&l.sizes)));
        }
        if !l.media.is_empty() {
            tag.push_str(&format!(" media=\"{}\"", esc(&l.media)));
        }
        if !l.as_.is_empty() {
            tag.push_str(&format!(" as=\"{}\"", esc(&l.as_)));
        }
        if !l.crossorigin.is_empty() {
            tag.push_str(&format!(" crossorigin=\"{}\"", esc(&l.crossorigin)));
        }
        tag.push_str("/>");
        s.push_str(&tag);
    }
    s
}

/// Module-level preview: assemble head table → HTML string.
pub fn make_head_html(table: &Value) -> String {
    render_head_links(&as_head_links(table))
}

/// Normalize icons table → (icons json array, site_head links, file routes).
pub fn normalize_icons(table: &Value) -> (Value, Vec<HeadLink>, Vec<IconRoute>) {
    let mut icons = Vec::new();
    let mut head = Vec::new();
    let mut routes = Vec::new();
    let mut saw_favicon_url = false;

    for m in rows_of(table) {
        let path = row_str(&m, &["路径", "path", "file", "Path"]);
        let mut rel = row_str(&m, &["关系", "rel", "Rel"]);
        if rel.is_empty() {
            rel = "icon".into();
        }
        let mut type_ = row_str(&m, &["类型", "type", "Type"]);
        let sizes = row_str(&m, &["尺寸", "sizes", "Sizes"]);
        let mut url = row_str(&m, &["地址", "url", "href", "Href", "URL"]);

        if type_.is_empty() && !path.is_empty() {
            type_ = mime_for_path(&path).to_string();
        }
        if url.is_empty() {
            if !path.is_empty() {
                let name = Path::new(&path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("icon");
                let lower = path.to_ascii_lowercase();
                if rel.to_ascii_lowercase().contains("icon") && lower.ends_with(".ico") {
                    url = "/favicon.ico".into();
                } else {
                    url = format!("/icons/{name}");
                }
            } else {
                continue;
            }
        }
        if url == "/favicon.ico" {
            saw_favicon_url = true;
        }

        icons.push(json!({
            "path": path,
            "rel": rel,
            "type": type_,
            "sizes": sizes,
            "url": url,
        }));

        if !url.is_empty() {
            head.push(HeadLink {
                rel: rel.clone(),
                href: url.clone(),
                type_: type_.clone(),
                sizes: sizes.clone(),
                media: String::new(),
                as_: String::new(),
                crossorigin: String::new(),
            });
        }

        if !path.is_empty() && !url.is_empty() {
            routes.push(IconRoute {
                url: url.clone(),
                path: PathBuf::from(&path),
                content_type: if type_.is_empty() {
                    mime_for_path(&path).to_string()
                } else {
                    type_.clone()
                },
            });
        }
    }

    // Ensure browser default path if we have an icon file but no /favicon.ico.
    if !saw_favicon_url {
        if let Some(first) = icons.iter().find(|v| {
            v.get("rel")
                .and_then(|r| r.as_str())
                .map(|r| r.to_ascii_lowercase().contains("icon"))
                .unwrap_or(false)
                && v.get("path")
                    .and_then(|p| p.as_str())
                    .map(|p| !p.is_empty())
                    .unwrap_or(false)
        }) {
            let path = first.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let type_ = first
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| mime_for_path(path));
            routes.push(IconRoute {
                url: "/favicon.ico".into(),
                path: PathBuf::from(path),
                content_type: type_.to_string(),
            });
            head.insert(
                0,
                HeadLink {
                    rel: "icon".into(),
                    href: "/favicon.ico".into(),
                    type_: type_.to_string(),
                    sizes: first
                        .get("sizes")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    media: String::new(),
                    as_: String::new(),
                    crossorigin: String::new(),
                },
            );
        }
    }

    (Value::Array(icons), head, routes)
}

/// Convention: if static_dir has favicon.ico|png|svg and icons empty, register defaults.
pub fn convention_favicon(static_dir: &Path) -> Option<(Vec<HeadLink>, Vec<IconRoute>)> {
    for name in ["favicon.ico", "favicon.png", "favicon.svg"] {
        let p = static_dir.join(name);
        if p.is_file() {
            let ct = mime_for_path(name).to_string();
            let head = vec![HeadLink {
                rel: "icon".into(),
                href: "/favicon.ico".into(),
                type_: ct.clone(),
                sizes: String::new(),
                media: String::new(),
                as_: String::new(),
                crossorigin: String::new(),
            }];
            let routes = vec![IconRoute {
                url: "/favicon.ico".into(),
                path: p,
                content_type: ct,
            }];
            return Some((head, routes));
        }
    }
    None
}

/// Merge site_head into a page value (dedupe by href+rel).
pub fn merge_site_head(page: &mut Value, site_head: &[HeadLink]) {
    if site_head.is_empty() {
        return;
    }
    let obj = match page.as_object_mut() {
        Some(o) => o,
        None => return,
    };
    let mut existing = obj
        .get("head")
        .map(head_links_from_json)
        .unwrap_or_default();
    for link in site_head {
        let dup = existing.iter().any(|e| {
            e.href == link.href && e.rel.eq_ignore_ascii_case(&link.rel)
        });
        if !dup {
            existing.push(link.clone());
        }
    }
    obj.insert("head".into(), head_links_to_json(&existing));
}

/// GFM image table → HTML fragment.
pub fn make_images_html(table: &Value) -> String {
    let mut figures = String::new();
    for m in rows_of(table) {
        let src = row_str(&m, &["源", "src", "Src", "url", "URL"]);
        if src.is_empty() {
            continue;
        }
        let alt = row_str(&m, &["替代", "alt", "Alt"]);
        let title = row_str(&m, &["标题", "title", "Title"]);
        let class = row_str(&m, &["类", "class", "Class"]);
        let href = row_str(&m, &["链接", "href", "link", "Href", "Link"]);
        let width = row_str(&m, &["宽度", "width", "Width"]);
        let height = row_str(&m, &["高度", "height", "Height"]);
        let mut loading = row_str(&m, &["加载", "loading", "Loading"]);
        if loading.is_empty() {
            loading = "lazy".into();
        }
        let caption = row_str(&m, &["图注", "caption", "Caption"]);

        let mut img = format!("<img src=\"{}\" alt=\"{}\"", esc(&src), esc(&alt));
        if !title.is_empty() {
            img.push_str(&format!(" title=\"{}\"", esc(&title)));
        }
        if !width.is_empty() {
            img.push_str(&format!(" width=\"{}\"", esc(&width)));
        }
        if !height.is_empty() {
            img.push_str(&format!(" height=\"{}\"", esc(&height)));
        }
        img.push_str(&format!(" loading=\"{}\"/>", esc(&loading)));

        let inner = if href.is_empty() {
            img
        } else {
            format!("<a href=\"{}\">{}</a>", esc(&href), img)
        };

        let fig_class = if class.is_empty() {
            "mq-img".into()
        } else {
            format!("mq-img {}", class)
        };
        figures.push_str(&format!("<figure class=\"{}\">{}", fig_class, inner));
        if !caption.is_empty() {
            figures.push_str(&format!("<figcaption>{}</figcaption>", esc(&caption)));
        }
        figures.push_str("</figure>");
    }
    if figures.is_empty() {
        return String::new();
    }
    format!("<div class=\"mq-images\" data-slot=\"main\">{figures}</div>")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn images_basic() {
        let t = json!([
            {"源": "/a.png", "替代": "A", "类": "logo", "加载": "eager"}
        ]);
        let h = make_images_html(&t);
        assert!(h.contains("mq-images"));
        assert!(h.contains("src=\"/a.png\""));
        assert!(h.contains("loading=\"eager\""));
        assert!(h.contains("class=\"mq-img logo\""));
    }

    #[test]
    fn head_script_and_icon() {
        let t = json!([
            {"关系": "icon", "地址": "/favicon.ico", "类型": "image/x-icon"},
            {"关系": "script", "地址": "/static/a.js"}
        ]);
        let h = make_head_html(&t);
        assert!(h.contains("rel=\"icon\""));
        assert!(h.contains("<script src=\"/static/a.js\"></script>"));
    }
}
