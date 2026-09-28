//! Web / Endpoint Artifact classification from bound Metadata.
//!
//! Spec: `doc/design/ext-web-artifact.md` · ADR 0007.
//! Does not add CORE_CONSTRUCTS — reuses flat frontmatter `type` / `类型`.

use crate::value::Value;

/// Kind of executable document recognized for the Web model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactKind {
    /// `type: web` / `类型: 网页`
    Web,
    /// `type: endpoint` / `类型: 端点`
    Endpoint,
    /// `type: prompt` / `类型: 提示` (AI stack; classified for inspect)
    Prompt,
    /// `type: llm` / other known types
    Other,
}

/// Read `type` or `类型` from bound metadata.
pub fn metadata_type(metadata: &[(String, Value)]) -> Option<String> {
    for key in ["type", "类型"] {
        if let Some(v) = meta_get(metadata, key) {
            let s = v.as_display();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

/// Classify artifact kind from metadata (None if no type key).
pub fn classify(metadata: &[(String, Value)]) -> Option<ArtifactKind> {
    let t = metadata_type(metadata)?;
    Some(match t.as_str() {
        "web" | "网页" => ArtifactKind::Web,
        "endpoint" | "端点" => ArtifactKind::Endpoint,
        "prompt" | "提示" => ArtifactKind::Prompt,
        _ => ArtifactKind::Other,
    })
}

/// True when this module is a Web Document Artifact.
pub fn is_web_document(metadata: &[(String, Value)]) -> bool {
    matches!(classify(metadata), Some(ArtifactKind::Web))
}

/// True when this module is an Endpoint Artifact.
pub fn is_endpoint(metadata: &[(String, Value)]) -> bool {
    matches!(classify(metadata), Some(ArtifactKind::Endpoint))
}

/// Lookup a metadata string (EN key, then ZH aliases).
pub fn meta_text(metadata: &[(String, Value)], en: &str, zh: &str) -> Option<String> {
    for key in [en, zh] {
        if let Some(v) = meta_get(metadata, key) {
            let s = v.as_display();
            if !s.is_empty() {
                return Some(s);
            }
        }
    }
    None
}

/// Route for a web document: `route` / `路由`, else None.
pub fn web_route(metadata: &[(String, Value)]) -> Option<String> {
    meta_text(metadata, "route", "路由")
}

/// HTTP method defaulting to GET.
pub fn http_method(metadata: &[(String, Value)]) -> String {
    meta_text(metadata, "method", "方法").unwrap_or_else(|| "GET".to_string())
}

/// Endpoint path: `path` / `路径`.
pub fn endpoint_path(metadata: &[(String, Value)]) -> Option<String> {
    meta_text(metadata, "path", "路径")
}

fn meta_get<'a>(metadata: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    metadata.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn classify_web_en_zh() {
        let en = vec![("type".into(), Value::Text("web".into()))];
        assert_eq!(classify(&en), Some(ArtifactKind::Web));
        let zh = vec![("类型".into(), Value::Text("网页".into()))];
        assert_eq!(classify(&zh), Some(ArtifactKind::Web));
    }

    #[test]
    fn classify_endpoint() {
        let m = vec![
            ("type".into(), Value::Text("endpoint".into())),
            ("path".into(), Value::Text("/api/x".into())),
            ("method".into(), Value::Text("POST".into())),
        ];
        assert!(is_endpoint(&m));
        assert_eq!(endpoint_path(&m).as_deref(), Some("/api/x"));
        assert_eq!(http_method(&m), "POST");
    }

    #[test]
    fn no_type() {
        let m = vec![("title".into(), Value::Text("x".into()))];
        assert_eq!(classify(&m), None);
    }
}
