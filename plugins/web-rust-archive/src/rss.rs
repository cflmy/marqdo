//! RSS 2.0 feed assembly (content-site W4).

use serde_json::Value;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
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

fn pick<'a>(m: &'a serde_json::Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|k| m.get(*k))
}

/// Build RSS 2.0 XML from channel metadata and row maps.
pub fn build_rss(
    title: &str,
    link: &str,
    description: &str,
    items: &[Value],
) -> String {
    let mut body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0"><channel>
<title>{t}</title>
<link>{l}</link>
<description>{d}</description>
"#,
        t = esc(title),
        l = esc(link),
        d = esc(description),
    );
    for row in items {
        let Some(m) = row.as_object() else {
            continue;
        };
        let item_title = pick(m, &["title", "标题"])
            .map(cell_str)
            .unwrap_or_default();
        let item_link = pick(m, &["link", "href", "链接", "slug"])
            .map(cell_str)
            .unwrap_or_default();
        let item_desc = pick(m, &["description", "summary", "摘要", "body", "正文"])
            .map(cell_str)
            .unwrap_or_default();
        let pub_date = pick(m, &["pubDate", "published", "created_at", "发布"])
            .map(cell_str)
            .unwrap_or_default();
        let full_link = if item_link.starts_with("http") {
            item_link.clone()
        } else if item_link.starts_with('/') {
            format!("{link}{item_link}")
        } else if !item_link.is_empty() {
            format!("{link}/{item_link}")
        } else {
            link.to_string()
        };
        body.push_str("<item>");
        body.push_str(&format!("<title>{}</title>", esc(&item_title)));
        body.push_str(&format!("<link>{}</link>", esc(&full_link)));
        body.push_str(&format!(
            "<description>{}</description>",
            esc(&item_desc)
        ));
        if !pub_date.is_empty() {
            body.push_str(&format!("<pubDate>{}</pubDate>", esc(&pub_date)));
        }
        body.push_str("</item>");
    }
    body.push_str("</channel></rss>");
    body
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rss_contains_item() {
        let xml = build_rss(
            "Blog",
            "http://example.com",
            "Feed",
            &[json!({"title": "Hi", "slug": "/post/a", "summary": "text"})],
        );
        assert!(xml.contains("<rss"));
        assert!(xml.contains("<title>Hi</title>"));
    }
}
