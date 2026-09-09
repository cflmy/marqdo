//! Streaming reverse proxy routes for `app.proxy` / `configure proxy=`.
//!
//! Browser → Marqdo (same origin) → upstream, piping response bodies
//! (including `text/event-stream`) without buffering the full stream.

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{MethodFilter, MethodRouter};
use axum::Router;
use futures_util::TryStreamExt;
use serde_json::{json, Map, Value};

use crate::middleware::cell_text;
use crate::table;

/// One declared proxy mount.
#[derive(Clone, Debug)]
pub struct ProxyRoute {
    pub upstream: String,
    pub stream: bool,
    pub strip_prefix: String,
    pub methods: Vec<String>,
    /// `ENV_NAME=Header-Name` pairs injected on the upstream request.
    pub headers_from_env: Vec<(String, String)>,
    pub timeout_ms: u64,
}

impl ProxyRoute {
    pub fn from_json(v: &Value) -> Option<Self> {
        let obj = v.as_object()?;
        let upstream = obj.get("upstream")?.as_str()?.trim().to_string();
        if upstream.is_empty() {
            return None;
        }
        let stream = obj
            .get("stream")
            .and_then(|x| x.as_bool())
            .unwrap_or(true);
        let strip_prefix = obj
            .get("strip_prefix")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let methods = obj
            .get("methods")
            .and_then(|x| x.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m.as_str().map(|s| s.to_uppercase()))
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| vec!["POST".into()]);
        let headers_from_env = parse_headers_from_env_value(obj.get("headers_from_env"));
        let timeout_ms = obj
            .get("timeout_ms")
            .and_then(|x| x.as_u64().or_else(|| x.as_i64().map(|i| i as u64)))
            .unwrap_or(120_000)
            .max(1);
        Some(Self {
            upstream,
            stream,
            strip_prefix,
            methods,
            headers_from_env,
            timeout_ms,
        })
    }
}

fn parse_headers_from_env_value(v: Option<&Value>) -> Vec<(String, String)> {
    let Some(v) = v else {
        return Vec::new();
    };
    match v {
        Value::String(s) => parse_headers_from_env_str(s),
        Value::Array(arr) => arr
            .iter()
            .filter_map(|x| x.as_str())
            .flat_map(parse_headers_from_env_str)
            .collect(),
        _ => Vec::new(),
    }
}

fn parse_headers_from_env_str(s: &str) -> Vec<(String, String)> {
    s.split(|c| c == ',' || c == ';')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() {
                return None;
            }
            let (env, hdr) = part.split_once('=')?;
            let env = env.trim();
            let hdr = hdr.trim();
            if env.is_empty() || hdr.is_empty() {
                return None;
            }
            Some((env.to_string(), hdr.to_string()))
        })
        .collect()
}

/// Expand `$VAR` / `${VAR}` in upstream URL strings.
pub fn expand_env(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                if let Some(end) = s[i + 2..].find('}') {
                    let name = &s[i + 2..i + 2 + end];
                    out.push_str(&std::env::var(name).unwrap_or_default());
                    i += 3 + end;
                    continue;
                }
            } else {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                if end > start {
                    let name = &s[start..end];
                    out.push_str(&std::env::var(name).unwrap_or_default());
                    i = end;
                    continue;
                }
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Normalize a proxy-routes GFM table into `app.proxy_routes` map shape.
pub fn proxy_routes_from_table(table_v: &Value) -> Value {
    let rows = table::as_rows(table_v);
    let mut routes = Map::new();
    if let Some(arr) = rows.as_array() {
        for row in arr {
            let col = |keys: &[&str]| -> String {
                keys.iter()
                    .find_map(|k| row.get(*k).map(cell_text))
                    .unwrap_or_default()
            };
            let path = normalize_path(&col(&["路径", "path", "Path"]));
            if path.is_empty() || path == "/" {
                continue;
            }
            let upstream = col(&["上游", "upstream", "Upstream"]);
            if upstream.is_empty() {
                continue;
            }
            let stream = {
                let s = col(&["流式", "stream", "Stream"]);
                if s.is_empty() {
                    true
                } else {
                    matches!(
                        s.to_ascii_lowercase().as_str(),
                        "true" | "1" | "yes" | "on" | "真"
                    )
                }
            };
            let strip_prefix = col(&["去前缀", "strip_prefix", "Strip-Prefix"]);
            let methods_raw = col(&["方法", "methods", "Methods"]);
            let methods: Vec<String> = if methods_raw.is_empty() {
                vec!["POST".into()]
            } else {
                methods_raw
                    .split(',')
                    .map(|m| m.trim().to_uppercase())
                    .filter(|m| !m.is_empty())
                    .collect()
            };
            let headers_from_env = col(&[
                "环境头",
                "headers_from_env",
                "Headers-From-Env",
            ]);
            let timeout_ms = col(&["超时", "timeout_ms", "Timeout"])
                .parse::<u64>()
                .unwrap_or(120_000);
            routes.insert(
                path.trim_start_matches('/').to_string(),
                json!({
                    "upstream": upstream,
                    "stream": stream,
                    "strip_prefix": strip_prefix,
                    "methods": methods,
                    "headers_from_env": headers_from_env,
                    "timeout_ms": timeout_ms,
                }),
            );
        }
    }
    Value::Object(routes)
}

fn normalize_path(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return String::new();
    }
    if s.starts_with('/') {
        s.to_string()
    } else {
        format!("/{s}")
    }
}

/// Mount all proxy routes onto the router.
pub fn mount_all(
    mut app: Router<Arc<crate::http::AppState>>,
    routes: &[(String, ProxyRoute)],
) -> Router<Arc<crate::http::AppState>> {
    for (path, route) in routes {
        app = mount_one(app, path, route.clone());
    }
    app
}

fn mount_one(
    app: Router<Arc<crate::http::AppState>>,
    path: &str,
    route: ProxyRoute,
) -> Router<Arc<crate::http::AppState>> {
    let mut filter: Option<MethodFilter> = None;
    for m in &route.methods {
        let f = match m.as_str() {
            "GET" => MethodFilter::GET,
            "POST" => MethodFilter::POST,
            "PUT" => MethodFilter::PUT,
            "PATCH" => MethodFilter::PATCH,
            "DELETE" => MethodFilter::DELETE,
            "HEAD" => MethodFilter::HEAD,
            "OPTIONS" => MethodFilter::OPTIONS,
            _ => continue,
        };
        filter = Some(match filter {
            None => f,
            Some(acc) => acc.or(f),
        });
    }
    let filter = filter.unwrap_or(MethodFilter::POST);
    let r = route.clone();
    let handler = MethodRouter::new().on(filter, move |State(_st): State<Arc<crate::http::AppState>>, req: Request| {
        let r = r.clone();
        async move { proxy_handler(r, req).await }
    });
    app.route(path, handler)
}

async fn proxy_handler(route: ProxyRoute, req: Request) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let in_headers = req.headers().clone();
    let body_bytes = match axum::body::to_bytes(req.into_body(), 64 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("read body: {e}"),
            )
                .into_response();
        }
    };

    let target = match build_upstream_url(&route, uri.path(), uri.query()) {
        Ok(u) => u,
        Err(e) => return (StatusCode::BAD_GATEWAY, e).into_response(),
    };

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(route.timeout_ms))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("client: {e}"),
            )
                .into_response();
        }
    };

    let mut builder = client.request(
        reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap_or(reqwest::Method::POST),
        &target,
    );

    // Forward selected inbound headers (skip hop-by-hop).
    for (name, value) in in_headers.iter() {
        let lname = name.as_str().to_ascii_lowercase();
        if is_hop_by_hop(&lname) || lname == "host" {
            continue;
        }
        if let Ok(v) = value.to_str() {
            builder = builder.header(name.as_str(), v);
        }
    }

    // Inject secrets from env (never from the browser).
    for (env_name, header_name) in &route.headers_from_env {
        if let Ok(val) = std::env::var(env_name) {
            if !val.is_empty() {
                // Common pattern: Authorization Bearer
                let hdr_val = if header_name.eq_ignore_ascii_case("authorization")
                    && !val.to_ascii_lowercase().starts_with("bearer ")
                {
                    format!("Bearer {val}")
                } else {
                    val
                };
                builder = builder.header(header_name.as_str(), hdr_val);
            }
        }
    }

    if !body_bytes.is_empty() {
        builder = builder.body(body_bytes);
    }

    let upstream = match builder.send().await {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, format!("upstream: {e}")).into_response();
        }
    };

    let status = StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let mut out_headers = HeaderMap::new();
    for (name, value) in upstream.headers().iter() {
        let lname = name.as_str().to_ascii_lowercase();
        if is_hop_by_hop(&lname) {
            continue;
        }
        if let (Ok(n), Ok(v)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()),
            HeaderValue::from_bytes(value.as_bytes()),
        ) {
            out_headers.append(n, v);
        }
    }
    // Ensure SSE content-type is preserved if present.
    if !out_headers.contains_key(header::CONTENT_TYPE) {
        if let Some(ct) = upstream.headers().get(reqwest::header::CONTENT_TYPE) {
            if let Ok(v) = HeaderValue::from_bytes(ct.as_bytes()) {
                out_headers.insert(header::CONTENT_TYPE, v);
            }
        }
    }

    if route.stream {
        // Industry SSE / reverse-proxy practice: disable buffering at CDN/Nginx.
        out_headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, no-transform"),
        );
        out_headers.insert(
            HeaderName::from_static("x-accel-buffering"),
            HeaderValue::from_static("no"),
        );
        let stream = upstream.bytes_stream().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e)
        });
        let body = Body::from_stream(stream);
        let mut resp = Response::new(body);
        *resp.status_mut() = status;
        *resp.headers_mut() = out_headers;
        resp
    } else {
        let bytes = match upstream.bytes().await {
            Ok(b) => b,
            Err(e) => {
                return (StatusCode::BAD_GATEWAY, format!("upstream body: {e}")).into_response();
            }
        };
        let mut resp = Response::new(Body::from(bytes));
        *resp.status_mut() = status;
        *resp.headers_mut() = out_headers;
        resp
    }
}

fn build_upstream_url(
    route: &ProxyRoute,
    req_path: &str,
    query: Option<&str>,
) -> Result<String, String> {
    let base = expand_env(&route.upstream);
    if base.is_empty() {
        return Err("proxy upstream empty after env expand".into());
    }
    // If upstream already looks like a full URL with path (contains scheme and
    // more than origin), and strip_prefix empties the remainder, use base as-is.
    let mut path = req_path.to_string();
    let strip = route.strip_prefix.trim();
    if !strip.is_empty() {
        let strip_n = normalize_path(strip);
        if path.starts_with(&strip_n) {
            path = path[strip_n.len()..].to_string();
            if path.is_empty() {
                path = "/".into();
            } else if !path.starts_with('/') {
                path = format!("/{path}");
            }
        }
    }

    let url = if base.contains("://") {
        let trimmed = base.trim_end_matches('/');
        // If base has no path beyond origin, append path; if base includes a
        // path suffix, append remaining path.
        if let Some(idx) = trimmed.find("://") {
            let after = &trimmed[idx + 3..];
            if after.contains('/') {
                // base includes path — append only if remaining path is not "/"
                if path == "/" {
                    trimmed.to_string()
                } else {
                    format!("{trimmed}{path}")
                }
            } else {
                format!("{trimmed}{path}")
            }
        } else {
            format!("{trimmed}{path}")
        }
    } else {
        return Err(format!("proxy upstream must be absolute URL, got `{base}`"));
    };

    Ok(match query {
        Some(q) if !q.is_empty() => {
            if url.contains('?') {
                format!("{url}&{q}")
            } else {
                format!("{url}?{q}")
            }
        }
        _ => url,
    })
}

fn is_hop_by_hop(name: &str) -> bool {
    matches!(
        name,
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailers"
            | "transfer-encoding"
            | "upgrade"
            | "content-length"
    )
}
