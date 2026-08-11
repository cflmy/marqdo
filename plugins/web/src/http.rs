//! Blocking HTTP listen: page shell, `/_part/{id}`, `/_form/{id}`, optional `/admin`.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Form, Path, State};
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
}

pub fn listen(
    page: &Value,
    db_url: Option<&str>,
    host: &str,
    port: u16,
    admin: bool,
    forms: HashMap<String, Value>,
) -> Result<Value, String> {
    let state = Arc::new(AppState {
        page: page.clone(),
        db_url: db_url.map(|s| s.to_string()),
        admin,
        forms,
    });

    let app = Router::new()
        .route("/", get(home))
        .route("/_part/{id}", get(part))
        .route("/_form/{id}", get(form_get).post(form_post))
        .route("/admin", get(admin_home))
        .route("/admin/{table}", get(admin_table))
        .with_state(state);

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
    let mut data = Map::new();
    for (k, v) in posted {
        data.insert(k, json!(v));
    }
    let data_v = Value::Object(data);
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
            Html(form::render(frm, &id, Some(&data_v), errors)).into_response()
        }
        Err(e) => Html(format!("<p>submit error: {}</p>", esc(&e))).into_response(),
    }
}

async fn admin_home(State(st): State<Arc<AppState>>) -> Response {
    if !st.admin {
        return Html(String::from("<p>admin disabled</p>")).into_response();
    }
    let Some(url) = st.db_url.as_deref() else {
        return Html(String::from("<p>no database</p>")).into_response();
    };
    let tables = db::list_tables(url).unwrap_or_default();
    let mut body = String::from(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"/><title>admin</title></head><body>",
    );
    body.push_str("<h1>admin</h1><ul>");
    for t in &tables {
        body.push_str(&format!("<li><a href=\"/admin/{t}\">{t}</a></li>"));
    }
    if !st.forms.is_empty() {
        body.push_str("</ul><h2>forms</h2><ul>");
        for id in st.forms.keys() {
            body.push_str(&format!("<li><a href=\"/_form/{id}\">{id}</a></li>"));
        }
    }
    body.push_str("</ul><p><a href=\"/\">home</a></p></body></html>");
    Html(body).into_response()
}

async fn admin_table(State(st): State<Arc<AppState>>, Path(table): Path<String>) -> Response {
    if !st.admin {
        return Redirect::to("/").into_response();
    }
    let Some(url) = st.db_url.as_deref() else {
        return Html(String::from("<p>no database</p>")).into_response();
    };
    let Ok(data) = db::select(url, &table, 200) else {
        return Html(format!("<p>cannot read {table}</p>")).into_response();
    };
    let rows = data
        .get("rows")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut body = format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"/><title>{table}</title></head><body>"
    );
    body.push_str(&format!(
        "<h1>{table}</h1><p><a href=\"/admin\">back</a></p><table border=\"1\" cellpadding=\"6\"><thead><tr>"
    ));
    let cols: Vec<String> = rows
        .first()
        .and_then(|r| r.as_object())
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    for c in &cols {
        body.push_str(&format!("<th>{c}</th>"));
    }
    body.push_str("</tr></thead><tbody>");
    for row in &rows {
        body.push_str("<tr>");
        let m = row.as_object().cloned().unwrap_or_else(Map::new);
        for c in &cols {
            let cell = m
                .get(c)
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    Value::Null => String::new(),
                    other => other.to_string(),
                })
                .unwrap_or_default();
            body.push_str(&format!("<td>{}</td>", esc(&cell)));
        }
        body.push_str("</tr>");
    }
    body.push_str("</tbody></table></body></html>");
    Html(body).into_response()
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
