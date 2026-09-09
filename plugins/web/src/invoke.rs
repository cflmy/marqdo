//! HTTP → user `##` invoke routes for `app.invoke` / `configure invoke=`.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use serde_json::{json, Map, Value};

use crate::middleware::{cell_text, JsonBody};
use crate::table;
use crate::call_lib_with_args;

/// One declared invoke mount.
#[derive(Clone, Debug)]
pub struct InvokeRoute {
    pub method: String,
    pub fn_path: String,
    pub body_mode: String,
    pub return_mode: String,
}

impl InvokeRoute {
    pub fn from_json(v: &Value) -> Option<Self> {
        let obj = v.as_object()?;
        let fn_path = obj
            .get("fn")
            .or_else(|| obj.get("function"))
            .and_then(|x| x.as_str())?
            .trim()
            .to_string();
        if fn_path.is_empty() {
            return None;
        }
        let method = obj
            .get("method")
            .and_then(|x| x.as_str())
            .unwrap_or("POST")
            .to_uppercase();
        let body_mode = obj
            .get("body")
            .and_then(|x| x.as_str())
            .unwrap_or("json")
            .to_ascii_lowercase();
        let return_mode = obj
            .get("return")
            .and_then(|x| x.as_str())
            .unwrap_or("json")
            .to_ascii_lowercase();
        Some(Self {
            method,
            fn_path,
            body_mode,
            return_mode,
        })
    }
}

/// Normalize invoke-routes GFM table into `app.invoke_routes` map.
pub fn invoke_routes_from_table(table_v: &Value) -> Value {
    let rows = table::as_rows(table_v);
    let mut routes = Map::new();
    if let Some(arr) = rows.as_array() {
        for row in arr {
            let col = |keys: &[&str]| -> String {
                keys.iter()
                    .find_map(|k| row.get(*k).map(cell_text))
                    .unwrap_or_default()
            };
            let path = col(&["路径", "path", "Path"])
                .trim()
                .trim_start_matches('/')
                .to_string();
            if path.is_empty() {
                continue;
            }
            let fn_path = col(&["函数", "fn", "function", "Function"]);
            if fn_path.is_empty() {
                continue;
            }
            let method = {
                let m = col(&["方法", "method", "Method"]).to_uppercase();
                if m.is_empty() {
                    "POST".into()
                } else {
                    m
                }
            };
            let body = {
                let b = col(&["正文", "body", "Body"]).to_ascii_lowercase();
                if b.is_empty() {
                    "json".into()
                } else {
                    b
                }
            };
            let ret = {
                let r = col(&["返回", "return", "Return"]).to_ascii_lowercase();
                if r.is_empty() {
                    "json".into()
                } else {
                    r
                }
            };
            routes.insert(
                path,
                json!({
                    "method": method,
                    "fn": fn_path,
                    "body": body,
                    "return": ret,
                }),
            );
        }
    }
    Value::Object(routes)
}

pub fn mount_all(
    mut app: Router<Arc<crate::http::AppState>>,
    routes: &[(String, InvokeRoute)],
) -> Router<Arc<crate::http::AppState>> {
    for (path, route) in routes {
        app = mount_one(app, path, route.clone());
    }
    app
}

fn mount_one(
    app: Router<Arc<crate::http::AppState>>,
    path: &str,
    route: InvokeRoute,
) -> Router<Arc<crate::http::AppState>> {
    let r = route.clone();
    match route.method.as_str() {
        "GET" => app.route(
            path,
            get(move |State(st): State<Arc<crate::http::AppState>>, req: Request| {
                let r = r.clone();
                async move { invoke_handler(st, r, req).await }
            }),
        ),
        _ => app.route(
            path,
            post(move |State(st): State<Arc<crate::http::AppState>>, req: Request| {
                let r = r.clone();
                async move { invoke_handler(st, r, req).await }
            }),
        ),
    }
}

async fn invoke_handler(
    _st: Arc<crate::http::AppState>,
    route: InvokeRoute,
    req: Request,
) -> Response {
    let (parts, body) = req.into_parts();
    let query_map = query_to_map(parts.uri.query());

    let payload = match route.body_mode.as_str() {
        "query" => Value::Object(query_map.clone()),
        "raw" => {
            let bytes = match axum::body::to_bytes(body, 8 * 1024 * 1024).await {
                Ok(b) => b,
                Err(e) => {
                    return err_json(StatusCode::BAD_REQUEST, &format!("body: {e}"));
                }
            };
            json!({ "payload": String::from_utf8_lossy(&bytes).to_string() })
        }
        "form" => {
            let bytes = match axum::body::to_bytes(body, 8 * 1024 * 1024).await {
                Ok(b) => b,
                Err(e) => {
                    return err_json(StatusCode::BAD_REQUEST, &format!("body: {e}"));
                }
            };
            form_to_map(&bytes)
        }
        _ => {
            // json (default)
            let bytes = match axum::body::to_bytes(body, 8 * 1024 * 1024).await {
                Ok(b) => b,
                Err(e) => {
                    return err_json(StatusCode::BAD_REQUEST, &format!("body: {e}"));
                }
            };
            if bytes.is_empty() {
                Value::Object(query_map.clone())
            } else {
                match serde_json::from_slice::<Value>(&bytes) {
                    Ok(Value::Object(mut m)) => {
                        for (k, v) in query_map {
                            m.entry(k).or_insert(v);
                        }
                        Value::Object(m)
                    }
                    Ok(other) => json!({ "payload": other }),
                    Err(e) => {
                        return err_json(StatusCode::BAD_REQUEST, &format!("json: {e}"));
                    }
                }
            }
        }
    };

    // Bind JSON object keys as named args; always offer `payload` too.
    let mut args = Map::new();
    if let Value::Object(ref m) = payload {
        for (k, v) in m {
            args.insert(k.clone(), v.clone());
        }
    }
    if !args.contains_key("payload") {
        args.insert("payload".to_string(), payload.clone());
    }

    let fn_path = route.fn_path.clone();
    let result = tokio::task::spawn_blocking(move || call_lib_with_args(&fn_path, &Value::Object(args)))
        .await;

    let value = match result {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            let status = if e.contains("unknown library")
                || e.contains("call_lib_path: need")
                || e.contains("not allowed")
                || e.contains("no site")
            {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            return err_json(status, &e);
        }
        Err(e) => {
            return err_json(StatusCode::INTERNAL_SERVER_ERROR, &format!("join: {e}"));
        }
    };

    match route.return_mode.as_str() {
        "text" => {
            if matches!(value, Value::Null) {
                return err_json(StatusCode::INTERNAL_SERVER_ERROR, "null result");
            }
            let text = match &value {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            (
                StatusCode::OK,
                [(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("text/plain; charset=utf-8"),
                )],
                text,
            )
                .into_response()
        }
        "status" => {
            if matches!(value, Value::Null) {
                return err_json(StatusCode::INTERNAL_SERVER_ERROR, "null result");
            }
            StatusCode::NO_CONTENT.into_response()
        }
        _ => {
            let body = match json_wrap_invoke_value(value) {
                Ok((status, body)) => {
                    let mut resp = JsonBody(body).into_response();
                    *resp.status_mut() = status;
                    resp
                }
                Err(resp) => resp,
            };
            body
        }
    }
}

/// Map host return value to invoke JSON. `Null` must not look like success (GAP-08).
pub(crate) fn json_wrap_invoke_value(
    value: Value,
) -> Result<(StatusCode, Value), Response> {
    match value {
        Value::Null => Err(err_json(
            StatusCode::INTERNAL_SERVER_ERROR,
            "null result",
        )),
        Value::Object(_) => Ok((StatusCode::OK, value)),
        other => Ok((StatusCode::OK, json!({ "ok": true, "result": other }))),
    }
}

fn query_to_map(q: Option<&str>) -> Map<String, Value> {
    let mut m = Map::new();
    let Some(q) = q else {
        return m;
    };
    for pair in q.split('&') {
        if pair.is_empty() {
            continue;
        }
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        let k = urlencoding_decode(k);
        let v = urlencoding_decode(v);
        if !k.is_empty() {
            m.insert(k, json!(v));
        }
    }
    m
}

fn form_to_map(bytes: &[u8]) -> Value {
    let s = String::from_utf8_lossy(bytes);
    Value::Object(query_to_map(Some(&s)))
}

fn urlencoding_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => {
                out.push(' ');
                i += 1;
            }
            b'%' if i + 2 < b.len() => {
                let h = u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or(""), 16);
                if let Ok(c) = h {
                    out.push(c as char);
                    i += 3;
                } else {
                    out.push('%');
                    i += 1;
                }
            }
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    out
}

fn err_json(status: StatusCode, msg: &str) -> Response {
    (
        status,
        JsonBody(json!({ "ok": false, "error": msg })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_result_is_not_fake_ok() {
        let err = json_wrap_invoke_value(Value::Null).expect_err("null must fail");
        assert_eq!(err.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let ok = json_wrap_invoke_value(json!({ "hits": [] })).expect("object ok");
        assert_eq!(ok.0, StatusCode::OK);
        let wrap = json_wrap_invoke_value(json!(1)).expect("scalar wrap");
        assert_eq!(wrap.0, StatusCode::OK);
        assert_eq!(wrap.1["ok"], true);
        assert_eq!(wrap.1["result"], 1);
    }
}
