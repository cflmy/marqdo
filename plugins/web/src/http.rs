//! Blocking HTTP listen: page shell, `/_part`, `/_form`, `/admin`, optional static dir.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Form, Multipart, Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::Router;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use tokio::runtime::Runtime;
use tower_http::services::ServeDir;

use crate::db;
use crate::form;
use crate::middleware::Middleware;
use crate::rate_limit;
use crate::render;
use crate::rss;
use crate::session;
use crate::sitemap;
use crate::storage;
use crate::upload;

fn with_site_head(base: &Value, site_head: &[crate::assets::HeadLink]) -> Value {
    let mut page = base.clone();
    crate::assets::merge_site_head(&mut page, site_head);
    page
}

fn serve_icon_file(path: PathBuf, content_type: String) -> Response {
    match std::fs::read(&path) {
        Ok(bytes) => {
            let ct = HeaderValue::from_str(&content_type)
                .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, ct),
                    (
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=86400"),
                    ),
                ],
                bytes,
            )
                .into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

#[derive(Clone)]
pub struct UploadRoute {
    pub field: String,
    pub storage_url: String,
    pub prefix: String,
    pub max_bytes: u64,
    pub types: Option<Value>,
}

#[derive(Clone)]
pub struct DownloadRoute {
    pub storage_url: String,
    pub disposition: String,
}

#[derive(Clone)]
pub struct AppState {
    page: Value,
    pub db_url: Option<String>,
    admin: bool,
    /// Built-in admin mount root (e.g. `/admin` or `/desk`); unused when `admin` is false.
    admin_prefix: String,
    login_redirect: String,
    logout_redirect: String,
    forms: HashMap<String, Value>,
    /// Page that owns an embedded form (for re-render on validation errors).
    form_owners: HashMap<String, Value>,
    /// Admin users table (`|用户名|密码|`) when login-gated admin is configured.
    auth_users: Option<Value>,
    session_ttl: u64,
    cookie_secure: bool,
    upload_routes: HashMap<String, UploadRoute>,
    download_routes: HashMap<String, DownloadRoute>,
    /// Optional assembled pages for 404 / 500.
    page_404: Option<Value>,
    page_500: Option<Value>,
    /// Default `<head>` links from `app.icons` / static favicon convention.
    site_head: Vec<crate::assets::HeadLink>,
}

impl AppState {
    fn ap(&self) -> &str {
        self.admin_prefix.trim_end_matches('/')
    }

    fn admin_href(&self, rest: &str) -> String {
        let p = self.ap();
        let rest = rest.trim().trim_start_matches('/');
        if rest.is_empty() {
            p.to_string()
        } else {
            format!("{p}/{rest}")
        }
    }

    fn login_path(&self) -> String {
        self.admin_href("login")
    }
}

fn collect_page_forms(
    page: &Value,
    forms: &mut HashMap<String, Value>,
    owners: &mut HashMap<String, Value>,
) {
    if let Some(obj) = page.get("forms").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            forms.insert(k.clone(), v.clone());
            owners.insert(k.clone(), page.clone());
        }
    }
    if let (Some(id), Some(f)) = (
        page.get("form_id").and_then(|v| v.as_str()),
        page.get("form"),
    ) {
        if !id.is_empty() {
            forms.insert(id.to_string(), f.clone());
            owners.insert(id.to_string(), page.clone());
        }
    }
}

pub fn listen(
    page: &Value,
    db_url: Option<&str>,
    host: &str,
    port: u16,
    admin: bool,
    admin_prefix: &str,
    login_redirect: &str,
    logout_redirect: &str,
    mut forms: HashMap<String, Value>,
    routes: HashMap<String, Value>,
    static_dir: Option<PathBuf>,
    static_mount: &str,
    auth_users: Option<Value>,
    session_ttl: u64,
    cookie_secure: bool,
    ws_routes: HashMap<String, crate::ws_hub::WsMode>,
    rss_routes: HashMap<String, Value>,
    upload_routes: HashMap<String, UploadRoute>,
    download_routes: HashMap<String, DownloadRoute>,
    redirects: HashMap<String, (String, bool)>,
    sitemap_routes: HashMap<String, Value>,
    robots_body: Option<String>,
    page_404: Option<Value>,
    page_500: Option<Value>,
    gates: Vec<(String, Vec<String>)>,
    gallery_routes: HashMap<String, Value>,
    middleware: &Middleware,
    mut site_head: Vec<crate::assets::HeadLink>,
    mut icon_routes: Vec<crate::assets::IconRoute>,
) -> Result<Value, String> {
    let mut form_owners = HashMap::new();
    collect_page_forms(page, &mut forms, &mut form_owners);
    for p in routes.values() {
        collect_page_forms(p, &mut forms, &mut form_owners);
    }
    session::configure(session::Config {
        db_url: db_url.map(|s| s.to_string()),
        ttl_sec: session_ttl,
        cookie_secure,
    });
    session::reset(session_ttl);
    rate_limit::reset();

    // Convention: static_dir/favicon.* → /favicon.ico when no explicit icons.
    if icon_routes.is_empty() {
        if let Some(dir) = static_dir.as_ref() {
            if let Some((head, routes)) = crate::assets::convention_favicon(dir) {
                site_head = head;
                icon_routes = routes;
            }
        }
    }

    // Resolve relative icon paths: try as-is absolute, else leave for entry_dir resolve in lib.
    // (Absolute paths preferred; relative already joined in web_listen.)

    let admin_prefix = {
        let mut p = admin_prefix.trim().to_string();
        if !p.starts_with('/') {
            p = format!("/{p}");
        }
        while p.len() > 1 && p.ends_with('/') {
            p.pop();
        }
        p
    };
    let state = Arc::new(AppState {
        page: page.clone(),
        db_url: db_url.map(|s| s.to_string()),
        admin,
        admin_prefix: admin_prefix.clone(),
        login_redirect: login_redirect.to_string(),
        logout_redirect: logout_redirect.to_string(),
        forms,
        form_owners,
        auth_users,
        session_ttl,
        cookie_secure,
        upload_routes: upload_routes.clone(),
        download_routes: download_routes.clone(),
        page_404: page_404.clone(),
        page_500: page_500.clone(),
        site_head: site_head.clone(),
    });

    let mut app = Router::new()
        .route("/", get(home))
        .route("/_part/{id}", get(home_part))
        .route("/_form/{id}", get(form_get).post(form_post));

    // Built-in admin only when enabled — otherwise `/admin…` is free for author routes.
    if admin {
        let admin_routes = Router::new()
            .route("/", get(admin_home))
            .route("/login", get(admin_login_get).post(admin_login_post))
            .route("/logout", get(admin_logout))
            .route("/{table}", get(admin_table))
            .route("/{table}/new", get(admin_new_get).post(admin_new_post))
            .route(
                "/{table}/{id}/edit",
                get(admin_edit_get).post(admin_edit_post),
            )
            .route("/{table}/{id}/delete", get(admin_delete));
        app = app.nest(&admin_prefix, admin_routes);
    }

    // Register each author route as an exact GET path + `{path}/_part/{id}`.
    // Paths containing `{param}` become dynamic routes (e.g. `/post/{slug}`);
    // the captured params are injected into the page before rendering.
    let mut paths: Vec<String> = routes.keys().cloned().collect();
    paths.sort();
    for path in paths {
        let page = routes.get(&path).cloned().unwrap_or_default();
        let dynamic = path.contains('{');
        if dynamic {
            let page_for_render = page.clone();
            app = app.route(
                &path,
                get(async move |State(st): State<Arc<AppState>>, headers: axum::http::HeaderMap, Path(params): Path<HashMap<String, String>>| {
                    let mut p = page_for_render.clone();
                    inject_params(&mut p, &params);
                    let p = with_site_head(&p, &st.site_head);
                    let (_, csrf, set_cookie) = resolve_session(&headers);
                    let mut resp = Html(render::render_page(&p, st.db_url.as_deref(), Some(&csrf))).into_response();
                    append_set_cookie(&mut resp, set_cookie);
                    resp
                }),
            );
            // Dynamic `{path}/_part/{id}` with the same path params.
            let part_path = format!("{path}/_part/{{id}}");
            let page_for_part = page.clone();
            app = app.route(
                &part_path,
                get(async move |State(st): State<Arc<AppState>>, Path((params, id)): Path<(HashMap<String, String>, String)>| {
                    let mut p = page_for_part.clone();
                    inject_params(&mut p, &params);
                    render_part_from_page(&p, &id, st.db_url.as_deref())
                }),
            );
        } else {
            let page_for_render = page.clone();
            app = app.route(
                &path,
                get(async move |State(st): State<Arc<AppState>>, headers: axum::http::HeaderMap| {
                    let (_, csrf, set_cookie) = resolve_session(&headers);
                    let p = with_site_head(&page_for_render, &st.site_head);
                    let mut resp = Html(render::render_page(&p, st.db_url.as_deref(), Some(&csrf)))
                        .into_response();
                    append_set_cookie(&mut resp, set_cookie);
                    resp
                }),
            );
            let part_path = format!("{path}/_part/{{id}}");
            let page_for_part = page.clone();
            app = app.route(
                &part_path,
                get(async move |State(st): State<Arc<AppState>>, Path(id): Path<String>| {
                    render_part_from_page(&page_for_part, &id, st.db_url.as_deref())
                }),
            );
        }
    }

    // WebSocket endpoints (`ws://…/{path}`).
    let mut ws_paths: Vec<String> = ws_routes.keys().cloned().collect();
    ws_paths.sort();
    for path in ws_paths {
        let mode = ws_routes
            .get(&path)
            .copied()
            .unwrap_or(crate::ws_hub::WsMode::Echo);
        let route_path = path.clone();
        app = app.route(
            &path,
            axum::routing::get(move |ws: axum::extract::WebSocketUpgrade| {
                ws_upgrade(ws, mode, route_path.clone())
            }),
        );
    }

    let mut rss_paths: Vec<String> = rss_routes.keys().cloned().collect();
    rss_paths.sort();
    for path in rss_paths {
        let cfg = rss_routes.get(&path).cloned().unwrap_or_default();
        let route_path = path.clone();
        app = app.route(
            &path,
            get(move |State(st): State<Arc<AppState>>| {
                let cfg = cfg.clone();
                let route_path = route_path.clone();
                async move { rss_feed(&st, &route_path, &cfg) }
            }),
        );
    }

    let mut upload_paths: Vec<String> = upload_routes.keys().cloned().collect();
    upload_paths.sort();
    for path in upload_paths {
        let route_path = path.clone();
        app = app.route(
            &path,
            post(move |State(st): State<Arc<AppState>>, headers: axum::http::HeaderMap, multipart: Multipart| {
                let route_path = route_path.clone();
                async move { upload_post(st, headers, route_path, multipart).await }
            }),
        );
    }

    let mut download_paths: Vec<String> = download_routes.keys().cloned().collect();
    download_paths.sort();
    for path in download_paths {
        let route_path = path.clone();
        app = app.route(
            &path,
            get(move |State(st): State<Arc<AppState>>, headers: axum::http::HeaderMap, Path(params): Path<HashMap<String, String>>| {
                let route_path = route_path.clone();
                async move { download_get(st, &route_path, params, &headers) }
            }),
        );
    }

    let mut gallery_paths: Vec<String> = gallery_routes.keys().cloned().collect();
    gallery_paths.sort();
    for path in gallery_paths {
        let cfg = gallery_routes.get(&path).cloned().unwrap_or_default();
        app = app.route(
            &path,
            get(move |State(st): State<Arc<AppState>>| {
                let cfg = cfg.clone();
                async move { gallery_page(&st, &cfg) }
            }),
        );
    }

    let mount = normalize_static_mount(static_mount);
    if let Some(dir) = static_dir {
        if !dir.is_dir() {
            return Err(format!(
                "static dir `{}` is not a directory",
                dir.display()
            ));
        }
        eprintln!(
            "marqdo web static: {} → {}",
            mount,
            dir.display()
        );
        let svc = ServeDir::new(dir);
        app = app.nest_service(&mount, svc);
    }

    // Root / custom icon file routes (favicon.ico, /icons/…).
    for ir in icon_routes {
        let url = ir.url.clone();
        if !url.starts_with('/') {
            return Err(format!("icon url `{url}` must start with /"));
        }
        if !ir.path.is_file() {
            return Err(format!(
                "icon file `{}` not found for `{url}`",
                ir.path.display()
            ));
        }
        eprintln!(
            "marqdo web icon: {} → {} ({})",
            url,
            ir.path.display(),
            ir.content_type
        );
        let path = ir.path.clone();
        let ct = ir.content_type.clone();
        app = app.route(
            &url,
            get(move || {
                let path = path.clone();
                let ct = ct.clone();
                async move { serve_icon_file(path, ct) }
            }),
        );
    }

    let mut redir_paths: Vec<String> = redirects.keys().cloned().collect();
    redir_paths.sort();
    for path in redir_paths {
        let (to, permanent) = redirects.get(&path).cloned().unwrap_or_default();
        app = app.route(
            &path,
            get(move || {
                let to = to.clone();
                let permanent = permanent;
                async move {
                    // 301 for permanent (SEO), 307 for temporary — not axum's 308/307 defaults.
                    let status = if permanent {
                        StatusCode::MOVED_PERMANENTLY
                    } else {
                        StatusCode::TEMPORARY_REDIRECT
                    };
                    match HeaderValue::from_str(&to) {
                        Ok(loc) => (status, [(header::LOCATION, loc)]).into_response(),
                        Err(_) => Redirect::to(&to).into_response(),
                    }
                }
            }),
        );
    }

    let mut sm_paths: Vec<String> = sitemap_routes.keys().cloned().collect();
    sm_paths.sort();
    for path in sm_paths {
        let cfg = sitemap_routes.get(&path).cloned().unwrap_or_default();
        app = app.route(
            &path,
            get(move |State(st): State<Arc<AppState>>| {
                let cfg = cfg.clone();
                async move { sitemap_response(&st, &cfg) }
            }),
        );
    }

    if let Some(body) = robots_body {
        app = app.route(
            "/robots.txt",
            get(move || {
                let body = body.clone();
                async move {
                    (
                        [(
                            header::CONTENT_TYPE,
                            HeaderValue::from_static("text/plain; charset=utf-8"),
                        )],
                        body,
                    )
                        .into_response()
                }
            }),
        );
    }

    app = app.fallback(get(fallback_404));

    let app = crate::middleware::apply(app, middleware);
    let app = if !gates.is_empty() {
        let gates_for_mw = gates.clone();
        let login_path_for_mw = state.login_path();
        let admin_prefix_for_mw = state.admin_prefix.clone();
        app.layer(axum::middleware::from_fn(
            move |req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| {
                let gates = gates_for_mw.clone();
                let login_path = login_path_for_mw.clone();
                let admin_prefix = admin_prefix_for_mw.clone();
                async move { rbac_middleware(gates, login_path, admin_prefix, req, next).await }
            },
        ))
    } else {
        app
    };
    let app = app.with_state(state);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| format!("bad listen addr {host}:{port}: {e}"))?;

    let rt = Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("bind {addr}: {e}"))?;
        eprintln!("marqdo web listening on http://{addr}");
        axum::serve(listener, app)
            .await
            .map_err(|e| format!("serve: {e}"))
    })?;
    Ok(json!({ "ok": true }))
}

pub fn normalize_static_mount(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return "/static".into();
    }
    let mut m = if s.starts_with('/') {
        s.to_string()
    } else {
        format!("/{s}")
    };
    while m.len() > 1 && m.ends_with('/') {
        m.pop();
    }
    m
}

fn rss_feed(st: &AppState, _path: &str, cfg: &Value) -> Response {
    let Some(url) = st.db_url.as_deref() else {
        return Html(String::from("<p>no database</p>")).into_response();
    };
    let table = cfg
        .get("table")
        .and_then(|v| v.as_str())
        .unwrap_or("posts");
    let limit = cfg
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(20);
    let order = cfg.get("order").and_then(|v| v.as_str());
    let title = cfg
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Feed");
    let link = cfg.get("link").and_then(|v| v.as_str()).unwrap_or("/");
    let description = cfg
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let rows = db::select_order(url, table, limit, None, order, None, None)
        .ok()
        .and_then(|v| v.get("rows").and_then(|r| r.as_array()).cloned())
        .unwrap_or_default();
    let xml = rss::build_rss(title, link, description, &rows);
    (
        [(axum::http::header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        xml,
    )
        .into_response()
}

async fn home(State(st): State<Arc<AppState>>, headers: axum::http::HeaderMap) -> Response {
    let (_, csrf, set_cookie) = resolve_session(&headers);
    let page = with_site_head(&st.page, &st.site_head);
    let html = render::render_page(&page, st.db_url.as_deref(), Some(&csrf));
    let mut resp = Html(html).into_response();
    append_set_cookie(&mut resp, set_cookie);
    resp
}

/// Inject captured dynamic-route params into a page copy as `params` (a map),
/// so render can resolve `{param}` placeholders in `query`/`intro`.
fn inject_params(page: &mut Value, params: &HashMap<String, String>) {
    if let Some(obj) = page.as_object_mut() {
        let mut map = Map::new();
        for (k, v) in params {
            map.insert(k.clone(), json!(v));
        }
        obj.insert("params".into(), Value::Object(map));
        // Propagate into every part (fragments render independently).
        if let Some(parts) = obj.get_mut("parts").and_then(|p| p.as_object_mut()) {
            for cfg in parts.values_mut() {
                if let Some(m) = cfg.as_object_mut() {
                    let mut pmap = Map::new();
                    for (k, v) in params {
                        pmap.insert(k.clone(), json!(v));
                    }
                    m.insert("params".into(), Value::Object(pmap));
                }
            }
        }
    }
}

fn render_part_from_page(page: &Value, id: &str, db_url: Option<&str>) -> Response {
    let Some(parts) = page.get("parts").and_then(|v| v.as_object()) else {
        return Html(String::from("<p>no parts</p>")).into_response();
    };
    let Some(cfg) = parts.get(id) else {
        return Html(format!("<p>unknown part {id}</p>")).into_response();
    };
    Html(render::render_fragment(cfg, db_url)).into_response()
}

async fn home_part(State(st): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    render_part_from_page(&st.page, &id, st.db_url.as_deref())
}

async fn form_get(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Response {
    let Some(frm) = st.forms.get(&id) else {
        return Html(format!("<p>unknown form {id}</p>")).into_response();
    };
    let (_, csrf, set_cookie) = resolve_session(&headers);
    let mut resp = Html(form::render(frm, &id, None, None, Some(&csrf))).into_response();
    append_set_cookie(&mut resp, set_cookie);
    resp
}

async fn form_post(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
    Form(posted): Form<HashMap<String, String>>,
) -> Response {
    let Some(frm) = st.forms.get(&id) else {
        return Html(format!("<p>unknown form {id}</p>")).into_response();
    };
    let Some(url) = st.db_url.as_deref() else {
        return Html(String::from("<p>no database</p>")).into_response();
    };
    let (_, csrf, set_cookie, posted) = match validate_csrf_post(&headers, posted) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let mut resp = submit_and_respond(&st, frm, &id, url, posted, None, Some(&csrf));
    append_set_cookie(&mut resp, set_cookie);
    resp
}

fn posted_to_value(posted: HashMap<String, String>) -> Value {
    let mut data = Map::new();
    for (k, v) in posted {
        data.insert(k, json!(v));
    }
    Value::Object(data)
}

fn with_flash(redirect: &str, flash: &str) -> String {
    if redirect.contains('?') {
        format!("{redirect}&flash={flash}")
    } else {
        format!("{redirect}?flash={flash}")
    }
}

fn submit_and_respond(
    st: &AppState,
    frm: &Value,
    form_id: &str,
    url: &str,
    posted: HashMap<String, String>,
    admin_table: Option<&str>,
    csrf: Option<&str>,
) -> Response {
    let data_v = posted_to_value(posted);
    match form::submit(frm, &data_v, url) {
        Ok(res) if res.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) => {
            let mut redirect = res
                .get("redirect")
                .and_then(|v| v.as_str())
                .unwrap_or("/")
                .to_string();
            if form_id.contains("-new") {
                redirect = with_flash(&redirect, "created");
            } else if form_id.contains("-edit") {
                redirect = with_flash(&redirect, "updated");
            }
            Redirect::to(&redirect).into_response()
        }
        Ok(res) => {
            let errors = res.get("errors");
            if let Some(table) = admin_table {
                Html(admin_form_page(
                    st,
                    table,
                    if form_id.contains("-edit") {
                        "Edit"
                    } else {
                        "New"
                    },
                    frm,
                    form_id,
                    Some(&data_v),
                    errors,
                    csrf,
                ))
                .into_response()
            } else if let Some(page) = st.form_owners.get(form_id) {
                Html(render::render_page_ex(
                    page,
                    st.db_url.as_deref(),
                    Some(&data_v),
                    errors,
                    csrf,
                ))
                .into_response()
            } else {
                Html(form::render(frm, form_id, Some(&data_v), errors, csrf)).into_response()
            }
        }
        Err(e) => Html(format!("<p>submit error: {}</p>", esc(&e))).into_response(),
    }
}

fn cookie_from_headers(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
}

fn client_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "127.0.0.1".into())
}

fn resolve_session(headers: &axum::http::HeaderMap) -> (String, String, Option<String>) {
    let cookie = cookie_from_headers(headers);
    let (id, set_cookie) = session::ensure_from_cookie(cookie);
    let csrf = session::csrf_for(&id).unwrap_or_default();
    (id, csrf, set_cookie)
}

fn append_set_cookie(resp: &mut Response, set_cookie: Option<String>) {
    if let Some(c) = set_cookie {
        let _ = resp.headers_mut().try_append(
            axum::http::header::SET_COOKIE,
            c.parse().unwrap(),
        );
    }
}

fn validate_csrf_post(
    headers: &axum::http::HeaderMap,
    posted: HashMap<String, String>,
) -> Result<(String, String, Option<String>, HashMap<String, String>), Response> {
    let (sid, csrf, set_cookie) = resolve_session(headers);
    let token = posted.get("_csrf").map(|s| s.as_str()).unwrap_or("");
    if !session::validate_csrf(&sid, token) {
        return Err(Html(
            "<p class=\"flash err\">Invalid or missing CSRF token. Refresh and try again.</p>"
                .to_string(),
        )
        .into_response());
    }
    let mut posted = posted;
    posted.remove("_csrf");
    Ok((sid, csrf, set_cookie, posted))
}

fn json_err(status: StatusCode, msg: &str) -> Response {
    let body = json!({ "ok": false, "error": msg });
    (status, axum::Json(body)).into_response()
}

async fn upload_post(
    st: Arc<AppState>,
    headers: axum::http::HeaderMap,
    route_path: String,
    mut multipart: Multipart,
) -> Response {
    let cfg = match st.upload_routes.get(&route_path) {
        Some(c) => c.clone(),
        None => return json_err(StatusCode::NOT_FOUND, "upload route not found"),
    };

    let had_cookie = cookie_from_headers(&headers).is_some();
    let (sid, _csrf, set_cookie) = resolve_session(&headers);

    let mut csrf_token = String::new();
    let mut file_name = String::new();
    let mut file_ct = String::from("application/octet-stream");
    let mut file_bytes: Option<Vec<u8>> = None;

    loop {
        let field = match multipart.next_field().await {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => return json_err(StatusCode::BAD_REQUEST, &format!("multipart: {e}")),
        };
        let name = field.name().unwrap_or("").to_string();
        if name == "_csrf" {
            csrf_token = field.text().await.unwrap_or_default();
            continue;
        }
        if name != cfg.field {
            // Drain unrelated fields.
            let _ = field.bytes().await;
            continue;
        }
        file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_default();
        if let Some(ct) = field.content_type() {
            file_ct = ct.to_string();
        }
        match field.bytes().await {
            Ok(b) => file_bytes = Some(b.to_vec()),
            Err(e) => return json_err(StatusCode::BAD_REQUEST, &format!("read file: {e}")),
        }
    }

    // Existing session cookie → require CSRF; brand-new anonymous upload → open.
    if had_cookie && !session::validate_csrf(&sid, &csrf_token) {
        return json_err(StatusCode::FORBIDDEN, "Invalid or missing CSRF token");
    }

    let bytes = match file_bytes {
        Some(b) if !b.is_empty() => b,
        _ => return json_err(StatusCode::BAD_REQUEST, "missing file field"),
    };
    let size = bytes.len() as u64;
    let check = match upload::validate(
        &file_name,
        &file_ct,
        size,
        cfg.max_bytes,
        cfg.types.as_ref(),
    ) {
        Ok(v) => v,
        Err(e) => return json_err(StatusCode::BAD_REQUEST, &e),
    };
    if check.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        let err = check
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("validation failed");
        return json_err(StatusCode::BAD_REQUEST, err);
    }
    let ct = check
        .get("content_type")
        .and_then(|v| v.as_str())
        .unwrap_or(&file_ct);

    let key = match upload::make_key(&cfg.prefix, &file_name) {
        Ok(k) => k,
        Err(e) => return json_err(StatusCode::BAD_REQUEST, &e),
    };
    let saved = match storage::put_bytes(&cfg.storage_url, &key, &bytes, ct) {
        Ok(v) => v,
        Err(e) => return json_err(StatusCode::INTERNAL_SERVER_ERROR, &e),
    };
    let mut body = json!({
        "ok": true,
        "key": key,
        "size": size,
        "content_type": ct,
    });
    if let Some(obj) = body.as_object_mut() {
        if let Some(sz) = saved.get("size") {
            obj.insert("size".into(), sz.clone());
        }
    }
    let mut resp = (StatusCode::OK, axum::Json(body)).into_response();
    append_set_cookie(&mut resp, set_cookie);
    resp
}

fn download_get(
    st: Arc<AppState>,
    route_path: &str,
    params: HashMap<String, String>,
    headers: &axum::http::HeaderMap,
) -> Response {
    let cfg = match st.download_routes.get(route_path) {
        Some(c) => c.clone(),
        None => return json_err(StatusCode::NOT_FOUND, "download route not found"),
    };
    let key = params
        .get("key")
        .or_else(|| params.values().next())
        .map(|s| s.trim_start_matches('/').to_string())
        .unwrap_or_default();
    if key.is_empty() {
        return json_err(StatusCode::BAD_REQUEST, "missing key");
    }
    let got = match storage::read_bytes(&cfg.storage_url, &key) {
        Ok(Some(v)) => v,
        Ok(None) => return json_err(StatusCode::NOT_FOUND, "not found"),
        Err(e) => return json_err(StatusCode::INTERNAL_SERVER_ERROR, &e),
    };
    let (bytes, content_type, filename) = got;
    let etag = weak_etag(&bytes);
    if let Some(inm) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
    {
        if inm
            .split(',')
            .any(|t| t.trim().eq_ignore_ascii_case(&etag))
        {
            return StatusCode::NOT_MODIFIED.into_response();
        }
    }
    let disp = if cfg.disposition.eq_ignore_ascii_case("inline") {
        "inline"
    } else {
        "attachment"
    };
    let safe_name = filename.replace('"', "_");
    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_str(&content_type)
                    .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
            ),
            (
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&format!("{disp}; filename=\"{safe_name}\""))
                    .unwrap_or_else(|_| HeaderValue::from_static("attachment")),
            ),
            (
                header::ETAG,
                HeaderValue::from_str(&etag).unwrap_or_else(|_| HeaderValue::from_static("W/\"0\"")),
            ),
        ],
        bytes,
    )
        .into_response()
}

fn weak_etag(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("W/\"{h:x}-{}\"", bytes.len())
}

fn path_matches(path: &str, pattern: &str) -> bool {
    let pat = pattern.trim_end_matches('*');
    if pattern.ends_with('*') {
        path == pat.trim_end_matches('/') || path.starts_with(pat)
    } else {
        path == pattern || path.starts_with(&format!("{pattern}/"))
    }
}

fn path_under_admin_prefix(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}/"))
}

async fn rbac_middleware(
    gates: Vec<(String, Vec<String>)>,
    login_path: String,
    admin_prefix: String,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    let path = req.uri().path().to_string();
    let cookie = req
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    for (prefix, roles) in &gates {
        if !path_matches(&path, prefix) {
            continue;
        }
        // Login page itself must stay reachable.
        if path == login_path || path.starts_with(&format!("{login_path}?")) {
            break;
        }
        let role = session::session_role(cookie.as_deref());
        if !session::role_allowed(&role, roles) {
            if role == "visitor" && path_under_admin_prefix(&path, &admin_prefix) {
                return Redirect::to(&login_path).into_response();
            }
            return (
                StatusCode::FORBIDDEN,
                Html(format!(
                    "<!doctype html><html><body><h1>403 Forbidden</h1><p>Role `{role}` cannot access `{path}`.</p></body></html>"
                )),
            )
                .into_response();
        }
        break;
    }
    next.run(req).await
}

fn gallery_page(_st: &AppState, cfg: &Value) -> Response {
    let storage_url = cfg
        .get("storage")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let prefix = cfg
        .get("prefix")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let title = cfg
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Gallery");
    let download_base = cfg
        .get("download_base")
        .and_then(|v| v.as_str())
        .unwrap_or("/_media");
    let listed = storage::list(storage_url, Some(prefix)).unwrap_or_else(|_| json!({ "keys": [] }));
    let keys = listed
        .get("keys")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut items = String::new();
    for k in &keys {
        let key = k.as_str().unwrap_or("");
        if key.is_empty() {
            continue;
        }
        let href = format!(
            "{}/{}",
            download_base.trim_end_matches('/'),
            key.trim_start_matches('/')
        );
        let name = key.rsplit('/').next().unwrap_or(key);
        let lower = name.to_ascii_lowercase();
        let is_img = lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".gif")
            || lower.ends_with(".webp")
            || lower.ends_with(".svg");
        if is_img {
            items.push_str(&format!(
                "<figure class=\"gal-item\"><a href=\"{h}\"><img src=\"{h}\" alt=\"{n}\"/></a><figcaption>{n}</figcaption></figure>",
                h = esc(&href),
                n = esc(name)
            ));
        } else {
            items.push_str(&format!(
                "<figure class=\"gal-item\"><a class=\"gal-file\" href=\"{h}\">{n}</a></figure>",
                h = esc(&href),
                n = esc(name)
            ));
        }
    }
    if items.is_empty() {
        items = "<p class=\"gal-empty\">No media yet.</p>".into();
    }
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en"><head>
<meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{title}</title>
<style>
body{{font-family:system-ui,sans-serif;margin:0;padding:1.5rem;background:#f6f4ef;color:#1a1a1a}}
h1{{margin:0 0 1rem;font-size:1.5rem}}
.gal{{display:grid;grid-template-columns:repeat(auto-fill,minmax(140px,1fr));gap:1rem}}
.gal-item{{margin:0;background:#fff;border:1px solid #ddd;border-radius:6px;overflow:hidden}}
.gal-item img{{display:block;width:100%;height:120px;object-fit:cover}}
.gal-item figcaption,.gal-file{{display:block;padding:.5rem;font-size:.85rem;word-break:break-all}}
.gal-empty{{color:#666}}
</style></head>
<body><h1>{title}</h1><div class="gal">{items}</div></body></html>"#,
        title = esc(title),
        items = items
    );
    let etag = weak_etag(html.as_bytes());
    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/html; charset=utf-8"),
            ),
            (
                header::ETAG,
                HeaderValue::from_str(&etag).unwrap_or_else(|_| HeaderValue::from_static("W/\"0\"")),
            ),
        ],
        html,
    )
        .into_response()
}

fn admin_gate(st: &AppState, cookie_header: Option<&str>) -> Option<Response> {
    // When admin=false the built-in routes are not mounted; defense-in-depth only.
    if !st.admin {
        return Some((StatusCode::NOT_FOUND, "Not Found").into_response());
    }
    if st.db_url.is_none() {
        return Some(Html(admin_shell(st, "Admin", None, "<p class=\"flash err\">No database bound to this app.</p>")).into_response());
    }
    if st.auth_users.is_some() && !admin_authed(st, cookie_header) {
        return Some(Redirect::to(&st.login_path()).into_response());
    }
    None
}

/// True when a session with role `admin` is present (legacy users without role → admin).
fn admin_authed(_st: &AppState, cookie_header: Option<&str>) -> bool {
    let Some(sid) = session::session_id_from_cookie(cookie_header) else {
        return false;
    };
    if session::session_get(&sid, "username").is_none() {
        return false;
    }
    let role = session::session_role(cookie_header);
    session::role_allowed(&role, &["admin".into()])
}

async fn admin_login_get(State(st): State<Arc<AppState>>, headers: axum::http::HeaderMap) -> Response {
    if st.auth_users.is_none() {
        return Redirect::to(&st.login_redirect).into_response();
    }
    if admin_authed(&st, cookie_from_headers(&headers)) {
        return Redirect::to(&st.login_redirect).into_response();
    }
    let (_, csrf, set_cookie) = resolve_session(&headers);
    let mut resp = Html(login_page(&st, None, Some(&csrf))).into_response();
    append_set_cookie(&mut resp, set_cookie);
    resp
}

async fn admin_login_post(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Form(posted): Form<HashMap<String, String>>,
) -> Response {
    let Some(users) = &st.auth_users else {
        return Redirect::to(&st.login_redirect).into_response();
    };
    let (_, csrf, set_cookie, posted) = match validate_csrf_post(&headers, posted) {
        Ok(v) => v,
        Err(r) => return r,
    };
    let username = posted.get("username").cloned().unwrap_or_default();
    let password = posted.get("password").cloned().unwrap_or_default();
    let ip = client_ip(&headers);
    if let Err(msg) = rate_limit::check(&ip, &username) {
        let mut resp = Html(login_page(&st, Some(&msg), Some(&csrf))).into_response();
        append_set_cookie(&mut resp, set_cookie);
        return resp;
    }
    let Some((user, role)) = session::check_credentials(users, &username, &password) else {
        rate_limit::record_failure(&ip, &username);
        let mut resp =
            Html(login_page(&st, Some("Invalid username or password."), Some(&csrf)))
                .into_response();
        append_set_cookie(&mut resp, set_cookie);
        return resp;
    };
    rate_limit::clear_success(&ip, &username);
    let sid = session::session_new(Some(st.session_ttl));
    session::session_set(&sid, "username", json!(user));
    session::session_set(&sid, "role", json!(role));
    let mut resp = Redirect::to(&st.login_redirect).into_response();
    let _ = resp.headers_mut().try_append(
        axum::http::header::SET_COOKIE,
        session::session_cookie(&sid, st.session_ttl, st.cookie_secure)
            .parse()
            .unwrap(),
    );
    append_set_cookie(&mut resp, set_cookie);
    resp
}

fn login_page(st: &AppState, error: Option<&str>, csrf: Option<&str>) -> String {
    let err = match error {
        Some(e) => format!("<p class=\"flash err\">{}</p>", esc(e)),
        None => String::new(),
    };
    let csrf_field = csrf
        .filter(|s| !s.is_empty())
        .map(|t| format!("<input type=\"hidden\" name=\"_csrf\" value=\"{}\"/>", esc(t)))
        .unwrap_or_default();
    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>Admin Login</title>
<style>
:root {{ --ink:#1c1917; --muted:#78716c; --paper:#fafaf9; --line:#e7e5e4; --accent:#0f766e; --err:#b91c1c; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; min-height:100vh; display:grid; place-items:center; background:var(--paper); color:var(--ink); font-family:"IBM Plex Sans","Noto Sans SC",sans-serif; }}
.login {{ background:#fff; border:1px solid var(--line); border-radius:10px; padding:2rem 2.25rem; width:min(92vw,22rem); box-shadow:0 8px 24px rgba(0,0,0,.05); }}
.login h1 {{ margin:0 0 .25rem; font-size:1.4rem; }}
.login .sub {{ color:var(--muted); margin:0 0 1.25rem; font-size:.9rem; }}
.login form {{ display:grid; gap:.9rem; }}
.login label {{ display:grid; gap:.25rem; font-size:.9rem; }}
.login input {{ padding:.55rem .65rem; border:1px solid var(--line); border-radius:6px; font:inherit; }}
.login button {{ background:var(--accent); color:#fff; border:0; padding:.6rem 1rem; border-radius:6px; cursor:pointer; font:inherit; }}
.login button:hover {{ filter:brightness(1.05); }}
.flash.err {{ background:#fef2f2; color:var(--err); border:1px solid #fecaca; padding:.6rem .8rem; border-radius:6px; margin:0 0 .9rem; font-size:.9rem; }}
</style>
</head>
<body>
<div class="login">
<h1>Admin</h1>
<p class="sub">Sign in to manage this site.</p>
{err}
<form method="post" action="{login_action}">
{csrf_field}
<label>Username<input name="username" autocomplete="username" required autofocus/></label>
<label>Password<input name="password" type="password" autocomplete="current-password" required/></label>
<button type="submit">Sign in</button>
</form>
</div>
</body></html>"#,
        err = err,
        csrf_field = csrf_field,
        login_action = esc(&st.login_path()),
    )
}

async fn admin_logout(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Response {
    if let Some(sid) = session::session_id_from_cookie(
        headers
            .get(axum::http::header::COOKIE)
            .and_then(|v| v.to_str().ok()),
    ) {
        session::session_destroy(&sid);
    }
    let mut resp = Redirect::to(&st.logout_redirect).into_response();
    if let Ok(h) = resp.headers_mut().try_append(
        axum::http::header::SET_COOKIE,
        "marqdo_sid=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax"
            .parse()
            .unwrap(),
    ) {
        let _ = h;
    }
    resp
}

fn flash_html(flash: Option<&str>) -> String {
    match flash {
        Some("created") => "<div class=\"flash ok\">Row created.</div>".into(),
        Some("updated") => "<div class=\"flash ok\">Row updated.</div>".into(),
        Some("deleted") => "<div class=\"flash ok\">Row deleted.</div>".into(),
        Some(other) if !other.is_empty() => {
            format!("<div class=\"flash ok\">{}</div>", esc(other))
        }
        _ => String::new(),
    }
}

fn admin_shell(st: &AppState, title: &str, active: Option<&str>, inner: &str) -> String {
    let tables = st
        .db_url
        .as_deref()
        .and_then(|u| db::list_tables(u).ok())
        .unwrap_or_default();
    let mut side = format!("<nav class=\"admin-nav\"><a class=\"brand\" href=\"{}\">Admin</a>", esc(st.ap()));
    if tables.is_empty() {
        side.push_str("<p class=\"muted\">No tables yet.</p>");
    } else {
        side.push_str("<ul>");
        for t in &tables {
            let cls = if active == Some(t.as_str()) {
                " class=\"active\""
            } else {
                ""
            };
            side.push_str(&format!(
                "<li><a{cls} href=\"{}\">{}</a></li>",
                esc(&st.admin_href(t)),
                esc(t)
            ));
        }
        side.push_str("</ul>");
    }
    if !st.forms.is_empty() {
        side.push_str("<p class=\"nav-label\">Forms</p><ul>");
        let mut ids: Vec<_> = st.forms.keys().cloned().collect();
        ids.sort();
        for id in ids {
            side.push_str(&format!(
                "<li><a href=\"/_form/{}\">{}</a></li>",
                esc(&id),
                esc(&id)
            ));
        }
        side.push_str("</ul>");
    }
    side.push_str("<div class=\"nav-foot\"><a href=\"/\">← Site</a></div></nav>");

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN"><head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>{title} · Admin</title>
<style>
:root {{ --ink:#1c1917; --muted:#78716c; --paper:#fafaf9; --line:#e7e5e4; --accent:#0f766e; --ok:#166534; --err:#b91c1c; --side:#f5f5f4; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; font-family:"IBM Plex Sans","Noto Sans SC",sans-serif; background:var(--paper); color:var(--ink); min-height:100vh; display:grid; grid-template-columns:14rem minmax(0,1fr); }}
@media (max-width:720px) {{ body {{ grid-template-columns:1fr; }} .admin-nav {{ border-right:0; border-bottom:1px solid var(--line); }} }}
.admin-nav {{ background:var(--side); border-right:1px solid var(--line); padding:1.1rem 1rem 1.5rem; }}
.admin-nav .brand {{ display:block; font-weight:600; font-size:1.05rem; margin-bottom:1rem; color:var(--ink); }}
.admin-nav ul {{ list-style:none; margin:0; padding:0; display:grid; gap:.2rem; }}
.admin-nav a {{ color:var(--ink); text-decoration:none; display:block; padding:.35rem .5rem; border-radius:4px; }}
.admin-nav a:hover,.admin-nav a.active {{ background:#fff; color:var(--accent); }}
.admin-nav .nav-label {{ margin:1.1rem 0 .35rem; font-size:.75rem; text-transform:uppercase; letter-spacing:.04em; color:var(--muted); }}
.admin-nav .nav-foot {{ margin-top:1.5rem; font-size:.9rem; }}
.admin-nav .muted {{ color:var(--muted); font-size:.9rem; }}
.admin-main {{ padding:1.25rem 1.5rem 2rem; min-width:0; width:100%; max-width:none; overflow-x:auto; }}
.admin-main h1 {{ margin:0 0 .35rem; font-size:1.65rem; }}
.crumbs {{ color:var(--muted); font-size:.85rem; margin-bottom:1rem; }}
.crumbs a {{ color:var(--muted); }}
.toolbar {{ display:flex; gap:.75rem; align-items:center; margin:0 0 1.1rem; flex-wrap:wrap; }}
.meta {{ color:var(--muted); font-size:.9rem; margin:0 0 1rem; }}
.btn {{ display:inline-block; background:var(--accent); color:#fff!important; padding:.45rem .85rem; border-radius:4px; text-decoration:none; border:0; font:inherit; cursor:pointer; }}
.btn:hover {{ filter:brightness(1.05); }}
.btn-muted {{ background:#78716c; }}
.btn-ghost {{ background:transparent; color:var(--accent)!important; border:1px solid var(--line); }}
.table-wrap {{ width:100%; overflow-x:auto; -webkit-overflow-scrolling:touch; border:1px solid var(--line); border-radius:6px; background:#fff; }}
table.data {{ width:100%; min-width:100%; border-collapse:collapse; background:#fff; }}
table.data th,table.data td {{ border-bottom:1px solid var(--line); padding:.55rem .7rem; text-align:left; vertical-align:top; font-size:.92rem; word-break:break-word; overflow-wrap:anywhere; white-space:pre-wrap; }}
table.data th {{ background:#f5f5f4; font-weight:600; white-space:nowrap; }}
table.data tr:last-child td {{ border-bottom:0; }}
table.data .actions {{ white-space:nowrap; overflow-wrap:normal; word-break:normal; }}
.empty {{ background:#fff; border:1px dashed var(--line); border-radius:6px; padding:2rem 1.25rem; text-align:center; color:var(--muted); }}
.empty strong {{ display:block; color:var(--ink); margin-bottom:.35rem; font-size:1.05rem; }}
.flash {{ padding:.65rem .85rem; border-radius:4px; margin:0 0 1rem; }}
.flash.ok {{ background:#ecfdf5; color:var(--ok); border:1px solid #a7f3d0; }}
.flash.err {{ background:#fef2f2; color:var(--err); border:1px solid #fecaca; }}
.danger {{ color:var(--err); }}
.site-form {{ max-width:min(48rem,100%); width:100%; margin-top:.5rem; }}
.site-form form {{ display:grid; gap:.85rem; }}
.site-form label {{ display:grid; gap:.25rem; font-size:.9rem; }}
.site-form input,.site-form textarea {{ padding:.5rem .6rem; border:1px solid var(--line); border-radius:4px; font:inherit; background:#fff; width:100%; box-sizing:border-box; }}
.site-form textarea {{ min-height:8rem; resize:vertical; field-sizing:content; }}
.site-form input[readonly] {{ background:#f5f5f4; color:var(--muted); }}
.site-form .err {{ color:var(--err); font-size:.85rem; }}
.site-form .actions {{ display:flex; gap:.75rem; align-items:center; flex-wrap:wrap; }}
.site-form button {{ background:var(--accent); color:#fff; border:0; padding:.55rem 1rem; border-radius:4px; cursor:pointer; font:inherit; }}
.site-form .meta {{ color:var(--muted); font-size:.9rem; }}
</style>
</head>
<body>
{side}
<main class="admin-main">{inner}</main>
</body></html>"#,
        title = esc(title),
        side = side,
        inner = inner
    )
}

fn admin_form_page(
    st: &AppState,
    table: &str,
    kind: &str,
    frm: &Value,
    form_id: &str,
    data: Option<&Value>,
    errors: Option<&Value>,
    csrf: Option<&str>,
) -> String {
    let mut body = format!(
        "<p class=\"crumbs\"><a href=\"{home}\">Admin</a> / <a href=\"{table_href}\">{t}</a> / {kind}</p><h1>{kind} {t}</h1>",
        home = esc(st.ap()),
        table_href = esc(&st.admin_href(table)),
        t = esc(table),
        kind = esc(kind),
    );
    body.push_str(&form::render_body(frm, form_id, data, errors, csrf));
    admin_shell(st, &format!("{kind} {table}"), Some(table), &body)
}

#[derive(Deserialize)]
struct FlashQuery {
    flash: Option<String>,
}

async fn admin_home(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(q): Query<FlashQuery>,
) -> Response {
    if let Some(r) = admin_gate(&st, cookie_from_headers(&headers)) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    let tables = db::list_tables(url).unwrap_or_default();
    let mut inner = String::from("<h1>Admin</h1><p class=\"meta\">SQLite tables for this site.</p>");
    inner.push_str(&flash_html(q.flash.as_deref()));
    if tables.is_empty() {
        inner.push_str(
            "<div class=\"empty\"><strong>No tables</strong>Initialize a table via <code>db.init</code> / <code>数据库.初始化</code>.</div>",
        );
    } else {
        inner.push_str("<div class=\"table-wrap\"><table class=\"data\"><thead><tr><th>Table</th><th></th></tr></thead><tbody>");
        for t in &tables {
            inner.push_str(&format!(
                "<tr><td><a href=\"{href}\">{t}</a></td><td class=\"actions\"><a class=\"btn\" href=\"{new_href}\">New</a></td></tr>",
                href = esc(&st.admin_href(t)),
                new_href = esc(&st.admin_href(&format!("{t}/new"))),
                t = esc(t)
            ));
        }
        inner.push_str("</tbody></table></div>");
    }
    Html(admin_shell(&st, "Admin", None, &inner)).into_response()
}

async fn admin_table(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(table): Path<String>,
    Query(q): Query<FlashQuery>,
) -> Response {
    if let Some(r) = admin_gate(&st, cookie_from_headers(&headers)) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    let Ok(data) = db::select(url, &table, 200, None) else {
        return Html(admin_shell(
            &st,
            &table,
            Some(&table),
            &format!("<p class=\"flash err\">Cannot read table {}.</p>", esc(&table)),
        ))
        .into_response();
    };
    let rows = data
        .get("rows")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut cols: Vec<String> = rows
        .first()
        .and_then(|r| r.as_object())
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    if cols.is_empty() {
        if let Ok(info) = db::table_info(url, &table) {
            cols = info.into_iter().map(|c| c.name).collect();
        }
    }
    if let Some(i) = cols.iter().position(|c| c == "id") {
        let idc = cols.remove(i);
        cols.insert(0, idc);
    }

    let mut inner = format!(
        "<p class=\"crumbs\"><a href=\"{home}\">Admin</a> / {t}</p><h1>{t}</h1>",
        home = esc(st.ap()),
        t = esc(&table)
    );
    inner.push_str(&flash_html(q.flash.as_deref()));
    inner.push_str(&format!(
        "<div class=\"toolbar\"><a class=\"btn\" href=\"{new_href}\">New row</a><span class=\"meta\">{} row{}</span></div>",
        rows.len(),
        if rows.len() == 1 { "" } else { "s" },
        new_href = esc(&st.admin_href(&format!("{table}/new"))),
    ));

    if rows.is_empty() {
        inner.push_str(&format!(
            "<div class=\"empty\"><strong>No rows in {}</strong><a class=\"btn\" href=\"{new_href}\">Create the first row</a></div>",
            esc(&table),
            new_href = esc(&st.admin_href(&format!("{table}/new"))),
        ));
    } else {
        inner.push_str("<div class=\"table-wrap\"><table class=\"data\"><thead><tr>");
        for c in &cols {
            inner.push_str(&format!("<th>{}</th>", esc(c)));
        }
        inner.push_str("<th></th></tr></thead><tbody>");
        for row in &rows {
            let m = row.as_object().cloned().unwrap_or_else(Map::new);
            inner.push_str("<tr>");
            for c in &cols {
                let cell = m
                    .get(c)
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        Value::Null => String::new(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default();
                // Show full cell text; CSS wraps / table-wrap scrolls when needed.
                inner.push_str(&format!("<td>{}</td>", esc(&cell)));
            }
            let id = m
                .get("id")
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            if !id.is_empty() {
                inner.push_str(&format!(
                    "<td class=\"actions\"><a href=\"{edit}\">Edit</a> · <a class=\"danger\" href=\"{del}\" onclick=\"return confirm('Delete row #{id}?')\">Delete</a></td>",
                    edit = esc(&st.admin_href(&format!("{table}/{id}/edit"))),
                    del = esc(&st.admin_href(&format!("{table}/{id}/delete"))),
                    id = esc(&id),
                ));
            } else {
                inner.push_str("<td></td>");
            }
            inner.push_str("</tr>");
        }
        inner.push_str("</tbody></table></div>");
    }
    Html(admin_shell(&st, &table, Some(&table), &inner)).into_response()
}

async fn admin_new_get(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(table): Path<String>,
) -> Response {
    if let Some(r) = admin_gate(&st, cookie_from_headers(&headers)) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    let (_, csrf, set_cookie) = resolve_session(&headers);
    match form::from_schema(url, &table, "insert", None, st.ap()) {
        Ok(frm) => {
            let mut resp = Html(admin_form_page(
                &st,
                &table,
                "New",
                &frm,
                &format!("admin-{table}-new"),
                None,
                None,
                Some(&csrf),
            ))
            .into_response();
            append_set_cookie(&mut resp, set_cookie);
            resp
        }
        Err(e) => Html(admin_shell(
            &st,
            &table,
            Some(&table),
            &format!("<p class=\"flash err\">{}</p>", esc(&e)),
        ))
        .into_response(),
    }
}

async fn admin_new_post(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(table): Path<String>,
    Form(posted): Form<HashMap<String, String>>,
) -> Response {
    if let Some(r) = admin_gate(&st, cookie_from_headers(&headers)) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    let (_, csrf, set_cookie, posted) = match validate_csrf_post(&headers, posted) {
        Ok(v) => v,
        Err(r) => return r,
    };
    match form::from_schema(url, &table, "insert", None, st.ap()) {
        Ok(frm) => {
            let mut resp = submit_and_respond(
                &st,
                &frm,
                &format!("admin-{table}-new"),
                url,
                posted,
                Some(&table),
                Some(&csrf),
            );
            append_set_cookie(&mut resp, set_cookie);
            resp
        }
        Err(e) => Html(admin_shell(
            &st,
            &table,
            Some(&table),
            &format!("<p class=\"flash err\">{}</p>", esc(&e)),
        ))
        .into_response(),
    }
}

async fn admin_edit_get(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path((table, id)): Path<(String, String)>,
) -> Response {
    if let Some(r) = admin_gate(&st, cookie_from_headers(&headers)) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    let row = match db::get(url, &table, &id, None) {
        Ok(Value::Null) => {
            return Html(admin_shell(
                &st,
                &table,
                Some(&table),
                &format!("<p class=\"flash err\">Row {id} not found.</p>"),
            ))
            .into_response();
        }
        Ok(v) => v,
        Err(e) => {
            return Html(admin_shell(
                &st,
                &table,
                Some(&table),
                &format!("<p class=\"flash err\">{}</p>", esc(&e)),
            ))
            .into_response();
        }
    };
    let (_, csrf, set_cookie) = resolve_session(&headers);
    match form::from_schema(url, &table, "update", Some(&id), st.ap()) {
        Ok(frm) => {
            let mut resp = Html(admin_form_page(
                &st,
                &table,
                "Edit",
                &frm,
                &format!("admin-{table}-edit"),
                Some(&row),
                None,
                Some(&csrf),
            ))
            .into_response();
            append_set_cookie(&mut resp, set_cookie);
            resp
        }
        Err(e) => Html(admin_shell(
            &st,
            &table,
            Some(&table),
            &format!("<p class=\"flash err\">{}</p>", esc(&e)),
        ))
        .into_response(),
    }
}

async fn admin_edit_post(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path((table, id)): Path<(String, String)>,
    Form(posted): Form<HashMap<String, String>>,
) -> Response {
    if let Some(r) = admin_gate(&st, cookie_from_headers(&headers)) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    let (_, csrf, set_cookie, posted) = match validate_csrf_post(&headers, posted) {
        Ok(v) => v,
        Err(r) => return r,
    };
    match form::from_schema(url, &table, "update", Some(&id), st.ap()) {
        Ok(frm) => {
            let mut resp = submit_and_respond(
                &st,
                &frm,
                &format!("admin-{table}-edit"),
                url,
                posted,
                Some(&table),
                Some(&csrf),
            );
            append_set_cookie(&mut resp, set_cookie);
            resp
        }
        Err(e) => Html(admin_shell(
            &st,
            &table,
            Some(&table),
            &format!("<p class=\"flash err\">{}</p>", esc(&e)),
        ))
        .into_response(),
    }
}

async fn admin_delete(
    State(st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path((table, id)): Path<(String, String)>,
) -> Response {
    if let Some(r) = admin_gate(&st, cookie_from_headers(&headers)) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    if let Err(e) = db::delete(url, &table, &id, None) {
        return Html(admin_shell(
            &st,
            &table,
            Some(&table),
            &format!("<p class=\"flash err\">Delete failed: {}</p>", esc(&e)),
        ))
        .into_response();
    }
    Redirect::to(&with_flash(&st.admin_href(&table), "deleted")).into_response()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn sitemap_response(st: &AppState, cfg: &Value) -> Response {
    let base = cfg
        .get("base")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let table = cfg.get("table").and_then(|v| v.as_str());
    let loc_col = cfg
        .get("loc")
        .and_then(|v| v.as_str())
        .unwrap_or("path");
    let limit = cfg
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(1000);
    let mut items = cfg.get("items").cloned().unwrap_or(json!([]));
    if let (Some(url), Some(table)) = (st.db_url.as_deref(), table) {
        if let Ok(rows) = db::select_order(url, table, limit, None, None, None, None) {
            if let Some(arr) = rows.get("rows").cloned() {
                // Normalize loc column → loc
                if let Some(list) = arr.as_array() {
                    let mut out = Vec::new();
                    for row in list {
                        let mut m = row.as_object().cloned().unwrap_or_default();
                        if !m.contains_key("loc") {
                            if let Some(v) = m.get(loc_col).cloned() {
                                m.insert("loc".into(), v);
                            }
                        }
                        out.push(Value::Object(m));
                    }
                    items = Value::Array(out);
                }
            }
        }
    }
    let xml = sitemap::build_sitemap(base, &items);
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/xml; charset=utf-8"),
        )],
        xml,
    )
        .into_response()
}

async fn fallback_404(State(st): State<Arc<AppState>>, headers: axum::http::HeaderMap) -> Response {
    let (_, csrf, set_cookie) = resolve_session(&headers);
    let html = if let Some(ref page) = st.page_404 {
        let page = with_site_head(page, &st.site_head);
        render::render_page(&page, st.db_url.as_deref(), Some(&csrf))
    } else {
        "<!doctype html><html><head><title>404</title></head><body><h1>404 Not Found</h1></body></html>"
            .to_string()
    };
    let mut resp = (StatusCode::NOT_FOUND, Html(html)).into_response();
    append_set_cookie(&mut resp, set_cookie);
    resp
}

/// Render a custom 500 page when configured.
#[allow(dead_code)]
fn error_500(st: &AppState, csrf: Option<&str>, msg: &str) -> Response {
    let html = if let Some(ref page) = st.page_500 {
        let page = with_site_head(page, &st.site_head);
        render::render_page(&page, st.db_url.as_deref(), csrf)
    } else {
        format!(
            "<!doctype html><html><head><title>500</title></head><body><h1>500</h1><p>{}</p></body></html>",
            esc(msg)
        )
    };
    (StatusCode::INTERNAL_SERVER_ERROR, Html(html)).into_response()
}

/// WebSocket server endpoint.
/// - `echo`: reply each text frame to the same socket
/// - `broadcast`: fan-out text frames to all sockets on this path
/// - `drain`: read and discard frames
async fn ws_upgrade(
    ws: axum::extract::WebSocketUpgrade,
    mode: crate::ws_hub::WsMode,
    path: String,
) -> Response {
    ws.on_upgrade(move |socket| ws_socket_loop(socket, mode, path))
}

async fn ws_socket_loop(
    mut socket: axum::extract::ws::WebSocket,
    mode: crate::ws_hub::WsMode,
    path: String,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};

    match mode {
        crate::ws_hub::WsMode::Echo => {
            while let Some(Ok(msg)) = socket.recv().await {
                match msg {
                    Message::Text(text) => {
                        if socket.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        }
        crate::ws_hub::WsMode::Drain => {
            while let Some(Ok(msg)) = socket.recv().await {
                if matches!(msg, Message::Close(_)) {
                    break;
                }
            }
        }
        crate::ws_hub::WsMode::Broadcast => {
            let mut rx = crate::ws_hub::subscribe(&path);
            let (mut sink, mut stream) = socket.split();
            loop {
                tokio::select! {
                    incoming = stream.next() => {
                        match incoming {
                            Some(Ok(Message::Text(text))) => {
                                crate::ws_hub::publish(&path, text.to_string());
                            }
                            Some(Ok(Message::Close(_))) | None => break,
                            Some(Ok(_)) => {}
                            Some(Err(_)) => break,
                        }
                    }
                    out = rx.recv() => {
                        match out {
                            Ok(text) => {
                                if sink.send(Message::Text(text.into())).await.is_err() {
                                    break;
                                }
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                            Err(_) => break,
                        }
                    }
                }
            }
        }
    }
}
