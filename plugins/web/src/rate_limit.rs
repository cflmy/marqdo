//! Login rate limiting — max failures per IP+username within a window.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_FAILS: u32 = 5;
const WINDOW_SEC: u64 = 900; // 15 minutes

struct Window {
    count: u32,
    start: u64,
}

static ATTEMPTS: Mutex<Option<HashMap<String, Window>>> = Mutex::new(None);

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn key(ip: &str, username: &str) -> String {
    format!("{ip}|{username}")
}

fn with_map<T>(f: impl FnOnce(&mut HashMap<String, Window>) -> T) -> T {
    let mut guard = ATTEMPTS.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    f(map)
}

/// Reset attempt counters (between listens / tests).
pub fn reset() {
    let mut guard = ATTEMPTS.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(HashMap::new());
}

/// Returns `Err(message)` when the client is temporarily locked out.
pub fn check(ip: &str, username: &str) -> Result<(), String> {
    let k = key(ip, username);
    let now = now_secs();
    with_map(|map| {
        if let Some(w) = map.get(&k) {
            if now.saturating_sub(w.start) < WINDOW_SEC && w.count >= MAX_FAILS {
                return Err(
                    "Too many failed login attempts. Try again in about 15 minutes.".into(),
                );
            }
            if now.saturating_sub(w.start) >= WINDOW_SEC {
                map.remove(&k);
            }
        }
        Ok(())
    })
}

pub fn record_failure(ip: &str, username: &str) {
    let k = key(ip, username);
    let now = now_secs();
    with_map(|map| {
        let entry = map.entry(k).or_insert(Window { count: 0, start: now });
        if now.saturating_sub(entry.start) >= WINDOW_SEC {
            entry.count = 0;
            entry.start = now;
        }
        entry.count = entry.count.saturating_add(1);
    });
}

pub fn clear_success(ip: &str, username: &str) {
    let k = key(ip, username);
    with_map(|map| {
        map.remove(&k);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locks_after_max_fails() {
        reset();
        let ip = "127.0.0.1";
        let user = "admin";
        for _ in 0..MAX_FAILS {
            record_failure(ip, user);
        }
        assert!(check(ip, user).is_err());
        clear_success(ip, user);
        assert!(check(ip, user).is_ok());
    }
}
