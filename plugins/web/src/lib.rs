//! Marqdo web plugin (C ABI v2): async HTTP, SQLite, page shell, admin.

mod db;
mod http;
mod page;
mod table_util;

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
    serde_json::from_str(s).map_err(|e| format!("json: {e}"))
}

fn arg_text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("missing `{key}`"))
}

macro_rules! web_ffi {
    ($name:ident, $body:expr, $label:expr) => {
        unsafe extern "C" fn $name(
            args_json: *const c_char,
            out_json: *mut *mut c_char,
            err_msg: *mut *mut c_char,
        ) -> c_int {
            match (|| -> Result<Value, String> {
                let args = parse_args(args_json)?;
                $body(&args)
            })() {
                Ok(v) => {
                    set_out(out_json, &v.to_string());
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

web_ffi!(
    web_render,
    |args: &Value| Ok(Value::String(page::render_page(args))),
    "web_render"
);

web_ffi!(
    web_as_links,
    |args: &Value| {
        let table = args.get("table").cloned().unwrap_or(Value::Null);
        Ok(table_util::as_links(&table))
    },
    "web_as_links"
);

web_ffi!(
    web_as_fields,
    |args: &Value| {
        let table = args.get("table").cloned().unwrap_or(Value::Null);
        Ok(table_util::as_fields(&table))
    },
    "web_as_fields"
);

web_ffi!(
    web_as_bind,
    |args: &Value| {
        let table = args.get("table").cloned().unwrap_or(Value::Null);
        Ok(table_util::as_bind(&table))
    },
    "web_as_bind"
);

web_ffi!(
    web_as_rows,
    |args: &Value| {
        let table = args.get("table").cloned().unwrap_or(Value::Null);
        Ok(table_util::as_rows(&table))
    },
    "web_as_rows"
);

web_ffi!(
    web_listen,
    |args: &Value| http::listen(args),
    "web_listen"
);

web_ffi!(
    web_db_migrate,
    |args: &Value| {
        let url = args
            .get("url")
            .or_else(|| args.get("db_url"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing `url`".to_string())?;
        let dir = args
            .get("dir")
            .and_then(|v| v.as_str())
            .unwrap_or("migrations");
        db::migrate(url, dir)
    },
    "web_db_migrate"
);

web_ffi!(
    web_db_define,
    |args: &Value| {
        let url = args
            .get("url")
            .or_else(|| args.get("db_url"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing `url`".to_string())?;
        let table = arg_text(args, "table")?;
        let fields = args.get("fields").cloned().unwrap_or(json!([]));
        let primary = args
            .get("primary")
            .and_then(|v| v.as_str())
            .unwrap_or("id");
        db::define_table(url, table, &fields, primary)
    },
    "web_db_define"
);

web_ffi!(
    web_db_all,
    |args: &Value| {
        let url = args
            .get("url")
            .or_else(|| args.get("db_url"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing `url`".to_string())?;
        let table = arg_text(args, "table")?;
        let limit = args
            .get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(200);
        db::query_all(url, table, limit)
    },
    "web_db_all"
);

web_ffi!(
    web_db_exec,
    |args: &Value| {
        let url = args
            .get("url")
            .or_else(|| args.get("db_url"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing `url`".to_string())?;
        let sql = arg_text(args, "sql")?;
        db::exec_sql(url, sql, args.get("args"))
    },
    "web_db_exec"
);

web_ffi!(
    web_db_query,
    |args: &Value| {
        let url = args
            .get("url")
            .or_else(|| args.get("db_url"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing `url`".to_string())?;
        let sql = arg_text(args, "sql")?;
        db::query_sql(url, sql, args.get("args"))
    },
    "web_db_query"
);

web_ffi!(
    web_db_insert,
    |args: &Value| {
        let url = args
            .get("url")
            .or_else(|| args.get("db_url"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing `url`".to_string())?;
        let table = arg_text(args, "table")?;
        if let Some(rows) = args.get("rows") {
            let normalized = table_util::as_rows(rows);
            return db::insert_rows(url, table, &normalized);
        }
        let row = args.get("row").cloned().unwrap_or(json!({}));
        let normalized = table_util::as_rows(&row);
        if let Some(arr) = normalized.as_array() {
            if arr.len() > 1 || row.as_object().is_some_and(|m| {
                m.values().any(|v| v.is_array())
            }) {
                return db::insert_rows(url, table, &normalized);
            }
            if let Some(one) = arr.first() {
                return db::insert_row(url, table, one);
            }
        }
        db::insert_row(url, table, &row)
    },
    "web_db_insert"
);

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
    if host.host_query.is_none() {
        return 1;
    }
    if register(host, "web_render", "title,nav,sidebar,footer,main", web_render) != 0 {
        return 1;
    }
    if register(host, "web_as_links", "table", web_as_links) != 0 {
        return 1;
    }
    if register(host, "web_as_fields", "table", web_as_fields) != 0 {
        return 1;
    }
    if register(host, "web_as_bind", "table", web_as_bind) != 0 {
        return 1;
    }
    if register(host, "web_as_rows", "table", web_as_rows) != 0 {
        return 1;
    }
    if register(host, "web_listen", "host,port", web_listen) != 0 {
        return 1;
    }
    if register(host, "web_db_migrate", "url,dir", web_db_migrate) != 0 {
        return 1;
    }
    if register(host, "web_db_define", "url,table,fields", web_db_define) != 0 {
        return 1;
    }
    if register(host, "web_db_all", "url,table", web_db_all) != 0 {
        return 1;
    }
    if register(host, "web_db_exec", "url,sql", web_db_exec) != 0 {
        return 1;
    }
    if register(host, "web_db_query", "url,sql", web_db_query) != 0 {
        return 1;
    }
    if register(host, "web_db_insert", "url,table,rows", web_db_insert) != 0 {
        return 1;
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn marqdo_plugin_shutdown() {
    HOST_ALLOC = None;
    HOST_FREE = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_contains_nav() {
        let html = page::render_page(&json!({
            "title": "T",
            "nav": [{"label": "Home", "href": "/"}],
            "main": "<p>hi</p>",
        }));
        assert!(html.contains("Home"));
        assert!(html.contains("<p>hi</p>"));
    }

    #[test]
    fn define_and_query_sqlite() {
        let dir = std::env::temp_dir().join(format!("mq-web-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let url = format!("sqlite:{}/t.db", dir.display());
        let fields = json!([
            {"name": "id", "type": "integer", "null": false},
            {"name": "title", "type": "text", "null": false},
        ]);
        db::define_table(&url, "posts", &fields, "id").unwrap();
        db::insert_row(&url, "posts", &json!({"title": "hello"})).unwrap();
        let rows = db::query_all(&url, "posts", 10).unwrap();
        assert_eq!(rows["count"], 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
