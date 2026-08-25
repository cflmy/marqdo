//! WebSocket client primitive (design ext-web-net §3.4).
//!
//! Single request–response style: connect, send one text `message`, collect all
//! server text replies until close, then return `{ok, messages}`.

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio_tungstenite::tungstenite::http::header::HeaderName;

/// Convert `headers` (a JSON map) into tungstenite header pairs.
fn headers_from_value(headers: Option<&Value>) -> Vec<(HeaderName, String)> {
    let Some(h) = headers else {
        return Vec::new();
    };
    match h {
        Value::Object(m) => m
            .iter()
            .filter_map(|(k, v)| {
                HeaderName::from_bytes(k.as_bytes())
                    .ok()
                    .map(|hn| (hn, v.as_str().unwrap_or("").to_string()))
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Connect to a WebSocket URL, send `message`, drain text replies until close.
pub fn connect(
    url: &str,
    message: &str,
    headers: Option<&Value>,
    timeout_sec: u64,
) -> Result<Value, String> {
    if !(url.starts_with("ws://") || url.starts_with("wss://")) {
        return Err("ws url must start with ws:// or wss://".into());
    }

    let hdrs = headers_from_value(headers);
    let mut builder = tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(
        url.to_string(),
    )
    .map_err(|e| format!("bad ws url: {e}"))?;
    for (hn, v) in &hdrs {
        builder
            .headers_mut()
            .insert(hn.clone(), v.parse().map_err(|e| format!("bad header value: {e}"))?);
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async move {
        let timeout = tokio::time::Duration::from_secs(timeout_sec.max(1));
        let fut = tokio_tungstenite::connect_async(builder);
        let (mut socket, _resp) = tokio::time::timeout(timeout, fut)
            .await
            .map_err(|_| "ws connect timeout".to_string())?
            .map_err(|e| format!("ws connect: {e}"))?;

        let send = tokio_tungstenite::tungstenite::Message::Text(message.to_string());
        socket
            .send(send)
            .await
            .map_err(|e| format!("ws send: {e}"))?;

        let mut messages = Vec::new();
        loop {
            match tokio::time::timeout(timeout, socket.next()).await {
                Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Text(t)))) => {
                    messages.push(t.to_string());
                }
                Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))))
                | Ok(Some(Err(_))) => break,
                Ok(None) => break,
                Ok(Some(Ok(_))) => {}
                Err(_) => break,
            }
        }
        Ok(json!({ "ok": true, "messages": messages }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::Message;

    #[test]
    fn echo_roundtrip() {
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let listener = rt.block_on(async { TcpListener::bind("127.0.0.1:0").await.expect("bind") });
        let addr = listener.local_addr().expect("addr");
        let server = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("server rt");
            rt.block_on(async move {
                let (stream, _) = listener.accept().await.expect("accept");
                let ws = tokio_tungstenite::accept_async(stream).await.expect("ws");
                let (mut sink, mut source) = ws.split();
                if let Some(Ok(Message::Text(t))) = source.next().await {
                    let _ = sink.send(Message::Text(t)).await;
                }
            });
        });

        let url = format!("ws://{addr}/");
        let got = connect(&url, "ping", None, 5).expect("connect");
        assert_eq!(got["ok"], true);
        assert_eq!(got["messages"], json!(["ping"]));
        server.join().expect("server join");
    }

    #[test]
    fn refuses_bad_url() {
        let err = connect("http://127.0.0.1:1", "hi", None, 1).unwrap_err();
        assert!(err.contains("ws://") || err.contains("ws url"));
    }

    #[test]
    fn refused_connection() {
        // Connect to a port that is closed; connect_async fails fast.
        let rt = tokio::runtime::Runtime::new().expect("rt");
        let listener = rt.block_on(async { TcpListener::bind("127.0.0.1:0").await.expect("bind") });
        let addr = listener.local_addr().expect("addr");
        drop(listener); // port now closed
        let url = format!("ws://{addr}/");
        let err = connect(&url, "hi", None, 2).unwrap_err();
        assert!(
            err.starts_with("ws connect") || err.contains("timeout"),
            "unexpected error: {err}"
        );
    }
}
