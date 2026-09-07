//! Marqdo linalg plugin (C ABI v2): formula-first matrix expressions.

mod matexpr;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use serde_json::{json, Value};

use matexpr::{Dim, Expr};

const ABI_VERSION: u32 = 2;

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

fn parse_args(args_json: *const c_char) -> Result<Value, String> {
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

fn reply(out_json: *mut *mut c_char, err_msg: *mut *mut c_char, r: Result<Value, String>) -> c_int {
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

macro_rules! la_ffi {
    ($name:ident, $body:expr) => {
        unsafe extern "C" fn $name(
            args_json: *const c_char,
            out_json: *mut *mut c_char,
            err_msg: *mut *mut c_char,
        ) -> c_int {
            let args = match parse_args(args_json) {
                Ok(v) => v,
                Err(e) => {
                    set_err(err_msg, &format!("{}: {e}", stringify!($name)));
                    return 1;
                }
            };
            let r: Result<Value, String> = (|| $body(&args))();
            reply(
                out_json,
                err_msg,
                r.map_err(|e| format!("{}: {e}", stringify!($name))),
            )
        }
    };
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

fn host_query_json(name: &str, args: &Value) -> Result<Value, String> {
    let query = unsafe { HOST_QUERY }.ok_or_else(|| "host_query not available".to_string())?;
    let userdata = unsafe { HOST_USERDATA };
    let c_name = CString::new(name).map_err(|e| e.to_string())?;
    let c_args = CString::new(args.to_string()).map_err(|e| e.to_string())?;
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

fn record_plot(svg: &str, path: Option<&str>) -> Result<(), String> {
    let mut args = json!({ "svg": svg });
    if let Some(p) = path {
        if !p.is_empty() {
            args.as_object_mut()
                .unwrap()
                .insert("path".into(), json!(p));
        }
    }
    host_query_json("record_plot", &args)?;
    Ok(())
}

fn arg_dim(v: &Value, key: &str) -> Result<Dim, String> {
    Dim::from_json(v.get(key).ok_or_else(|| format!("missing `{key}`"))?)
}

fn arg_expr_opt(v: &Value, keys: &[&str]) -> Result<Expr, String> {
    for k in keys {
        if let Some(e) = v.get(*k) {
            if !e.is_null() {
                return Expr::from_value(e);
            }
        }
    }
    Err(format!("missing one of {:?}", keys))
}

fn wrap(expr: Expr) -> Result<Value, String> {
    expr.to_value()
}

la_ffi!(linalg_ping, |_args: &Value| {
    Ok(json!({
        "ok": true,
        "name": "linalg",
        "abi": ABI_VERSION,
        "features": ["matexpr", "simplify"],
    }))
});

la_ffi!(linalg_symbol, |args: &Value| {
    let name = args
        .get("name")
        .or_else(|| args.get("名"))
        .and_then(|x| x.as_str())
        .ok_or_else(|| "missing `name`".to_string())?;
    let rows = arg_dim(args, "rows").or_else(|_| arg_dim(args, "行"))?;
    let cols = arg_dim(args, "cols").or_else(|_| arg_dim(args, "列"))?;
    wrap(matexpr::symbol(name, rows, cols))
});

la_ffi!(linalg_eye, |args: &Value| {
    let n = arg_dim(args, "n")
        .or_else(|_| arg_dim(args, "阶"))
        .or_else(|_| arg_dim(args, "rows"))?;
    wrap(matexpr::eye(n))
});

la_ffi!(linalg_zeros, |args: &Value| {
    let rows = arg_dim(args, "rows").or_else(|_| arg_dim(args, "行"))?;
    let cols = arg_dim(args, "cols").or_else(|_| arg_dim(args, "列"))?;
    wrap(matexpr::zeros(rows, cols))
});

la_ffi!(linalg_from_list, |args: &Value| {
    let data = args
        .get("data")
        .or_else(|| args.get("数据"))
        .ok_or_else(|| "missing `data`".to_string())?;
    let rows = data
        .as_array()
        .ok_or_else(|| "`data` must be list of rows".to_string())?;
    let mut mat = Vec::new();
    for row in rows {
        let cells = row
            .as_array()
            .ok_or_else(|| "each row must be a list".to_string())?;
        let mut r = Vec::new();
        for c in cells {
            let n = c
                .as_f64()
                .or_else(|| c.as_i64().map(|i| i as f64))
                .or_else(|| c.as_u64().map(|u| u as f64))
                .ok_or_else(|| "cell must be a number".to_string())?;
            r.push(n);
        }
        mat.push(r);
    }
    wrap(matexpr::from_list(mat)?)
});

la_ffi!(linalg_mul, |args: &Value| {
    let a = arg_expr_opt(args, &["a", "左"])?;
    let b = arg_expr_opt(args, &["b", "右"])?;
    wrap(matexpr::mul(a, b)?)
});

la_ffi!(linalg_add, |args: &Value| {
    let a = arg_expr_opt(args, &["a", "左"])?;
    let b = arg_expr_opt(args, &["b", "右"])?;
    wrap(matexpr::add(a, b)?)
});

la_ffi!(linalg_sub, |args: &Value| {
    let a = arg_expr_opt(args, &["a", "左"])?;
    let b = arg_expr_opt(args, &["b", "右"])?;
    wrap(matexpr::sub(a, b)?)
});

la_ffi!(linalg_transpose, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    wrap(matexpr::transpose(e)?)
});

la_ffi!(linalg_inv, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    wrap(matexpr::inv(e)?)
});

la_ffi!(linalg_simplify, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    wrap(matexpr::simplify(e)?)
});

la_ffi!(linalg_ascii, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    Ok(json!(e.ascii()))
});

la_ffi!(linalg_latex, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    Ok(json!(e.latex()))
});

la_ffi!(linalg_shape, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    let (r, c) = e.shape()?;
    Ok(json!([r.to_json(), c.to_json()]))
});

la_ffi!(linalg_show, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    let latex = e.latex();
    let ascii = e.ascii();
    let path = args
        .get("path")
        .or_else(|| args.get("路径"))
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty());
    // Minimal SVG so view/plot channel can embed the formula text.
    let escaped = latex
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"480\" height=\"64\" data-linalg=\"formula\">\
<text x=\"12\" y=\"40\" font-size=\"18\" font-family=\"serif\">{escaped}</text></svg>"
    );
    let _ = record_plot(&svg, path);
    let mut out = e.to_value()?;
    if let Some(obj) = out.as_object_mut() {
        obj.insert("shown".into(), json!(true));
        obj.insert("ascii".into(), json!(ascii));
        obj.insert("latex".into(), json!(latex));
    }
    Ok(out)
});

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_abi_version() -> u32 {
    ABI_VERSION
}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_shutdown() {}

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
    let register = match host.register_fn {
        Some(f) => f,
        None => return 1,
    };
    let regs = [
        ("linalg_ping", "", linalg_ping as PluginFn),
        ("linalg_symbol", "name,rows,cols", linalg_symbol as PluginFn),
        ("linalg_eye", "n", linalg_eye as PluginFn),
        ("linalg_zeros", "rows,cols", linalg_zeros as PluginFn),
        ("linalg_from_list", "data", linalg_from_list as PluginFn),
        ("linalg_mul", "a,b", linalg_mul as PluginFn),
        ("linalg_add", "a,b", linalg_add as PluginFn),
        ("linalg_sub", "a,b", linalg_sub as PluginFn),
        ("linalg_transpose", "expr", linalg_transpose as PluginFn),
        ("linalg_inv", "expr", linalg_inv as PluginFn),
        ("linalg_simplify", "expr", linalg_simplify as PluginFn),
        ("linalg_ascii", "expr", linalg_ascii as PluginFn),
        ("linalg_latex", "expr", linalg_latex as PluginFn),
        ("linalg_shape", "expr", linalg_shape as PluginFn),
        ("linalg_show", "expr,path", linalg_show as PluginFn),
    ];
    for (name, params, f) in regs {
        let c_name = CString::new(name).unwrap();
        let c_params = CString::new(params).unwrap();
        if register(host.userdata, c_name.as_ptr(), c_params.as_ptr(), f) != 0 {
            return 1;
        }
    }
    0
}
