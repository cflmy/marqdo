//! Marqdo web plugin (C ABI v2): SQLite, page assemble, HTTP listen.

mod compose;
mod db;
mod http;
mod render;
mod table;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

use serde_json::{json, Value};

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

fn call_lib(path: &str) -> Result<Value, String> {
    host_query_json("call_lib_path", &json!({ "path": path }))
}

fn arg_text(v: &Value, key: &str) -> Result<String, String> {
    match v.get(key) {
        None | Some(Value::Null) => Err(format!("missing `{key}`")),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(Value::Number(n)) => Ok(n.to_string()),
        Some(Value::Bool(b)) => Ok(b.to_string()),
        Some(other) => Ok(other.to_string()),
    }
}

fn arg_str<'a>(v: &'a Value, key: &str) -> Result<&'a str, String> {
    v.get(key)
        .and_then(|x| x.as_str())
        .ok_or_else(|| format!("missing `{key}`"))
}

fn arg_str_opt<'a>(v: &'a Value, key: &str) -> Option<&'a str> {
    v.get(key).and_then(|x| x.as_str()).filter(|s| !s.is_empty())
}

fn db_url_of(args: &Value) -> Result<String, String> {
    if let Some(s) = arg_str_opt(args, "url").or_else(|| arg_str_opt(args, "db_url")) {
        return Ok(s.to_string());
    }
    if let Some(db) = args.get("db") {
        if let Some(s) = db.get("url").and_then(|v| v.as_str()) {
            return Ok(s.to_string());
        }
    }
    Err("missing db url".into())
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

macro_rules! web_ffi {
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

web_ffi!(web_page_new, |args: &Value| {
    let title = arg_str_opt(args, "title").unwrap_or("Marqdo Web");
    let intro = arg_str_opt(args, "intro").unwrap_or("");
    Ok(json!({
        "title": title,
        "intro": intro,
    }))
});

web_ffi!(web_compose_components, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let components = args
        .get("components")
        .cloned()
        .ok_or_else(|| "missing `components`".to_string())?;
    compose::compose_components(&page, &components, |path| call_lib(path))
});

web_ffi!(web_compose_main, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let main = args
        .get("main")
        .cloned()
        .ok_or_else(|| "missing `main`".to_string())?;
    compose::compose_main(&page, &main, |path| call_lib(path))
});

web_ffi!(web_render, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or_else(|| args.clone());
    let db_url = db_url_of(args).ok();
    let html = render::render_page(&page, db_url.as_deref());
    Ok(Value::String(html))
});

web_ffi!(web_db_new, |args: &Value| {
    let url = arg_str_opt(args, "url").unwrap_or("sqlite:site.db");
    Ok(json!({ "url": url }))
});

web_ffi!(web_db_init, |args: &Value| {
    let url = db_url_of(args)?;
    let name = arg_str(args, "name")
        .or_else(|_| arg_str(args, "table"))?
        .to_string();
    let fields = args.get("fields").cloned().unwrap_or(json!([]));
    db::init(&url, &name, &fields)
});

web_ffi!(web_db_insert, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let rows = args
        .get("rows")
        .or_else(|| args.get("row"))
        .cloned()
        .unwrap_or(json!([]));
    db::insert(&url, &table, &rows)
});

web_ffi!(web_db_select, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(200);
    db::select(&url, &table, limit)
});

web_ffi!(web_db_get, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let id = arg_text(args, "id")?;
    db::get(&url, &table, &id)
});

web_ffi!(web_db_update, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let id = arg_text(args, "id")?;
    let row = args.get("row").cloned().unwrap_or(json!({}));
    db::update(&url, &table, &id, &row)
});

web_ffi!(web_db_delete, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let id = arg_text(args, "id")?;
    db::delete(&url, &table, &id)
});

web_ffi!(web_db_exec, |args: &Value| {
    let url = db_url_of(args)?;
    let sql = arg_str(args, "sql")?.to_string();
    let args_v = args.get("args");
    db::exec(&url, &sql, args_v)
});

web_ffi!(web_db_list_tables, |args: &Value| {
    let url = db_url_of(args)?;
    let tables = db::list_tables(&url)?;
    Ok(json!({ "tables": tables }))
});

web_ffi!(web_app_new, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let db = args.get("db").cloned().unwrap_or(Value::Null);
    let host = arg_str_opt(args, "host").unwrap_or("127.0.0.1");
    let port = args
        .get("port")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .unwrap_or(18081);
    let admin = match args.get("admin") {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => matches!(s.as_str(), "true" | "True" | "1" | "yes"),
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
        _ => false,
    };
    Ok(json!({
        "page": page,
        "db": db,
        "host": host,
        "port": port,
        "admin": admin,
    }))
});

web_ffi!(web_listen, |args: &Value| {
    let (page, db_url, host, port, admin) = if args.get("page").is_some() || args.get("host").is_some()
    {
        let page = args.get("page").cloned().unwrap_or(json!({}));
        let db_url = db_url_of(args).ok();
        let host = arg_str_opt(args, "host").unwrap_or("127.0.0.1");
        let port = args
            .get("port")
            .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
            .unwrap_or(18081) as u16;
        let admin = match args.get("admin") {
            Some(Value::Bool(b)) => *b,
            _ => false,
        };
        (page, db_url, host.to_string(), port, admin)
    } else {
        let app = args.get("app").cloned().unwrap_or_else(|| args.clone());
        let page = app.get("page").cloned().unwrap_or(json!({}));
        let db_url = app
            .get("db")
            .and_then(|d| d.get("url"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| arg_str_opt(&app, "db_url").map(|s| s.to_string()));
        let host = app
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or("127.0.0.1")
            .to_string();
        let port = app
            .get("port")
            .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
            .unwrap_or(18081) as u16;
        let admin = app
            .get("admin")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        (page, db_url, host, port, admin)
    };
    http::listen(&page, db_url.as_deref(), &host, port, admin)
});

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
    if host.host_query.is_none() {
        return 1;
    }

    let regs = [
        ("web_page_new", "title,intro", web_page_new as PluginFn),
        (
            "web_compose_components",
            "page,components",
            web_compose_components as PluginFn,
        ),
        ("web_compose_main", "page,main", web_compose_main as PluginFn),
        ("web_render", "page", web_render as PluginFn),
        ("web_db_new", "url", web_db_new as PluginFn),
        ("web_db_init", "url,name,fields", web_db_init as PluginFn),
        ("web_db_insert", "url,table,rows", web_db_insert as PluginFn),
        ("web_db_select", "url,table,limit", web_db_select as PluginFn),
        ("web_db_get", "url,table,id", web_db_get as PluginFn),
        ("web_db_update", "url,table,id,row", web_db_update as PluginFn),
        ("web_db_delete", "url,table,id", web_db_delete as PluginFn),
        ("web_db_exec", "url,sql,args", web_db_exec as PluginFn),
        ("web_db_list_tables", "url", web_db_list_tables as PluginFn),
        (
            "web_app_new",
            "page,db,admin,host,port",
            web_app_new as PluginFn,
        ),
        ("web_listen", "app", web_listen as PluginFn),
    ];
    for (name, params, f) in regs {
        if register(host, name, params, f) != 0 {
            return 1;
        }
    }
    0
}
