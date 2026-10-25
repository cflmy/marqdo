//! Project snapshot analysis cache (M05 / F14): keyed by project_root + source digests.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

#[derive(Clone)]
struct CacheEntry {
    digest_key: String,
    #[allow(dead_code)]
    compiled_at: Instant,
}

fn cache() -> &'static Mutex<HashMap<String, CacheEntry>> {
    static C: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Fingerprint of authorized project sources (shallow: entry + .mq.md under root, capped).
pub fn project_source_digest(project_root: &Path) -> String {
    let mut h = Sha256::new();
    h.update(project_root.to_string_lossy().as_bytes());
    let mut paths: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(project_root) {
        for ent in rd.flatten().take(200) {
            let p = ent.path();
            if p.extension().and_then(|e| e.to_str()) == Some("md")
                || p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".mq.md"))
                    .unwrap_or(false)
            {
                paths.push(p);
            }
        }
    }
    paths.sort();
    for p in paths {
        h.update(p.to_string_lossy().as_bytes());
        if let Ok(bytes) = std::fs::read(&p) {
            h.update(&bytes);
        }
    }
    format!("{:x}", h.finalize())
}

/// Record/reuse analysis for concurrent same-snapshot reads. Returns true if cache hit.
pub fn note_analysis(project_root: &Path) -> bool {
    let key = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf())
        .to_string_lossy()
        .to_string();
    let digest = project_source_digest(project_root);
    let mut g = cache().lock().unwrap();
    if let Some(e) = g.get(&key) {
        if e.digest_key == digest {
            return true;
        }
    }
    g.insert(
        key,
        CacheEntry {
            digest_key: digest,
            compiled_at: Instant::now(),
        },
    );
    false
}

/// Two project roots must not share cache entries even with identical relative paths.
pub fn cache_keys_isolated(a: &Path, b: &Path) -> bool {
    let ka = a
        .canonicalize()
        .unwrap_or_else(|_| a.to_path_buf())
        .to_string_lossy()
        .to_string();
    let kb = b
        .canonicalize()
        .unwrap_or_else(|_| b.to_path_buf())
        .to_string_lossy()
        .to_string();
    ka != kb
}
