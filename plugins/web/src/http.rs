//! Blocking HTTP listen: page shell, `/_part/{id}`, `/_form/{id}`, `/admin` CRUD.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Form, Path, State};
use axum::http::Uri;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;
use serde_json::{json, Map, Value};
use tokio::runtime::Runtime;

use crate::db;
use crate::form;
use crate::render;

#[derive(Clone)]
struct AppState {
    page: Value,
    db_url: Option<String>,
    admin: bool,
    forms: HashMap<String, Value>,
    routes: HashMap<String, Value>,
}

pub fn listen(
    page: &Value,
    db_url: Option<&str>,
    host: &str,
    port: u16,
    admin: bool,
    forms: HashMap<String, Value>,
    routes: HashMap<String, Value>,
) -> Result<Value, String> {
    let state = Arc::new(AppState {
        page: page.clone(),
        db_url: db_url.map(|s| s.to_string()),
        admin,
        forms,
        routes: routes.clone(),
    });

    let mut app = Router::new()
        .route("/", get(home))
        .route("/_part/{id}", get(part))
        .route("/_form/{id}", get(form_get).post(form_post))
        .route("/admin", get(admin_home))
        .route("/admin/{table}", get(admin_table))
        .route("/admin/{table}/new", get(admin_new_get).post(admin_new_post))
        .route(
            "/admin/{table}/{id}/edit",
            get(admin_edit_get).post(admin_edit_post),
        )
        .route("/admin/{table}/{id}/delete", get(admin_delete));

    // Register each author route as an exact GET path.
    let mut paths: Vec<String> = routes.keys().cloned().collect();
    paths.sort();
    for path in paths {
        app = app.route(&path, get(routed_page));
    }

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

async fn home(State(st): State<Arc<AppState>>) -> Html<String> {
    let html = render::render_page(&st.page, st.db_url.as_deref());
    Html(html)
}

async fn routed_page(State(st): State<Arc<AppState>>, uri: Uri) -> Response {
    let mut path = uri.path().to_string();
    while path.len() > 1 && path.ends_with('/') {
        path.pop();
    }
    let Some(page) = st.routes.get(&path) else {
        return Html(format!("<p>404 not found: {}</p>", esc(&path))).into_response();
    };
    Html(render::render_page(page, st.db_url.as_deref())).into_response()
}

async fn part(State(st): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let Some(parts) = st.page.get("parts").and_then(|v| v.as_object()) else {
        return Html(String::from("<p>no parts</p>")).into_response();
    };
    let Some(cfg) = parts.get(&id) else {
        return Html(format!("<p>unknown part {id}</p>")).into_response();
    };
    let html = render::render_fragment(cfg, st.db_url.as_deref());
    Html(html).into_response()
}

async fn form_get(State(st): State<Arc<AppState>>, Path(id): Path<String>) -> Response {
    let Some(frm) = st.forms.get(&id) else {
        return Html(format!("<p>unknown form {id}</p>")).into_response();
    };
    Html(form::render(frm, &id, None, None)).into_response()
}

async fn form_post(
    State(st): State<Arc<AppState>>,
    Path(id): Path<String>,
    Form(posted): Form<HashMap<String, String>>,
) -> Response {
    let Some(frm) = st.forms.get(&id) else {
        return Html(format!("<p>unknown form {id}</p>")).into_response();
    };
    let Some(url) = st.db_url.as_deref() else {
        return Html(String::from("<p>no database</p>")).into_response();
    };
    submit_and_respond(frm, &id, url, posted)
}

fn posted_to_value(posted: HashMap<String, String>) -> Value {
    let mut data = Map::new();
    for (k, v) in posted {
        data.insert(k, json!(v));
    }
    Value::Object(data)
}

fn submit_and_respond(
    frm: &Value,
    form_id: &str,
    url: &str,
    posted: HashMap<String, String>,
) -> Response {
    let data_v = posted_to_value(posted);
    match form::submit(frm, &data_v, url) {
        Ok(res) if res.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) => {
            let redirect = res
                .get("redirect")
                .and_then(|v| v.as_str())
                .unwrap_or("/");
            Redirect::to(redirect).into_response()
        }
        Ok(res) => {
            let errors = res.get("errors");
            Html(form::render(frm, form_id, Some(&data_v), errors)).into_response()
        }
        Err(e) => Html(format!("<p>submit error: {}</p>", esc(&e))).into_response(),
    }
}

fn admin_gate(st: &AppState) -> Option<Response> {
    if !st.admin {
        return Some(Html(String::from("<p>admin disabled</p>")).into_response());
    }
    if st.db_url.is_none() {
        return Some(Html(String::from("<p>no database</p>")).into_response());
    }
    None
}

fn admin_shell(title: &str, inner: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width, initial-scale=1"/><title>{title}</title>
<style>
body{{font-family:"IBM Plex Sans","Noto Sans SC",sans-serif;margin:1.5rem;background:#fafaf9;color:#1c1917}}
a{{color:#0f766e;text-decoration:none}} a:hover{{text-decoration:underline}}
table{{border-collapse:collapse;background:#fff}} th,td{{border:1px solid #e7e5e4;padding:.45rem .65rem;text-align:left}}
.toolbar{{display:flex;gap:1rem;align-items:center;margin:.75rem 0 1.25rem;flex-wrap:wrap}}
.btn{{display:inline-block;background:#0f766e;color:#fff!important;padding:.4rem .75rem;border-radius:4px}}
.btn-muted{{background:#78716c}}
.danger{{color:#b91c1c}}
</style></head><body>{inner}</body></html>"#,
        title = esc(title),
        inner = inner
    )
}

async fn admin_home(State(st): State<Arc<AppState>>) -> Response {
    if let Some(r) = admin_gate(&st) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    let tables = db::list_tables(url).unwrap_or_default();
    let mut inner = String::from("<h1>admin</h1><div class=\"toolbar\"><a href=\"/\">home</a></div><h2>tables</h2><ul>");
    for t in &tables {
        inner.push_str(&format!("<li><a href=\"/admin/{t}\">{t}</a></li>"));
    }
    inner.push_str("</ul>");
    if !st.forms.is_empty() {
        inner.push_str("<h2>mounted forms</h2><ul>");
        for id in st.forms.keys() {
            inner.push_str(&format!("<li><a href=\"/_form/{id}\">{id}</a></li>"));
        }
        inner.push_str("</ul>");
    }
    Html(admin_shell("admin", &inner)).into_response()
}

async fn admin_table(State(st): State<Arc<AppState>>, Path(table): Path<String>) -> Response {
    if let Some(r) = admin_gate(&st) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    let Ok(data) = db::select(url, &table, 200) else {
        return Html(format!("<p>cannot read {table}</p>")).into_response();
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
    // prefer id first
    if let Some(i) = cols.iter().position(|c| c == "id") {
        let idc = cols.remove(i);
        cols.insert(0, idc);
    }

    let mut inner = format!("<h1>{table}</h1>");
    inner.push_str(&format!(
        "<div class=\"toolbar\"><a class=\"btn\" href=\"/admin/{table}/new\">New</a><a href=\"/admin\">back</a><a href=\"/\">home</a></div>"
    ));
    inner.push_str("<table><thead><tr>");
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
                "<td><a href=\"/admin/{table}/{id}/edit\">edit</a> · <a class=\"danger\" href=\"/admin/{table}/{id}/delete\" onclick=\"return confirm('Delete #{id}?')\">delete</a></td>"
            ));
        } else {
            inner.push_str("<td></td>");
        }
        inner.push_str("</tr>");
    }
    if rows.is_empty() {
        inner.push_str("<tr><td colspan=\"99\">(empty)</td></tr>");
    }
    inner.push_str("</tbody></table>");
    Html(admin_shell(&table, &inner)).into_response()
}

async fn admin_new_get(State(st): State<Arc<AppState>>, Path(table): Path<String>) -> Response {
    if let Some(r) = admin_gate(&st) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    match form::from_schema(url, &table, "insert", None) {
        Ok(frm) => Html(form::render(&frm, &format!("admin-{table}-new"), None, None))
            .into_response(),
        Err(e) => Html(format!("<p>{}</p>", esc(&e))).into_response(),
    }
}

async fn admin_new_post(
    State(st): State<Arc<AppState>>,
    Path(table): Path<String>,
    Form(posted): Form<HashMap<String, String>>,
) -> Response {
    if let Some(r) = admin_gate(&st) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    match form::from_schema(url, &table, "insert", None) {
        Ok(frm) => submit_and_respond(&frm, &format!("admin-{table}-new"), url, posted),
        Err(e) => Html(format!("<p>{}</p>", esc(&e))).into_response(),
    }
}

async fn admin_edit_get(
    State(st): State<Arc<AppState>>,
    Path((table, id)): Path<(String, String)>,
) -> Response {
    if let Some(r) = admin_gate(&st) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    let row = match db::get(url, &table, &id) {
        Ok(Value::Null) => {
            return Html(format!("<p>row {id} not found</p>")).into_response();
        }
        Ok(v) => v,
        Err(e) => return Html(format!("<p>{}</p>", esc(&e))).into_response(),
    };
    match form::from_schema(url, &table, "update", Some(&id)) {
        Ok(frm) => Html(form::render(
            &frm,
            &format!("admin-{table}-edit"),
            Some(&row),
            None,
        ))
        .into_response(),
        Err(e) => Html(format!("<p>{}</p>", esc(&e))).into_response(),
    }
}

async fn admin_edit_post(
    State(st): State<Arc<AppState>>,
    Path((table, id)): Path<(String, String)>,
    Form(posted): Form<HashMap<String, String>>,
) -> Response {
    if let Some(r) = admin_gate(&st) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    match form::from_schema(url, &table, "update", Some(&id)) {
        Ok(frm) => submit_and_respond(&frm, &format!("admin-{table}-edit"), url, posted),
        Err(e) => Html(format!("<p>{}</p>", esc(&e))).into_response(),
    }
}

async fn admin_delete(
    State(st): State<Arc<AppState>>,
    Path((table, id)): Path<(String, String)>,
) -> Response {
    if let Some(r) = admin_gate(&st) {
        return r;
    }
    let url = st.db_url.as_deref().unwrap();
    if let Err(e) = db::delete(url, &table, &id) {
        return Html(format!("<p>delete failed: {}</p>", esc(&e))).into_response();
    }
    Redirect::to(&format!("/admin/{table}")).into_response()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
