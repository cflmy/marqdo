//! Document extraction protocol (M01): DocumentSource → ExtractedDocument.
//!
//! Format parsers live here / in extensions — language core does not OCR.
//! First slice: plain text and Markdown. PDF/DOCX plug in later via the same shape.

use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};
use serde_json::{json, Value};

pub const EXTRACTOR_VERSION: &str = "marqdo-document-extract/1";

#[derive(Debug, Clone)]
pub struct DocumentSource {
    pub path: std::path::PathBuf,
    pub format_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExtractedSegment {
    pub id: String,
    pub kind: String,
    pub text: String,
    pub locator: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ExtractedDocument {
    pub source_uri: String,
    pub source_digest: String,
    pub format: String,
    pub extractor_version: String,
    pub text: String,
    pub segments: Vec<ExtractedSegment>,
    pub missing: Vec<String>,
    pub diagnostics: Vec<String>,
    pub status: String,
}

impl ExtractedDocument {
    pub fn to_json(&self) -> Value {
        json!({
            "schema_version": 1,
            "source_uri": self.source_uri,
            "source_digest": self.source_digest,
            "format": self.format,
            "extractor_version": self.extractor_version,
            "status": self.status,
            "text": self.text,
            "segments": self.segments.iter().map(|s| json!({
                "id": s.id,
                "kind": s.kind,
                "text": s.text,
                "locator": s.locator,
                "confidence": s.confidence,
            })).collect::<Vec<_>>(),
            "missing": self.missing,
            "diagnostics": self.diagnostics,
        })
    }
}

/// Extract text/Markdown from a file. Uploaded `.mq.md` is treated as untrusted
/// source text and is never executed here.
pub fn extract_document(source: &DocumentSource) -> ExtractedDocument {
    let path = &source.path;
    let uri = path.to_string_lossy().replace('\\', "/");
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let format = source
        .format_hint
        .clone()
        .unwrap_or_else(|| {
            if name.ends_with(".mq.md") || ext == "md" {
                "markdown".into()
            } else if matches!(ext.as_str(), "txt" | "text") {
                "text".into()
            } else if matches!(ext.as_str(), "pdf" | "docx") {
                ext.clone()
            } else {
                "unknown".into()
            }
        });

    if matches!(
        format.as_str(),
        "pdf" | "docx" | "ocr" | "png" | "jpg" | "jpeg" | "webp" | "tiff"
    ) || matches!(ext.as_str(), "pdf" | "docx" | "png" | "jpg" | "jpeg" | "webp" | "tiff")
    {
        let kind = if matches!(format.as_str(), "ocr")
            || matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "tiff")
        {
            "ocr/image".to_string()
        } else {
            format.clone()
        };
        return ExtractedDocument {
            source_uri: uri,
            source_digest: String::new(),
            format,
            extractor_version: EXTRACTOR_VERSION.into(),
            text: String::new(),
            segments: vec![],
            missing: vec!["full_document".into()],
            diagnostics: vec![format!(
                "format `{kind}` requires an external extractor plugin (not in core); incomplete extraction is explicit failure"
            )],
            status: "failed".into(),
        };
    }

    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            return ExtractedDocument {
                source_uri: uri,
                source_digest: String::new(),
                format,
                extractor_version: EXTRACTOR_VERSION.into(),
                text: String::new(),
                segments: vec![],
                missing: vec!["full_document".into()],
                diagnostics: vec![format!("read error: {e}")],
                status: "failed".into(),
            };
        }
    };

    if bytes.is_empty() {
        return ExtractedDocument {
            source_uri: uri,
            source_digest: digest_bytes(&bytes),
            format,
            extractor_version: EXTRACTOR_VERSION.into(),
            text: String::new(),
            segments: vec![],
            missing: vec!["content".into()],
            diagnostics: vec!["empty document".into()],
            status: "failed".into(),
        };
    }

    // Resource limit: refuse to fully buffer huge uploads as text (M01).
    const MAX_EXTRACT_BYTES: usize = 8 << 20;
    if bytes.len() > MAX_EXTRACT_BYTES {
        let head = &bytes[..MAX_EXTRACT_BYTES];
        let lossy = String::from_utf8_lossy(head).into_owned();
        let mut doc = extract_markdown_or_text(&uri, &digest_bytes(&bytes), &format, &lossy);
        doc.missing.push("tail_beyond_limit".into());
        doc.diagnostics.push(format!(
            "document truncated at {MAX_EXTRACT_BYTES} bytes (got {}); extraction is partial",
            bytes.len()
        ));
        doc.status = "partial".into();
        return doc;
    }

    let text = match String::from_utf8(bytes.clone()) {
        Ok(t) => t,
        Err(e) => {
            let lossy = String::from_utf8_lossy(&bytes).into_owned();
            let mut doc = extract_markdown_or_text(&uri, &digest_bytes(&bytes), &format, &lossy);
            doc.diagnostics
                .push(format!("invalid utf-8 ({e}); decoded lossy"));
            doc.status = if doc.text.is_empty() {
                "failed".into()
            } else {
                "partial".into()
            };
            return doc;
        }
    };

    extract_markdown_or_text(&uri, &digest_bytes(&bytes), &format, &text)
}

fn extract_markdown_or_text(
    uri: &str,
    digest: &str,
    format: &str,
    text: &str,
) -> ExtractedDocument {
    let mut segments = Vec::new();
    let mut n = 0u32;
    for (i, line) in text.lines().enumerate() {
        let line_no = i + 1;
        if let Some(rest) = line.trim().strip_prefix('#') {
            n += 1;
            let heading = rest.trim_start_matches('#').trim();
            segments.push(ExtractedSegment {
                id: format!("seg-{n}"),
                kind: "heading".into(),
                text: heading.to_string(),
                locator: format!("L{line_no}"),
                confidence: 1.0,
            });
        } else if line.trim().starts_with('|') {
            n += 1;
            segments.push(ExtractedSegment {
                id: format!("seg-{n}"),
                kind: "table_row".into(),
                text: line.to_string(),
                locator: format!("L{line_no}"),
                confidence: 0.9,
            });
        } else if !line.trim().is_empty() {
            n += 1;
            segments.push(ExtractedSegment {
                id: format!("seg-{n}"),
                kind: "paragraph".into(),
                text: line.to_string(),
                locator: format!("L{line_no}"),
                confidence: 1.0,
            });
        }
    }
    ExtractedDocument {
        source_uri: uri.to_string(),
        source_digest: digest.to_string(),
        format: format.to_string(),
        extractor_version: EXTRACTOR_VERSION.into(),
        text: text.to_string(),
        segments,
        missing: vec![],
        diagnostics: vec![],
        status: "ok".into(),
    }
}

fn digest_bytes(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("sha256:{:x}", h.finalize())
}

/// Convenience for tests / CLI: extract if path exists.
pub fn extract_path(path: &Path) -> ExtractedDocument {
    extract_document(&DocumentSource {
        path: path.to_path_buf(),
        format_hint: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn extracts_markdown_headings() {
        let dir = std::env::temp_dir().join(format!("mq-extract-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("note.md");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "# rate\n\nhello").unwrap();
        let doc = extract_path(&path);
        assert_eq!(doc.status, "ok");
        assert!(doc.segments.iter().any(|s| s.kind == "heading" && s.text == "rate"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn pdf_reports_missing_extractor() {
        let doc = extract_document(&DocumentSource {
            path: Path::new("x.pdf").into(),
            format_hint: Some("pdf".into()),
        });
        assert_eq!(doc.status, "failed");
        assert!(!doc.diagnostics.is_empty());
    }

    #[test]
    fn ocr_image_reports_missing_extractor() {
        let doc = extract_document(&DocumentSource {
            path: Path::new("scan.png").into(),
            format_hint: Some("ocr".into()),
        });
        assert_eq!(doc.status, "failed");
        assert!(doc.missing.contains(&"full_document".into()));
        assert!(doc.diagnostics.iter().any(|d| d.contains("ocr") || d.contains("extractor")));
    }
}
