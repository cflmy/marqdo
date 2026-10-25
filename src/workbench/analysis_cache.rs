//! Project snapshot analysis cache (M05 / F14): keyed by project_root + source digests.
//! Concurrent same-snapshot requests coalesce: waiters reuse the leader's result.

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::Instant;

#[derive(Clone)]
struct CacheEntry {
    digest_key: String,
    #[allow(dead_code)]
    compiled_at: Instant,
}

struct CacheState {
    entries: HashMap<String, CacheEntry>,
    /// project_key → refcount of leaders currently analyzing.
    inflight: HashMap<String, usize>,
}

fn cache() -> &'static (Mutex<CacheState>, Condvar) {
    static C: OnceLock<(Mutex<CacheState>, Condvar)> = OnceLock::new();
    C.get_or_init(|| {
        (
            Mutex::new(CacheState {
                entries: HashMap::new(),
                inflight: HashMap::new(),
            }),
            Condvar::new(),
        )
    })
}

static HITS: AtomicU64 = AtomicU64::new(0);
static MISSES: AtomicU64 = AtomicU64::new(0);
static MERGES: AtomicU64 = AtomicU64::new(0);

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

fn project_key(project_root: &Path) -> String {
    project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf())
        .to_string_lossy()
        .to_string()
}

/// Ticket for a leader analysis; must call [`finish_analysis`] (or drop via finish).
pub struct AnalysisTicket {
    key: String,
    digest: String,
    /// true ⇒ this caller is the leader and must run compile then finish.
    pub is_leader: bool,
}

/// Begin analysis: cache hit / wait for in-flight leader / become leader.
pub fn begin_analysis(project_root: &Path) -> AnalysisTicket {
    let key = project_key(project_root);
    let digest = project_source_digest(project_root);
    let (lock, cv) = cache();
    let mut g = lock.lock().unwrap();
    loop {
        if let Some(e) = g.entries.get(&key) {
            if e.digest_key == digest {
                HITS.fetch_add(1, Ordering::Relaxed);
                return AnalysisTicket {
                    key,
                    digest,
                    is_leader: false,
                };
            }
        }
        if g.inflight.contains_key(&key) {
            MERGES.fetch_add(1, Ordering::Relaxed);
            g = cv.wait(g).unwrap();
            continue;
        }
        *g.inflight.entry(key.clone()).or_insert(0) += 1;
        MISSES.fetch_add(1, Ordering::Relaxed);
        return AnalysisTicket {
            key,
            digest,
            is_leader: true,
        };
    }
}

pub fn finish_analysis(ticket: AnalysisTicket) {
    if !ticket.is_leader {
        return;
    }
    let (lock, cv) = cache();
    let mut g = lock.lock().unwrap();
    g.entries.insert(
        ticket.key.clone(),
        CacheEntry {
            digest_key: ticket.digest,
            compiled_at: Instant::now(),
        },
    );
    if let Some(n) = g.inflight.get_mut(&ticket.key) {
        *n = n.saturating_sub(1);
        if *n == 0 {
            g.inflight.remove(&ticket.key);
        }
    }
    cv.notify_all();
}

/// Compatibility: note without holding across compile (prefer begin/finish).
pub fn note_analysis(project_root: &Path) -> bool {
    let t = begin_analysis(project_root);
    let hit = !t.is_leader;
    finish_analysis(t);
    hit
}

pub fn cache_keys_isolated(a: &Path, b: &Path) -> bool {
    project_key(a) != project_key(b)
}

pub fn cache_stats() -> (u64, u64, u64) {
    (
        HITS.load(Ordering::Relaxed),
        MISSES.load(Ordering::Relaxed),
        MERGES.load(Ordering::Relaxed),
    )
}

pub fn reset_cache_for_tests() {
    let (lock, cv) = cache();
    let mut g = lock.lock().unwrap();
    g.entries.clear();
    g.inflight.clear();
    HITS.store(0, Ordering::Relaxed);
    MISSES.store(0, Ordering::Relaxed);
    MERGES.store(0, Ordering::Relaxed);
    cv.notify_all();
}
