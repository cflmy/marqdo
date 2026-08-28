//! HTTP upload validation and safe object keys (W5).

use serde_json::{json, Value};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Validate size / MIME / extension against an optional allowlist table.
///
/// `types` may be:
/// - `null` / missing → allow any
/// - string CSV of MIME types (`image/png,image/jpeg`)
/// - GFM-style rows: `|类型|扩展名|` / `|type|ext|` (ext may be comma-separated)
pub fn validate(
    filename: &str,
    content_type: &str,
    size: u64,
    max_bytes: u64,
    types: Option<&Value>,
) -> Result<Value, String> {
    if size > max_bytes {
        return Ok(json!({
            "ok": false,
            "error": format!("file too large: {size} > {max_bytes}"),
        }));
    }
    let name = filename.trim();
    if name.is_empty() {
        return Ok(json!({ "ok": false, "error": "empty filename" }));
    }
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        // Basename only is expected from clients; reject path fragments.
        return Ok(json!({ "ok": false, "error": "unsafe filename" }));
    }

    let ct = content_type.trim().to_ascii_lowercase();
    let ct = ct.split(';').next().unwrap_or(&ct).trim();
    let ext = name
        .rsplit('.')
        .next()
        .filter(|e| !e.is_empty() && *e != name)
        .unwrap_or("")
        .to_ascii_lowercase();

    if let Some(t) = types {
        if !types_allow(t, ct, &ext) {
            return Ok(json!({
                "ok": false,
                "error": format!("type not allowed: content_type={ct} ext={ext}"),
            }));
        }
    }

    Ok(json!({
        "ok": true,
        "filename": name,
        "content_type": if ct.is_empty() { "application/octet-stream" } else { ct },
        "size": size,
        "ext": ext,
    }))
}

fn types_allow(types: &Value, content_type: &str, ext: &str) -> bool {
    match types {
        Value::Null => true,
        Value::String(s) => {
            let s = s.trim();
            if s.is_empty() {
                return true;
            }
            s.split(',')
                .map(|p| p.trim().to_ascii_lowercase())
                .filter(|p| !p.is_empty())
                .any(|p| p == content_type)
        }
        Value::Array(rows) => {
            if rows.is_empty() {
                return true;
            }
            for row in rows {
                if row_allows(row, content_type, ext) {
                    return true;
                }
            }
            false
        }
        Value::Object(obj) => {
            // Shapes:
            // 1) Columnar GFM: `{类型: [mime…], 扩展名: [ext…]}` (Marqdo default)
            // 2) Single allowlist row: `{类型|type, 扩展名|ext}` with string values
            // 3) MIME→ext map: `{ "image/png": "png", … }`
            let col_mime = obj
                .get("类型")
                .or_else(|| obj.get("type"))
                .or_else(|| obj.get("mime"));
            let col_ext = obj
                .get("扩展名")
                .or_else(|| obj.get("ext"))
                .or_else(|| obj.get("extension"));
            if let (Some(Value::Array(mimes)), Some(exts_v)) = (col_mime, col_ext) {
                let exts: Vec<String> = match exts_v {
                    Value::Array(a) => a
                        .iter()
                        .map(|x| x.as_str().unwrap_or("").to_ascii_lowercase())
                        .collect(),
                    Value::String(s) => vec![s.to_ascii_lowercase()],
                    _ => Vec::new(),
                };
                if mimes.is_empty() {
                    return true;
                }
                for (i, mime_v) in mimes.iter().enumerate() {
                    let mime = mime_v
                        .as_str()
                        .unwrap_or("")
                        .trim()
                        .to_ascii_lowercase();
                    let row_ext = exts.get(i).map(|s| s.as_str()).unwrap_or("");
                    let mime_ok = mime.is_empty() || mime == content_type;
                    let ext_ok = if row_ext.is_empty() {
                        true
                    } else {
                        row_ext
                            .split(',')
                            .map(|e| e.trim().trim_start_matches('.'))
                            .any(|e| e == ext)
                    };
                    if mime_ok && ext_ok {
                        return true;
                    }
                }
                return false;
            }
            if col_mime.is_some() || col_ext.is_some() {
                return row_allows(&Value::Object(obj.clone()), content_type, ext);
            }
            if obj.is_empty() {
                return true;
            }
            for (mime, exts_v) in obj {
                let mime = mime.trim().to_ascii_lowercase();
                let exts = match exts_v {
                    Value::String(s) => s.clone(),
                    Value::Array(a) => a
                        .iter()
                        .filter_map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                    _ => String::new(),
                };
                let exts = exts.trim().to_ascii_lowercase();
                let mime_ok = mime.is_empty() || mime == content_type;
                let ext_ok = if exts.is_empty() {
                    true
                } else {
                    exts.split(',')
                        .map(|e| e.trim().trim_start_matches('.'))
                        .any(|e| e == ext)
                };
                if mime_ok && ext_ok {
                    return true;
                }
            }
            false
        }
        _ => true,
    }
}

fn row_allows(row: &Value, content_type: &str, ext: &str) -> bool {
    let Some(obj) = row.as_object() else {
        if let Some(s) = row.as_str() {
            return s.trim().eq_ignore_ascii_case(content_type);
        }
        return false;
    };
    let mime = obj
        .get("类型")
        .or_else(|| obj.get("type"))
        .or_else(|| obj.get("mime"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let exts = obj
        .get("扩展名")
        .or_else(|| obj.get("ext"))
        .or_else(|| obj.get("extension"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let mime_ok = mime.is_empty() || mime == content_type;
    let ext_ok = if exts.is_empty() {
        true
    } else {
        exts.split(',')
            .map(|e| e.trim().trim_start_matches('.'))
            .any(|e| e == ext)
    };
    mime_ok && ext_ok
}

/// Build `prefix` + timestamp-random + sanitized basename.
pub fn make_key(prefix: &str, filename: &str) -> Result<String, String> {
    let base = filename
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(filename)
        .trim();
    if base.is_empty() {
        return Err("empty filename".into());
    }
    let safe: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if safe.contains("..") {
        return Err("unsafe filename".into());
    }
    let mut pref = prefix.trim().trim_start_matches('/').replace('\\', "/");
    if !pref.is_empty() && !pref.ends_with('/') {
        pref.push('/');
    }
    if pref.contains("..") {
        return Err("prefix must not contain `..`".into());
    }
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut rnd = [0u8; 4];
    getrandom::getrandom(&mut rnd).map_err(|e| e.to_string())?;
    let hex = hex::encode(rnd);
    Ok(format!("{pref}{secs}-{hex}-{safe}"))
}

/// Offline save: read local `path` bytes into storage under `key` (or auto key).
pub fn save(
    storage_url: &str,
    key: Option<&str>,
    path: &str,
    content_type: Option<&str>,
    prefix: Option<&str>,
) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|e| format!("read `{path}`: {e}"))?;
    let filename = path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("upload.bin");
    let ct = content_type
        .filter(|s| !s.is_empty())
        .unwrap_or("application/octet-stream");
    let object_key = match key.filter(|k| !k.is_empty()) {
        Some(k) => k.to_string(),
        None => make_key(prefix.unwrap_or("uploads/"), filename)?,
    };
    crate::storage::put_bytes(storage_url, &object_key, &bytes, ct)
}
