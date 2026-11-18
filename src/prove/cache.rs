//! 证书缓存（设计 §8.3）：复合键、命中跳判定、失效。
//!
//! 键 = sha256(源码 ⊕ plan.json ⊕ schema ⊕ 库 sha256 ⊕ lean --version ⊕ marqdo 版本)。
//! 证书落在产物目录（`.marqdo/prove/<srcKey8>/certificate.json`）；键不合即失效。

use std::path::Path;

use anyhow::Result;
use sha2::{Digest, Sha256};

use crate::prove::Verdict;

pub const CERT_SCHEMA: u32 = 1;

/// 单条定理的判定记录。
#[derive(Debug, Clone, PartialEq)]
pub struct ClaimRecord {
    pub id: String,
    pub function: String,
    pub status: Verdict,
    pub source_line: Option<u32>,
}

/// 一份判定证书（复核收据，不是发布许可）。
#[derive(Debug, Clone)]
pub struct Certificate {
    pub key: String,
    pub status: Verdict,
    pub tier: &'static str,
    pub source_sha256: String,
    pub library_sha256: String,
    pub toolchain: String,
    pub module_sha256: String,
    pub claims: Vec<ClaimRecord>,
    /// lean 行 → mq 行。
    pub span_map: Vec<(u32, u32)>,
    pub marqdo: String,
}

/// 复合缓存键。
pub fn cache_key(
    source_sha256: &str,
    plan_sha256: &str,
    library_sha256: &str,
    toolchain: &str,
    marqdo_version: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_sha256.as_bytes());
    hasher.update(b"\x00");
    hasher.update(plan_sha256.as_bytes());
    hasher.update(b"\x00");
    hasher.update(library_sha256.as_bytes());
    hasher.update(b"\x00");
    hasher.update(toolchain.as_bytes());
    hasher.update(b"\x00");
    hasher.update(marqdo_version.as_bytes());
    hex(&hasher.finalize())
}

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex(&hasher.finalize())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl Certificate {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "schema": CERT_SCHEMA,
            "key": self.key,
            "status": self.status.as_str(),
            "tier": self.tier,
            "source_sha256": self.source_sha256,
            "library_sha256": self.library_sha256,
            "toolchain": self.toolchain,
            "module_sha256": self.module_sha256,
            "claims": self.claims.iter().map(|c| serde_json::json!({
                "id": c.id,
                "function": c.function,
                "status": c.status.as_str(),
                "source_line": c.source_line,
            })).collect::<Vec<_>>(),
            "span_map": self.span_map.iter().map(|(l, m)| serde_json::json!({
                "lean_line": l, "mq_line": m,
            })).collect::<Vec<_>>(),
            "marqdo": self.marqdo,
        })
    }

    fn from_json(value: &serde_json::Value) -> Option<Certificate> {
        use serde_json::Value;
        if value.get("schema")?.as_u64()? != CERT_SCHEMA as u64 {
            return None;
        }
        let status = match value.get("status")?.as_str()? {
            "proved" => Verdict::Proved,
            "checked" => Verdict::Checked,
            "unverified" => Verdict::Unverified,
            "unproved" => Verdict::Unproved,
            "rejected" => Verdict::Rejected,
            "error" => Verdict::Error,
            _ => return None,
        };
        let parse_line = |v: &Value| v.as_u64().map(|n| n as u32);
        Some(Certificate {
            key: value.get("key")?.as_str()?.to_string(),
            status,
            tier: match value.get("tier")?.as_str()? {
                "lean" => "lean",
                "offline" => "offline",
                _ => "none",
            },
            source_sha256: value.get("source_sha256")?.as_str()?.to_string(),
            library_sha256: value.get("library_sha256")?.as_str()?.to_string(),
            toolchain: value.get("toolchain")?.as_str()?.to_string(),
            module_sha256: value.get("module_sha256")?.as_str()?.to_string(),
            claims: value
                .get("claims")?
                .as_array()?
                .iter()
                .map(|c| {
                    Some(ClaimRecord {
                        id: c.get("id")?.as_str()?.to_string(),
                        function: c.get("function")?.as_str()?.to_string(),
                        status: match c.get("status")?.as_str()? {
                            "proved" => Verdict::Proved,
                            "checked" => Verdict::Checked,
                            "unverified" => Verdict::Unverified,
                            "unproved" => Verdict::Unproved,
                            "rejected" => Verdict::Rejected,
                            "error" => Verdict::Error,
                            _ => return None,
                        },
                        source_line: c.get("source_line").and_then(parse_line),
                    })
                })
                .collect::<Option<Vec<_>>>()?,
            span_map: value
                .get("span_map")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|e| {
                            Some((
                                e.get("lean_line")?.as_u64()? as u32,
                                e.get("mq_line")?.as_u64()? as u32,
                            ))
                        })
                        .collect()
                })
                .unwrap_or_default(),
            marqdo: value.get("marqdo")?.as_str()?.to_string(),
        })
    }
}

/// 读证书：文件缺失/损坏/键不合 → None（调用方重判）。
pub fn load(artifact_dir: &Path, key: &str) -> Option<Certificate> {
    let path = artifact_dir.join("certificate.json");
    let raw = std::fs::read_to_string(path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let cert = Certificate::from_json(&value)?;
    (cert.key == key).then_some(cert)
}

/// 写证书（原子：临时文件 + 改名）。
pub fn store(artifact_dir: &Path, cert: &Certificate) -> Result<()> {
    std::fs::create_dir_all(artifact_dir)?;
    let path = artifact_dir.join("certificate.json");
    let tmp = artifact_dir.join(".certificate.json.tmp");
    std::fs::write(&tmp, cert.to_json().to_string())?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_order_sensitive_and_stable() {
        let a = cache_key("s", "p", "l", "lean 4", "2.8.0");
        let b = cache_key("s", "p", "l", "lean 4", "2.8.0");
        let c = cache_key("p", "s", "l", "lean 4", "2.8.0");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn certificate_roundtrip_preserves_verdicts() {
        let cert = Certificate {
            key: "k".into(),
            status: Verdict::Unproved,
            tier: "lean",
            source_sha256: "s".into(),
            library_sha256: "l".into(),
            toolchain: "lean 4.35.0".into(),
            module_sha256: "m".into(),
            claims: vec![ClaimRecord {
                id: "大额必经人工".into(),
                function: "审批".into(),
                status: Verdict::Unproved,
                source_line: Some(21),
            }],
            span_map: vec![(41, 21)],
            marqdo: "2.8.0".into(),
        };
        let parsed = Certificate::from_json(&cert.to_json()).expect("roundtrip");
        assert_eq!(parsed.status, Verdict::Unproved);
        assert_eq!(parsed.claims.len(), 1);
        assert_eq!(parsed.claims[0].status, Verdict::Unproved);
        assert_eq!(parsed.claims[0].source_line, Some(21));
    }

    #[test]
    fn load_rejects_wrong_key_and_corrupt_file() {
        let dir = std::env::temp_dir().join(format!("mq-cache-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(load(&dir, "k").is_none());
        let cert = Certificate {
            key: "k1".into(),
            status: Verdict::Proved,
            tier: "lean",
            source_sha256: "s".into(),
            library_sha256: "l".into(),
            toolchain: "t".into(),
            module_sha256: "m".into(),
            claims: vec![],
            span_map: vec![],
            marqdo: "2.8.0".into(),
        };
        store(&dir, &cert).unwrap();
        assert!(load(&dir, "k1").is_some());
        assert!(load(&dir, "k2").is_none(), "键不合必须失效");
        std::fs::write(dir.join("certificate.json"), "{broken").unwrap();
        assert!(load(&dir, "k1").is_none(), "坏证书必须失效，不 panic");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
