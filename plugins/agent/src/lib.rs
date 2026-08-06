//! Agent layout plugin (C ABI v1): find_root, ensure_layout, probe, scaffold.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::{Component, Path, PathBuf};
use std::ptr;

const ABI_VERSION: u32 = 1;
const DEFAULT_MARKERS: &str = "agents,runbooks,marqdo.agent.json";
const LAYOUT_DIRS: &[&str] = &["agents", "runbooks", "templates", "reports"];

type PluginFn = unsafe extern "C" fn(
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
}

static mut HOST_FREE: Option<unsafe extern "C" fn(*mut c_void)> = None;
static mut HOST_ALLOC: Option<unsafe extern "C" fn(usize) -> *mut c_void> = None;

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

fn register(host: &MarqdoHostApi, name: &str, params: &str, fn_ptr: PluginFn) -> c_int {
    let register = match host.register_fn {
        Some(f) => f,
        None => return 1,
    };
    let n = CString::new(name).unwrap();
    let p = CString::new(params).unwrap();
    unsafe { register(host.userdata, n.as_ptr(), p.as_ptr(), fn_ptr) }
}

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
    0
}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_shutdown() {
    HOST_ALLOC = None;
    HOST_FREE = None;
}
