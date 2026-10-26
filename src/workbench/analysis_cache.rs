//! AnalysisCache v2 (M-R5): SnapshotKey → Arc<Compilation> with single-flight.
//! Cache hits must reuse the compilation result, not merely a digest ticket.

use sha2::{Digest, Sha256};
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::sema::Compilation;

/// Full-project snapshot identity (design §8.1). Built from the compiler's
/// source enumeration — never a truncated `take(200)` directory listing.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SnapshotKey {
    pub project_identity: String,
    pub source_digest: String,
    pub compiler_version: String,
}

impl SnapshotKey {
    pub fn compute(project_root: &Path) -> Result<Self, String> {
        let root = crate::project::ProjectRoot::new(project_root)
            .map_err(|e| format!("snapshot.project: {e:#}"))?;
        let files = root
            .source_files()
            .map_err(|e| format!("snapshot.enumerate: {e:#}"))?;
        let mut h = Sha256::new();
        h.update(root.path().to_string_lossy().as_bytes());
        for p in &files {
            let rel = p
                .strip_prefix(root.path())
                .unwrap_or(p)
                .to_string_lossy();
            h.update(rel.as_bytes());
            let meta =
                std::fs::metadata(p).map_err(|e| format!("snapshot.meta {}: {e}", p.display()))?;
            h.update(meta.len().to_le_bytes());
            let bytes =
                std::fs::read(p).map_err(|e| format!("snapshot.read {}: {e}", p.display()))?;
            h.update(&bytes);
        }
        let identity = root
            .path()
            .canonicalize()
            .unwrap_or_else(|_| root.path().to_path_buf())
            .to_string_lossy()
            .to_string();
        Ok(Self {
            project_identity: identity,
            source_digest: format!("{:x}", h.finalize()),
            compiler_version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    pub fn cache_key(&self) -> String {
        format!(
            "{}::{}::{}",
            self.project_identity, self.source_digest, self.compiler_version
        )
    }
}

struct CacheEntry {
    #[allow(dead_code)]
    key: SnapshotKey,
    compilation: Arc<Compilation>,
    bytes: usize,
    #[allow(dead_code)]
    compiled_at: Instant,
}

struct InflightSlot {
    waiters: usize,
    result: Option<Result<Arc<Compilation>, String>>,
}

struct CacheState {
    order: VecDeque<String>,
    entries: HashMap<String, CacheEntry>,
    inflight: HashMap<String, InflightSlot>,
    bytes_used: usize,
    budget_bytes: usize,
}

fn cache() -> &'static (Mutex<CacheState>, Condvar) {
    static C: OnceLock<(Mutex<CacheState>, Condvar)> = OnceLock::new();
    C.get_or_init(|| {
        (
            Mutex::new(CacheState {
                order: VecDeque::new(),
                entries: HashMap::new(),
                inflight: HashMap::new(),
                bytes_used: 0,
                budget_bytes: default_budget_bytes(),
            }),
            Condvar::new(),
        )
    })
}

fn default_budget_bytes() -> usize {
    512 * 1024 * 1024
}

static HITS: AtomicU64 = AtomicU64::new(0);
static MISSES: AtomicU64 = AtomicU64::new(0);
static MERGES: AtomicU64 = AtomicU64::new(0);
static COMPILE_INVOCATIONS: AtomicU64 = AtomicU64::new(0);
static EVICTIONS: AtomicU64 = AtomicU64::new(0);
static RESULT_HITS: AtomicU64 = AtomicU64::new(0);

fn key_compiles() -> &'static Mutex<HashMap<String, u64>> {
    static K: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();
    K.get_or_init(|| Mutex::new(HashMap::new()))
}

fn estimate_bytes(c: &Compilation) -> usize {
    let arts = c.semantics.artifacts.len() * 512;
    let refs = c.semantics.refs.len() * 128;
    let steps = c.semantics.steps.len() * 128;
    let diags = c.diagnostics.len() * 64;
    4096 + arts + refs + steps + diags
}

enum Admit {
    Hit(Arc<Compilation>),
    Wait,
    Lead,
}

fn admit(key: &str) -> Admit {
    let (lock, _cv) = cache();
    let mut g = lock.lock().unwrap();
    if let Some(e) = g.entries.get(key) {
        let arc = Arc::clone(&e.compilation);
        if let Some(pos) = g.order.iter().position(|k| k == key) {
            g.order.remove(pos);
        }
        g.order.push_back(key.to_string());
        HITS.fetch_add(1, Ordering::Relaxed);
        RESULT_HITS.fetch_add(1, Ordering::Relaxed);
        return Admit::Hit(arc);
    }
    if g.inflight.contains_key(key) {
        if let Some(slot) = g.inflight.get_mut(key) {
            slot.waiters += 1;
        }
        MERGES.fetch_add(1, Ordering::Relaxed);
        return Admit::Wait;
    }
    g.inflight.insert(
        key.to_string(),
        InflightSlot {
            waiters: 0,
            result: None,
        },
    );
    MISSES.fetch_add(1, Ordering::Relaxed);
    Admit::Lead
}

fn wait_for(key: &str, deadline: Instant) -> Result<Arc<Compilation>, String> {
    let (lock, cv) = cache();
    let mut g = lock.lock().unwrap();
    loop {
        if let Some(e) = g.entries.get(key) {
            let arc = Arc::clone(&e.compilation);
            if let Some(slot) = g.inflight.get_mut(key) {
                slot.waiters = slot.waiters.saturating_sub(1);
                if slot.waiters == 0 {
                    g.inflight.remove(key);
                }
            }
            HITS.fetch_add(1, Ordering::Relaxed);
            RESULT_HITS.fetch_add(1, Ordering::Relaxed);
            return Ok(arc);
        }
        if let Some(slot) = g.inflight.get(key) {
            if let Some(res) = &slot.result {
                let out = res.clone();
                let waiters = slot.waiters.saturating_sub(1);
                if let Some(slot) = g.inflight.get_mut(key) {
                    slot.waiters = waiters;
                    if slot.waiters == 0 {
                        g.inflight.remove(key);
                    }
                }
                HITS.fetch_add(1, Ordering::Relaxed);
                RESULT_HITS.fetch_add(1, Ordering::Relaxed);
                return out;
            }
        } else {
            // Leader finished without leaving a result — retry admit.
            return Err("analysis.retry".into());
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            if let Some(slot) = g.inflight.get_mut(key) {
                slot.waiters = slot.waiters.saturating_sub(1);
                if slot.waiters == 0 && slot.result.is_none() {
                    g.inflight.remove(key);
                }
            }
            return Err("analysis.wait_deadline".into());
        }
        let (guard, timeout) = cv.wait_timeout(g, remaining).unwrap();
        g = guard;
        if timeout.timed_out() {
            if let Some(slot) = g.inflight.get_mut(key) {
                slot.waiters = slot.waiters.saturating_sub(1);
                if slot.waiters == 0 && slot.result.is_none() {
                    g.inflight.remove(key);
                }
            }
            return Err("analysis.wait_deadline".into());
        }
    }
}

fn finish_leader(key: &str, snap: SnapshotKey, compiled: Result<Arc<Compilation>, String>) {
    if compiled.is_ok() {
        let mut kc = key_compiles().lock().unwrap();
        *kc.entry(key.to_string()).or_insert(0) += 1;
    }
    let (lock, cv) = cache();
    let mut g = lock.lock().unwrap();
    if let Ok(ref arc) = compiled {
        insert_entry(&mut g, snap, Arc::clone(arc));
    }
    if let Some(slot) = g.inflight.get_mut(key) {
        slot.result = Some(compiled);
        if slot.waiters == 0 {
            g.inflight.remove(key);
        }
    }
    cv.notify_all();
}

/// Compile once per SnapshotKey; followers await the same Arc<Compilation>.
pub fn get_or_compile(project_root: &Path) -> Result<Arc<Compilation>, String> {
    let snap = SnapshotKey::compute(project_root)?;
    let key = snap.cache_key();
    let deadline = Instant::now() + Duration::from_secs(300);
    loop {
        match admit(&key) {
            Admit::Hit(arc) => return Ok(arc),
            Admit::Wait => match wait_for(&key, deadline) {
                Err(e) if e == "analysis.retry" => continue,
                other => return other,
            },
            Admit::Lead => {
                let compiled = compile_leader(project_root);
                COMPILE_INVOCATIONS.fetch_add(1, Ordering::Relaxed);
                let out = compiled.clone();
                finish_leader(&key, snap, compiled);
                return out;
            }
        }
    }
}

fn compile_leader(project_root: &Path) -> Result<Arc<Compilation>, String> {
    let options = crate::sema::CompileOptions::for_path(project_root)
        .map_err(|e| format!("view.compile_failed: {e:#}"))?;
    let c = crate::sema::compile_project(&options)
        .map_err(|e| format!("view.compile_failed: {e:#}"))?;
    Ok(Arc::new(c))
}

fn insert_entry(g: &mut CacheState, key: SnapshotKey, compilation: Arc<Compilation>) {
    let ck = key.cache_key();
    let bytes = estimate_bytes(&compilation);
    while g.bytes_used + bytes > g.budget_bytes && !g.order.is_empty() {
        if let Some(old) = g.order.pop_front() {
            if let Some(e) = g.entries.remove(&old) {
                g.bytes_used = g.bytes_used.saturating_sub(e.bytes);
                EVICTIONS.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
    if bytes > g.budget_bytes || g.entries.contains_key(&ck) {
        return;
    }
    g.bytes_used += bytes;
    g.order.push_back(ck.clone());
    g.entries.insert(
        ck,
        CacheEntry {
            key,
            compilation,
            bytes,
            compiled_at: Instant::now(),
        },
    );
}

pub fn project_source_digest(project_root: &Path) -> String {
    SnapshotKey::compute(project_root)
        .map(|k| k.source_digest)
        .unwrap_or_else(|_| "digest-error".into())
}

fn project_key(project_root: &Path) -> String {
    project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf())
        .to_string_lossy()
        .to_string()
}

pub struct AnalysisTicket {
    #[allow(dead_code)]
    key: String,
    #[allow(dead_code)]
    digest: String,
    pub is_leader: bool,
}

pub fn begin_analysis(project_root: &Path) -> AnalysisTicket {
    match get_or_compile(project_root) {
        Ok(_) => AnalysisTicket {
            key: project_key(project_root),
            digest: project_source_digest(project_root),
            is_leader: false,
        },
        Err(_) => AnalysisTicket {
            key: project_key(project_root),
            digest: String::new(),
            is_leader: true,
        },
    }
}

pub fn finish_analysis(_ticket: AnalysisTicket) {}

pub fn note_analysis(project_root: &Path) -> bool {
    // True iff this SnapshotKey was already cached (result hit), independent of
    // other projects compiled concurrently in the same process.
    let before = compile_count_for(project_root);
    let _ = get_or_compile(project_root);
    let after = compile_count_for(project_root);
    after == before && before > 0
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

pub fn compile_invocations() -> u64 {
    COMPILE_INVOCATIONS.load(Ordering::Relaxed)
}

/// Successful compile count for one SnapshotKey (M-R5 / MR-06).
pub fn compile_count_for(project_root: &Path) -> u64 {
    let Ok(snap) = SnapshotKey::compute(project_root) else {
        return 0;
    };
    key_compiles()
        .lock()
        .unwrap()
        .get(&snap.cache_key())
        .copied()
        .unwrap_or(0)
}

pub fn result_hits() -> u64 {
    RESULT_HITS.load(Ordering::Relaxed)
}

pub fn eviction_count() -> u64 {
    EVICTIONS.load(Ordering::Relaxed)
}

pub fn reset_cache_for_tests() {
    let (lock, cv) = cache();
    let mut g = lock.lock().unwrap();
    g.entries.clear();
    g.order.clear();
    g.inflight.clear();
    g.bytes_used = 0;
    HITS.store(0, Ordering::Relaxed);
    MISSES.store(0, Ordering::Relaxed);
    MERGES.store(0, Ordering::Relaxed);
    COMPILE_INVOCATIONS.store(0, Ordering::Relaxed);
    EVICTIONS.store(0, Ordering::Relaxed);
    RESULT_HITS.store(0, Ordering::Relaxed);
    key_compiles().lock().unwrap().clear();
    cv.notify_all();
}

/// Serialize tests that call [`reset_cache_for_tests`] so parallel suites cannot
/// clear another test's SnapshotKey mid-flight. Recovers from poison so one
/// failing cache test does not brick the rest of the suite.
pub fn test_serial_guard() -> std::sync::MutexGuard<'static, ()> {
    static G: OnceLock<Mutex<()>> = OnceLock::new();
    let m = G.get_or_init(|| Mutex::new(()));
    m.lock().unwrap_or_else(|p| p.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn same_snapshot_compiles_once_under_load() {
        let _g = test_serial_guard();
        reset_cache_for_tests();
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/executable_knowledge/baseline/02-extract");
        let n = 32usize;
        let barrier = Arc::new(Barrier::new(n));
        let mut handles = Vec::with_capacity(n);
        for _ in 0..n {
            let barrier = Arc::clone(&barrier);
            let root = root.clone();
            handles.push(thread::spawn(move || {
                barrier.wait();
                get_or_compile(&root).expect("compile")
            }));
        }
        let first = handles.pop().unwrap().join().unwrap();
        for h in handles {
            let arc = h.join().unwrap();
            assert!(Arc::ptr_eq(&first, &arc), "followers must share the same Arc");
        }
        assert_eq!(compile_count_for(&root), 1);
        assert_eq!(compile_invocations(), 1);
    }
}
