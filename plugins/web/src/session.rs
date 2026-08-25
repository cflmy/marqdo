//! In-memory session store + cookie helpers (design ext-web-net §3.1 / §3.2).
//!
//! Sessions live for the lifetime of the plugin process (same as `web_listen`).
//! Session ids are random hex strings; cookies are `marqdo_sid=<id>` with
//! `HttpOnly; Path=/; SameSite=Lax`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Map, Value};

type SessionMap = HashMap<String, Value>;
struct Store {
    ttl_sec: u64,
    data: HashMap<String, (u64, SessionMap)>,
}

static STORE: Mutex<Option<Store>> = Mutex::new(None);

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn with_store<T>(f: impl FnOnce(&mut Store) -> T) -> T {
    let mut guard = STORE.lock().unwrap_or_else(|e| e.into_inner());
    let store = guard.get_or_insert_with(|| Store {
        ttl_sec: 3600,
        data: HashMap::new(),
    });
    f(store)
}

fn prune(store: &mut Store) {
    let now = now_secs();
    store.data.retain(|_, (exp, _)| *exp > now);
}

fn random_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    // Cheap but adequate unique-enough id for an in-memory session store.
    format!("{pid:x}{nanos:x}{:x}", rand_word())
}

fn rand_word() -> u64 {
    // xorshift64 from a counter; not cryptographically strong but fine for
    // session cookies when combined with pid+time. (Plugin is single-process.)
    static COUNTER: Mutex<u64> = Mutex::new(0x9e3779b97f4a7c15);
    let mut c = COUNTER.lock().unwrap_or_else(|e| e.into_inner());
    let mut x = *c;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *c = x;
    x
}

/// Reset the store (used between listens / for tests).
pub fn reset(ttl_sec: u64) {
    let mut guard = STORE.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(Store {
        ttl_sec,
        data: HashMap::new(),
    });
}

pub fn session_new(ttl_sec: Option<u64>) -> String {
    with_store(|store| {
        if let Some(ttl) = ttl_sec {
            if ttl > 0 {
                store.ttl_sec = ttl;
            }
        }
        prune(store);
        let id = random_id();
        let exp = now_secs() + store.ttl_sec;
        store.data.insert(id.clone(), (exp, SessionMap::new()));
        id
    })
}

pub fn session_set(id: &str, key: &str, value: Value) -> bool {
    with_store(|store| {
        prune(store);
        let Some((exp, m)) = store.data.get_mut(id) else {
            return false;
        };
        if *exp <= now_secs() {
            return false;
        }
        m.insert(key.to_string(), value);
        true
    })
}

pub fn session_get(id: &str, key: &str) -> Option<Value> {
    with_store(|store| {
        prune(store);
        let (exp, m) = store.data.get(id)?;
        if *exp <= now_secs() {
            return None;
        }
        m.get(key).cloned()
    })
}

pub fn session_del(id: &str, key: &str) -> bool {
    with_store(|store| {
        prune(store);
        let Some((exp, m)) = store.data.get_mut(id) else {
            return false;
        };
        if *exp <= now_secs() {
            return false;
        }
        m.remove(key).is_some()
    })
}

pub fn session_destroy(id: &str) -> bool {
    with_store(|store| {
        prune(store);
        store.data.remove(id).is_some()
    })
}

/// Parse the session id from a `Cookie` request header (looks for `marqdo_sid=…`).
pub fn session_id_from_cookie(cookie_header: Option<&str>) -> Option<String> {
    let header = cookie_header?;
    for part in header.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("marqdo_sid=") {
            let v = v.trim_matches('"');
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

/// `Set-Cookie: marqdo_sid=<id>; HttpOnly; Path=/; SameSite=Lax; Max-Age=<ttl>`
pub fn session_cookie(id: &str, ttl_sec: u64) -> String {
    format!(
        "marqdo_sid={id}; HttpOnly; Path=/; SameSite=Lax; Max-Age={ttl_sec}"
    )
}

/// Validate `username`/`password` against an admin-users table (`|用户名|密码|`
/// with headers `username`/`密码` or `用户`/`密码`, or plain `username`/`password`).
/// Returns the matched username on success.
pub fn check_credentials(users: &Value, username: &str, password: &str) -> Option<String> {
    let rows = match users {
        Value::Array(a) => a,
        _ => return None,
    };
    for row in rows {
        let m = match row {
            Value::Object(m) => m,
            _ => continue,
        };
        let u = pick_cell(m, &["用户名", "用户", "username", "user", "账号"])
            .map(cell_str)
            .unwrap_or_default();
        let p = pick_cell(m, &["密码", "password", "pass", "口令"])
            .map(cell_str)
            .unwrap_or_default();
        if u == username && p == password {
            return Some(u);
        }
    }
    None
}

fn pick_cell<'a>(m: &'a Map<String, Value>, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|k| m.get(*k))
}

fn cell_str(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// ABI helpers.
pub fn abi_session_new(args: &Value) -> Result<Value, String> {
    let ttl = args
        .get("ttl_sec")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)));
    let id = session_new(ttl);
    Ok(json!({ "id": id }))
}

pub fn abi_session_set(args: &Value) -> Result<Value, String> {
    let id = arg_str(args, "id")?;
    let key = arg_str(args, "key")?;
    let value = args.get("value").cloned().unwrap_or(Value::Null);
    Ok(json!({ "ok": session_set(id, key, value) }))
}

pub fn abi_session_get(args: &Value) -> Result<Value, String> {
    let id = arg_str(args, "id")?;
    let key = arg_str(args, "key")?;
    match session_get(id, key) {
        Some(v) => Ok(json!({ "ok": true, "value": v })),
        None => Ok(json!({ "ok": false })),
    }
}

pub fn abi_session_del(args: &Value) -> Result<Value, String> {
    let id = arg_str(args, "id")?;
    let key = arg_str(args, "key")?;
    Ok(json!({ "ok": session_del(id, key) }))
}

pub fn abi_session_destroy(args: &Value) -> Result<Value, String> {
    let id = arg_str(args, "id")?;
    Ok(json!({ "ok": session_destroy(id) }))
}

pub fn abi_auth_login(args: &Value) -> Result<Value, String> {
    let username = arg_str(args, "username")?;
    let password = arg_str(args, "password")?;
    let users = args.get("users").cloned().unwrap_or(Value::Null);
    let ttl = args
        .get("session_ttl")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)));
    match check_credentials(&users, username, password) {
        Some(_) => {
            let id = session_new(ttl);
            session_set(&id, "username", json!(username));
            Ok(json!({ "ok": true, "session_id": id, "username": username }))
        }
        None => Ok(json!({ "ok": false })),
    }
}

pub fn abi_auth_check(args: &Value) -> Result<Value, String> {
    let id = arg_str(args, "session_id")?;
    match session_get(id, "username") {
        Some(Value::String(u)) => Ok(json!({ "ok": true, "username": u })),
        _ => Ok(json!({ "ok": false })),
    }
}

pub fn abi_auth_logout(args: &Value) -> Result<Value, String> {
    let id = arg_str(args, "session_id")?;
    Ok(json!({ "ok": session_destroy(id) }))
}

fn arg_str<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .ok_or_else(|| format!("missing `{key}`"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn users() -> Value {
        json!([
            { "username": "admin", "password": "secret" },
            { "用户": "站长", "密码": "pw123" }
        ])
    }

    #[test]
    fn session_set_get_del() {
        reset(3600);
        let id = session_new(None);
        assert!(!id.is_empty());
        assert!(session_set(&id, "username", json!("admin")));
        assert_eq!(
            session_get(&id, "username"),
            Some(json!("admin"))
        );
        assert!(session_del(&id, "username"));
        assert_eq!(session_get(&id, "username"), None);
        assert!(session_destroy(&id));
    }

    #[test]
    fn auth_login_check_logout() {
        reset(3600);
        let r = abi_auth_login(&json!({
            "username": "admin",
            "password": "secret",
            "users": users(),
        }))
        .unwrap();
        assert_eq!(r["ok"].as_bool(), Some(true));
        let sid = r["session_id"].as_str().unwrap().to_string();
        let check = abi_auth_check(&json!({ "session_id": sid })).unwrap();
        assert_eq!(check["ok"].as_bool(), Some(true));
        assert_eq!(check["username"], "admin");

        let bad = abi_auth_login(&json!({
            "username": "admin",
            "password": "wrong",
            "users": users(),
        }))
        .unwrap();
        assert_eq!(bad["ok"].as_bool(), Some(false));

        // Chinese column headers also work.
        let zh = abi_auth_login(&json!({
            "username": "站长",
            "password": "pw123",
            "users": users(),
        }))
        .unwrap();
        assert_eq!(zh["ok"].as_bool(), Some(true));

        let out = abi_auth_logout(&json!({ "session_id": sid })).unwrap();
        assert_eq!(out["ok"].as_bool(), Some(true));
        let check2 = abi_auth_check(&json!({ "session_id": sid })).unwrap();
        assert_eq!(check2["ok"].as_bool(), Some(false));
    }

    #[test]
    fn cookie_roundtrip() {
        reset(3600);
        let id = session_new(None);
        let cookie = session_cookie(&id, 3600);
        assert!(cookie.starts_with("marqdo_sid="));
        let got = session_id_from_cookie(Some(&format!(
            "theme=light; {cookie}"
        )));
        assert_eq!(got.as_deref(), Some(id.as_str()));
        assert_eq!(session_id_from_cookie(Some("theme=light")), None);
        assert_eq!(session_id_from_cookie(None), None);
    }

    #[test]
    fn check_credentials_missing() {
        assert!(check_credentials(&json!(null), "a", "b").is_none());
        assert!(check_credentials(&json!("x"), "a", "b").is_none());
        assert!(check_credentials(&json!([]), "a", "b").is_none());
    }
}
