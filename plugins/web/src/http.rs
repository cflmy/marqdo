//! Async HTTP server (tokio + axum).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Path as AxPath, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Form, Router};
use serde_json::{json, Value};
use tower_http::services::ServeDir;

use crate::db;
use crate::page;

#[derive(Clone)]
struct AppState {
    /// Pre-rendered static HTML (optional).
    pages: HashMap<String, String>,
    /// Live page configs re-rendered per request (path → page JSON).
    page_cfgs: HashMap<String, Value>,
    db_url: Option<String>,
    admin_tables: Vec<String>,
}

fn arg_str(args: &Value, key: &str, default: &str) -> String {
    args.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

fn arg_u16(args: &Value, key: &str, default: u16) -> u16 {
    match args.get(key) {
        Some(Value::Number(n)) => n.as_u64().unwrap_or(default as u64) as u16,
        Some(Value::String(s)) => s.parse().unwrap_or(default),
        _ => default,
    }
}

fn normalize_path(p: &str) -> String {
    if p.is_empty() || p == "/" {
        return "/".into();
    }
    let mut s = p.to_string();
    if !s.starts_with('/') {
        s.insert(0, '/');
    }
    if s.len() > 1 && s.ends_with('/') {
        s.pop();
    }
    s
}

fn build_pages(args: &Value) -> (HashMap<String, String>, HashMap<String, Value>) {
    let mut pages = HashMap::new();
    let mut cfgs = HashMap::new();

    if let Some(page_cfg) = args.get("page") {
        cfgs.insert("/".into(), page_cfg.clone());
    }

    if let Some(routes) = args.get("routes").and_then(|v| v.as_array()) {
        for r in routes {
            let path = r
                .get("path")
                .or_else(|| r.get("href"))
                .and_then(|v| v.as_str())
                .unwrap_or("/");
            let path = normalize_path(path);
            if let Some(cfg) = r.get("page") {
                cfgs.insert(path.clone(), cfg.clone());
            } else if let Some(h) = r.get("html").and_then(|v| v.as_str()) {
                pages.insert(path, h.to_string());
            } else if r.get("nav").is_some() || r.get("main").is_some() || r.get("intro").is_some()
            {
                cfgs.insert(path, r.clone());
            } else {
                pages.insert(path, page::render_page(r));
            }
        }
    }

    if pages.is_empty() && cfgs.is_empty() {
        pages.insert("/".into(), page::default_hello_html());
    }
    (pages, cfgs)
}

fn resolve_admin_tables(args: &Value, db_url: Option<&str>) -> Vec<String> {
    let from_args: Vec<String> = args
        .get("admin_tables")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();
    if !from_args.is_empty() {
        return from_args;
    }
    let Some(url) = db_url else {
        return Vec::new();
    };
    db::list_user_tables(url).unwrap_or_default()
}

fn render_live(st: &AppState, cfg: &Value) -> String {
    page::render_page_with_db(cfg, st.db_url.as_deref())
}

async fn serve_page(
    State(st): State<Arc<AppState>>,
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
) -> Response {
    let path = normalize_path(uri.path());
    if let Some(cfg) = st.page_cfgs.get(&path) {
        return Html(render_live(&st, cfg)).into_response();
    }
    if let Some(html) = st.pages.get(&path) {
        return Html(html.clone()).into_response();
    }
    (
        StatusCode::NOT_FOUND,
        Html(page::render_page(&json!({
            "title": "404",
            "main": "<h1>404</h1><p>Not found</p>",
            "nav": [{"label": "Home", "href": "/"}],
        }))),
    )
        .into_response()
}

fn admin_tables_now(st: &AppState) -> Vec<String> {
    if !st.admin_tables.is_empty() {
        return st.admin_tables.clone();
    }
    let Some(url) = st.db_url.as_ref() else {
        return Vec::new();
    };
    db::list_user_tables(url).unwrap_or_default()
}

async fn admin_home(State(st): State<Arc<AppState>>) -> Response {
    let tables = admin_tables_now(&st);
    Html(db::admin_home_html(st.db_url.as_deref(), &tables)).into_response()
}

async fn admin_log_page(State(st): State<Arc<AppState>>) -> Response {
    let Some(url) = st.db_url.as_ref() else {
        return Html("<p>No database</p>").into_response();
    };
    match db::admin_log_html(url) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(page::html_escape(&e)),
        )
            .into_response(),
    }
}

async fn admin_table(State(st): State<Arc<AppState>>, AxPath(table): AxPath<String>) -> Response {
    let Some(url) = st.db_url.as_ref() else {
        return Html("<p>No database</p>").into_response();
    };
    let tables = admin_tables_now(&st);
    if !tables.iter().any(|t| t == &table) {
        return (StatusCode::NOT_FOUND, Html("<p>Unknown table</p>")).into_response();
    }
    match db::admin_list_html(url, &table) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(page::html_escape(&e)),
        )
            .into_response(),
    }
}

async fn admin_new_get(State(st): State<Arc<AppState>>, AxPath(table): AxPath<String>) -> Response {
    let Some(url) = st.db_url.as_ref() else {
        return Html("<p>No database</p>").into_response();
    };
    let tables = admin_tables_now(&st);
    if !tables.iter().any(|t| t == &table) {
        return (StatusCode::NOT_FOUND, "unknown").into_response();
    }
    match db::admin_new_form_html(url, &table) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Html(page::html_escape(&e))).into_response(),
    }
}

async fn admin_new_post(
    State(st): State<Arc<AppState>>,
    AxPath(table): AxPath<String>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    let Some(url) = st.db_url.as_ref() else {
        return Html("<p>No database</p>").into_response();
    };
    let tables = admin_tables_now(&st);
    if !tables.iter().any(|t| t == &table) {
        return (StatusCode::NOT_FOUND, "unknown").into_response();
    }
    let mut map = serde_json::Map::new();
    for (k, v) in form {
        if k == "id" {
            continue;
        }
        map.insert(k, Value::String(v));
    }
    match db::insert_row(url, &table, &Value::Object(map)) {
        Ok(_) => Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, format!("/admin/{table}"))
            .body(Body::empty())
            .unwrap(),
        Err(e) => (StatusCode::BAD_REQUEST, Html(page::html_escape(&e))).into_response(),
    }
}

async fn admin_edit_get(
    State(st): State<Arc<AppState>>,
    AxPath((table, id)): AxPath<(String, String)>,
) -> Response {
    let Some(url) = st.db_url.as_ref() else {
        return Html("<p>No database</p>").into_response();
    };
    let tables = admin_tables_now(&st);
    if !tables.iter().any(|t| t == &table) {
        return (StatusCode::NOT_FOUND, "unknown").into_response();
    }
    match db::admin_edit_form_html(url, &table, &id) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, Html(page::html_escape(&e))).into_response(),
    }
}

async fn admin_edit_post(
    State(st): State<Arc<AppState>>,
    AxPath((table, id)): AxPath<(String, String)>,
    Form(form): Form<HashMap<String, String>>,
) -> Response {
    let Some(url) = st.db_url.as_ref() else {
        return Html("<p>No database</p>").into_response();
    };
    let tables = admin_tables_now(&st);
    if !tables.iter().any(|t| t == &table) {
        return (StatusCode::NOT_FOUND, "unknown").into_response();
    }
    let mut map = serde_json::Map::new();
    for (k, v) in form {
        if k == "id" {
            continue;
        }
        map.insert(k, Value::String(v));
    }
    match db::update_row(url, &table, &id, &Value::Object(map)) {
        Ok(_) => Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, format!("/admin/{table}"))
            .body(Body::empty())
            .unwrap(),
        Err(e) => (StatusCode::BAD_REQUEST, Html(page::html_escape(&e))).into_response(),
    }
}

async fn admin_delete_post(
    State(st): State<Arc<AppState>>,
    AxPath((table, id)): AxPath<(String, String)>,
) -> Response {
    let Some(url) = st.db_url.as_ref() else {
        return Html("<p>No database</p>").into_response();
    };
    let tables = admin_tables_now(&st);
    if !tables.iter().any(|t| t == &table) {
        return (StatusCode::NOT_FOUND, "unknown").into_response();
    }
    match db::delete_row(url, &table, &id) {
        Ok(_) => Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, format!("/admin/{table}"))
            .body(Body::empty())
            .unwrap(),
        Err(e) => (StatusCode::BAD_REQUEST, Html(page::html_escape(&e))).into_response(),
    }
}

/// Blocking entry used by ABI: run async server until Ctrl-C or optional duration_ms.
pub fn listen(args: &Value) -> Result<Value, String> {
    let host = arg_str(args, "host", "127.0.0.1");
    let port = arg_u16(args, "port", 8080);
    let (pages, page_cfgs) = build_pages(args);
    let static_dir = args
        .get("static_dir")
        .and_then(|v| v.as_str())
        .map(PathBuf::from);
    let db_url = args
        .get("db_url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let admin = args
        .get("admin")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let admin_tables = resolve_admin_tables(args, db_url.as_deref());
    let duration_ms = match args.get("duration_ms") {
        Some(Value::Number(n)) => n.as_u64(),
        Some(Value::String(s)) => s.parse().ok(),
        _ => None,
    };

    let state = Arc::new(AppState {
        pages,
        page_cfgs,
        db_url: db_url.clone(),
        admin_tables,
    });

    let mut app = Router::new().route("/", get(serve_page));
    let mut paths: Vec<String> = state.pages.keys().cloned().collect();
    paths.extend(state.page_cfgs.keys().cloned());
    for path in paths {
        if path == "/" {
            continue;
        }
        if admin && (path == "/admin" || path.starts_with("/admin/")) {
            continue;
        }
        app = app.route(&path, get(serve_page));
    }
    app = app.fallback(get(serve_page));

    if admin {
        if let Some(url) = db_url.as_deref() {
            let _ = db::ensure_admin_log(url);
        }
        app = app
            .route("/admin", get(admin_home))
            .route("/admin/log", get(admin_log_page))
            .route("/admin/{table}", get(admin_table))
            .route(
                "/admin/{table}/new",
                get(admin_new_get).post(admin_new_post),
            )
            .route(
                "/admin/{table}/{id}/edit",
                get(admin_edit_get).post(admin_edit_post),
            )
            .route("/admin/{table}/{id}/delete", axum::routing::post(admin_delete_post));
    }

    if let Some(dir) = static_dir {
        if dir.is_dir() {
            app = app.nest_service("/static", ServeDir::new(dir));
        }
    }

    let app = app.with_state(state);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| format!("bad addr: {e}"))?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("tokio runtime: {e}"))?;

    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("bind {addr}: {e}"))?;
        let bound = listener
            .local_addr()
            .map_err(|e| format!("local_addr: {e}"))?;
        eprintln!("marqdo web listening on http://{bound}");

        let server = axum::serve(listener, app);
        if let Some(ms) = duration_ms {
            tokio::select! {
                r = server => r.map_err(|e| format!("server: {e}"))?,
                _ = tokio::time::sleep(Duration::from_millis(ms)) => {}
            }
        } else {
            server
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await
                .map_err(|e| format!("server: {e}"))?;
        }
        Ok(json!({ "ok": true, "addr": bound.to_string() }))
    })
}
