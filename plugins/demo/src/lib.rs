//! Demo Marqdo plugin (C ABI v1). Registers `demo_add` and `demo_echo`.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

const ABI_VERSION: u32 = 1;

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

unsafe extern "C" fn demo_add(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    if args_json.is_null() {
        set_err(err_msg, "demo_add: null args");
        return 1;
    }
    let s = match CStr::from_ptr(args_json).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_err(err_msg, "demo_add: args not utf-8");
            return 1;
        }
    };
    let v: serde_json::Value = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("demo_add: {e}"));
            return 1;
        }
    };
    let a = v.get("a").and_then(|x| x.as_i64()).unwrap_or(0);
    let b = v.get("b").and_then(|x| x.as_i64()).unwrap_or(0);
    set_out(out_json, &(a + b).to_string());
    0
}

unsafe extern "C" fn demo_echo(
    args_json: *const c_char,
    out_json: *mut *mut c_char,
    err_msg: *mut *mut c_char,
) -> c_int {
    if args_json.is_null() {
        set_err(err_msg, "demo_echo: null args");
        return 1;
    }
    let s = match CStr::from_ptr(args_json).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_err(err_msg, "demo_echo: args not utf-8");
            return 1;
        }
    };
    let v: serde_json::Value = match serde_json::from_str(s) {
        Ok(v) => v,
        Err(e) => {
            set_err(err_msg, &format!("demo_echo: {e}"));
            return 1;
        }
    };
    let text = v
        .get("text")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    let out = serde_json::Value::String(text.to_string()).to_string();
    set_out(out_json, &out);
    0
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
    let register = match host.register_fn {
        Some(f) => f,
        None => return 1,
    };
    let name_add = CString::new("demo_add").unwrap();
    let params_add = CString::new("a,b").unwrap();
    if register(
        host.userdata,
        name_add.as_ptr(),
        params_add.as_ptr(),
        demo_add,
    ) != 0
    {
        return 1;
    }
    let name_echo = CString::new("demo_echo").unwrap();
    let params_echo = CString::new("text").unwrap();
    if register(
        host.userdata,
        name_echo.as_ptr(),
        params_echo.as_ptr(),
        demo_echo,
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
