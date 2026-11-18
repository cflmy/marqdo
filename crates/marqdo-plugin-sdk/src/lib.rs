//! Shared ABI plumbing for Marqdo native plugins (`<TAB>` slimming).
//!
//! Every plugin used to copy the same `MarqdoHostApi` struct, the alloc/free
//! statics, `host_strdup`, `set_out`/`set_err`, `parse_args` and `reply`
//! (about 140 lines each × 5 crates). This crate holds the single copy; the
//! `marqdo_plugin!` macro emits the `#[no_mangle]` exports each `.so` needs.
//!
//! The ABI itself is unchanged: `include/marqdo_abi.h` and the host loader
//! (`src/host/plugin.rs`) still see exactly the same symbols and layout.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

pub use serde_json::{json, Value};

/// Highest ABI version this SDK speaks.
pub const ABI_VERSION: u32 = 2;

pub type PluginFn = unsafe extern "C" fn(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int;

pub type HostQueryFn = unsafe extern "C" fn(
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

pub static mut HOST_FREE: Option<unsafe extern "C" fn(*mut c_void)> = None;
pub static mut HOST_ALLOC: Option<unsafe extern "C" fn(usize) -> *mut c_void> = None;
pub static mut HOST_QUERY: Option<HostQueryFn> = None;
pub static mut HOST_USERDATA: *mut c_void = ptr::null_mut();

/// Duplicate `s` into host-allocated memory (the host frees it).
pub unsafe fn host_strdup(s: &str) -> *mut c_char {
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

pub fn set_out(out: *mut *mut c_char, s: &str) {
    if out.is_null() {
        return;
    }
    unsafe {
        *out = host_strdup(s);
    }
}

pub fn set_err(err: *mut *mut c_char, s: &str) {
    if err.is_null() {
        return;
    }
    unsafe {
        *err = host_strdup(s);
    }
}

/// Parse the JSON args object handed to a plugin entry point.
pub fn parse_args(args_json: *const c_char) -> Result<Value, String> {
    if args_json.is_null() {
        return Ok(json!({}));
    }
    let s = unsafe { CStr::from_ptr(args_json) }
        .to_str()
        .map_err(|_| "args not utf-8".to_string())?;
    if s.trim().is_empty() {
        return Ok(json!({}));
    }
    serde_json::from_str(s).map_err(|e| e.to_string())
}

/// Write a plugin result (or error) back through the ABI pointers.
pub fn reply(out_json: *mut *mut c_char, err_msg: *mut *mut c_char, r: Result<Value, String>) -> c_int {
    match r {
        Ok(v) => {
            set_out(out_json, &v.to_string());
            0
        }
        Err(e) => {
            set_err(err_msg, &e);
            1
        }
    }
}

/// Ask the host for a builtin with no arguments (`host_query`).
pub fn host_query_json(name: &str) -> Result<Value, String> {
    host_query_json_with(name, &json!({}))
}

/// Ask the host for a builtin, passing `args` as its argument object.
pub fn host_query_json_with(name: &str, args: &Value) -> Result<Value, String> {
    let q = unsafe { HOST_QUERY }.ok_or("host_query unavailable")?;
    let n = CString::new(name).map_err(|e| e.to_string())?;
    let a = CString::new(args.to_string()).map_err(|e| e.to_string())?;
    let mut out: *mut c_char = ptr::null_mut();
    let mut err: *mut c_char = ptr::null_mut();
    let rc = unsafe { q(HOST_USERDATA, n.as_ptr(), a.as_ptr(), &mut out, &mut err) };
    if rc != 0 {
        return Err(take_host_string(err).unwrap_or_else(|| "host_query failed".into()));
    }
    let text = take_host_string(out).ok_or("host_query returned nothing")?;
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// Copy a host-allocated C string into a Rust `String` and free the original.
pub fn take_host_string(p: *mut c_char) -> Option<String> {
    if p.is_null() {
        return None;
    }
    let s = unsafe { CStr::from_ptr(p) }
        .to_string_lossy()
        .into_owned();
    if let Some(free) = unsafe { HOST_FREE } {
        unsafe { free(p as *mut c_void) };
    }
    Some(s)
}

fn register(host: &MarqdoHostApi, name: &str, params: &str, f: PluginFn) -> c_int {
    let Some(reg) = host.register_fn else {
        return 1;
    };
    let n = CString::new(name).unwrap();
    let p = CString::new(params).unwrap();
    unsafe { reg(host.userdata, n.as_ptr(), p.as_ptr(), f) }
}

/// Store the host callbacks and register every entry point.
pub unsafe fn init_plugin(host: *const MarqdoHostApi, regs: &[(&str, &str, PluginFn)]) -> c_int {
    if host.is_null() {
        return 1;
    }
    let host = &*host;
    HOST_ALLOC = host.alloc;
    HOST_FREE = host.free;
    HOST_QUERY = host.host_query;
    HOST_USERDATA = host.userdata;
    for (name, params, f) in regs {
        if register(host, name, params, *f) != 0 {
            return 1;
        }
    }
    0
}

/// Emit the four `#[no_mangle]` exports every plugin `.so` must provide,
/// wiring `marqdo_plugin_init` to the given registration table.
#[macro_export]
macro_rules! marqdo_plugin {
    ($version:expr, [ $( ($name:expr, $params:expr, $f:expr) ),* $(,)? ]) => {
        #[no_mangle]
        pub unsafe extern "C" fn marqdo_plugin_abi_version() -> u32 {
            $version
        }

        #[no_mangle]
        pub unsafe extern "C" fn marqdo_plugin_shutdown() {}

        #[no_mangle]
        pub unsafe extern "C" fn marqdo_plugin_init(
            host: *const $crate::MarqdoHostApi,
        ) -> i32 {
            $crate::init_plugin(host, &[ $( ($name, $params, $f as $crate::PluginFn) ),* ])
        }
    };
}
