//! `# cache` — Redis or in-process memory (design ext-web-drivers).

use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

struct Entry {
    value: String,
    /// Absolute expiry unix secs; `None` = no TTL.
    exp: Option<u64>,
}

struct MemStore {
    data: HashMap<String, Entry>,
}

enum Backend {
    Memory(MemStore),
    /// Redis URL — connections opened per call (simple; enough for W4).
    RedisUrl(String),
}

static HANDLES: Mutex<Option<HashMap<String, Backend>>> = Mutex::new(None);

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn with_map<T>(f: impl FnOnce(&mut HashMap<String, Backend>) -> T) -> T {
    let mut g = HANDLES.lock().unwrap_or_else(|e| e.into_inner());
    let m = g.get_or_insert_with(HashMap::new);
    f(m)
}

fn is_memory(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.is_empty() || u == "memory:" || u == "memory://" || u.starts_with("memory:")
}

fn is_redis(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.starts_with("redis://") || u.starts_with("rediss://")
}

/// Open (or reuse) a cache handle. Returns `{ _type, url }`.
pub fn open(url: &str) -> Result<Value, String> {
    let url = if url.trim().is_empty() {
        "memory:"
    } else {
        url.trim()
    };
    if !is_memory(url) && !is_redis(url) {
        return Err(format!(
            "cache url must be `memory:` or `redis://…`, got `{url}`"
        ));
    }
    with_map(|m| {
        if !m.contains_key(url) {
            let backend = if is_memory(url) {
                Backend::Memory(MemStore {
                    data: HashMap::new(),
                })
            } else {
                Backend::RedisUrl(url.to_string())
            };
            m.insert(url.to_string(), backend);
        }
    });
    Ok(json!({ "_type": "cache", "url": url }))
}

fn prune_mem(store: &mut MemStore) {
    let t = now();
    store.data.retain(|_, e| e.exp.map(|x| x > t).unwrap_or(true));
}

fn with_backend<T>(url: &str, f: impl FnOnce(&mut Backend) -> Result<T, String>) -> Result<T, String> {
    with_map(|m| {
        let b = m
            .get_mut(url)
            .ok_or_else(|| format!("cache `{url}` not opened — call cache.new first"))?;
        f(b)
    })
}

pub fn get(url: &str, key: &str) -> Result<Value, String> {
    with_backend(url, |b| match b {
        Backend::Memory(store) => {
            prune_mem(store);
            match store.data.get(key) {
                Some(e) => Ok(json!({ "ok": true, "value": e.value })),
                None => Ok(json!({ "ok": false })),
            }
        }
        Backend::RedisUrl(u) => redis_get(u, key),
    })
}

pub fn set(url: &str, key: &str, value: &str, ttl_sec: Option<u64>) -> Result<Value, String> {
    with_backend(url, |b| match b {
        Backend::Memory(store) => {
            prune_mem(store);
            let exp = ttl_sec.filter(|t| *t > 0).map(|t| now() + t);
            store.data.insert(
                key.to_string(),
                Entry {
                    value: value.to_string(),
                    exp,
                },
            );
            Ok(json!({ "ok": true }))
        }
        Backend::RedisUrl(u) => redis_set(u, key, value, ttl_sec),
    })
}

pub fn del(url: &str, key: &str) -> Result<Value, String> {
    with_backend(url, |b| match b {
        Backend::Memory(store) => {
            let ok = store.data.remove(key).is_some();
            Ok(json!({ "ok": ok }))
        }
        Backend::RedisUrl(u) => redis_del(u, key),
    })
}

pub fn exists(url: &str, key: &str) -> Result<Value, String> {
    with_backend(url, |b| match b {
        Backend::Memory(store) => {
            prune_mem(store);
            Ok(json!({ "ok": store.data.contains_key(key) }))
        }
        Backend::RedisUrl(u) => redis_exists(u, key),
    })
}

pub fn ttl(url: &str, key: &str) -> Result<Value, String> {
    with_backend(url, |b| match b {
        Backend::Memory(store) => {
            prune_mem(store);
            match store.data.get(key) {
                Some(e) => {
                    let left = match e.exp {
                        Some(x) => x.saturating_sub(now()) as i64,
                        None => -1,
                    };
                    Ok(json!({ "ok": true, "ttl": left }))
                }
                None => Ok(json!({ "ok": false, "ttl": -2 })),
            }
        }
        Backend::RedisUrl(u) => redis_ttl(u, key),
    })
}

fn redis_get(url: &str, key: &str) -> Result<Value, String> {
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    let mut con = client.get_connection().map_err(|e| e.to_string())?;
    let v: Option<String> = redis::cmd("GET")
        .arg(key)
        .query(&mut con)
        .map_err(|e| e.to_string())?;
    match v {
        Some(s) => Ok(json!({ "ok": true, "value": s })),
        None => Ok(json!({ "ok": false })),
    }
}

fn redis_set(url: &str, key: &str, value: &str, ttl_sec: Option<u64>) -> Result<Value, String> {
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    let mut con = client.get_connection().map_err(|e| e.to_string())?;
    match ttl_sec.filter(|t| *t > 0) {
        Some(t) => {
            redis::cmd("SETEX")
                .arg(key)
                .arg(t)
                .arg(value)
                .query::<()>(&mut con)
                .map_err(|e| e.to_string())?;
        }
        None => {
            redis::cmd("SET")
                .arg(key)
                .arg(value)
                .query::<()>(&mut con)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(json!({ "ok": true }))
}

fn redis_del(url: &str, key: &str) -> Result<Value, String> {
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    let mut con = client.get_connection().map_err(|e| e.to_string())?;
    let n: i64 = redis::cmd("DEL")
        .arg(key)
        .query(&mut con)
        .map_err(|e| e.to_string())?;
    Ok(json!({ "ok": n > 0 }))
}

fn redis_exists(url: &str, key: &str) -> Result<Value, String> {
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    let mut con = client.get_connection().map_err(|e| e.to_string())?;
    let n: i64 = redis::cmd("EXISTS")
        .arg(key)
        .query(&mut con)
        .map_err(|e| e.to_string())?;
    Ok(json!({ "ok": n > 0 }))
}

fn redis_ttl(url: &str, key: &str) -> Result<Value, String> {
    let client = redis::Client::open(url).map_err(|e| e.to_string())?;
    let mut con = client.get_connection().map_err(|e| e.to_string())?;
    let t: i64 = redis::cmd("TTL")
        .arg(key)
        .query(&mut con)
        .map_err(|e| e.to_string())?;
    Ok(json!({ "ok": t >= -1, "ttl": t }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_roundtrip() {
        let url = "memory:test-cache";
        open(url).unwrap();
        set(url, "a", "1", Some(60)).unwrap();
        let g = get(url, "a").unwrap();
        assert_eq!(g["ok"], true);
        assert_eq!(g["value"], "1");
        assert_eq!(exists(url, "a").unwrap()["ok"], true);
        del(url, "a").unwrap();
        assert_eq!(get(url, "a").unwrap()["ok"], false);
    }
}
