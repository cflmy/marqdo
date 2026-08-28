//! Declarative middleware for the web app.
//!
//! All cross-cutting HTTP capabilities (CORS, security headers, compression,
//! body limits, JSON API routes) are configured as *data* on the `app` object
//! and applied to the axum router here. This keeps the authoring surface
//! declarative: a GFM table in Marqdo becomes a `middleware` map, and this
//! module turns it into axum layers — 配置即数据、装配即函数.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderValue, Method, Request, StatusCode};
use axum::middleware::{self as axum_mw, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde_json::{json, Map, Value};
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::db;

/// Render a cell value as text (strings stay, numbers/bools stringify).
fn cell_text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// Parsed middleware configuration (from `app["middleware"]`).
#[derive(Clone, Default)]
pub struct Middleware {
    pub cors: Option<CorsConfig>,
    pub security: Vec<(String, String)>,
    pub compress: bool,
    pub body_limit: Option<u64>,
    /// Log `METHOD path status duration_ms` to stderr for each request.
    pub access_log: bool,
    /// Global `Cache-Control` response header (e.g. `public, max-age=3600`).
    pub cache_control: Option<String>,
    /// JSON API routes: `path -> { method, table, where?, order?, limit? }`.
    pub json_routes: Vec<(String, JsonRoute)>,
}

#[derive(Clone)]
pub struct CorsConfig {
    pub allow_origins: Vec<String>,
    pub methods: Vec<String>,
    pub headers: Vec<String>,
    pub expose_headers: Vec<String>,
    pub credentials: bool,
}

#[derive(Clone)]
pub struct JsonRoute {
    pub method: String,
    pub table: String,
    pub where_v: Option<Value>,
    pub order: Option<String>,
    pub limit: i64,
}

/// Parse `app["middleware"]` (a map) into a `Middleware`.
pub fn parse(app: &Value) -> Middleware {
    let mut mw = Middleware::default();
    let Some(obj) = app.get("middleware").and_then(|v| v.as_object()) else {
        return mw;
    };
    if let Some(c) = obj.get("cors") {
        mw.cors = parse_cors(c);
    }
    if let Some(sec) = obj.get("security").and_then(|v| v.as_object()) {
        mw.security = sec
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .filter(|(_, v)| !v.is_empty())
            .collect();
    }
    mw.compress = obj
        .get("compress")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    mw.access_log = obj
        .get("access_log")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    mw.cache_control = obj
        .get("cache_control")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    mw.body_limit = obj
        .get("body_limit")
        .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|i| i as u64)))
        .filter(|n| *n > 0);
    if let Some(routes) = obj.get("json_routes").and_then(|v| v.as_object()) {
        for (path, spec) in routes {
            if let Some(route) = parse_json_route(path, spec) {
                mw.json_routes.push(route);
            }
        }
        mw.json_routes.sort_by(|a, b| a.0.cmp(&b.0));
    }
    mw
}

fn parse_cors(v: &Value) -> Option<CorsConfig> {
    let obj = v.as_object()?;
    let str_list = |key: &str| -> Vec<String> {
        obj.get(key)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    };
    Some(CorsConfig {
        allow_origins: str_list("allow_origins"),
        methods: str_list("methods"),
        headers: str_list("headers"),
        expose_headers: str_list("expose_headers"),
        credentials: obj
            .get("credentials")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    })
}

fn parse_json_route(path: &str, spec: &Value) -> Option<(String, JsonRoute)> {
    let spec = spec.as_object()?;
    let method = spec
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET")
        .to_uppercase();
    let table = spec.get("table")?.as_str()?.to_string();
    let order = spec.get("order").and_then(|v| v.as_str()).map(|s| s.to_string());
    let limit = spec
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(200);
    Some((
        format!("/{}", path.trim_start_matches('/')),
        JsonRoute {
            method,
            table,
            where_v: spec.get("where").cloned(),
            order,
            limit,
        },
    ))
}

/// Apply the middleware configuration to a router (state already provided).
pub fn apply(
    router: Router<Arc<crate::http::AppState>>,
    mw: &Middleware,
) -> Router<Arc<crate::http::AppState>> {
    let mut app = router;

    // JSON API routes first: axum layers only wrap routes registered before
    // the `.layer()` call, so these must be mounted ahead of the pipeline.
    for (path, route) in &mw.json_routes {
        app = mount_json(app, path, route.clone());
    }

    // CORS
    if let Some(c) = &mw.cors {
        let mut layer = CorsLayer::new();
        if c.allow_origins.is_empty() {
            layer = layer.allow_origin(Any);
        } else {
            layer = layer.allow_origin(
                c.allow_origins
                    .iter()
                    .filter_map(|o| o.parse::<HeaderValue>().ok())
                    .collect::<Vec<_>>(),
            );
        }
        if c.methods.is_empty() {
            layer = layer.allow_methods(Any);
        } else {
            layer = layer.allow_methods(
                c.methods
                    .iter()
                    .filter_map(|m| m.parse::<Method>().ok())
                    .collect::<Vec<_>>(),
            );
        }
        if c.headers.is_empty() {
            layer = layer.allow_headers(Any);
        } else {
            layer = layer.allow_headers(
                c.headers
                    .iter()
                    .filter_map(|h| h.parse::<axum::http::HeaderName>().ok())
                    .collect::<Vec<_>>(),
            );
        }
        if !c.expose_headers.is_empty() {
            layer = layer.expose_headers(
                c.expose_headers
                    .iter()
                    .filter_map(|h| h.parse::<axum::http::HeaderName>().ok())
                    .collect::<Vec<_>>(),
            );
        }
        layer = layer.allow_credentials(c.credentials);
        app = app.layer(layer);
    }

    // Security response headers
    for (name, value) in &mw.security {
        let Some(header_name) = header_name_from(&name.to_ascii_lowercase()) else {
            continue;
        };
        if let Ok(v) = HeaderValue::from_str(value) {
            app = app.layer(SetResponseHeaderLayer::overriding(header_name, v));
        }
    }

    // Compression
    if mw.compress {
        app = app.layer(CompressionLayer::new());
    }

    if let Some(ref cc) = mw.cache_control {
        if let Ok(v) = HeaderValue::from_str(cc) {
            app = app.layer(SetResponseHeaderLayer::overriding(
                header::CACHE_CONTROL,
                v,
            ));
        }
    }

    // Request body limit
    if let Some(bytes) = mw.body_limit {
        app = app.layer(RequestBodyLimitLayer::new(bytes as usize));
    }

    // Access log (outermost so status/duration include other layers).
    if mw.access_log {
        app = app.layer(axum_mw::from_fn(access_log_mw));
    }

    app
}

async fn access_log_mw(req: Request<Body>, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = std::time::Instant::now();
    let resp = next.run(req).await;
    eprintln!(
        "marqdo web {} {} {} {}ms",
        method,
        path,
        resp.status().as_u16(),
        start.elapsed().as_millis()
    );
    resp
}

fn header_name_from(name: &str) -> Option<axum::http::HeaderName> {
    let h = match name {
        "x_frame_options" | "x-frame-options" => header::X_FRAME_OPTIONS,
        "content_security_policy" | "content-security-policy" => header::CONTENT_SECURITY_POLICY,
        "x_content_type_options" | "x-content-type-options" => header::X_CONTENT_TYPE_OPTIONS,
        "referrer_policy" | "referrer-policy" => header::REFERRER_POLICY,
        "strict_transport_security" | "strict-transport-security" | "hsts" => {
            header::STRICT_TRANSPORT_SECURITY
        }
        "x_xss_protection" | "x-xss-protection" => header::X_XSS_PROTECTION,
        _ => return None,
    };
    Some(h)
}

/// Mount a JSON API route backed by a db query.
fn mount_json(
    app: Router<Arc<crate::http::AppState>>,
    path: &str,
    route: JsonRoute,
) -> Router<Arc<crate::http::AppState>> {
    let handler = match route.method.as_str() {
        "POST" => post(move |State(st): State<Arc<crate::http::AppState>>| {
            json_handler(st, route.clone())
        }),
        _ => get(move |State(st): State<Arc<crate::http::AppState>>| {
            json_handler(st, route.clone())
        }),
    };
    app.route(path, handler)
}

async fn json_handler(st: Arc<crate::http::AppState>, route: JsonRoute) -> Response {
    let Some(url) = st.db_url.as_deref() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonBody(json!({ "error": "no database" })),
        )
            .into_response();
    };
    let res = match &route.order {
        Some(o) if !o.trim().is_empty() => {
            db::select_order(url, &route.table, route.limit, route.where_v.as_ref(), Some(o), None, None)
        }
        _ => db::select(url, &route.table, route.limit, route.where_v.as_ref()),
    };
    match res {
        Ok(data) => JsonBody(data).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            JsonBody(json!({ "error": e })),
        )
            .into_response(),
    }
}

/// A JSON body response wrapper (sets `Content-Type: application/json`).
pub struct JsonBody(pub Value);

impl IntoResponse for JsonBody {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .body(Body::from(self.0.to_string()))
            .unwrap()
    }
}

/// Normalize a `security` table into the `middleware.security` map shape.
/// Accepts a GFM table `|头|值|` (list of rows) or an already-shaped map.
pub fn security_from_table(table: &Value) -> Value {
    let rows = crate::table::as_rows(table);
    let mut m = Map::new();
    if let Some(arr) = rows.as_array() {
        for row in arr {
            let col = |keys: &[&str]| -> String {
                keys.iter()
                    .find_map(|k| row.get(*k).map(cell_text))
                    .unwrap_or_default()
            };
            let k = col(&["头", "header", "Header", "name"]);
            let v = col(&["值", "value", "Value"]);
            if !k.is_empty() {
                m.insert(k, json!(v));
            }
        }
    }
    Value::Object(m)
}

/// Normalize a CORS table `|允许来源|方法|头|暴露头|凭证|` (one row per origin)
/// into the `middleware.cors` map shape.
pub fn cors_from_table(table: &Value) -> Value {
    let rows = crate::table::as_rows(table);
    let mut allow_origins = Vec::new();
    let mut methods = Vec::new();
    let mut headers = Vec::new();
    let mut expose_headers = Vec::new();
    let mut credentials = false;
    if let Some(arr) = rows.as_array() {
        for row in arr {
            let col = |keys: &[&str]| -> String {
                keys.iter()
                    .find_map(|k| row.get(*k).map(cell_text))
                    .unwrap_or_default()
            };
            let origin = col(&["允许来源", "origin", "Origin"]);
            if !origin.is_empty() && origin != "*" {
                allow_origins.push(origin);
            }
            for m in col(&["方法", "methods", "Methods"]).split(',') {
                let m = m.trim();
                if !m.is_empty() {
                    methods.push(m.to_string());
                }
            }
            for h in col(&["头", "headers", "Headers"]).split(',') {
                let h = h.trim();
                if !h.is_empty() {
                    headers.push(h.to_string());
                }
            }
            for h in col(&["暴露头", "expose_headers", "Expose-Headers"]).split(',') {
                let h = h.trim();
                if !h.is_empty() {
                    expose_headers.push(h.to_string());
                }
            }
            let cred = col(&["凭证", "credentials", "Credentials"]);
            if cred.eq_ignore_ascii_case("true")
                || cred == "1"
                || cred.eq_ignore_ascii_case("yes")
            {
                credentials = true;
            }
        }
    }
    json!({
        "allow_origins": allow_origins,
        "methods": methods,
        "headers": headers,
        "expose_headers": expose_headers,
        "credentials": credentials,
    })
}

/// Normalize a JSON-routes table into `middleware.json_routes` map shape.
/// Accepts a GFM table `|路径|方法|表|条件|排序|上限|` (rows).
pub fn json_routes_from_table(table: &Value) -> Value {
    let rows = crate::table::as_rows(table);
    let mut routes = Map::new();
    if let Some(arr) = rows.as_array() {
        for row in arr {
            let col = |keys: &[&str]| -> String {
                keys.iter()
                    .find_map(|k| row.get(*k).map(cell_text))
                    .unwrap_or_default()
            };
            let path = col(&["路径", "path", "Path"]).trim_start_matches('/').to_string();
            if path.is_empty() {
                continue;
            }
            let method = col(&["方法", "method", "Method"]).to_uppercase();
            let method = if method.is_empty() { "GET".into() } else { method };
            let table = col(&["表", "table", "Table"]);
            if table.is_empty() {
                continue;
            }
            let order = col(&["排序", "order", "Order"]);
            let limit = col(&["上限", "limit", "Limit"]).parse::<i64>().unwrap_or(200);
            let mut spec = Map::new();
            spec.insert("method".into(), json!(method));
            spec.insert("table".into(), json!(table));
            if !order.is_empty() {
                spec.insert("order".into(), json!(order));
            }
            spec.insert("limit".into(), json!(limit));
            routes.insert(path, Value::Object(spec));
        }
    }
    Value::Object(routes)
}

/// Read the `middleware` map out of an app object (for logging/debug).
pub fn summary(app: &Value) -> String {
    let mw = parse(app);
    let mut parts = Vec::new();
    if mw.cors.is_some() {
        parts.push("cors".to_string());
    }
    if !mw.security.is_empty() {
        parts.push("security".to_string());
    }
    if mw.compress {
        parts.push("compress".to_string());
    }
    if mw.body_limit.is_some() {
        parts.push("body_limit".to_string());
    }
    if mw.access_log {
        parts.push("access_log".to_string());
    }
    if mw.cache_control.is_some() {
        parts.push("cache_control".to_string());
    }
    if !mw.json_routes.is_empty() {
        parts.push(format!("json:{}", mw.json_routes.len()));
    }
    if parts.is_empty() {
        String::new()
    } else {
        parts.join(" + ")
    }
}
