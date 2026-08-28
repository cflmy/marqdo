//! Session store + cookie helpers + CSRF tokens (ext-web-net W3).
//!
//! - **CSPRNG** session ids (`getrandom`).
//! - **SQLite persistence** when `configure` receives a `db_url`; otherwise in-memory (offline gold).
//! - **Sliding expiry** on read/write.
//! - **CSRF** token per session (`_csrf` key).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use getrandom::getrandom;
use serde_json::{json, Map, Value};

use crate::db;
use crate::password;

type SessionMap = HashMap<String, Value>;

#[derive(Clone, Default)]
pub struct Config {
    pub db_url: Option<String>,
    pub ttl_sec: u64,
    pub cookie_secure: bool,
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

struct MemStore {
    ttl_sec: u64,
    data: HashMap<String, (u64, SessionMap)>,
}

static MEM: Mutex<Option<MemStore>> = Mutex::new(None);

const SESSION_TABLE: &str = "_marqdo_sessions";
const CSRF_KEY: &str = "_csrf";

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn cfg() -> Config {
    CONFIG
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or(Config {
            db_url: None,
            ttl_sec: 3600,
            cookie_secure: false,
        })
}

/// Configure session backend before `listen` or offline ABI calls.
pub fn configure(config: Config) {
    let ttl = if config.ttl_sec > 0 {
        config.ttl_sec
    } else {
        3600
    };
    let mut c = config;
    c.ttl_sec = ttl;
    if let Some(ref url) = c.db_url {
        let _ = ensure_session_table(url);
    }
    let mut guard = CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(c);
}

/// Reset in-memory sessions (tests / each listen).
pub fn reset(ttl_sec: u64) {
    let ttl = if ttl_sec > 0 { ttl_sec } else { 3600 };
    {
        let mut guard = CONFIG.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(ref mut c) = *guard {
            c.ttl_sec = ttl;
        } else {
            *guard = Some(Config {
                db_url: None,
                ttl_sec: ttl,
                cookie_secure: false,
            });
        }
    }
    let mut mem = MEM.lock().unwrap_or_else(|e| e.into_inner());
    *mem = Some(MemStore {
        ttl_sec: ttl,
        data: HashMap::new(),
    });
}

fn ensure_session_table(url: &str) -> Result<(), String> {
    db::exec(
        url,
        &format!(
            "CREATE TABLE IF NOT EXISTS \"{SESSION_TABLE}\" (
                id TEXT PRIMARY KEY,
                expires_at INTEGER NOT NULL,
                data TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_marqdo_sessions_exp ON \"{SESSION_TABLE}\"(expires_at);"
        ),
        None,
        None,
    )?;
    Ok(())
}

fn prune_mem(store: &mut MemStore) {
    let now = now_secs();
    store.data.retain(|_, (exp, _)| *exp > now);
}

fn prune_sql(url: &str) {
    let now = now_secs() as i64;
    let _ = db::exec(
        url,
        &format!("DELETE FROM \"{SESSION_TABLE}\" WHERE expires_at <= ?1"),
        Some(&json!([now])),
        None,
    );
}

fn secure_random_id() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom(&mut bytes).map_err(|e| format!("getrandom: {e}"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn random_csrf() -> Result<String, String> {
    let mut bytes = [0u8; 24];
    getrandom(&mut bytes).map_err(|e| format!("getrandom: {e}"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

fn map_to_json(m: &SessionMap) -> Result<String, String> {
    let obj: Map<String, Value> = m.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    serde_json::to_string(&Value::Object(obj)).map_err(|e| e.to_string())
}

fn json_to_map(s: &str) -> SessionMap {
    match serde_json::from_str::<Value>(s) {
        Ok(Value::Object(m)) => m.into_iter().collect(),
        _ => HashMap::new(),
    }
}

fn load_sql(url: &str, id: &str) -> Option<(u64, SessionMap)> {
    let q = db::query(
        url,
        &format!("SELECT expires_at, data FROM \"{SESSION_TABLE}\" WHERE id = ?1"),
        Some(&json!([id])),
        None,
    )
    .ok()?;
    let rows = q.get("rows")?.as_array()?;
    let row = rows.first()?;
    let exp = row.get("expires_at")?.as_i64()? as u64;
    let data_s = row.get("data")?.as_str()?;
    Some((exp, json_to_map(data_s)))
}

fn save_sql(url: &str, id: &str, exp: u64, data: &SessionMap) -> Result<(), String> {
    let data_s = map_to_json(data)?;
    db::exec(
        url,
        &format!(
            "INSERT INTO \"{SESSION_TABLE}\" (id, expires_at, data) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET expires_at = excluded.expires_at, data = excluded.data"
        ),
        Some(&json!([id, exp as i64, data_s])),
        None,
    )?;
    Ok(())
}

fn delete_sql(url: &str, id: &str) {
    let _ = db::exec(
        url,
        &format!("DELETE FROM \"{SESSION_TABLE}\" WHERE id = ?1"),
        Some(&json!([id])),
        None,
    );
}

fn touch_expiry(ttl: u64) -> u64 {
    now_secs() + ttl
}

fn ensure_csrf(data: &mut SessionMap) -> Result<String, String> {
    if let Some(Value::String(t)) = data.get(CSRF_KEY) {
        if !t.is_empty() {
            return Ok(t.clone());
        }
    }
    let t = random_csrf()?;
    data.insert(CSRF_KEY.to_string(), json!(t));
    Ok(t)
}

pub fn session_new(ttl_sec: Option<u64>) -> String {
    let cfg = cfg();
    let ttl = ttl_sec.unwrap_or(cfg.ttl_sec);
    if ttl > 0 {
        if let Ok(mut guard) = CONFIG.lock() {
            if let Some(ref mut c) = *guard {
                c.ttl_sec = ttl;
            }
        }
    }
    let id = secure_random_id().unwrap_or_else(|_| format!("fallback-{}", now_secs()));
    let exp = touch_expiry(ttl);
    let mut data = SessionMap::new();
    let _ = ensure_csrf(&mut data);
    if let Some(ref url) = cfg.db_url {
        prune_sql(url);
        let _ = save_sql(url, &id, exp, &data);
    } else {
        with_mem(|store| {
            store.ttl_sec = ttl;
            prune_mem(store);
            store.data.insert(id.clone(), (exp, data));
        });
    }
    id
}

fn with_mem<T>(f: impl FnOnce(&mut MemStore) -> T) -> T {
    let mut guard = MEM.lock().unwrap_or_else(|e| e.into_inner());
    let store = guard.get_or_insert_with(|| MemStore {
        ttl_sec: cfg().ttl_sec,
        data: HashMap::new(),
    });
    f(store)
}

fn get_record(id: &str) -> Option<(u64, SessionMap)> {
    let cfg = cfg();
    let now = now_secs();
    if let Some(ref url) = cfg.db_url {
        prune_sql(url);
        let (exp, data) = load_sql(url, id)?;
        if exp <= now {
            delete_sql(url, id);
            return None;
        }
        Some((exp, data))
    } else {
        with_mem(|store| {
            prune_mem(store);
            let (exp, data) = store.data.get(id)?.clone();
            if exp <= now {
                store.data.remove(id);
                return None;
            }
            Some((exp, data))
        })
    }
}

fn put_record(id: &str, exp: u64, data: SessionMap) -> bool {
    let cfg = cfg();
    if let Some(ref url) = cfg.db_url {
        save_sql(url, id, exp, &data).is_ok()
    } else {
        with_mem(|store| {
            store.data.insert(id.to_string(), (exp, data));
            true
        })
    }
}

pub fn session_set(id: &str, key: &str, value: Value) -> bool {
    let cfg = cfg();
    let Some((_, mut data)) = get_record(id) else {
        return false;
    };
    data.insert(key.to_string(), value);
    put_record(id, touch_expiry(cfg.ttl_sec), data)
}

pub fn session_get(id: &str, key: &str) -> Option<Value> {
    let cfg = cfg();
    let (_, data) = get_record(id)?;
    let v = data.get(key).cloned()?;
    put_record(id, touch_expiry(cfg.ttl_sec), data);
    Some(v)
}

pub fn session_del(id: &str, key: &str) -> bool {
    let cfg = cfg();
    let Some((_, mut data)) = get_record(id) else {
        return false;
    };
    let ok = data.remove(key).is_some();
    if ok {
        put_record(id, touch_expiry(cfg.ttl_sec), data);
    }
    ok
}

pub fn session_destroy(id: &str) -> bool {
    let cfg = cfg();
    if let Some(ref url) = cfg.db_url {
        delete_sql(url, id);
        true
    } else {
        with_mem(|store| store.data.remove(id).is_some())
    }
}

/// CSRF token for an existing session (creates one if missing).
pub fn csrf_for(id: &str) -> Option<String> {
    let cfg = cfg();
    let (_, mut data) = get_record(id)?;
    let token = ensure_csrf(&mut data).ok()?;
    put_record(id, touch_expiry(cfg.ttl_sec), data);
    Some(token)
}

pub fn validate_csrf(id: &str, token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    match session_get(id, CSRF_KEY) {
        Some(Value::String(stored)) => stored == token,
        _ => false,
    }
}

/// Resolve session from cookie: reuse valid id or create a new one.
/// Returns `(session_id, set_cookie_header)`.
pub fn ensure_from_cookie(cookie_header: Option<&str>) -> (String, Option<String>) {
    if let Some(id) = session_id_from_cookie(cookie_header) {
        if get_record(&id).is_some() {
            let _ = csrf_for(&id);
            return (id, None);
        }
    }
    let id = session_new(None);
    let cfg = cfg();
    (
        id.clone(),
        Some(session_cookie(&id, cfg.ttl_sec, cfg.cookie_secure)),
    )
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

/// `Set-Cookie: marqdo_sid=<id>; HttpOnly; Path=/; SameSite=Lax; Max-Age=<ttl>[; Secure]`
pub fn session_cookie(id: &str, ttl_sec: u64, secure: bool) -> String {
    let mut s = format!(
        "marqdo_sid={id}; HttpOnly; Path=/; SameSite=Lax; Max-Age={ttl_sec}"
    );
    if secure {
        s.push_str("; Secure");
    }
    s
}

/// Validate credentials; supports argon2 hashes or legacy plaintext in the password column.
/// Returns `(username, role)`. Missing role column defaults to `admin` (legacy admin tables).
pub fn check_credentials(users: &Value, username: &str, password: &str) -> Option<(String, String)> {
    let rows = crate::table::as_rows(users);
    let rows = match rows {
        Value::Array(a) => a,
        _ => return None,
    };
    for row in &rows {
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
        if u == username && password::verify_password(password, &p) {
            let role = pick_cell(m, &["角色", "role", "roles"])
                .map(cell_str)
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "admin".into());
            return Some((u, role.to_ascii_lowercase()));
        }
    }
    None
}

/// Session role (defaults to `visitor` when anonymous / missing).
pub fn session_role(cookie_header: Option<&str>) -> String {
    let Some(sid) = session_id_from_cookie(cookie_header) else {
        return "visitor".into();
    };
    match session_get(&sid, "role") {
        Some(Value::String(r)) if !r.is_empty() => r.to_ascii_lowercase(),
        _ => {
            if session_get(&sid, "username").is_some() {
                "admin".into()
            } else {
                "visitor".into()
            }
        }
    }
}

pub fn role_allowed(role: &str, allowed: &[String]) -> bool {
    if allowed.is_empty() {
        return true;
    }
    let role = role.to_ascii_lowercase();
    allowed.iter().any(|a| a.eq_ignore_ascii_case(&role))
}

pub fn parse_roles_csv(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
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
        Some((u, role)) => {
            let id = session_new(ttl);
            session_set(&id, "username", json!(u));
            session_set(&id, "role", json!(role));
            Ok(json!({ "ok": true, "session_id": id, "username": u, "role": role }))
        }
        None => Ok(json!({ "ok": false })),
    }
}

pub fn abi_auth_check(args: &Value) -> Result<Value, String> {
    let id = arg_str(args, "session_id")?;
    match session_get(id, "username") {
        Some(Value::String(u)) => {
            let role = match session_get(id, "role") {
                Some(Value::String(r)) => r,
                _ => "admin".into(),
            };
            Ok(json!({ "ok": true, "username": u, "role": role }))
        }
        _ => Ok(json!({ "ok": false })),
    }
}

pub fn abi_auth_logout(args: &Value) -> Result<Value, String> {
    let id = arg_str(args, "session_id")?;
    Ok(json!({ "ok": session_destroy(id) }))
}

pub fn abi_password_hash(args: &Value) -> Result<Value, String> {
    let plain = arg_str(args, "password")?;
    let hash = password::hash_password(plain)?;
    Ok(json!({ "hash": hash }))
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
        assert_eq!(session_get(&id, "username"), Some(json!("admin")));
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

        let bad = abi_auth_login(&json!({
            "username": "admin",
            "password": "wrong",
            "users": users(),
        }))
        .unwrap();
        assert_eq!(bad["ok"].as_bool(), Some(false));

        let out = abi_auth_logout(&json!({ "session_id": sid })).unwrap();
        assert_eq!(out["ok"].as_bool(), Some(true));
    }

    #[test]
    fn csrf_roundtrip() {
        reset(3600);
        let id = session_new(None);
        let t = csrf_for(&id).unwrap();
        assert!(validate_csrf(&id, &t));
        assert!(!validate_csrf(&id, "bad"));
    }

    #[test]
    fn cookie_roundtrip() {
        reset(3600);
        let id = session_new(None);
        let cookie = session_cookie(&id, 3600, false);
        assert!(cookie.starts_with("marqdo_sid="));
        let got = session_id_from_cookie(Some(&format!("theme=light; {cookie}")));
        assert_eq!(got.as_deref(), Some(id.as_str()));
    }

    #[test]
    fn hashed_password_login() {
        reset(3600);
        let hash = password::hash_password("secret").unwrap();
        let users = json!([{ "username": "admin", "password": hash }]);
        let r = abi_auth_login(&json!({
            "username": "admin",
            "password": "secret",
            "users": users,
        }))
        .unwrap();
        assert_eq!(r["ok"].as_bool(), Some(true));
    }

    #[test]
    fn session_persists_in_sqlite() {
        let dir = std::env::temp_dir().join(format!("marqdo_web_sess_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("persist.db");
        let _ = std::fs::remove_file(&path);
        let url = format!("sqlite:{}", path.display());
        configure(Config {
            db_url: Some(url.clone()),
            ttl_sec: 3600,
            cookie_secure: false,
        });
        let id = session_new(None);
        assert!(session_set(&id, "theme", json!("dark")));
        // Re-configure (simulates listen restart with same db).
        configure(Config {
            db_url: Some(url),
            ttl_sec: 3600,
            cookie_secure: false,
        });
        assert_eq!(session_get(&id, "theme"), Some(json!("dark")));
        let _ = std::fs::remove_dir_all(dir);
    }
}
