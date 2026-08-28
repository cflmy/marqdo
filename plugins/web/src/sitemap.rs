//! sitemap.xml / robots.txt helpers (W7).

use serde_json::{json, Value};

fn cell_str(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Build a sitemap.xml document from rows.
///
/// Each row may use keys: `loc`/`url`/`路径`, optional `lastmod`/`更新`,
/// `changefreq`/`频率`, `priority`/`优先级`.
pub fn build_sitemap(base: &str, items: &Value) -> String {
    let base = base.trim_end_matches('/');
    let rows: Vec<Value> = match items {
        Value::Array(a) => a
            .iter()
            .map(|row| match row {
                Value::String(s) => json!({ "loc": s }),
                Value::Object(_) => row.clone(),
                other => json!({ "loc": cell_str(other) }),
            })
            .collect(),
        Value::Object(m) => {
            // Columnar GFM: { loc: [...], ... }
            let locs = m
                .get("loc")
                .or_else(|| m.get("路径"))
                .or_else(|| m.get("path"))
                .or_else(|| m.get("url"));
            match locs {
                Some(Value::Array(a)) => a
                    .iter()
                    .enumerate()
                    .map(|(i, loc)| {
                        let mut obj = serde_json::Map::new();
                        obj.insert("loc".into(), loc.clone());
                        for (k, v) in m {
                            if k == "loc" || k == "路径" || k == "path" || k == "url" {
                                continue;
                            }
                            if let Some(Value::Array(col)) = Some(v) {
                                if let Some(cell) = col.get(i) {
                                    obj.insert(k.clone(), cell.clone());
                                }
                            }
                        }
                        Value::Object(obj)
                    })
                    .collect(),
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    };
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for row in &rows {
        let Some(obj) = row.as_object() else {
            continue;
        };
        let loc_raw = obj
            .get("loc")
            .or_else(|| obj.get("url"))
            .or_else(|| obj.get("路径"))
            .or_else(|| obj.get("path"))
            .map(cell_str)
            .unwrap_or_default();
        if loc_raw.is_empty() {
            continue;
        }
        let loc = if loc_raw.starts_with("http://") || loc_raw.starts_with("https://") {
            loc_raw
        } else if loc_raw.starts_with('/') {
            format!("{base}{loc_raw}")
        } else {
            format!("{base}/{loc_raw}")
        };
        out.push_str("  <url>\n");
        out.push_str(&format!("    <loc>{}</loc>\n", xml_escape(&loc)));
        if let Some(lm) = obj
            .get("lastmod")
            .or_else(|| obj.get("更新"))
            .map(cell_str)
            .filter(|s| !s.is_empty())
        {
            out.push_str(&format!("    <lastmod>{}</lastmod>\n", xml_escape(&lm)));
        }
        if let Some(cf) = obj
            .get("changefreq")
            .or_else(|| obj.get("频率"))
            .map(cell_str)
            .filter(|s| !s.is_empty())
        {
            out.push_str(&format!(
                "    <changefreq>{}</changefreq>\n",
                xml_escape(&cf)
            ));
        }
        if let Some(pr) = obj
            .get("priority")
            .or_else(|| obj.get("优先级"))
            .map(cell_str)
            .filter(|s| !s.is_empty())
        {
            out.push_str(&format!("    <priority>{}</priority>\n", xml_escape(&pr)));
        }
        out.push_str("  </url>\n");
    }
    out.push_str("</urlset>\n");
    out
}

/// Default robots.txt body allowing all and pointing at a sitemap URL.
pub fn build_robots(sitemap_url: Option<&str>) -> String {
    let mut out = String::from("User-agent: *\nAllow: /\n");
    if let Some(u) = sitemap_url.filter(|s| !s.is_empty()) {
        out.push_str(&format!("Sitemap: {u}\n"));
    }
    out
}

/// ABI helper returning `{ body }`.
#[allow(dead_code)]
pub fn robots_json(sitemap_url: Option<&str>, body: Option<&str>) -> Value {
    let text = match body.filter(|s| !s.is_empty()) {
        Some(b) => b.to_string(),
        None => build_robots(sitemap_url),
    };
    json!({ "body": text })
}
