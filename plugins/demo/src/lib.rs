//! Demo Marqdo plugin (C ABI v1). Registers `demo_add` and `demo_echo`.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use marqdo_plugin_sdk::*;


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

marqdo_plugin!(1, [
    ("demo_add", "a,b", demo_add),
    ("demo_echo", "text", demo_echo),
]);
