//! Agent plugin (C ABI v2): layout helpers + session bag + context via host_query + agent-kb.

mod kb;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Component, Path, PathBuf};
use std::ptr;
use std::sync::{LazyLock, Mutex};

const ABI_VERSION: u32 = 2;
const DEFAULT_MARKERS: &str = "agents,runbooks,marqdo.agent.json";
const LAYOUT_DIRS: &[&str] = &["agents", "runbooks", "templates", "reports"];

type PluginFn = unsafe extern "C" fn(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int;

type HostQueryFn = unsafe extern "C" fn(
    userdata: *mut c_void,
    name: *const c_char,
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int;

#[repr(C)]
pub struct MarqdoHostApi {
    pub userdata: *mut c_void,
    pub register_fn: Option<
        unsafe extern "C" fn(
            userdata: *mut c_void,
            name: *const c_char,
            params: *const c_char,
            fn_ptr: PluginFn,
        ) -> c_int,
    >,
    pub alloc: Option<unsafe extern "C" fn(n: usize) -> *mut c_void>,
    pub free: Option<unsafe extern "C" fn(p: *mut c_void)>,
    pub host_query: Option<HostQueryFn>,
}

static mut HOST_FREE: Option<unsafe extern "C" fn(*mut c_void)> = None;
static mut HOST_ALLOC: Option<unsafe extern "C" fn(usize) -> *mut c_void> = None;
static mut HOST_QUERY: Option<HostQueryFn> = None;
static mut HOST_USERDATA: *mut c_void = ptr::null_mut();

static HISTORIES: LazyLock<Mutex<HashMap<String, Vec<serde_json::Value>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static AGENT_SEQ: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

unsafe fn host_strdup(s: &str) -> *mut c_char {
    let alloc = HOST_ALLOC.expect("host alloc");
    let bytes = s.as_bytes();
    let p = alloc(bytes.len() + 1) as *mut u8;
    if p.is_null() {
        return ptr::null_mut();
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
    *p.add(bytes.len()) = 0;
    p as *mut c_char
}

fn set_out(out: *mut *mut c_char, s: &str) {
    if out.is_null() {
        return;
    }
    unsafe {
        *out = host_strdup(s);
    }
}

fn set_err(err: *mut *mut c_char, s: &str) {
    if err.is_null() {
        return;
    }
    unsafe {
        *err = host_strdup(s);
    }
}

fn parse_args(args_json: *const c_char) -> Result<serde_json::Value, String> {
    if args_json.is_null() {
        return Err("null args".into());
    }
    let s = unsafe { CStr::from_ptr(args_json) }
        .to_str()
        .map_err(|_| "args not utf-8".to_string())?;
    serde_json::from_str(s).map_err(|e| e.to_string())
}

fn arg_str<'a>(v: &'a serde_json::Value, key: &str) -> Result<&'a str, String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .ok_or_else(|| format!("missing text `{key}`"))
}

fn has_parent_escape(rel: &Path) -> bool {
    rel.components().any(|c| matches!(c, Component::ParentDir))
}

fn find_root(start: &str, markers: &str) -> Result<PathBuf, String> {
    let markers: Vec<&str> = markers
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if markers.is_empty() {
        return Err("empty markers".into());
    }
    let mut cur = PathBuf::from(start);
    if cur.as_os_str().is_empty() {
        cur = std::env::current_dir().map_err(|e| e.to_string())?;
    }
    if !cur.exists() {
        return Err(format!("start path missing: {}", cur.display()));
    }
    let mut cur = std::fs::canonicalize(&cur).map_err(|e| e.to_string())?;
    loop {
        for m in &markers {
            let p = cur.join(m);
            let ok = if m.contains('.') {
                p.is_file()
            } else {
                p.is_dir()
            };
            if ok {
                return Ok(cur);
            }
        }
        if !cur.pop() {
            break;
        }
    }
    Err("agent project root not found".into())
}

fn ensure_layout(root: &Path) -> Result<i64, String> {
    let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    let mut created = 0i64;
    for d in LAYOUT_DIRS {
        let p = root.join(d);
        if !p.exists() {
            std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
            created += 1;
        } else if !p.is_dir() {
            return Err(format!("{} exists and is not a directory", p.display()));
        }
    }
    Ok(created)
}

fn probe(root: &Path) -> Result<serde_json::Value, String> {
    let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    let mut map = serde_json::Map::new();
    map.insert(
        "root".into(),
        serde_json::Value::String(root.to_string_lossy().into_owned()),
    );
    for d in LAYOUT_DIRS {
        let key = format!("has_{d}");
        map.insert(key, serde_json::Value::Bool(root.join(d).is_dir()));
    }
    Ok(serde_json::Value::Object(map))
}

fn scaffold(root: &Path, name: &str, template: &str, dest: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("name is empty".into());
    }
    let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
    let tmpl_rel = Path::new(template);
    let dest_rel = Path::new(dest);
    if has_parent_escape(tmpl_rel) {
        return Err("template path escapes project root".into());
    }
    if has_parent_escape(dest_rel) {
        return Err("dest path escapes project root".into());
    }
    let tmpl_path = root.join(tmpl_rel);
    if !tmpl_path.is_file() {
        return Err(format!("template not found: {}", tmpl_path.display()));
    }
    let tmpl_can = std::fs::canonicalize(&tmpl_path).map_err(|e| e.to_string())?;
    if !tmpl_can.starts_with(&root) {
        return Err("template path escapes project root".into());
    }
    let dest_path = root.join(dest_rel);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        let parent_can = std::fs::canonicalize(parent).map_err(|e| e.to_string())?;
        if !parent_can.starts_with(&root) {
            return Err("dest path escapes project root".into());
        }
    }
    let body = std::fs::read_to_string(&tmpl_can).map_err(|e| e.to_string())?;
    let body = body.replace("{{name}}", name);
    std::fs::write(&dest_path, body).map_err(|e| e.to_string())?;
    Ok(dest_path.to_string_lossy().into_owned())
}

fn map_get_str<'a>(m: &'a serde_json::Value, keys: &[&str]) -> Option<&'a str> {
    let obj = m.as_object()?;
    for k in keys {
        if let Some(v) = obj.get(*k) {
            if let Some(s) = v.as_str() {
                return Some(s);
            }
            if let Some(n) = v.as_i64() {
                // allow numeric fields to be read as display later
                let _ = n;
            }
        }
    }
    None
}

fn map_get_i64(m: &serde_json::Value, keys: &[&str]) -> i64 {
    let Some(obj) = m.as_object() else {
        return i64::MAX;
    };
    for k in keys {
        if let Some(v) = obj.get(*k) {
            if let Some(n) = v.as_i64() {
                return n;
            }
            if let Some(s) = v.as_str() {
                if let Ok(n) = s.parse::<i64>() {
                    return n;
                }
            }
        }
    }
    i64::MAX
}

fn skills_text(m: &serde_json::Value) -> String {
    map_get_str(m, &["技能", "skills", "skill"])
        .unwrap_or("")
        .to_string()
}

fn match_skill(skill: &str, members: &serde_json::Value) -> Result<serde_json::Value, String> {
    let arr = members
        .as_array()
        .ok_or_else(|| "members must be a JSON array".to_string())?;
    let skill = skill.trim();
    if skill.is_empty() {
        return Err("skill is empty".into());
    }
    let mut best: Option<&serde_json::Value> = None;
    let mut best_load = i64::MAX;
    for m in arr {
        let sk = skills_text(m);
        if sk.split(',').any(|p| p.trim() == skill) || sk.contains(skill) {
            let load = map_get_i64(m, &["负载", "load"]);
            if load < best_load {
                best_load = load;
                best = Some(m);
            }
        }
    }
    Ok(best.cloned().unwrap_or(serde_json::Value::Null))
}

fn bump_load(member: &serde_json::Value, delta: i64) -> Result<serde_json::Value, String> {
    let mut obj = member
        .as_object()
        .cloned()
        .ok_or_else(|| "member must be a JSON object".to_string())?;
    let key = if obj.contains_key("负载") {
        "负载"
    } else if obj.contains_key("load") {
        "load"
    } else {
        obj.insert("负载".into(), serde_json::Value::Number(0.into()));
        "负载"
    };
    let cur = match obj.get(key) {
        Some(serde_json::Value::Number(n)) => n.as_i64().unwrap_or(0),
        Some(serde_json::Value::String(s)) => s.parse().unwrap_or(0),
        _ => 0,
    };
    obj.insert(
        key.into(),
        serde_json::Value::Number((cur + delta).into()),
    );
    Ok(serde_json::Value::Object(obj))
}

unsafe extern "C" fn agent_find_root(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_find_root: {e}"));
            return 1;
        }
    };
    let start = match arg_str(&v, "start") {
        Ok(s) => s,
        Err(e) => {
            set_err(err_msg, &format!("agent_find_root: {e}"));
            return 1;
        }
    };
    let markers = v
        .get("markers")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_MARKERS);
    match find_root(start, markers) {
        Ok(p) => {
            let out = serde_json::Value::String(p.to_string_lossy().into_owned()).to_string();
            set_out(out_json, &out);
            0
        }
        Err(e) => {
            set_err(err_msg, &format!("agent_find_root: {e}"));
            1
        }
    }
}

unsafe extern "C" fn agent_ensure_layout(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_ensure_layout: {e}"));
            return 1;
        }
    };
    let root = match arg_str(&v, "root") {
        Ok(s) => s,
        Err(e) => {
            set_err(err_msg, &format!("agent_ensure_layout: {e}"));
            return 1;
        }
    };
    match ensure_layout(Path::new(root)) {
        Ok(n) => {
            set_out(out_json, &n.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &format!("agent_ensure_layout: {e}"));
            1
        }
    }
}

unsafe extern "C" fn agent_probe(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_probe: {e}"));
            return 1;
        }
    };
    let root = match arg_str(&v, "root") {
        Ok(s) => s,
        Err(e) => {
            set_err(err_msg, &format!("agent_probe: {e}"));
            return 1;
        }
    };
    match probe(Path::new(root)) {
        Ok(j) => {
            set_out(out_json, &j.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &format!("agent_probe: {e}"));
            1
        }
    }
}

unsafe extern "C" fn agent_scaffold(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_scaffold: {e}"));
            return 1;
        }
    };
    let root = match arg_str(&v, "root") {
        Ok(s) => s,
        Err(e) => {
            set_err(err_msg, &format!("agent_scaffold: {e}"));
            return 1;
        }
    };
    let name = match arg_str(&v, "name") {
        Ok(s) => s,
        Err(e) => {
            set_err(err_msg, &format!("agent_scaffold: {e}"));
            return 1;
        }
    };
    let template = match arg_str(&v, "template") {
        Ok(s) => s,
        Err(e) => {
            set_err(err_msg, &format!("agent_scaffold: {e}"));
            return 1;
        }
    };
    let dest = match arg_str(&v, "dest") {
        Ok(s) => s,
        Err(e) => {
            set_err(err_msg, &format!("agent_scaffold: {e}"));
            return 1;
        }
    };
    match scaffold(Path::new(root), name, template, dest) {
        Ok(p) => {
            let out = serde_json::Value::String(p).to_string();
            set_out(out_json, &out);
            0
        }
        Err(e) => {
            set_err(err_msg, &format!("agent_scaffold: {e}"));
            1
        }
    }
}

unsafe extern "C" fn agent_match_skill(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_match_skill: {e}"));
            return 1;
        }
    };
    let skill = match arg_str(&v, "skill") {
        Ok(s) => s,
        Err(e) => {
            set_err(err_msg, &format!("agent_match_skill: {e}"));
            return 1;
        }
    };
    let members = match v.get("members") {
        Some(m) => m.clone(),
        None => {
            set_err(err_msg, "agent_match_skill: missing members");
            return 1;
        }
    };
    // members may be a JSON string (from Marqdo stringify) or already an array
    let members = if let Some(s) = members.as_str() {
        match serde_json::from_str::<serde_json::Value>(s) {
            Ok(j) => j,
            Err(e) => {
                set_err(err_msg, &format!("agent_match_skill: members JSON: {e}"));
                return 1;
            }
        }
    } else {
        members
    };
    match match_skill(skill, &members) {
        Ok(j) => {
            set_out(out_json, &j.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &format!("agent_match_skill: {e}"));
            1
        }
    }
}

unsafe extern "C" fn agent_bump_load(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_bump_load: {e}"));
            return 1;
        }
    };
    let member = match v.get("member") {
        Some(m) => m.clone(),
        None => {
            set_err(err_msg, "agent_bump_load: missing member");
            return 1;
        }
    };
    let member = if let Some(s) = member.as_str() {
        match serde_json::from_str::<serde_json::Value>(s) {
            Ok(j) => j,
            Err(e) => {
                set_err(err_msg, &format!("agent_bump_load: member JSON: {e}"));
                return 1;
            }
        }
    } else {
        member
    };
    let delta = v.get("delta").and_then(|x| x.as_i64()).unwrap_or(1);
    match bump_load(&member, delta) {
        Ok(j) => {
            set_out(out_json, &j.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &format!("agent_bump_load: {e}"));
            1
        }
    }
}

fn register(host: &MarqdoHostApi, name: &str, params: &str, fn_ptr: PluginFn) -> c_int {
    let register = match host.register_fn {
        Some(f) => f,
        None => return 1,
    };
    let n = CString::new(name).unwrap();
    let p = CString::new(params).unwrap();
    unsafe { register(host.userdata, n.as_ptr(), p.as_ptr(), fn_ptr) }
}

pub(crate) fn host_query_json(name: &str) -> Result<serde_json::Value, String> {
    let query = unsafe { HOST_QUERY }.ok_or_else(|| "host_query not available".to_string())?;
    let userdata = unsafe { HOST_USERDATA };
    let c_name = CString::new(name).map_err(|e| e.to_string())?;
    let c_args = CString::new("{}").unwrap();
    let mut out_ptr: *mut c_char = ptr::null_mut();
    let mut err_ptr: *mut c_char = ptr::null_mut();
    let rc = unsafe {
        query(
            userdata,
            c_name.as_ptr(),
            c_args.as_ptr(),
            &mut out_ptr,
            &mut err_ptr,
        )
    };
    let err = take_host_string(err_ptr);
    let out = take_host_string(out_ptr);
    if rc != 0 {
        return Err(err.unwrap_or_else(|| format!("host_query `{name}` failed")));
    }
    let out = out.unwrap_or_else(|| "null".into());
    serde_json::from_str(&out).map_err(|e| format!("host_query `{name}` bad JSON: {e}"))
}

fn take_host_string(p: *mut c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(p) }
        .to_str()
        .ok()
        .map(|s| s.to_string());
    unsafe {
        if let Some(free) = HOST_FREE {
            free(p as *mut c_void);
        }
    }
    s
}

unsafe extern "C" fn agent_alloc(
    _args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let mut seq = match AGENT_SEQ.lock() {
        Ok(g) => g,
        Err(_) => {
            set_err(err_msg, "agent_alloc: lock poisoned");
            return 1;
        }
    };
    *seq = seq.wrapping_add(1);
    let id = format!("agent-{seq}");
    drop(seq);
    if let Ok(mut map) = HISTORIES.lock() {
        map.insert(id.clone(), Vec::new());
    }
    let out = serde_json::Value::String(id).to_string();
    set_out(out_json, &out);
    0
}

unsafe extern "C" fn agent_history_get(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_history_get: {e}"));
            return 1;
        }
    };
    let id = match arg_str(&v, "id") {
        Ok(s) => s.to_string(),
        Err(e) => {
            set_err(err_msg, &format!("agent_history_get: {e}"));
            return 1;
        }
    };
    let list = HISTORIES
        .lock()
        .map(|m| m.get(&id).cloned().unwrap_or_default())
        .unwrap_or_default();
    set_out(out_json, &serde_json::Value::Array(list).to_string());
    0
}

unsafe extern "C" fn agent_history_clear(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_history_clear: {e}"));
            return 1;
        }
    };
    let id = match arg_str(&v, "id") {
        Ok(s) => s.to_string(),
        Err(e) => {
            set_err(err_msg, &format!("agent_history_clear: {e}"));
            return 1;
        }
    };
    if let Ok(mut map) = HISTORIES.lock() {
        map.insert(id, Vec::new());
    }
    set_out(out_json, "null");
    0
}

unsafe extern "C" fn agent_history_append(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_history_append: {e}"));
            return 1;
        }
    };
    let id = match arg_str(&v, "id") {
        Ok(s) => s.to_string(),
        Err(e) => {
            set_err(err_msg, &format!("agent_history_append: {e}"));
            return 1;
        }
    };
    let item = match v.get("item") {
        Some(i) => i.clone(),
        None => {
            set_err(err_msg, "agent_history_append: missing item");
            return 1;
        }
    };
    let list = match HISTORIES.lock() {
        Ok(mut map) => {
            let list = map.entry(id).or_default();
            list.push(item);
            list.clone()
        }
        Err(_) => {
            set_err(err_msg, "agent_history_append: lock poisoned");
            return 1;
        }
    };
    set_out(out_json, &serde_json::Value::Array(list).to_string());
    0
}

unsafe extern "C" fn agent_module_source(
    _args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    match host_query_json("module_source") {
        Ok(j) => {
            set_out(out_json, &j.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &e);
            1
        }
    }
}

unsafe extern "C" fn agent_call_site(
    _args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    match host_query_json("call_site") {
        Ok(j) => {
            set_out(out_json, &j.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &e);
            1
        }
    }
}

unsafe extern "C" fn agent_marqdo_skill(
    _args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    match host_query_json("marqdo_skill") {
        Ok(j) => {
            set_out(out_json, &j.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &e);
            1
        }
    }
}

fn tool_names(tools: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    let rows = match tools {
        serde_json::Value::Array(a) => a.as_slice(),
        serde_json::Value::String(s) => {
            if let Ok(serde_json::Value::Array(a)) = serde_json::from_str(s) {
                return tool_names(&serde_json::Value::Array(a));
            }
            return out;
        }
        _ => return out,
    };
    for row in rows {
        match row {
            serde_json::Value::String(s) if !s.is_empty() => out.push(s.clone()),
            serde_json::Value::Object(m) => {
                for key in ["工具", "tools", "name"] {
                    if let Some(serde_json::Value::String(s)) = m.get(key) {
                        if !s.is_empty() {
                            out.push(s.clone());
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    out
}

unsafe extern "C" fn agent_format_tools(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_format_tools: {e}"));
            return 1;
        }
    };
    let tools = match v.get("tools") {
        Some(t) => t,
        None => {
            set_err(err_msg, "agent_format_tools: missing tools");
            return 1;
        }
    };
    let mut text =
        String::from("Available tools (invoke via CALL:<name> or 调用:<name>; runs as subtask):\n");
    for name in tool_names(tools) {
        text.push_str("- ");
        text.push_str(&name);
        text.push('\n');
    }
    set_out(out_json, &serde_json::Value::String(text).to_string());
    0
}

unsafe extern "C" fn agent_tool_allowed(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let v = match parse_args(args_json) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("agent_tool_allowed: {e}"));
            return 1;
        }
    };
    let tools = match v.get("tools") {
        Some(t) => t,
        None => {
            set_err(err_msg, "agent_tool_allowed: missing tools");
            return 1;
        }
    };
    let name = match arg_str(&v, "name") {
        Ok(s) => s,
        Err(_) => {
            set_out(out_json, "false");
            return 0;
        }
    };
    let ok = tool_names(tools).iter().any(|n| n == name);
    set_out(out_json, if ok { "true" } else { "false" });
    0
}

macro_rules! kb_ffi {
    ($fn_name:ident, $call:expr, $label:expr) => {
        unsafe extern "C" fn $fn_name(
            args_json: *const c_char,
            out_json: *mut *mut c_char,
            err_msg: *mut *mut c_char,
        ) -> c_int {
            let v = match parse_args(args_json) {
                Ok(v) => v,
                Err(e) => {
                    set_err(err_msg, &format!("{}: {e}", $label));
                    return 1;
                }
            };
            match $call(&v) {
                Ok(out) => {
                    set_out(out_json, &out.to_string());
                    0
                }
                Err(e) => {
                    set_err(err_msg, &format!("{}: {e}", $label));
                    1
                }
            }
        }
    };
}

kb_ffi!(agent_goal_sig, kb::goal_sig, "agent_goal_sig");
kb_ffi!(agent_goal_slug, kb::goal_slug, "agent_goal_slug");
kb_ffi!(agent_kb_canonicalize, kb::kb_canonicalize, "agent_kb_canonicalize");
kb_ffi!(agent_kb_lookup, kb::kb_lookup, "agent_kb_lookup");
kb_ffi!(agent_kb_list_tasks, kb::kb_list_tasks, "agent_kb_list_tasks");
kb_ffi!(agent_kb_add_alias, kb::kb_add_alias, "agent_kb_add_alias");
kb_ffi!(agent_kb_promote, kb::kb_promote, "agent_kb_promote");
kb_ffi!(agent_kb_record_hit, kb::kb_record_hit, "agent_kb_record_hit");
kb_ffi!(
    agent_workbook_solidify,
    kb::workbook_solidify,
    "agent_workbook_solidify"
);
kb_ffi!(agent_kb_task_files, kb::kb_task_files, "agent_kb_task_files");

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_abi_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_init(host: *const MarqdoHostApi) -> c_int {
    if host.is_null() {
        return 1;
    }
    let host = &*host;
    HOST_ALLOC = host.alloc;
    HOST_FREE = host.free;
    HOST_QUERY = host.host_query;
    HOST_USERDATA = host.userdata;
    if host.host_query.is_none() {
        return 1;
    }
    if register(host, "agent_find_root", "start,markers", agent_find_root) != 0 {
        return 1;
    }
    if register(host, "agent_ensure_layout", "root", agent_ensure_layout) != 0 {
        return 1;
    }
    if register(host, "agent_probe", "root", agent_probe) != 0 {
        return 1;
    }
    if register(
        host,
        "agent_scaffold",
        "root,name,template,dest",
        agent_scaffold,
    ) != 0
    {
        return 1;
    }
    if register(host, "agent_match_skill", "skill,members", agent_match_skill) != 0 {
        return 1;
    }
    if register(host, "agent_bump_load", "member,delta", agent_bump_load) != 0 {
        return 1;
    }
    if register(host, "agent_alloc", "", agent_alloc) != 0 {
        return 1;
    }
    if register(host, "agent_history_get", "id", agent_history_get) != 0 {
        return 1;
    }
    if register(host, "agent_history_clear", "id", agent_history_clear) != 0 {
        return 1;
    }
    if register(host, "agent_history_append", "id,item", agent_history_append) != 0 {
        return 1;
    }
    if register(host, "agent_module_source", "", agent_module_source) != 0 {
        return 1;
    }
    if register(host, "agent_call_site", "", agent_call_site) != 0 {
        return 1;
    }
    if register(host, "agent_marqdo_skill", "", agent_marqdo_skill) != 0 {
        return 1;
    }
    if register(host, "agent_format_tools", "tools", agent_format_tools) != 0 {
        return 1;
    }
    if register(host, "agent_tool_allowed", "tools,name", agent_tool_allowed) != 0 {
        return 1;
    }
    if register(host, "agent_goal_sig", "goal", agent_goal_sig) != 0 {
        return 1;
    }
    if register(host, "agent_goal_slug", "goal", agent_goal_slug) != 0 {
        return 1;
    }
    if register(host, "agent_kb_canonicalize", "goal", agent_kb_canonicalize) != 0 {
        return 1;
    }
    if register(host, "agent_kb_lookup", "kb_dir,goal", agent_kb_lookup) != 0 {
        return 1;
    }
    if register(host, "agent_kb_list_tasks", "kb_dir", agent_kb_list_tasks) != 0 {
        return 1;
    }
    if register(host, "agent_kb_add_alias", "kb_dir,slug,alias", agent_kb_add_alias) != 0 {
        return 1;
    }
    if register(
        host,
        "agent_kb_promote",
        "kb_dir,goal,workbook",
        agent_kb_promote,
    ) != 0
    {
        return 1;
    }
    if register(host, "agent_kb_record_hit", "kb_dir,goal", agent_kb_record_hit) != 0 {
        return 1;
    }
    if register(host, "agent_workbook_solidify", "path", agent_workbook_solidify) != 0 {
        return 1;
    }
    if register(host, "agent_kb_task_files", "kb_dir,goal", agent_kb_task_files) != 0 {
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_shutdown() {
    HOST_ALLOC = None;
    HOST_FREE = None;
    HOST_QUERY = None;
    HOST_USERDATA = ptr::null_mut();
    if let Ok(mut map) = HISTORIES.lock() {
        map.clear();
    }
    if let Ok(mut seq) = AGENT_SEQ.lock() {
        *seq = 0;
    }
}
