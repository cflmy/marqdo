//! `# storage` — local `file:` blobs; `s3://` reserved for live S3/MinIO (SigV4 later).

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn is_file(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.starts_with("file:") || u.starts_with("file://")
}

fn is_s3(url: &str) -> bool {
    url.trim().to_ascii_lowercase().starts_with("s3://")
}

fn file_root(url: &str) -> Result<PathBuf, String> {
    let raw = url
        .trim()
        .strip_prefix("file://")
        .or_else(|| url.trim().strip_prefix("file:"))
        .unwrap_or(url.trim());
    let p = PathBuf::from(raw);
    if !p.as_os_str().is_empty() {
        fs::create_dir_all(&p).map_err(|e| format!("storage mkdir: {e}"))?;
    }
    Ok(p)
}

fn safe_key(key: &str) -> Result<String, String> {
    let k = key.trim().trim_start_matches('/');
    if k.is_empty() {
        return Err("storage key is empty".into());
    }
    if k.contains("..") {
        return Err("storage key must not contain `..`".into());
    }
    Ok(k.replace('\\', "/"))
}

fn path_for(root: &Path, key: &str) -> Result<PathBuf, String> {
    let k = safe_key(key)?;
    Ok(root.join(k))
}

/// Open storage. For `file:` ensures the directory exists.
pub fn open(url: &str) -> Result<Value, String> {
    let url = url.trim();
    if is_file(url) {
        let _ = file_root(url)?;
        Ok(json!({ "_type": "storage", "url": url, "backend": "file" }))
    } else if is_s3(url) {
        // Validate shape early so author typos fail at open, not at put.
        let rest = url
            .strip_prefix("s3://")
            .ok_or_else(|| "bad s3 url".to_string())?;
        let bucket = rest.split('?').next().unwrap_or("");
        if bucket.is_empty() {
            return Err("s3 url missing bucket (s3://bucket?endpoint=…)".into());
        }
        Ok(json!({ "_type": "storage", "url": url, "backend": "s3" }))
    } else {
        Err(format!(
            "storage url must be `file:…` or `s3://bucket?…`, got `{url}`"
        ))
    }
}

fn load_bytes(body: Option<&str>, path: Option<&str>) -> Result<Vec<u8>, String> {
    if let Some(p) = path.filter(|s| !s.is_empty()) {
        return fs::read(p).map_err(|e| format!("read path `{p}`: {e}"));
    }
    if let Some(b) = body {
        return Ok(b.as_bytes().to_vec());
    }
    Err("storage.put needs `body` or `path`".into())
}

pub fn put(
    url: &str,
    key: &str,
    body: Option<&str>,
    path: Option<&str>,
    content_type: Option<&str>,
) -> Result<Value, String> {
    let bytes = load_bytes(body, path)?;
    let ct = content_type.unwrap_or("application/octet-stream");
    if is_file(url) {
        let root = file_root(url)?;
        let dest = path_for(&root, key)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
        let _ = fs::write(ctype_path(&dest), ct);
        Ok(json!({ "ok": true, "key": key, "size": bytes.len() }))
    } else if is_s3(url) {
        s3_put_http(url, key, &bytes, ct)
    } else {
        Err("unknown storage backend".into())
    }
}

fn ctype_path(dest: &Path) -> PathBuf {
    PathBuf::from(format!("{}.ctype", dest.display()))
}

pub fn get(url: &str, key: &str) -> Result<Value, String> {
    if is_file(url) {
        let root = file_root(url)?;
        let dest = path_for(&root, key)?;
        if !dest.is_file() {
            return Ok(json!({ "ok": false }));
        }
        let bytes = fs::read(&dest).map_err(|e| e.to_string())?;
        let ct = fs::read_to_string(ctype_path(&dest))
            .unwrap_or_else(|_| "application/octet-stream".into());
        match String::from_utf8(bytes.clone()) {
            Ok(s) => Ok(json!({
                "ok": true,
                "key": key,
                "body": s,
                "content_type": ct,
                "size": s.len(),
            })),
            Err(_) => Ok(json!({
                "ok": true,
                "key": key,
                "path": dest.to_string_lossy(),
                "content_type": ct,
                "size": bytes.len(),
                "binary": true,
            })),
        }
    } else if is_s3(url) {
        s3_get_http(url, key)
    } else {
        Err("unknown storage backend".into())
    }
}

pub fn delete(url: &str, key: &str) -> Result<Value, String> {
    if is_file(url) {
        let root = file_root(url)?;
        let dest = path_for(&root, key)?;
        let ok = if dest.is_file() {
            fs::remove_file(&dest).map_err(|e| e.to_string())?;
            let _ = fs::remove_file(ctype_path(&dest));
            true
        } else {
            false
        };
        Ok(json!({ "ok": ok }))
    } else if is_s3(url) {
        s3_delete_http(url, key)
    } else {
        Err("unknown storage backend".into())
    }
}

pub fn list(url: &str, prefix: Option<&str>) -> Result<Value, String> {
    let prefix = prefix.unwrap_or("").trim_start_matches('/');
    if is_file(url) {
        let root = file_root(url)?;
        let mut keys = Vec::new();
        walk_keys(&root, &root, prefix, &mut keys)?;
        keys.sort();
        Ok(json!({ "ok": true, "keys": keys, "count": keys.len() }))
    } else if is_s3(url) {
        s3_list_http(url, prefix)
    } else {
        Err("unknown storage backend".into())
    }
}

fn walk_keys(root: &Path, dir: &Path, prefix: &str, out: &mut Vec<String>) -> Result<(), String> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    for ent in entries {
        let ent = ent.map_err(|e| e.to_string())?;
        let path = ent.path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".ctype"))
        {
            continue;
        }
        if path.is_dir() {
            walk_keys(root, &path, prefix, out)?;
        } else if path.is_file() {
            let rel = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .replace('\\', "/");
            if prefix.is_empty() || rel.starts_with(prefix) {
                out.push(rel);
            }
        }
    }
    Ok(())
}

/// Live S3/MinIO via AWS SigV4. Requires `endpoint` query for path-style (MinIO).
fn s3_put_http(url: &str, key: &str, body: &[u8], content_type: &str) -> Result<Value, String> {
    let cfg = S3Conf::parse(url)?;
    let target = cfg.object_url(key)?;
    let status = cfg.request("PUT", &target, body, content_type)?;
    if (200..300).contains(&status) {
        Ok(json!({ "ok": true, "key": key, "size": body.len() }))
    } else {
        Err(format!("s3 put HTTP {status}"))
    }
}

fn s3_get_http(url: &str, key: &str) -> Result<Value, String> {
    let cfg = S3Conf::parse(url)?;
    let target = cfg.object_url(key)?;
    let (status, ct, bytes) = cfg.request_bytes("GET", &target)?;
    if status == 404 {
        return Ok(json!({ "ok": false }));
    }
    if !(200..300).contains(&status) {
        return Err(format!("s3 get HTTP {status}"));
    }
    match String::from_utf8(bytes.clone()) {
        Ok(s) => Ok(json!({
            "ok": true, "key": key, "body": s, "content_type": ct, "size": s.len()
        })),
        Err(_) => Ok(json!({
            "ok": true, "key": key, "content_type": ct, "size": bytes.len(), "binary": true
        })),
    }
}

fn s3_delete_http(url: &str, key: &str) -> Result<Value, String> {
    let cfg = S3Conf::parse(url)?;
    let target = cfg.object_url(key)?;
    let status = cfg.request("DELETE", &target, &[], "")?;
    if (200..300).contains(&status) || status == 204 || status == 404 {
        Ok(json!({ "ok": status != 404 }))
    } else {
        Err(format!("s3 delete HTTP {status}"))
    }
}

fn s3_list_http(url: &str, prefix: &str) -> Result<Value, String> {
    let cfg = S3Conf::parse(url)?;
    let list_url = cfg.list_url(prefix);
    let (status, _ct, bytes) = cfg.request_bytes("GET", &list_url)?;
    let text = String::from_utf8_lossy(&bytes).into_owned();
    if !(200..300).contains(&status) {
        return Err(format!("s3 list HTTP {status}: {text}"));
    }
    let mut keys = Vec::new();
    for part in text.split("<Key>").skip(1) {
        if let Some(end) = part.find("</Key>") {
            keys.push(part[..end].to_string());
        }
    }
    Ok(json!({ "ok": true, "keys": keys, "count": keys.len() }))
}

struct S3Conf {
    bucket: String,
    endpoint: String,
    region: String,
    access_key: String,
    secret_key: String,
    path_style: bool,
}

impl S3Conf {
    fn parse(url: &str) -> Result<Self, String> {
        let rest = url
            .trim()
            .strip_prefix("s3://")
            .ok_or_else(|| "not an s3 url".to_string())?;
        let (bucket, query) = match rest.split_once('?') {
            Some((b, q)) => (b.to_string(), q),
            None => (rest.to_string(), ""),
        };
        if bucket.is_empty() {
            return Err("s3 url missing bucket".into());
        }
        let mut endpoint = String::new();
        let mut region = "us-east-1".to_string();
        let mut access_key = std::env::var("AWS_ACCESS_KEY_ID")
            .or_else(|_| std::env::var("S3_ACCESS_KEY"))
            .unwrap_or_default();
        let mut secret_key = std::env::var("AWS_SECRET_ACCESS_KEY")
            .or_else(|_| std::env::var("S3_SECRET_KEY"))
            .unwrap_or_default();
        let mut path_style = true;
        for pair in query.split('&').filter(|p| !p.is_empty()) {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            let v = pct_decode(v);
            match k {
                "endpoint" => endpoint = v,
                "region" => region = v,
                "access_key" | "accessKey" => access_key = v,
                "secret_key" | "secretKey" => secret_key = v,
                "path_style" => {
                    path_style = matches!(v.as_str(), "1" | "true" | "True" | "yes")
                }
                _ => {}
            }
        }
        if endpoint.is_empty() {
            return Err(
                "s3 url needs `endpoint=` for MinIO/path-style (e.g. s3://bucket?endpoint=http://127.0.0.1:9000)"
                    .into(),
            );
        }
        if access_key.is_empty() || secret_key.is_empty() {
            return Err(
                "s3 credentials missing (access_key/secret_key query or AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY)"
                    .into(),
            );
        }
        Ok(Self {
            bucket,
            endpoint: endpoint.trim_end_matches('/').to_string(),
            region,
            access_key,
            secret_key,
            path_style,
        })
    }

    fn object_url(&self, key: &str) -> Result<String, String> {
        let k = safe_key(key)?;
        if self.path_style {
            Ok(format!("{}/{}/{}", self.endpoint, self.bucket, k))
        } else {
            Err("virtual-hosted s3 not implemented in this wave — set path_style=true".into())
        }
    }

    fn list_url(&self, prefix: &str) -> String {
        format!(
            "{}/{}?list-type=2&prefix={}",
            self.endpoint,
            self.bucket,
            uri_encode(prefix)
        )
    }

    fn request(
        &self,
        method: &str,
        url: &str,
        body: &[u8],
        content_type: &str,
    ) -> Result<u16, String> {
        let (status, _, _) = self.request_inner(method, url, body, content_type)?;
        Ok(status)
    }

    fn request_bytes(&self, method: &str, url: &str) -> Result<(u16, String, Vec<u8>), String> {
        self.request_inner(method, url, &[], "")
    }

    fn request_inner(
        &self,
        method: &str,
        url: &str,
        body: &[u8],
        content_type: &str,
    ) -> Result<(u16, String, Vec<u8>), String> {
        use hmac::{Hmac, Mac};
        use sha2::{Digest, Sha256};
        type HmacSha256 = Hmac<Sha256>;

        let (scheme_host, path_query) = split_url(url)?;
        let (path, query) = match path_query.split_once('?') {
            Some((p, q)) => (p, q),
            None => (path_query.as_str(), ""),
        };
        let host = scheme_host
            .split("://")
            .nth(1)
            .ok_or_else(|| "bad url host".to_string())?;

        let amz_date = amz_timestamp();
        let date_stamp = &amz_date[..8];
        let payload_hash = hex::encode(Sha256::digest(body));

        let mut hdrs: Vec<(String, String)> = vec![
            ("host".into(), host.into()),
            ("x-amz-content-sha256".into(), payload_hash.clone()),
            ("x-amz-date".into(), amz_date.clone()),
        ];
        if method == "PUT" && !content_type.is_empty() {
            hdrs.push(("content-type".into(), content_type.into()));
        }
        hdrs.sort_by(|a, b| a.0.cmp(&b.0));
        let signed_headers = hdrs.iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>().join(";");
        let canonical_headers = hdrs
            .iter()
            .map(|(k, v)| format!("{k}:{}\n", v.trim()))
            .collect::<String>();
        let canonical_query = canonical_query_string(query);
        let canonical_request = format!(
            "{method}\n{path}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );
        let scope = format!("{date_stamp}/{}/s3/aws4_request", self.region);
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{scope}\n{}",
            hex::encode(Sha256::digest(canonical_request.as_bytes()))
        );

        let mut mac =
            HmacSha256::new_from_slice(format!("AWS4{}", self.secret_key).as_bytes()).unwrap();
        mac.update(date_stamp.as_bytes());
        let k_date = mac.finalize().into_bytes();
        let mut mac = HmacSha256::new_from_slice(&k_date).unwrap();
        mac.update(self.region.as_bytes());
        let k_region = mac.finalize().into_bytes();
        let mut mac = HmacSha256::new_from_slice(&k_region).unwrap();
        mac.update(b"s3");
        let k_service = mac.finalize().into_bytes();
        let mut mac = HmacSha256::new_from_slice(&k_service).unwrap();
        mac.update(b"aws4_request");
        let k_signing = mac.finalize().into_bytes();
        let mut mac = HmacSha256::new_from_slice(&k_signing).unwrap();
        mac.update(string_to_sign.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        let auth = format!(
            "AWS4-HMAC-SHA256 Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
            self.access_key
        );

        let agent = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(30))
            .build();
        let mut req = agent.request(method, url);
        req = req
            .set("Authorization", &auth)
            .set("x-amz-content-sha256", &payload_hash)
            .set("x-amz-date", &amz_date);
        if method == "PUT" && !content_type.is_empty() {
            req = req.set("Content-Type", content_type);
        }
        let result = if method == "PUT" {
            req.send_bytes(body)
        } else {
            req.call()
        };
        match result {
            Ok(r) => {
                let status = r.status();
                let ct = r
                    .header("content-type")
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let mut buf = Vec::new();
                let _ = std::io::Read::read_to_end(&mut r.into_reader(), &mut buf);
                Ok((status, ct, buf))
            }
            Err(ureq::Error::Status(code, r)) => {
                let ct = r
                    .header("content-type")
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let mut buf = Vec::new();
                let _ = std::io::Read::read_to_end(&mut r.into_reader(), &mut buf);
                Ok((code, ct, buf))
            }
            Err(e) => Err(e.to_string()),
        }
    }
}

fn split_url(url: &str) -> Result<(String, String), String> {
    let without = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .ok_or_else(|| "url must be http(s)".to_string())?;
    let scheme = if url.starts_with("https://") {
        "https"
    } else {
        "http"
    };
    let (hostport, rest) = match without.find('/') {
        Some(i) => (&without[..i], &without[i..]),
        None => (without, "/"),
    };
    Ok((format!("{scheme}://{hostport}"), rest.to_string()))
}

fn amz_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // UTC YYYYMMDDTHHMMSSZ via a tiny formatter (no chrono dep).
    let days = secs / 86400;
    let tod = secs % 86400;
    let hour = tod / 3600;
    let min = (tod % 3600) / 60;
    let sec = tod % 60;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}{m:02}{d:02}T{hour:02}{min:02}{sec:02}Z")
}

/// Algorithm from Howard Hinnant's date algorithms (days since 1970-01-01 → Y-M-D).
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

fn uri_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn canonical_query_string(query: &str) -> String {
    if query.is_empty() {
        return String::new();
    }
    let mut pairs: Vec<(String, String)> = query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (pct_decode(k), pct_decode(v))
        })
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", uri_encode(k), uri_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn pct_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or(""), 16)
            {
                out.push(v as char);
                i += 3;
                continue;
            }
        }
        out.push(b[i] as char);
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn file_put_get_list_delete() {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("marqdo-storage-{nanos}"));
        let url = format!("file:{}", dir.display());
        open(&url).unwrap();
        put(&url, "a/hello.txt", Some("hi"), None, Some("text/plain")).unwrap();
        let g = get(&url, "a/hello.txt").unwrap();
        assert_eq!(g["ok"], true);
        assert_eq!(g["body"], "hi");
        let ls = list(&url, Some("a/")).unwrap();
        assert_eq!(ls["count"], 1);
        delete(&url, "a/hello.txt").unwrap();
        assert_eq!(get(&url, "a/hello.txt").unwrap()["ok"], false);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn s3_open_validates_bucket() {
        let r = open("s3://mybucket?endpoint=http://127.0.0.1:9000");
        assert!(r.is_ok());
        assert!(open("s3://").is_err());
    }
}
