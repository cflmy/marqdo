//! Native plugin loader (C ABI v1). Shared libs are not linked into marqdo.

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

use libloading::{Library, Symbol};

use crate::host::json::{json_to_value, value_to_json};
use crate::host::HostContext;
use crate::value::Value;

pub const ABI_VERSION: u32 = 1;

type PluginFn = unsafe extern "C" fn(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int;

type AbiVersionFn = unsafe extern "C" fn() -> u32;
type InitFn = unsafe extern "C" fn(host: *const MarqdoHostApi) -> c_int;
type ShutdownFn = unsafe extern "C" fn();

#[repr(C)]
struct MarqdoHostApi {
    userdata: *mut c_void,
    register_fn: Option<
        unsafe extern "C" fn(
            userdata: *mut c_void,
            name: *const c_char,
            params: *const c_char,
            fn_ptr: PluginFn,
        ) -> c_int,
    >,
    alloc: Option<unsafe extern "C" fn(n: usize) -> *mut c_void>,
    free: Option<unsafe extern "C" fn(p: *mut c_void)>,
}

struct RegisterBuf {
    fns: HashMap<String, RegisteredFn>,
    error: Option<String>,
}

#[derive(Clone)]
pub struct RegisteredFn {
    pub params: Vec<String>,
    fn_ptr: PluginFn,
}

pub struct PluginState {
    libs: Vec<Library>,
    fns: HashMap<String, RegisteredFn>,
}

impl Default for PluginState {
    fn default() -> Self {
        Self {
            libs: Vec::new(),
            fns: HashMap::new(),
        }
    }
}

impl std::fmt::Debug for PluginState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginState")
            .field("libs", &self.libs.len())
            .field("fns", &self.list_names())
            .finish()
    }
}

impl Clone for PluginState {
    fn clone(&self) -> Self {
        // `Library` handles are not cloneable; a cloned host starts with no plugins.
        Self::default()
    }
}

impl PluginState {
    pub fn get(&self, name: &str) -> Option<&RegisteredFn> {
        self.fns.get(name)
    }

    pub fn list_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.fns.keys().cloned().collect();
        names.sort();
        names
    }
}

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} must be text")),
    }
}

extern "C" {
    fn malloc(n: usize) -> *mut c_void;
    fn free(p: *mut c_void);
}

unsafe extern "C" fn host_alloc(n: usize) -> *mut c_void {
    malloc(n)
}

unsafe extern "C" fn host_free(p: *mut c_void) {
    if !p.is_null() {
        free(p);
    }
}

unsafe extern "C" fn host_register(
    userdata: *mut c_void,
    name: *const c_char,
    params: *const c_char,
    fn_ptr: PluginFn,
) -> c_int {
    if userdata.is_null() || name.is_null() {
        return 1;
    }
    let buf = &mut *(userdata as *mut RegisterBuf);
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            buf.error = Some("plugin register: name is not UTF-8".into());
            return 1;
        }
    };
    if name.is_empty() {
        buf.error = Some("plugin register: empty name".into());
        return 1;
    }
    let params_s = if params.is_null() {
        ""
    } else {
        match CStr::from_ptr(params).to_str() {
            Ok(s) => s,
            Err(_) => {
                buf.error = Some("plugin register: params is not UTF-8".into());
                return 1;
            }
        }
    };
    let param_list: Vec<String> = if params_s.trim().is_empty() {
        Vec::new()
    } else {
        params_s
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect()
    };
    if buf.fns.contains_key(&name) {
        buf.error = Some(format!("plugin register: duplicate `{name}`"));
        return 1;
    }
    buf.fns.insert(
        name,
        RegisteredFn {
            params: param_list,
            fn_ptr,
        },
    );
    0
}

pub fn load(ctx: &mut HostContext, path: &Value) -> Result<Value, String> {
    if !ctx.allow_plugin() {
        return Err("plugin_load denied by host policy".into());
    }
    let rel = as_text(path, "path")?;
    let file = match ctx.resolve_path(rel) {
        Ok(p) => p,
        Err(e) => {
            let p = std::path::Path::new(rel);
            let abs = if p.is_absolute() {
                p.to_path_buf()
            } else {
                ctx.cwd.join(p)
            };
            if abs.is_file() && crate::ext_cli::path_is_trusted_plugin(&abs) {
                abs
            } else {
                return Err(e);
            }
        }
    };
    if !file.is_file() {
        return Err(format!("plugin_load: not a file: {}", file.display()));
    }

    let lib = unsafe { Library::new(&file) }
        .map_err(|e| format!("plugin_load {}: {e}", file.display()))?;

    let version_fn: Symbol<AbiVersionFn> = unsafe {
        lib.get(b"marqdo_plugin_abi_version\0")
            .map_err(|e| format!("plugin missing marqdo_plugin_abi_version: {e}"))?
    };
    let ver = unsafe { version_fn() };
    if ver != ABI_VERSION {
        return Err(format!(
            "plugin ABI version {ver} != host {ABI_VERSION} ({})",
            file.display()
        ));
    }

    let init_fn: Symbol<InitFn> = unsafe {
        lib.get(b"marqdo_plugin_init\0")
            .map_err(|e| format!("plugin missing marqdo_plugin_init: {e}"))?
    };

    let mut buf = RegisterBuf {
        fns: HashMap::new(),
        error: None,
    };
    let api = MarqdoHostApi {
        userdata: &mut buf as *mut RegisterBuf as *mut c_void,
        register_fn: Some(host_register),
        alloc: Some(host_alloc),
        free: Some(host_free),
    };

    let rc = unsafe { init_fn(&api) };
    if rc != 0 {
        return Err(buf
            .error
            .unwrap_or_else(|| format!("plugin init failed (code {rc})")));
    }
    if let Some(e) = buf.error {
        return Err(e);
    }

    for k in buf.fns.keys() {
        if ctx.plugins.fns.contains_key(k) {
            return Err(format!("plugin function `{k}` already registered"));
        }
    }
    let added = buf.fns.len();
    for (k, v) in buf.fns {
        ctx.plugins.fns.insert(k, v);
    }
    ctx.plugins.libs.push(lib);
    Ok(Value::Int(added as i64))
}

pub fn unload(ctx: &mut HostContext) -> Result<Value, String> {
    if !ctx.allow_plugin() {
        return Err("plugin_unload denied by host policy".into());
    }
    for lib in ctx.plugins.libs.drain(..) {
        if let Ok(shutdown) = unsafe { lib.get::<ShutdownFn>(b"marqdo_plugin_shutdown\0") } {
            unsafe { shutdown() };
        }
        drop(lib);
    }
    ctx.plugins.fns.clear();
    Ok(Value::None)
}

pub fn list(ctx: &HostContext) -> Result<Value, String> {
    Ok(Value::List(
        ctx.plugins
            .list_names()
            .into_iter()
            .map(Value::Text)
            .collect(),
    ))
}

/// Call a registered plugin function with already-bound args (param name → Value).
pub fn call_registered(
    ctx: &HostContext,
    name: &str,
    bound: &HashMap<String, Value>,
) -> Result<Value, String> {
    if !ctx.allow_plugin() {
        return Err("plugin call denied by host policy".into());
    }
    let reg = ctx
        .plugins
        .get(name)
        .ok_or_else(|| format!("unknown function `{name}`"))?;

    let mut map = serde_json::Map::new();
    for p in &reg.params {
        let v = bound.get(p).unwrap_or(&Value::None);
        map.insert(p.clone(), value_to_json(v)?);
    }
    for (k, v) in bound {
        if !map.contains_key(k) {
            map.insert(k.clone(), value_to_json(v)?);
        }
    }
    let args_json = serde_json::Value::Object(map).to_string();
    let c_args = CString::new(args_json).map_err(|e| format!("plugin args: {e}"))?;

    let mut out_ptr: *mut c_char = std::ptr::null_mut();
    let mut err_ptr: *mut c_char = std::ptr::null_mut();
    let rc = unsafe { (reg.fn_ptr)(c_args.as_ptr(), &mut out_ptr, &mut err_ptr) };

    let err_s = take_c_string(err_ptr);
    let out_s = take_c_string(out_ptr);

    if rc != 0 {
        return Err(err_s.unwrap_or_else(|| format!("plugin `{name}` failed (code {rc})")));
    }
    let out_s = out_s.unwrap_or_else(|| "null".to_string());
    let j: serde_json::Value =
        serde_json::from_str(&out_s).map_err(|e| format!("plugin `{name}` bad JSON result: {e}"))?;
    json_to_value(&j)
}

fn take_c_string(p: *mut c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(p) }
        .to_str()
        .map(|s| s.to_string())
        .ok();
    unsafe { host_free(p as *mut c_void) };
    s
}
