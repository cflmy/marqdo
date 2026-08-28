//! Blocking HTTP listen: page shell, `/_part`, `/_form`, `/admin`, optional static dir.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Form, Path, Query, State};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
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

#[derive(Clone)]
pub struct AppState {
    page: Value,
    pub db_url: Option<String>,
    admin: bool,
    forms: HashMap<String, Value>,
    /// Page that owns an embedded form (for re-render on validation errors).
    form_owners: HashMap<String, Value>,
    /// Admin users table (`|用户名|密码|`) when login-gated admin is configured.
    auth_users: Option<Value>,
    session_ttl: u64,
    cookie_secure: bool,
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
    mut forms: HashMap<String, Value>,
    routes: HashMap<String, Value>,
    static_dir: Option<PathBuf>,
    static_mount: &str,
    auth_users: Option<Value>,
    session_ttl: u64,
    cookie_secure: bool,
    ws_routes: HashMap<String, bool>,
    rss_routes: HashMap<String, Value>,
    middleware: &Middleware,
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
    let state = Arc::new(AppState {
        page: page.clone(),
        db_url: db_url.map(|s| s.to_string()),
        admin,
        forms,
        form_owners,
        auth_users,
        session_ttl,
        cookie_secure,
    });

    let mut app = Router::new()
        .route("/", get(home))
        .route("/_part/{id}", get(home_part))
        .route("/_form/{id}", get(form_get).post(form_post))
        .route("/admin", get(admin_home))
        .route("/admin/login", get(admin_login_get).post(admin_login_post))
        .route("/admin/logout", get(admin_logout))
        .route("/admin/{table}", get(admin_table))
        .route("/admin/{table}/new", get(admin_new_get).post(admin_new_post))
        .route(
            "/admin/{table}/{id}/edit",
            get(admin_edit_get).post(admin_edit_post),
        )
        .route("/admin/{table}/{id}/delete", get(admin_delete));

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
                    let mut resp = Html(render::render_page(&page_for_render, st.db_url.as_deref(), Some(&csrf)))
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

    // WebSocket endpoints (`ws://…/{path}`), echo-mode by default.
    let mut ws_paths: Vec<String> = ws_routes.keys().cloned().collect();
    ws_paths.sort();
    for path in ws_paths {
        let echo = ws_routes.get(&path).copied().unwrap_or(true);
        app = app.route(
            &path,
            axum::routing::get(move |ws: axum::extract::WebSocketUpgrade| {
                ws_upgrade(ws, echo)
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

    let app = crate::middleware::apply(app, middleware);
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
    let html = render::render_page(&st.page, st.db_url.as_deref(), Some(&csrf));
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

fn admin_gate(st: &AppState, cookie_header: Option<&str>) -> Option<Response> {
    if !st.admin {
        return Some(Html(admin_shell(st, "Admin", None, "<p class=\"flash err\">Admin is disabled for this app.</p>")).into_response());
    }
    if st.db_url.is_none() {
        return Some(Html(admin_shell(st, "Admin", None, "<p class=\"flash err\">No database bound to this app.</p>")).into_response());
    }
    // Login-gated admin: require a valid session cookie.
    if st.auth_users.is_some() && !admin_authed(st, cookie_header) {
        return Some(
            Redirect::to("/admin/login")
                .into_response(),
        );
    }
    None
}

/// True when a valid admin session cookie is present.
fn admin_authed(_st: &AppState, cookie_header: Option<&str>) -> bool {
    let Some(sid) = session::session_id_from_cookie(cookie_header) else {
        return false;
    };
    session::session_get(&sid, "username").is_some()
}

async fn admin_login_get(State(st): State<Arc<AppState>>, headers: axum::http::HeaderMap) -> Response {
    if st.auth_users.is_none() {
        return Redirect::to("/admin").into_response();
    }
    if admin_authed(&st, cookie_from_headers(&headers)) {
        return Redirect::to("/admin").into_response();
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
        return Redirect::to("/admin").into_response();
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
    if session::check_credentials(users, &username, &password).is_none() {
        rate_limit::record_failure(&ip, &username);
        let mut resp =
            Html(login_page(&st, Some("Invalid username or password."), Some(&csrf)))
                .into_response();
        append_set_cookie(&mut resp, set_cookie);
        return resp;
    }
    rate_limit::clear_success(&ip, &username);
    let sid = session::session_new(Some(st.session_ttl));
    session::session_set(&sid, "username", json!(username));
    let mut resp = Redirect::to("/admin").into_response();
    let _ = resp.headers_mut().try_append(
        axum::http::header::SET_COOKIE,
        session::session_cookie(&sid, st.session_ttl, st.cookie_secure)
            .parse()
            .unwrap(),
    );
    append_set_cookie(&mut resp, set_cookie);
    resp
}

fn login_page(_st: &AppState, error: Option<&str>, csrf: Option<&str>) -> String {
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
<form method="post" action="/admin/login">
{csrf_field}
<label>Username<input name="username" autocomplete="username" required autofocus/></label>
<label>Password<input name="password" type="password" autocomplete="current-password" required/></label>
<button type="submit">Sign in</button>
</form>
</div>
</body></html>"#,
        err = err,
        csrf_field = csrf_field,
    )
}

async fn admin_logout(
    State(_st): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Response {
    if let Some(sid) = session::session_id_from_cookie(
        headers
            .get(axum::http::header::COOKIE)
            .and_then(|v| v.to_str().ok()),
    ) {
        session::session_destroy(&sid);
    }
    let mut resp = Redirect::to("/admin/login").into_response();
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

fn truncate_cell(s: &str, max: usize) -> String {
    let mut it = s.chars();
    let head: String = it.by_ref().take(max).collect();
    if it.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

fn admin_shell(st: &AppState, title: &str, active: Option<&str>, inner: &str) -> String {
    let tables = st
        .db_url
        .as_deref()
        .and_then(|u| db::list_tables(u).ok())
        .unwrap_or_default();
    let mut side = String::from("<nav class=\"admin-nav\"><a class=\"brand\" href=\"/admin\">Admin</a>");
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
                "<li><a{cls} href=\"/admin/{}\">{}</a></li>",
                esc(t),
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
body {{ margin:0; font-family:"IBM Plex Sans","Noto Sans SC",sans-serif; background:var(--paper); color:var(--ink); min-height:100vh; display:grid; grid-template-columns:14rem 1fr; }}
@media (max-width:720px) {{ body {{ grid-template-columns:1fr; }} .admin-nav {{ border-right:0; border-bottom:1px solid var(--line); }} }}
.admin-nav {{ background:var(--side); border-right:1px solid var(--line); padding:1.1rem 1rem 1.5rem; }}
.admin-nav .brand {{ display:block; font-weight:600; font-size:1.05rem; margin-bottom:1rem; color:var(--ink); }}
.admin-nav ul {{ list-style:none; margin:0; padding:0; display:grid; gap:.2rem; }}
.admin-nav a {{ color:var(--ink); text-decoration:none; display:block; padding:.35rem .5rem; border-radius:4px; }}
.admin-nav a:hover,.admin-nav a.active {{ background:#fff; color:var(--accent); }}
.admin-nav .nav-label {{ margin:1.1rem 0 .35rem; font-size:.75rem; text-transform:uppercase; letter-spacing:.04em; color:var(--muted); }}
.admin-nav .nav-foot {{ margin-top:1.5rem; font-size:.9rem; }}
.admin-nav .muted {{ color:var(--muted); font-size:.9rem; }}
.admin-main {{ padding:1.25rem 1.5rem 2rem; max-width:56rem; }}
.admin-main h1 {{ margin:0 0 .35rem; font-size:1.65rem; }}
.crumbs {{ color:var(--muted); font-size:.85rem; margin-bottom:1rem; }}
.crumbs a {{ color:var(--muted); }}
.toolbar {{ display:flex; gap:.75rem; align-items:center; margin:0 0 1.1rem; flex-wrap:wrap; }}
.meta {{ color:var(--muted); font-size:.9rem; margin:0 0 1rem; }}
.btn {{ display:inline-block; background:var(--accent); color:#fff!important; padding:.45rem .85rem; border-radius:4px; text-decoration:none; border:0; font:inherit; cursor:pointer; }}
.btn:hover {{ filter:brightness(1.05); }}
.btn-muted {{ background:#78716c; }}
.btn-ghost {{ background:transparent; color:var(--accent)!important; border:1px solid var(--line); }}
table.data {{ width:100%; border-collapse:collapse; background:#fff; border:1px solid var(--line); border-radius:6px; overflow:hidden; }}
table.data th,table.data td {{ border-bottom:1px solid var(--line); padding:.55rem .7rem; text-align:left; vertical-align:top; font-size:.92rem; }}
table.data th {{ background:#f5f5f4; font-weight:600; }}
table.data tr:last-child td {{ border-bottom:0; }}
table.data .actions {{ white-space:nowrap; }}
.empty {{ background:#fff; border:1px dashed var(--line); border-radius:6px; padding:2rem 1.25rem; text-align:center; color:var(--muted); }}
.empty strong {{ display:block; color:var(--ink); margin-bottom:.35rem; font-size:1.05rem; }}
.flash {{ padding:.65rem .85rem; border-radius:4px; margin:0 0 1rem; }}
.flash.ok {{ background:#ecfdf5; color:var(--ok); border:1px solid #a7f3d0; }}
.flash.err {{ background:#fef2f2; color:var(--err); border:1px solid #fecaca; }}
.danger {{ color:var(--err); }}
.site-form {{ max-width:28rem; margin-top:.5rem; }}
.site-form form {{ display:grid; gap:.85rem; }}
.site-form label {{ display:grid; gap:.25rem; font-size:.9rem; }}
.site-form input,.site-form textarea {{ padding:.5rem .6rem; border:1px solid var(--line); border-radius:4px; font:inherit; background:#fff; }}
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
        "<p class=\"crumbs\"><a href=\"/admin\">Admin</a> / <a href=\"/admin/{t}\">{t}</a> / {kind}</p><h1>{kind} {t}</h1>",
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
        inner.push_str("<table class=\"data\"><thead><tr><th>Table</th><th></th></tr></thead><tbody>");
        for t in &tables {
            inner.push_str(&format!(
                "<tr><td><a href=\"/admin/{t}\">{t}</a></td><td class=\"actions\"><a class=\"btn\" href=\"/admin/{t}/new\">New</a></td></tr>",
                t = esc(t)
            ));
        }
        inner.push_str("</tbody></table>");
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
        "<p class=\"crumbs\"><a href=\"/admin\">Admin</a> / {t}</p><h1>{t}</h1>",
        t = esc(&table)
    );
    inner.push_str(&flash_html(q.flash.as_deref()));
    inner.push_str(&format!(
        "<div class=\"toolbar\"><a class=\"btn\" href=\"/admin/{}/new\">New row</a><span class=\"meta\">{} row{}</span></div>",
        esc(&table),
        rows.len(),
        if rows.len() == 1 { "" } else { "s" }
    ));

    if rows.is_empty() {
        inner.push_str(&format!(
            "<div class=\"empty\"><strong>No rows in {}</strong><a class=\"btn\" href=\"/admin/{}/new\">Create the first row</a></div>",
            esc(&table),
            esc(&table)
        ));
    } else {
        inner.push_str("<table class=\"data\"><thead><tr>");
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
                inner.push_str(&format!("<td>{}</td>", esc(&truncate_cell(&cell, 80))));
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
                    "<td class=\"actions\"><a href=\"/admin/{table}/{id}/edit\">Edit</a> · <a class=\"danger\" href=\"/admin/{table}/{id}/delete\" onclick=\"return confirm('Delete row #{id}?')\">Delete</a></td>"
                ));
            } else {
                inner.push_str("<td></td>");
            }
            inner.push_str("</tr>");
        }
        inner.push_str("</tbody></table>");
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
    match form::from_schema(url, &table, "insert", None) {
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
    match form::from_schema(url, &table, "insert", None) {
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
    match form::from_schema(url, &table, "update", Some(&id)) {
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
    match form::from_schema(url, &table, "update", Some(&id)) {
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
    Redirect::to(&with_flash(&format!("/admin/{table}"), "deleted")).into_response()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// WebSocket server endpoint: `echo=true` replies with each received text frame;
/// otherwise it just drains frames (extension point for custom logic).
async fn ws_upgrade(
    ws: axum::extract::WebSocketUpgrade,
    echo: bool,
) -> Response {
    ws.on_upgrade(move |socket| ws_echo_loop(socket, echo))
}

async fn ws_echo_loop(mut socket: axum::extract::ws::WebSocket, echo: bool) {
    use axum::extract::ws::Message;
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                if echo {
                    if socket.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
