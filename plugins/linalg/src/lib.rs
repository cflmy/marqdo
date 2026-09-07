//! Marqdo linalg plugin (C ABI v2): formula-first matrix expressions.

mod complex;
mod dense;
mod draw;
mod factor;
mod matexpr;
mod metrics;

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
                return coerce_expr(e);
            }
        }
    }
    Err(format!("missing one of {:?}", keys))
}

fn coerce_expr(v: &Value) -> Result<Expr, String> {
    if let Ok(e) = Expr::from_value(v) {
        return Ok(e);
    }
    let data = dense::from_value(v)?;
    Ok(Expr::Dense { data })
}

fn wrap(expr: Expr) -> Result<Value, String> {
    expr.to_value()
}

la_ffi!(linalg_ping, |_args: &Value| {
    Ok(json!({
        "ok": true,
        "name": "linalg",
        "abi": ABI_VERSION,
        "features": [
            "matexpr",
            "simplify",
            "dense",
            "solve",
            "block",
            "kron",
            "factorize",
            "draw",
            "lstsq",
            "norm",
            "cond",
            "rank",
            "complex",
            "declare",
        ],
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
    if complex::data_has_complex(data) {
        return complex::to_dense_value(&complex::from_data(data)?);
    }
    let m = dense::from_value(data)?;
    wrap(matexpr::from_list(m)?)
});

la_ffi!(linalg_declare, |args: &Value| {
    let table = args
        .get("table")
        .or_else(|| args.get("rows"))
        .or_else(|| args.get("表"))
        .or_else(|| args.get("行"))
        .ok_or_else(|| "missing `table` (list of {name,rows,cols})".to_string())?;
    let rows = table
        .as_array()
        .ok_or_else(|| "`table` must be a list of row maps".to_string())?;
    let mut out = serde_json::Map::new();
    out.insert("_type".into(), json!("matrix_env"));
    for (i, row) in rows.iter().enumerate() {
        let obj = row
            .as_object()
            .ok_or_else(|| format!("declare row {i} must be a map"))?;
        let name = obj
            .get("name")
            .or_else(|| obj.get("名"))
            .and_then(|x| x.as_str())
            .ok_or_else(|| format!("declare row {i} missing name"))?
            .to_string();
        let r = Dim::from_json(
            obj.get("rows")
                .or_else(|| obj.get("行"))
                .ok_or_else(|| format!("declare `{name}` missing rows"))?,
        )?;
        let c = Dim::from_json(
            obj.get("cols")
                .or_else(|| obj.get("列"))
                .ok_or_else(|| format!("declare `{name}` missing cols"))?,
        )?;
        let expr = matexpr::symbol(&name, r, c);
        out.insert(name, expr.to_value()?);
    }
    Ok(Value::Object(out))
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

fn display_expr(args: &Value, e: matexpr::Expr) -> Result<matexpr::Expr, String> {
    let raw = args
        .get("raw")
        .or_else(|| args.get("原始"))
        .and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            Value::String(s) => Some(matches!(s.to_ascii_lowercase().as_str(), "true" | "1" | "yes")),
            _ => v.as_bool(),
        })
        .unwrap_or(false);
    if raw {
        Ok(e)
    } else {
        matexpr::simplify(e)
    }
}

la_ffi!(linalg_ascii, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    let e = display_expr(args, e)?;
    Ok(json!(e.ascii()))
});

la_ffi!(linalg_latex, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    let e = display_expr(args, e)?;
    Ok(json!(e.latex()))
});

la_ffi!(linalg_shape, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    let (r, c) = e.shape()?;
    Ok(json!([r.to_json(), c.to_json()]))
});

la_ffi!(linalg_show, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    let e = display_expr(args, e)?;
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

la_ffi!(linalg_from_formula, |args: &Value| {
    let f = args
        .get("formula")
        .or_else(|| args.get("公式"))
        .or_else(|| args.get("expr"))
        .or_else(|| args.get("式"))
        .ok_or_else(|| "missing `formula`".to_string())?;
    wrap(coerce_expr(f)?)
});

la_ffi!(linalg_explicit, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"]).or_else(|_| {
        let f = args
            .get("formula")
            .or_else(|| args.get("公式"))
            .ok_or_else(|| "missing `expr`".to_string())?;
        coerce_expr(f)
    })?;
    let m = dense::explicit(&e)?;
    dense::to_dense_value(&m)
});

la_ffi!(linalg_det, |args: &Value| {
    let m = dense_arg(args, &["expr", "a", "matrix", "式", "矩阵"])?;
    Ok(json!(dense::det(&m)?))
});

la_ffi!(linalg_trace, |args: &Value| {
    let m = dense_arg(args, &["expr", "a", "matrix", "式", "矩阵"])?;
    Ok(json!(dense::trace(&m)?))
});

la_ffi!(linalg_solve, |args: &Value| {
    let b = dense_arg(args, &["b", "B", "rhs", "右"])?;
    if let Some(f) = args
        .get("factor")
        .or_else(|| args.get("分解"))
        .filter(|v| !v.is_null())
    {
        let x = factor::solve_lu_factor(f, &b)?;
        return dense::to_dense_value(&x).map(|xv| {
            json!({
                "_type": "linalg_solve",
                "method": "lu",
                "x": xv,
            })
        });
    }
    let a = dense_arg(args, &["a", "A", "matrix", "矩阵", "左"])?;
    dense::solve_result(&a, &b)
});

la_ffi!(linalg_matmul, |args: &Value| {
    let a = dense_arg(args, &["a", "左"])?;
    let b = dense_arg(args, &["b", "右"])?;
    dense::to_dense_value(&dense::matmul(&a, &b)?)
});

la_ffi!(linalg_kron, |args: &Value| {
    let a = arg_expr_opt(args, &["a", "左"])?;
    let b = arg_expr_opt(args, &["b", "右"])?;
    wrap(matexpr::kron(a, b)?)
});

la_ffi!(linalg_block, |args: &Value| {
    let blocks_v = args
        .get("blocks")
        .or_else(|| args.get("块"))
        .ok_or_else(|| "missing `blocks`".to_string())?;
    let grid = blocks_v
        .as_array()
        .ok_or_else(|| "`blocks` must be a list of rows".to_string())?;
    let mut blocks = Vec::new();
    for row in grid {
        let cells = row
            .as_array()
            .ok_or_else(|| "each block row must be a list".to_string())?;
        let mut r = Vec::new();
        for c in cells {
            r.push(coerce_expr(c)?);
        }
        blocks.push(r);
    }
    wrap(matexpr::block(blocks)?)
});

la_ffi!(linalg_collapse, |args: &Value| {
    let e = arg_expr_opt(args, &["expr", "a", "式"])?;
    wrap(matexpr::collapse(e)?)
});

la_ffi!(linalg_factorize, |args: &Value| {
    let m = dense_arg(args, &["matrix", "expr", "a", "矩阵", "式"])?;
    let kind = args
        .get("kind")
        .or_else(|| args.get("类型"))
        .and_then(|x| x.as_str())
        .unwrap_or("lu");
    factor::factorize(&m, kind)
});

la_ffi!(linalg_draw, |args: &Value| {
    let factor_v = args
        .get("factor")
        .or_else(|| args.get("分解"))
        .or_else(|| args.get("expr"))
        .or_else(|| args.get("matrix"))
        .or_else(|| args.get("式"))
        .or_else(|| args.get("矩阵"))
        .ok_or_else(|| "missing `factor`".to_string())?;
    let kind = args
        .get("kind")
        .or_else(|| args.get("类型"))
        .and_then(|x| x.as_str());
    let path = args
        .get("path")
        .or_else(|| args.get("路径"))
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty());
    let theme = args
        .get("theme")
        .or_else(|| args.get("主题"))
        .and_then(|x| x.as_str())
        .map(draw::ThemeName::parse)
        .transpose()?
        .unwrap_or(draw::ThemeName::Light);
    let svg = draw::draw_factor(factor_v, kind, path, theme)?;
    let _ = record_plot(&svg, path);
    Ok(json!({
        "_type": "linalg_svg",
        "kind": kind
            .or_else(|| factor_v.get("kind").and_then(|k| k.as_str()))
            .unwrap_or("structure"),
        "theme": theme.as_str(),
        "svg": svg,
    }))
});

la_ffi!(linalg_lstsq, |args: &Value| {
    let a = dense_arg(args, &["a", "A", "matrix", "矩阵", "左"])?;
    let b = dense_arg(args, &["b", "B", "rhs", "右"])?;
    metrics::lstsq(&a, &b)
});

la_ffi!(linalg_norm, |args: &Value| {
    let ord = args
        .get("ord")
        .or_else(|| args.get("order"))
        .or_else(|| args.get("范数"))
        .and_then(|x| x.as_str())
        .unwrap_or("fro");
    if let Some(v) = args
        .get("expr")
        .or_else(|| args.get("a"))
        .or_else(|| args.get("matrix"))
        .or_else(|| args.get("式"))
        .or_else(|| args.get("矩阵"))
    {
        if v.get("dtype").and_then(|d| d.as_str()) == Some("complex")
            || complex::from_value(v).is_ok()
        {
            if !matches!(ord, "fro" | "frobenius" | "f" | "") {
                return Err("complex norm only supports fro for now".into());
            }
            let cm = complex::from_value(v).or_else(|_| {
                v.get("data")
                    .ok_or_else(|| "missing complex data".to_string())
                    .and_then(complex::from_data)
            })?;
            return Ok(json!(complex::frobenius_norm(&cm)?));
        }
    }
    let m = dense_arg(args, &["expr", "a", "matrix", "式", "矩阵"])?;
    Ok(json!(metrics::norm(&m, ord)?))
});

la_ffi!(linalg_cond, |args: &Value| {
    let m = dense_arg(args, &["expr", "a", "matrix", "式", "矩阵"])?;
    metrics::cond(&m)
});

la_ffi!(linalg_rank, |args: &Value| {
    let m = dense_arg(args, &["expr", "a", "matrix", "式", "矩阵"])?;
    Ok(json!(metrics::rank(&m)?))
});

fn dense_arg(args: &Value, keys: &[&str]) -> Result<dense::Mat, String> {
    for k in keys {
        if let Some(v) = args.get(*k) {
            if v.is_null() {
                continue;
            }
            if v.get("dtype").and_then(|d| d.as_str()) == Some("complex") {
                return Err("real dense op got dtype=complex (use norm fro on complex; factorize/solve stay real)".into());
            }
            if let Ok(e) = Expr::from_value(v) {
                return dense::explicit(&e);
            }
            return dense::from_value(v);
        }
    }
    Err(format!("missing dense matrix among {:?}", keys))
}

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
        ("linalg_declare", "table", linalg_declare as PluginFn),
        ("linalg_from_formula", "formula", linalg_from_formula as PluginFn),
        ("linalg_mul", "a,b", linalg_mul as PluginFn),
        ("linalg_add", "a,b", linalg_add as PluginFn),
        ("linalg_sub", "a,b", linalg_sub as PluginFn),
        ("linalg_transpose", "expr", linalg_transpose as PluginFn),
        ("linalg_inv", "expr", linalg_inv as PluginFn),
        ("linalg_simplify", "expr", linalg_simplify as PluginFn),
        ("linalg_ascii", "expr,raw", linalg_ascii as PluginFn),
        ("linalg_latex", "expr,raw", linalg_latex as PluginFn),
        ("linalg_shape", "expr", linalg_shape as PluginFn),
        ("linalg_show", "expr,path,raw", linalg_show as PluginFn),
        ("linalg_explicit", "expr", linalg_explicit as PluginFn),
        ("linalg_det", "expr", linalg_det as PluginFn),
        ("linalg_trace", "expr", linalg_trace as PluginFn),
        ("linalg_solve", "a,b,factor", linalg_solve as PluginFn),
        ("linalg_matmul", "a,b", linalg_matmul as PluginFn),
        ("linalg_kron", "a,b", linalg_kron as PluginFn),
        ("linalg_block", "blocks", linalg_block as PluginFn),
        ("linalg_collapse", "expr", linalg_collapse as PluginFn),
        ("linalg_factorize", "matrix,kind", linalg_factorize as PluginFn),
        ("linalg_draw", "factor,kind,path,theme", linalg_draw as PluginFn),
        ("linalg_lstsq", "a,b", linalg_lstsq as PluginFn),
        ("linalg_norm", "expr,ord", linalg_norm as PluginFn),
        ("linalg_cond", "expr", linalg_cond as PluginFn),
        ("linalg_rank", "expr", linalg_rank as PluginFn),
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
