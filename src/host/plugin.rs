//! Native plugin loader (C ABI v1/v2). Shared libs are not linked into marqdo.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};

use libloading::{Library, Symbol};

use crate::host::agent_rt;
use crate::host::json::{json_to_value, value_to_json};
use crate::host::HostContext;
use crate::value::Value;

/// Highest ABI version this host speaks.
pub const ABI_VERSION: u32 = 2;
/// Oldest plugin ABI still accepted.
pub const ABI_VERSION_MIN: u32 = 1;

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
    host_query: Option<HostQueryFn>,
}

thread_local! {
    /// Set around plugin calls so `host_query` can read/write host context (plots, etc.).
    static CURRENT_HOST: Cell<*mut HostContext> = const { Cell::new(std::ptr::null_mut()) };
    /// Optional `lib.member` resolver for `host_query("call_lib_path")` (site entry module).
    static LIB_PATH_CALL: RefCell<Option<LibPathCall>> = const { RefCell::new(None) };
}

/// Hook: resolve bare `lib.member` (and longer paths) in the site module's import tree.
pub struct LibPathCall {
    pub call: fn(*mut (), &str) -> Result<Value, String>,
    pub data: *mut (),
}

/// Run `body` with a temporary lib-path call hook (nested-safe).
pub fn with_lib_path_call<R>(hook: LibPathCall, body: impl FnOnce() -> R) -> R {
    LIB_PATH_CALL.with(|slot| {
        let prev = slot.replace(Some(hook));
        let out = body();
        slot.replace(prev);
        out
    })
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

    pub fn has_fn(&self, name: &str) -> bool {
        self.fns.contains_key(name)
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

unsafe fn host_strdup(s: &str) -> *mut c_char {
    let bytes = s.as_bytes();
    let p = host_alloc(bytes.len() + 1) as *mut u8;
    if p.is_null() {
        return std::ptr::null_mut();
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
    *p.add(bytes.len()) = 0;
    p as *mut c_char
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

/// Allowlisted host introspection for ABI v2 plugins (`host_query`).
unsafe extern "C" fn host_query(
    _userdata: *mut c_void,
    name: *const c_char,
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    let set_err = |msg: &str| {
        if !err_msg.is_null() {
            *err_msg = host_strdup(msg);
        }
    };
    let set_out = |msg: &str| {
        if !out_json.is_null() {
            *out_json = host_strdup(msg);
        }
    };

    if name.is_null() {
        set_err("host_query: null name");
        return 1;
    }
    let name = match CStr::from_ptr(name).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_err("host_query: name not utf-8");
            return 1;
        }
    };

    let ctx_ptr = CURRENT_HOST.with(|c| c.get());
    if ctx_ptr.is_null() {
        set_err("host_query: no active host context (call during plugin fn)");
        return 1;
    }
    let ctx = &mut *ctx_ptr;

    let args_owned: Option<String> = if args_json.is_null() {
        None
    } else {
        CStr::from_ptr(args_json)
            .to_str()
            .ok()
            .map(|s| s.to_string())
    };

    let result = match name {
        "module_source" => agent_rt::module_source(ctx).and_then(|v| value_to_json(&v)),
        "call_site" => agent_rt::call_site(ctx, Some(ctx.current_line)).and_then(|v| value_to_json(&v)),
        "marqdo_skill" => agent_rt::marqdo_skill(ctx).and_then(|v| value_to_json(&v)),
        "cwd" => Ok(serde_json::Value::String(
            ctx.cwd.to_string_lossy().into_owned(),
        )),
        "entry_dir" => Ok(serde_json::Value::String(
            ctx.entry_path
                .as_ref()
                .map(|p| {
                    if p.is_dir() {
                        p.clone()
                    } else {
                        p.parent().unwrap_or(p).to_path_buf()
                    }
                })
                .unwrap_or_else(|| ctx.cwd.clone())
                .to_string_lossy()
                .into_owned(),
        )),
        "call_lib_path" => (|| {
            let raw = args_owned.as_deref().unwrap_or("{}");
            let args: serde_json::Value = serde_json::from_str(if raw.trim().is_empty() {
                "{}"
            } else {
                raw
            })
            .map_err(|e| format!("call_lib_path args: {e}"))?;
            let path = args
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "call_lib_path requires `path`".to_string())?
                .to_string();
            LIB_PATH_CALL.with(|slot| {
                let hook = slot.borrow();
                let hook = hook
                    .as_ref()
                    .ok_or_else(|| "call_lib_path: no site lib resolver".to_string())?;
                let v = (hook.call)(hook.data, &path)?;
                value_to_json(&v)
            })
        })(),
        "record_plot" => (|| {
            let raw = args_owned.as_deref().unwrap_or("{}");
            let args: serde_json::Value = serde_json::from_str(if raw.trim().is_empty() {
                "{}"
            } else {
                raw
            })
            .map_err(|e| format!("record_plot args: {e}"))?;
            let svg = args
                .get("svg")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "record_plot requires `svg`".to_string())?
                .to_string();
            let path_s = args
                .get("path")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            if let Some(ref p) = path_s {
                crate::host::fs::write_text(
                    ctx,
                    &Value::Text(p.clone()),
                    &Value::Text(svg.clone()),
                )?;
            }
            ctx.plots.push(crate::host::PlotArtifact {
                path: path_s,
                svg,
            });
            Ok(serde_json::json!({ "ok": true }))
        })(),
        other => Err(format!("host_query: unknown `{other}`")),
    };

    match result {
        Ok(j) => {
            set_out(&j.to_string());
            0
        }
        Err(e) => {
            set_err(&e);
            1
        }
    }
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
    if ver < ABI_VERSION_MIN || ver > ABI_VERSION {
        return Err(format!(
            "plugin ABI version {ver} not in host range {ABI_VERSION_MIN}..={ABI_VERSION} ({})",
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
        host_query: Some(host_query),
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
            // Idempotent re-load of the same plugin surface (e.g. multiple `agent` ctors).
            if buf.fns.keys().all(|name| ctx.plugins.fns.contains_key(name)) {
                return Ok(Value::Int(0));
            }
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
    ctx: &mut HostContext,
    name: &str,
    bound: &HashMap<String, Value>,
) -> Result<Value, String> {
    if !ctx.allow_plugin() {
        return Err("plugin call denied by host policy".into());
    }
    let reg = ctx
        .plugins
        .get(name)
        .ok_or_else(|| format!("unknown function `{name}`"))?
        .clone();

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

    CURRENT_HOST.with(|c| c.set(ctx as *mut HostContext));
    let rc = unsafe { (reg.fn_ptr)(c_args.as_ptr(), &mut out_ptr, &mut err_ptr) };
    CURRENT_HOST.with(|c| c.set(std::ptr::null_mut()));

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
