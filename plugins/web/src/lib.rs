//! Marqdo web plugin (C ABI v2): SQLite, page assemble, HTTP listen.

mod compose;
mod db;
mod db_pg;
mod driver;
mod form;
mod http;
mod cache;
mod markdown;
mod middleware;
mod password;
mod rate_limit;
mod render;
mod rss;
mod session;
mod sitemap;
mod storage;
mod table;
mod upload;
mod ws;
mod ws_hub;

use crate::table::as_css_named;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
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

/// Absolute directory of the entry `.mq.md` (falls back to process cwd).
fn entry_dir() -> PathBuf {
    host_query_json("entry_dir", &json!({}))
        .ok()
        .and_then(|v| v.as_str().map(|s| PathBuf::from(s)))
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        })
}

/// Resolve a DB URL for the plugin.
///
/// - `postgres://` / `postgresql://` — passed through unchanged.
/// - `sqlite:` / bare relative path — made absolute against `entry_dir()`.
fn resolve_db_url(url: &str) -> String {
    let lower = url.to_ascii_lowercase();
    if lower.starts_with("postgres://") || lower.starts_with("postgresql://") {
        return url.to_string();
    }
    let stripped = url
        .strip_prefix("sqlite:")
        .or_else(|| url.strip_prefix("SQLITE:"))
        .unwrap_or(url);
    let abs = PathBuf::from(stripped);
    let abs = if abs.is_absolute() {
        abs
    } else {
        entry_dir().join(abs)
    };
    if stripped.len() == url.len() {
        abs.to_string_lossy().into_owned()
    } else if url.starts_with("sqlite:") {
        format!("sqlite:{}", abs.to_string_lossy())
    } else {
        format!("SQLITE:{}", abs.to_string_lossy())
    }
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
    let raw = if let Some(s) = arg_str_opt(args, "url").or_else(|| arg_str_opt(args, "db_url"))
    {
        Some(s.to_string())
    } else if let Some(db) = args.get("db") {
        if let Some(s) = db.get("url").and_then(|v| v.as_str()) {
            Some(s.to_string())
        } else {
            None
        }
    } else {
        None
    };
    match raw {
        Some(s) => Ok(resolve_db_url(&s)),
        None => Err("missing db url".into()),
    }
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

web_ffi!(web_page_query, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let query = args.get("query").cloned().unwrap_or(json!({}));
    let mut obj = page.as_object().cloned().unwrap_or_default();
    obj.insert("query".into(), query);
    Ok(Value::Object(obj))
});

web_ffi!(web_page_order, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let order = arg_str(args, "order")?.to_string();
    let mut obj = page.as_object().cloned().unwrap_or_default();
    obj.insert("order".into(), json!(order));
    Ok(Value::Object(obj))
});

web_ffi!(web_page_link_prefix, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let prefix = arg_str(args, "prefix")?.to_string();
    let mut obj = page.as_object().cloned().unwrap_or_default();
    obj.insert("link_prefix".into(), json!(prefix));
    Ok(Value::Object(obj))
});

web_ffi!(web_page_css, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let css = arg_str_opt(args, "css").unwrap_or("").to_string();
    let mut obj = page.as_object().cloned().unwrap_or_default();
    if !css.trim().is_empty() {
        let prev = obj
            .get("styles_css")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        obj.insert("styles_css".into(), json!(format!("{prev}\n{css}")));
    }
    Ok(Value::Object(obj))
});

// Assemble a GFM style table into CSS text.
//
// Two shapes are accepted:
// 1. Rule rows `|选择器|属性|值|` → `selector { prop: value; }`
// 2. Property rows `|属性|值|` → `.name { prop: value; }`
//
// The `name` argument names the CSS class for shape 2, and is ignored for
// shape 1. This is how Marqdo-side theme modules turn style tables into a
// stylesheet with 文档即代码 — styles are data, assembly is a function.
web_ffi!(web_style, |args: &Value| {
    let name = arg_str_opt(args, "name").unwrap_or("").to_string();
    let table = args.get("table").cloned().unwrap_or(json!([]));
    Ok(Value::String(as_css_named(&name, &table)))
});

web_ffi!(web_page_detail, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let on = args
        .get("detail")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let mut obj = page.as_object().cloned().unwrap_or_default();
    obj.insert("detail".into(), json!(on));
    Ok(Value::Object(obj))
});

web_ffi!(web_page_meta, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let meta = args
        .get("meta")
        .or_else(|| args.get("table"))
        .cloned()
        .unwrap_or(json!({}));
    let mut obj = page.as_object().cloned().unwrap_or_default();
    obj.insert("meta".into(), Value::Object(crate::table::as_meta_map(&meta)));
    Ok(Value::Object(obj))
});

web_ffi!(web_page_paginate, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let offset = args
        .get("offset")
        .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
        .unwrap_or(0);
    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
        .unwrap_or(10);
    let path = arg_str_opt(args, "path").unwrap_or("/").to_string();
    let mut obj = page.as_object().cloned().unwrap_or_default();
    obj.insert(
        "paginate".into(),
        json!({ "offset": offset, "limit": limit, "path": path }),
    );
    Ok(Value::Object(obj))
});

web_ffi!(web_rss_build, |args: &Value| {
    let title = arg_str_opt(args, "title").unwrap_or("Feed");
    let link = arg_str_opt(args, "link").unwrap_or("/");
    let description = arg_str_opt(args, "description").unwrap_or("");
    let items = args
        .get("items")
        .or_else(|| args.get("rows"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    Ok(Value::String(rss::build_rss(
        title,
        link,
        description,
        &items,
    )))
});

web_ffi!(web_app_route_rss, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let path = normalize_route_path(arg_str(args, "path")?)?;
    let table = arg_str(args, "table")?.to_string();
    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
        .unwrap_or(20);
    let order = arg_str_opt(args, "order").unwrap_or("-created_at").to_string();
    let title = arg_str_opt(args, "title").unwrap_or("Feed").to_string();
    let link = arg_str_opt(args, "link").unwrap_or("/").to_string();
    let description = arg_str_opt(args, "description")
        .unwrap_or("")
        .to_string();
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let mut rss_routes = obj
        .get("rss_routes")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    rss_routes.insert(
        path,
        json!({
            "table": table,
            "limit": limit,
            "order": order,
            "title": title,
            "link": link,
            "description": description,
        }),
    );
    obj.insert("rss_routes".into(), Value::Object(rss_routes));
    Ok(app)
});

web_ffi!(web_app_redirect, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let from = normalize_route_path(arg_str(args, "from").or_else(|_| arg_str(args, "path"))?)?;
    let to = arg_str(args, "to")?.to_string();
    let permanent = match args
        .get("permanent")
        .or_else(|| args.get("永久"))
        .cloned()
        .unwrap_or(json!(false))
    {
        Value::Bool(b) => b,
        Value::String(s) => matches!(s.as_str(), "true" | "True" | "1" | "yes" | "真"),
        Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
        _ => false,
    };
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let mut redirects = obj
        .get("redirects")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    redirects.insert(from, json!({ "to": to, "permanent": permanent }));
    obj.insert("redirects".into(), Value::Object(redirects));
    Ok(app)
});

web_ffi!(web_app_error_page, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let status = args
        .get("status")
        .or_else(|| args.get("状态"))
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .unwrap_or(404) as u16;
    let page = args
        .get("page")
        .or_else(|| args.get("页面"))
        .cloned()
        .ok_or_else(|| "missing `page`".to_string())?;
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let key = match status {
        500 => "page_500",
        _ => "page_404",
    };
    obj.insert(key.into(), page);
    Ok(app)
});

web_ffi!(web_app_sitemap, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let path = normalize_route_path(arg_str(args, "path").unwrap_or("/sitemap.xml"))?;
    let base = arg_str_opt(args, "base").unwrap_or("").to_string();
    let table = arg_str_opt(args, "table").map(|s| s.to_string());
    let loc = arg_str_opt(args, "loc").unwrap_or("path").to_string();
    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(1000);
    let items = args.get("items").cloned().unwrap_or(json!([]));
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let mut routes = obj
        .get("sitemap_routes")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    routes.insert(
        path,
        json!({
            "base": base,
            "table": table,
            "loc": loc,
            "limit": limit,
            "items": items,
        }),
    );
    obj.insert("sitemap_routes".into(), Value::Object(routes));
    Ok(app)
});

web_ffi!(web_app_robots, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let body = arg_str_opt(args, "body").map(|s| s.to_string());
    let sitemap_url = arg_str_opt(args, "sitemap").map(|s| s.to_string());
    let text = match body.filter(|s| !s.is_empty()) {
        Some(b) => b,
        None => crate::sitemap::build_robots(sitemap_url.as_deref()),
    };
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    obj.insert("robots_body".into(), json!(text));
    Ok(app)
});

web_ffi!(web_sitemap_build, |args: &Value| {
    let base = arg_str_opt(args, "base").unwrap_or("");
    let items = args.get("items").cloned().unwrap_or(json!([]));
    Ok(crate::sitemap::sitemap_json(base, &items))
});

fn storage_url_arg(args: &Value, key: &str) -> Result<String, String> {
    if let Some(s) = arg_str_opt(args, key) {
        return Ok(s.to_string());
    }
    let v = args
        .get(key)
        .ok_or_else(|| format!("missing `{key}`"))?;
    if let Some(u) = v.get("url").and_then(|x| x.as_str()) {
        return Ok(u.to_string());
    }
    Err(format!("`{key}` must be a storage url string or storage handle"))
}

web_ffi!(web_app_upload, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let path = normalize_route_path(arg_str(args, "path").unwrap_or("/_upload"))?;
    let field = arg_str_opt(args, "field").unwrap_or("file").to_string();
    let storage_url = storage_url_arg(args, "storage")?;
    let prefix = arg_str_opt(args, "prefix").unwrap_or("uploads/").to_string();
    let max_bytes = args
        .get("max_bytes")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .unwrap_or(5_242_880);
    let types = args.get("types").cloned().filter(|v| !matches!(v, Value::Null));
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let mut upload_routes = obj
        .get("upload_routes")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    upload_routes.insert(
        path,
        json!({
            "field": field,
            "storage_url": storage_url,
            "prefix": prefix,
            "max_bytes": max_bytes,
            "types": types,
        }),
    );
    obj.insert("upload_routes".into(), Value::Object(upload_routes));
    Ok(app)
});

web_ffi!(web_app_download, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let path = arg_str(args, "path").unwrap_or("/_media/{*key}");
    let path = {
        let s = path.trim();
        if s.is_empty() {
            return Err("download path is empty".into());
        }
        let mut p = if s.starts_with('/') {
            s.to_string()
        } else {
            format!("/{s}")
        };
        while p.len() > 1 && p.ends_with('/') {
            p.pop();
        }
        p
    };
    if !path.contains('{') {
        return Err("download path must include a `{key}` or `{*key}` capture".into());
    }
    let storage_url = storage_url_arg(args, "storage")?;
    let disposition = arg_str_opt(args, "disposition")
        .unwrap_or("attachment")
        .to_string();
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let mut download_routes = obj
        .get("download_routes")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    download_routes.insert(
        path,
        json!({
            "storage_url": storage_url,
            "disposition": disposition,
        }),
    );
    obj.insert("download_routes".into(), Value::Object(download_routes));
    Ok(app)
});

web_ffi!(web_upload_validate, |args: &Value| {
    let filename = arg_str(args, "filename")?;
    let content_type = arg_str_opt(args, "content_type").unwrap_or("application/octet-stream");
    let size = args
        .get("size")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .ok_or_else(|| "missing `size`".to_string())?;
    let max_bytes = args
        .get("max_bytes")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .unwrap_or(5_242_880);
    let types = args.get("types");
    upload::validate(filename, content_type, size, max_bytes, types)
});

web_ffi!(web_upload_save, |args: &Value| {
    let storage_url = if let Ok(u) = storage_url_arg(args, "storage") {
        u
    } else if let Ok(u) = storage_url_arg(args, "url") {
        u
    } else {
        arg_str(args, "storage_url")?.to_string()
    };
    let path = arg_str(args, "path")?;
    let key = arg_str_opt(args, "key");
    let content_type = arg_str_opt(args, "content_type");
    let prefix = arg_str_opt(args, "prefix");
    upload::save(&storage_url, key, path, content_type, prefix)
});

web_ffi!(web_media_new, |args: &Value| {
    let storage = args.get("storage").cloned().unwrap_or(Value::Null);
    Ok(json!({ "_type": "media", "storage": storage }))
});

web_ffi!(web_compose_form, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or(json!({}));
    let form = args
        .get("form")
        .cloned()
        .ok_or_else(|| "missing `form`".to_string())?;
    let id = arg_str(args, "id")?.to_string();
    compose::compose_form(&page, &form, &id)
});

web_ffi!(web_render, |args: &Value| {
    let page = args.get("page").cloned().unwrap_or_else(|| args.clone());
    let db_url = db_url_of(args).ok();
    let html = render::render_page(&page, db_url.as_deref(), None);
    Ok(Value::String(html))
});

web_ffi!(web_db_new, |args: &Value| {
    let url = arg_str_opt(args, "url").unwrap_or("sqlite:site.db");
    Ok(json!({ "url": resolve_db_url(url), "_type": "db" }))
});

web_ffi!(web_cache_new, |args: &Value| {
    let url = arg_str_opt(args, "url").unwrap_or("memory:");
    cache::open(url)
});

web_ffi!(web_cache_get, |args: &Value| {
    let url = arg_str(args, "url")?;
    let key = arg_str(args, "key")?;
    cache::get(url, key)
});

web_ffi!(web_cache_set, |args: &Value| {
    let url = arg_str(args, "url")?;
    let key = arg_str(args, "key")?;
    let value = arg_text(args, "value")?;
    let ttl = args
        .get("ttl")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)));
    cache::set(url, key, &value, ttl)
});

web_ffi!(web_cache_del, |args: &Value| {
    let url = arg_str(args, "url")?;
    let key = arg_str(args, "key")?;
    cache::del(url, key)
});

web_ffi!(web_cache_exists, |args: &Value| {
    let url = arg_str(args, "url")?;
    let key = arg_str(args, "key")?;
    cache::exists(url, key)
});

web_ffi!(web_cache_ttl, |args: &Value| {
    let url = arg_str(args, "url")?;
    let key = arg_str(args, "key")?;
    cache::ttl(url, key)
});

web_ffi!(web_storage_new, |args: &Value| {
    let url = arg_str(args, "url").unwrap_or("file:data/blobs");
    storage::open(url)
});

web_ffi!(web_storage_put, |args: &Value| {
    let url = arg_str(args, "url")?;
    let key = arg_str(args, "key")?;
    let body = arg_str_opt(args, "body");
    let path = arg_str_opt(args, "path");
    let content_type = arg_str_opt(args, "content_type");
    storage::put(url, key, body, path, content_type)
});

web_ffi!(web_storage_get, |args: &Value| {
    let url = arg_str(args, "url")?;
    let key = arg_str(args, "key")?;
    storage::get(url, key)
});

web_ffi!(web_storage_delete, |args: &Value| {
    let url = arg_str(args, "url")?;
    let key = arg_str(args, "key")?;
    storage::delete(url, key)
});

web_ffi!(web_storage_list, |args: &Value| {
    let url = arg_str(args, "url")?;
    let prefix = arg_str_opt(args, "prefix");
    storage::list(url, prefix)
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
    let txn = args.get("txn").and_then(|v| v.as_str());
    db::insert(&url, &table, &rows, txn)
});

web_ffi!(web_db_select, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(200);
    let where_v = match args.get("where") {
        None | Some(Value::Null) => None,
        Some(Value::String(s))
            if s.is_empty()
                || s.eq_ignore_ascii_case("none")
                || s.eq_ignore_ascii_case("null") =>
        {
            None
        }
        Some(v) => Some(v),
    };
    let order = args.get("order").and_then(|v| v.as_str());
    let offset = args
        .get("offset")
        .or_else(|| args.get("跳过"))
        .and_then(|v| v.as_i64());
    let txn = args.get("txn").and_then(|v| v.as_str());
    db::select_order(&url, &table, limit, where_v, order, offset, txn)
});

web_ffi!(web_db_query, |args: &Value| {
    let url = db_url_of(args)?;
    let sql = arg_str(args, "sql")?.to_string();
    let args_v = args.get("args");
    let txn = args.get("txn").and_then(|v| v.as_str());
    db::query(&url, &sql, args_v, txn)
});

web_ffi!(web_db_count, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let where_v = args.get("where").filter(|v| !v.is_null());
    let txn = args.get("txn").and_then(|v| v.as_str());
    db::count(&url, &table, where_v, txn)
});

web_ffi!(web_db_migrate, |args: &Value| {
    let url = db_url_of(args)?;
    let steps = args
        .get("steps")
        .ok_or_else(|| "missing `steps`".to_string())?;
    db::migrate(&url, steps)
});

web_ffi!(web_db_fts_create, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let columns = args
        .get("columns")
        .ok_or_else(|| "missing `columns`".to_string())?;
    let name = arg_str_opt(args, "name");
    db::fts_create(&url, &table, columns, name)
});

web_ffi!(web_db_search, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let q = arg_str(args, "q")
        .or_else(|_| arg_str(args, "query"))
        .or_else(|_| arg_str(args, "关键词"))?
        .to_string();
    let limit = args
        .get("limit")
        .and_then(|v| v.as_i64().or_else(|| v.as_u64().map(|u| u as i64)))
        .unwrap_or(20);
    let name = arg_str_opt(args, "name");
    db::search(&url, &table, &q, limit, name)
});

web_ffi!(web_db_begin, |args: &Value| {
    let url = db_url_of(args)?;
    db::begin(&url)
});

web_ffi!(web_db_commit, |args: &Value| {
    let txn = arg_str(args, "txn")?.to_string();
    db::commit(&txn)
});

web_ffi!(web_db_rollback, |args: &Value| {
    let txn = arg_str(args, "txn")?.to_string();
    db::rollback(&txn)
});

web_ffi!(web_db_get, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let id = arg_text(args, "id")?;
    let txn = args.get("txn").and_then(|v| v.as_str());
    db::get(&url, &table, &id, txn)
});

web_ffi!(web_db_update, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let id = arg_text(args, "id")?;
    let row = args.get("row").cloned().unwrap_or(json!({}));
    let txn = args.get("txn").and_then(|v| v.as_str());
    db::update(&url, &table, &id, &row, txn)
});

web_ffi!(web_db_delete, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let id = arg_text(args, "id")?;
    let txn = args.get("txn").and_then(|v| v.as_str());
    db::delete(&url, &table, &id, txn)
});

web_ffi!(web_db_exec, |args: &Value| {
    let url = db_url_of(args)?;
    let sql = arg_str(args, "sql")?.to_string();
    let args_v = args.get("args");
    let txn = args.get("txn").and_then(|v| v.as_str());
    db::exec(&url, &sql, args_v, txn)
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
        "forms": {},
        "routes": {},
        "ws_routes": {},
        "static_dir": null,
        "static_mount": "/static",
    }))
});

fn normalize_route_path(raw: &str) -> Result<String, String> {
    let s = raw.trim();
    if s.is_empty() {
        return Err("route path is empty".into());
    }
    if s == "/" {
        return Err("route path `/` is reserved for the home page".into());
    }
    let mut path = if s.starts_with('/') {
        s.to_string()
    } else {
        format!("/{s}")
    };
    while path.len() > 1 && path.ends_with('/') {
        path.pop();
    }
    // reserved prefixes
    if path == "/admin"
        || path.starts_with("/admin/")
        || path == "/_form"
        || path.starts_with("/_form/")
        || path == "/_part"
        || path.starts_with("/_part/")
        || path == "/static"
        || path.starts_with("/static/")
    {
        return Err(format!("route path `{path}` is reserved"));
    }
    Ok(path)
}

web_ffi!(web_app_route, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let path = normalize_route_path(arg_str(args, "path")?)?;
    let mut page = args
        .get("page")
        .cloned()
        .ok_or_else(|| "missing `page` for route".to_string())?;
    // Stamp mount path so render can emit `{path}/_part/{id}` slot sources.
    if let Some(obj) = page.as_object_mut() {
        obj.insert("_route".into(), json!(&path));
    }
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let routes = obj
        .entry("routes".to_string())
        .or_insert_with(|| json!({}));
    let rmap = routes
        .as_object_mut()
        .ok_or_else(|| "app.routes must be a map".to_string())?;
    rmap.insert(path, page);
    Ok(app)
});

web_ffi!(web_app_route_ws, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let path = normalize_route_path(arg_str(args, "path")?)?;
    let mode = if let Some(m) = args.get("mode").or_else(|| args.get("模式")) {
        crate::ws_hub::WsMode::parse(m)
    } else {
        crate::ws_hub::WsMode::parse(&args.get("echo").cloned().unwrap_or(json!(true)))
    };
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let ws_routes = obj
        .entry("ws_routes".to_string())
        .or_insert_with(|| json!({}));
    let wmap = ws_routes
        .as_object_mut()
        .ok_or_else(|| "app.ws_routes must be a map".to_string())?;
    wmap.insert(path, json!({ "mode": mode.as_str() }));
    Ok(app)
});

web_ffi!(web_ws_connect, |args: &Value| {
    let url = arg_str(args, "url")?.to_string();
    let message = arg_str_opt(args, "message").unwrap_or("");
    let headers = args.get("headers");
    let timeout_sec = args
        .get("timeout_sec")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .unwrap_or(30);
    match crate::ws::connect(&url, message, headers, timeout_sec) {
        Ok(v) => Ok(v),
        Err(e) => Ok(json!({ "ok": false, "error": e })),
    }
});

web_ffi!(web_session_new, |args: &Value| {
    session::abi_session_new(args)
});

web_ffi!(web_session_set, |args: &Value| {
    session::abi_session_set(args)
});

web_ffi!(web_session_get, |args: &Value| {
    session::abi_session_get(args)
});

web_ffi!(web_session_del, |args: &Value| {
    session::abi_session_del(args)
});

web_ffi!(web_session_destroy, |args: &Value| {
    session::abi_session_destroy(args)
});

web_ffi!(web_auth_login, |args: &Value| {
    session::abi_auth_login(args)
});

web_ffi!(web_auth_check, |args: &Value| {
    session::abi_auth_check(args)
});

web_ffi!(web_auth_logout, |args: &Value| {
    session::abi_auth_logout(args)
});

web_ffi!(web_password_hash, |args: &Value| {
    session::abi_password_hash(args)
});

web_ffi!(web_auth_new, |args: &Value| {
    let users = args.get("users").cloned().unwrap_or(Value::Null);
    let session_ttl = args
        .get("session_ttl")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .unwrap_or(3600);
    Ok(json!({ "users": users, "session_ttl": session_ttl }))
});

web_ffi!(web_app_auth, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let users = args.get("users").cloned().unwrap_or(Value::Null);
    let session_ttl = args
        .get("session_ttl")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .unwrap_or(3600);
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    obj.insert(
        "auth".to_string(),
        json!({ "users": users, "session_ttl": session_ttl }),
    );
    // Default RBAC: `/admin*` requires role `admin` when auth is configured.
    let mut gates = obj
        .get("gates")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let has_admin_gate = gates.iter().any(|g| {
        g.get("path")
            .and_then(|v| v.as_str())
            .is_some_and(|p| p == "/admin" || p == "/admin*")
    });
    if !has_admin_gate {
        gates.push(json!({ "path": "/admin*", "roles": ["admin"] }));
        obj.insert("gates".into(), Value::Array(gates));
    }
    Ok(app)
});

web_ffi!(web_app_gate, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let path = arg_str(args, "path")?.to_string();
    let roles_raw = arg_str_opt(args, "roles")
        .or_else(|| arg_str_opt(args, "角色"))
        .unwrap_or("admin");
    let roles: Vec<String> = session::parse_roles_csv(roles_raw);
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let mut gates = obj
        .get("gates")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    gates.push(json!({ "path": path, "roles": roles }));
    obj.insert("gates".into(), Value::Array(gates));
    Ok(app)
});

web_ffi!(web_app_gallery, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let path = normalize_route_path(
        arg_str(args, "path")
            .or_else(|_| arg_str(args, "路径"))
            .unwrap_or("/gallery"),
    )?;
    let storage_url = storage_url_arg(args, "storage")
        .or_else(|_| storage_url_arg(args, "存储"))?;
    let prefix = arg_str_opt(args, "prefix")
        .or_else(|| arg_str_opt(args, "前缀"))
        .unwrap_or("uploads/")
        .to_string();
    let title = arg_str_opt(args, "title")
        .or_else(|| arg_str_opt(args, "标题"))
        .unwrap_or("Gallery")
        .to_string();
    let download_base = arg_str_opt(args, "download_base")
        .or_else(|| arg_str_opt(args, "media"))
        .or_else(|| arg_str_opt(args, "下载基址"))
        .unwrap_or("/_media")
        .to_string();
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let mut routes = obj
        .get("gallery_routes")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    routes.insert(
        path,
        json!({
            "storage": storage_url,
            "prefix": prefix,
            "title": title,
            "download_base": download_base,
        }),
    );
    obj.insert("gallery_routes".into(), Value::Object(routes));
    Ok(app)
});

web_ffi!(web_app_mount_form, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let id = arg_str(args, "id")?.to_string();
    let form_v = args
        .get("form")
        .cloned()
        .ok_or_else(|| "missing `form`".to_string())?;
    let obj = app.as_object_mut().ok_or_else(|| "app must be a map".to_string())?;
    let forms = obj
        .entry("forms".to_string())
        .or_insert_with(|| json!({}));
    let fmap = forms
        .as_object_mut()
        .ok_or_else(|| "app.forms must be a map".to_string())?;
    fmap.insert(id, form_v);
    Ok(app)
});

web_ffi!(web_app_static, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let dir = arg_str(args, "dir")?.to_string();
    if dir.trim().is_empty() {
        return Err("static `dir` is empty".into());
    }
    let mount = arg_str_opt(args, "mount").unwrap_or("/static");
    let mount = http::normalize_static_mount(mount);
    // Also reserve custom mounts against later routes.
    if mount == "/admin"
        || mount.starts_with("/admin/")
        || mount == "/_form"
        || mount.starts_with("/_form/")
        || mount == "/_part"
        || mount.starts_with("/_part/")
    {
        return Err(format!("static mount `{mount}` is reserved"));
    }
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    obj.insert("static_dir".into(), json!(dir));
    obj.insert("static_mount".into(), json!(mount));
    Ok(app)
});

// Configure cross-cutting HTTP middleware on an app: CORS, security response
// headers, gzip compression, request-body limits, and JSON API routes.
//
// Every capability is passed in as *data* (GFM tables from the Marqdo side,
// normalized here into the `middleware` map): 配置即数据、装配即函数.
//
// - `cors`       : `|允许来源|方法|头|暴露头|凭证|` table (one row per origin).
// - `security`   : `|头|值|` table of response headers (e.g. `X-Frame-Options`).
// - `compress`   : bool — enable gzip response compression.
// - `body_limit` : number — max request body bytes (e.g. `1048576`).
// - `json_routes`: `|路径|方法|表|条件|排序|上限|` table of JSON API endpoints
//                  backed by db queries (method GET/POST, `表`=table name).
web_ffi!(web_app_middleware, |args: &Value| {
    let mut app = args.get("app").cloned().unwrap_or(json!({}));
    let obj = app
        .as_object_mut()
        .ok_or_else(|| "app must be a map".to_string())?;
    let mw = obj
        .entry("middleware".to_string())
        .or_insert_with(|| json!({}));
    let m = mw
        .as_object_mut()
        .ok_or_else(|| "app.middleware must be a map".to_string())?;

    if let Some(cors) = args.get("cors") {
        m.insert(
            "cors".into(),
            middleware::cors_from_table(cors),
        );
    }
    if let Some(security) = args.get("security") {
        m.insert(
            "security".into(),
            middleware::security_from_table(security),
        );
    }
    if let Some(compress) = args.get("compress") {
        let on = match compress {
            Value::Bool(b) => *b,
            Value::String(s) => matches!(s.as_str(), "true" | "True" | "1" | "yes" | "on"),
            Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
            _ => false,
        };
        m.insert("compress".into(), json!(on));
    }
    if let Some(al) = args
        .get("access_log")
        .or_else(|| args.get("访问日志"))
    {
        let on = match al {
            Value::Bool(b) => *b,
            Value::String(s) => matches!(s.as_str(), "true" | "True" | "1" | "yes" | "on" | "真"),
            Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
            _ => false,
        };
        m.insert("access_log".into(), json!(on));
    }
    if let Some(cc) = args
        .get("cache_control")
        .or_else(|| args.get("缓存控制"))
        .and_then(|v| v.as_str())
    {
        let s = cc.trim();
        if !s.is_empty() {
            m.insert("cache_control".into(), json!(s));
        }
    }
    if let Some(bl) = args.get("body_limit") {
        let n = bl
            .as_u64()
            .or_else(|| bl.as_i64().map(|i| i as u64))
            .unwrap_or(0);
        if n > 0 {
            m.insert("body_limit".into(), json!(n));
        }
    }
    if let Some(routes) = args.get("json_routes") {
        m.insert(
            "json_routes".into(),
            middleware::json_routes_from_table(routes),
        );
    }
    Ok(app)
});

web_ffi!(web_form_new, |args: &Value| {
    let table = arg_str_opt(args, "table");
    let action = arg_str_opt(args, "action").unwrap_or("insert");
    let id = arg_str_opt(args, "id");
    Ok(form::form_new(table, action, id))
});

web_ffi!(web_form_fields, |args: &Value| {
    let form_v = args.get("form").cloned().unwrap_or(json!({}));
    let fields = args
        .get("fields")
        .cloned()
        .ok_or_else(|| "missing `fields`".to_string())?;
    Ok(form::set_fields(&form_v, &fields))
});

web_ffi!(web_form_rules, |args: &Value| {
    let form_v = args.get("form").cloned().unwrap_or(json!({}));
    let rules = args
        .get("rules")
        .cloned()
        .ok_or_else(|| "missing `rules`".to_string())?;
    Ok(form::set_rules(&form_v, &rules))
});

web_ffi!(web_form_validate, |args: &Value| {
    let form_v = args.get("form").cloned().unwrap_or(json!({}));
    let data = args
        .get("data")
        .cloned()
        .ok_or_else(|| "missing `data`".to_string())?;
    let rules = args.get("rules");
    Ok(form::validate(&form_v, rules, &data))
});

web_ffi!(web_form_render, |args: &Value| {
    let form_v = args.get("form").cloned().unwrap_or(json!({}));
    let id = arg_str_opt(args, "id").unwrap_or("form");
    let data = args.get("data");
    let errors = args.get("errors");
    Ok(Value::String(form::render(
        &form_v,
        id,
        data,
        errors,
        arg_str_opt(args, "csrf"),
    )))
});

web_ffi!(web_form_submit, |args: &Value| {
    let form_v = args.get("form").cloned().unwrap_or(json!({}));
    let data = args
        .get("data")
        .cloned()
        .ok_or_else(|| "missing `data`".to_string())?;
    let url = db_url_of(args)?;
    form::submit(&form_v, &data, &url)
});

web_ffi!(web_form_from_schema, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let action = arg_str_opt(args, "action").unwrap_or("insert");
    let id = match args.get("id") {
        None | Some(Value::Null) => None,
        Some(_) => Some(arg_text(args, "id")?),
    };
    form::from_schema(&url, &table, action, id.as_deref())
});

web_ffi!(web_db_table_info, |args: &Value| {
    let url = db_url_of(args)?;
    let table = arg_str(args, "table")?.to_string();
    let cols = db::table_info(&url, &table)?;
    let arr: Vec<Value> = cols
        .into_iter()
        .map(|c| {
            json!({
                "name": c.name,
                "type": c.sql_type,
                "notnull": c.notnull,
                "pk": c.pk,
            })
        })
        .collect();
    Ok(json!({ "columns": arr }))
});

web_ffi!(web_listen, |args: &Value| {
    use std::collections::HashMap;
    use std::path::PathBuf;
    let (
        page,
        db_url,
        host,
        port,
        admin,
        forms,
        routes,
        static_dir,
        static_mount,
        auth_users,
        session_ttl,
        ws_routes,
        rss_routes,
        upload_routes,
        download_routes,
        redirects,
        sitemap_routes,
        robots_body,
        page_404,
        page_500,
        gates,
        gallery_routes,
        middleware,
        cookie_secure,
    ) = if args.get("page").is_some() || args.get("host").is_some() {
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
        let static_dir = arg_str_opt(args, "static_dir").map(PathBuf::from);
        let static_mount = arg_str_opt(args, "static_mount")
            .unwrap_or("/static")
            .to_string();
        let auth_users = args
            .get("auth")
            .or_else(|| args.get("users"))
            .cloned()
            .filter(|v| !matches!(v, Value::Null));
        let session_ttl = args
            .get("session_ttl")
            .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
            .unwrap_or(3600);
        let cookie_secure = args
            .get("cookie_secure")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        (
            page,
            db_url,
            host.to_string(),
            port,
            admin,
            HashMap::new(),
            HashMap::new(),
            static_dir,
            static_mount,
            auth_users,
            session_ttl,
            HashMap::<String, crate::ws_hub::WsMode>::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            None,
            None,
            None,
            Vec::<(String, Vec<String>)>::new(),
            HashMap::new(),
            middleware::Middleware::default(),
            cookie_secure,
        )
    } else {
        let app = args.get("app").cloned().unwrap_or_else(|| args.clone());
        let page = app.get("page").cloned().unwrap_or(json!({}));
        let db_url = app
            .get("db")
            .and_then(|d| d.get("url"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| arg_str_opt(&app, "db_url").map(|s| s.to_string()))
            .map(|s| resolve_db_url(&s));
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
        let mut forms = HashMap::new();
        if let Some(obj) = app.get("forms").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                forms.insert(k.clone(), v.clone());
            }
        }
        let mut routes = HashMap::new();
        if let Some(obj) = app.get("routes").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                routes.insert(k.clone(), v.clone());
            }
        }
        let mut ws_routes = HashMap::new();
        if let Some(obj) = app.get("ws_routes").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                ws_routes.insert(k.clone(), crate::ws_hub::WsMode::parse(v));
            }
        }
        let mut rss_routes = HashMap::new();
        if let Some(obj) = app.get("rss_routes").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                rss_routes.insert(k.clone(), v.clone());
            }
        }
        let mut upload_routes = HashMap::new();
        if let Some(obj) = app.get("upload_routes").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                let field = v
                    .get("field")
                    .and_then(|x| x.as_str())
                    .unwrap_or("file")
                    .to_string();
                let storage_url = v
                    .get("storage_url")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let prefix = v
                    .get("prefix")
                    .and_then(|x| x.as_str())
                    .unwrap_or("uploads/")
                    .to_string();
                let max_bytes = v
                    .get("max_bytes")
                    .and_then(|x| x.as_u64().or_else(|| x.as_i64().map(|i| i as u64)))
                    .unwrap_or(5_242_880);
                let types = v.get("types").cloned().filter(|t| !matches!(t, Value::Null));
                upload_routes.insert(
                    k.clone(),
                    http::UploadRoute {
                        field,
                        storage_url,
                        prefix,
                        max_bytes,
                        types,
                    },
                );
            }
        }
        let mut download_routes = HashMap::new();
        if let Some(obj) = app.get("download_routes").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                let storage_url = v
                    .get("storage_url")
                    .and_then(|x| x.as_str())
                    .unwrap_or("")
                    .to_string();
                let disposition = v
                    .get("disposition")
                    .and_then(|x| x.as_str())
                    .unwrap_or("attachment")
                    .to_string();
                download_routes.insert(
                    k.clone(),
                    http::DownloadRoute {
                        storage_url,
                        disposition,
                    },
                );
            }
        }
        let mut redirects = HashMap::new();
        if let Some(obj) = app.get("redirects").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                let to = v
                    .get("to")
                    .and_then(|x| x.as_str())
                    .unwrap_or("/")
                    .to_string();
                let permanent = v
                    .get("permanent")
                    .and_then(|x| x.as_bool())
                    .unwrap_or(false);
                redirects.insert(k.clone(), (to, permanent));
            }
        }
        let mut sitemap_routes = HashMap::new();
        if let Some(obj) = app.get("sitemap_routes").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                sitemap_routes.insert(k.clone(), v.clone());
            }
        }
        let robots_body = app
            .get("robots_body")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let page_404 = app.get("page_404").cloned().filter(|v| !v.is_null());
        let page_500 = app.get("page_500").cloned().filter(|v| !v.is_null());
        let mut gates: Vec<(String, Vec<String>)> = Vec::new();
        if let Some(arr) = app.get("gates").and_then(|v| v.as_array()) {
            for g in arr {
                let path = g
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if path.is_empty() {
                    continue;
                }
                let roles = match g.get("roles") {
                    Some(Value::Array(a)) => a
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_ascii_lowercase()))
                        .collect(),
                    Some(Value::String(s)) => session::parse_roles_csv(s),
                    _ => vec!["admin".into()],
                };
                gates.push((path, roles));
            }
        }
        let mut gallery_routes = HashMap::new();
        if let Some(obj) = app.get("gallery_routes").and_then(|v| v.as_object()) {
            for (k, v) in obj {
                gallery_routes.insert(k.clone(), v.clone());
            }
        }
        if app.get("tls_cert").is_some() || app.get("tls_key").is_some() {
            eprintln!(
                "marqdo web: in-process TLS is not enabled; terminate HTTPS at a reverse proxy (nginx/caddy) and set cookie_secure=True"
            );
        }
        let static_dir = app
            .get("static_dir")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from);
        let static_mount = app
            .get("static_mount")
            .and_then(|v| v.as_str())
            .unwrap_or("/static")
            .to_string();
        // `auth` may hold the users table directly or `{users:…, ttl:…}`.
        let (auth_users, session_ttl) = match app.get("auth") {
            Some(Value::Object(m)) if m.contains_key("users") => (
                m.get("users").cloned(),
                m.get("ttl")
                    .or_else(|| m.get("session_ttl"))
                    .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
                    .unwrap_or(3600),
            ),
            Some(other) if !matches!(other, Value::Null) => (
                Some(other.clone()),
                app.get("session_ttl")
                    .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
                    .unwrap_or(3600),
            ),
            _ => (
                None,
                app.get("session_ttl")
                    .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
                    .unwrap_or(3600),
            ),
        };
        let middleware = middleware::parse(&app);
        let mw_summary = middleware::summary(&app);
        if !mw_summary.is_empty() {
            eprintln!("marqdo web middleware: {mw_summary}");
        }
        let cookie_secure = app
            .get("cookie_secure")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        (
            page,
            db_url,
            host,
            port,
            admin,
            forms,
            routes,
            static_dir,
            static_mount,
            auth_users,
            session_ttl,
            ws_routes,
            rss_routes,
            upload_routes,
            download_routes,
            redirects,
            sitemap_routes,
            robots_body,
            page_404,
            page_500,
            gates,
            gallery_routes,
            middleware,
            cookie_secure,
        )
    };
    let static_dir = static_dir.map(|p| {
        if p.is_absolute() {
            p
        } else {
            entry_dir().join(p)
        }
    });
    http::listen(
        &page,
        db_url.as_deref(),
        &host,
        port,
        admin,
        forms,
        routes,
        static_dir,
        &static_mount,
        auth_users,
        session_ttl,
        cookie_secure,
        ws_routes,
        rss_routes,
        upload_routes,
        download_routes,
        redirects,
        sitemap_routes,
        robots_body,
        page_404,
        page_500,
        gates,
        gallery_routes,
        &middleware,
    )
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
        (
            "web_page_query",
            "page,query",
            web_page_query as PluginFn,
        ),
        (
            "web_page_order",
            "page,order",
            web_page_order as PluginFn,
        ),
        (
            "web_page_link_prefix",
            "page,prefix",
            web_page_link_prefix as PluginFn,
        ),
        (
            "web_page_css",
            "page,css",
            web_page_css as PluginFn,
        ),
        (
            "web_page_detail",
            "page,detail",
            web_page_detail as PluginFn,
        ),
        (
            "web_page_meta",
            "page,meta",
            web_page_meta as PluginFn,
        ),
        (
            "web_page_paginate",
            "page,offset,limit,path",
            web_page_paginate as PluginFn,
        ),
        (
            "web_rss_build",
            "title,link,description,items",
            web_rss_build as PluginFn,
        ),
        (
            "web_app_route_rss",
            "app,path,table,limit,order,title,link,description",
            web_app_route_rss as PluginFn,
        ),
        (
            "web_app_redirect",
            "app,from,to,permanent",
            web_app_redirect as PluginFn,
        ),
        (
            "web_app_error_page",
            "app,status,page",
            web_app_error_page as PluginFn,
        ),
        (
            "web_app_sitemap",
            "app,path,base,table,loc,limit,items",
            web_app_sitemap as PluginFn,
        ),
        ("web_app_robots", "app,body,sitemap", web_app_robots as PluginFn),
        ("web_sitemap_build", "base,items", web_sitemap_build as PluginFn),
        (
            "web_app_upload",
            "app,path,field,storage,prefix,max_bytes,types",
            web_app_upload as PluginFn,
        ),
        (
            "web_app_download",
            "app,path,storage,disposition",
            web_app_download as PluginFn,
        ),
        (
            "web_upload_validate",
            "filename,content_type,size,max_bytes,types",
            web_upload_validate as PluginFn,
        ),
        (
            "web_upload_save",
            "storage,path,key,content_type,prefix",
            web_upload_save as PluginFn,
        ),
        ("web_media_new", "storage", web_media_new as PluginFn),
        (
            "web_style",
            "name,table",
            web_style as PluginFn,
        ),
        (
            "web_compose_form",
            "page,form,id",
            web_compose_form as PluginFn,
        ),
        ("web_render", "page", web_render as PluginFn),
        ("web_db_new", "url", web_db_new as PluginFn),
        ("web_cache_new", "url", web_cache_new as PluginFn),
        ("web_cache_get", "url,key", web_cache_get as PluginFn),
        ("web_cache_set", "url,key,value,ttl", web_cache_set as PluginFn),
        ("web_cache_del", "url,key", web_cache_del as PluginFn),
        ("web_cache_exists", "url,key", web_cache_exists as PluginFn),
        ("web_cache_ttl", "url,key", web_cache_ttl as PluginFn),
        ("web_storage_new", "url", web_storage_new as PluginFn),
        (
            "web_storage_put",
            "url,key,body,path,content_type",
            web_storage_put as PluginFn,
        ),
        ("web_storage_get", "url,key", web_storage_get as PluginFn),
        ("web_storage_delete", "url,key", web_storage_delete as PluginFn),
        ("web_storage_list", "url,prefix", web_storage_list as PluginFn),
        ("web_db_init", "url,name,fields", web_db_init as PluginFn),
        ("web_db_insert", "url,table,rows,txn", web_db_insert as PluginFn),
        (
            "web_db_select",
            "url,table,where,limit,order,offset,txn",
            web_db_select as PluginFn,
        ),
        (
            "web_db_query",
            "url,sql,args,txn",
            web_db_query as PluginFn,
        ),
        (
            "web_db_count",
            "url,table,where,txn",
            web_db_count as PluginFn,
        ),
        ("web_db_migrate", "url,steps", web_db_migrate as PluginFn),
        (
            "web_db_fts_create",
            "url,table,columns,name",
            web_db_fts_create as PluginFn,
        ),
        (
            "web_db_search",
            "url,table,q,limit,name",
            web_db_search as PluginFn,
        ),
        ("web_db_begin", "url", web_db_begin as PluginFn),
        ("web_db_commit", "txn", web_db_commit as PluginFn),
        ("web_db_rollback", "txn", web_db_rollback as PluginFn),
        ("web_db_get", "url,table,id,txn", web_db_get as PluginFn),
        ("web_db_update", "url,table,id,row,txn", web_db_update as PluginFn),
        ("web_db_delete", "url,table,id,txn", web_db_delete as PluginFn),
        ("web_db_exec", "url,sql,args,txn", web_db_exec as PluginFn),
        ("web_db_list_tables", "url", web_db_list_tables as PluginFn),
        (
            "web_app_new",
            "page,db,admin,host,port",
            web_app_new as PluginFn,
        ),
        ("web_app_route", "app,path,page", web_app_route as PluginFn),
        (
            "web_app_mount_form",
            "app,id,form",
            web_app_mount_form as PluginFn,
        ),
        (
            "web_app_static",
            "app,dir,mount",
            web_app_static as PluginFn,
        ),
        (
            "web_app_middleware",
            "app,cors,security,compress,body_limit,json_routes,access_log,cache_control",
            web_app_middleware as PluginFn,
        ),
        ("web_form_new", "table,action,id", web_form_new as PluginFn),
        ("web_form_fields", "form,fields", web_form_fields as PluginFn),
        ("web_form_rules", "form,rules", web_form_rules as PluginFn),
        (
            "web_form_validate",
            "form,rules,data",
            web_form_validate as PluginFn,
        ),
        ("web_form_render", "form,id", web_form_render as PluginFn),
        ("web_form_submit", "form,data,url", web_form_submit as PluginFn),
        (
            "web_form_from_schema",
            "url,table,action",
            web_form_from_schema as PluginFn,
        ),
        ("web_db_table_info", "url,table", web_db_table_info as PluginFn),
        ("web_session_new", "ttl_sec", web_session_new as PluginFn),
        ("web_session_set", "id,key,value", web_session_set as PluginFn),
        ("web_session_get", "id,key", web_session_get as PluginFn),
        ("web_session_del", "id,key", web_session_del as PluginFn),
        ("web_session_destroy", "id", web_session_destroy as PluginFn),
        (
            "web_auth_login",
            "username,password,users,session_ttl",
            web_auth_login as PluginFn,
        ),
        ("web_auth_check", "session_id", web_auth_check as PluginFn),
        ("web_auth_logout", "session_id", web_auth_logout as PluginFn),
        ("web_password_hash", "password", web_password_hash as PluginFn),
        ("web_auth_new", "users,session_ttl", web_auth_new as PluginFn),
        ("web_app_auth", "app,users,session_ttl", web_app_auth as PluginFn),
        ("web_app_gate", "app,path,roles", web_app_gate as PluginFn),
        (
            "web_app_gallery",
            "app,path,storage,prefix,title,download_base",
            web_app_gallery as PluginFn,
        ),
        (
            "web_app_route_ws",
            "app,path,echo,mode",
            web_app_route_ws as PluginFn,
        ),
        (
            "web_ws_connect",
            "url,message,headers,timeout_sec",
            web_ws_connect as PluginFn,
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
