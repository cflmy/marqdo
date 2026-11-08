//! A recoverable failure is data; permission errors remain explicit failures.
use crate::value::Value;

pub(super) fn capture(operation: &str, result: Result<Value, String>) -> Value {
    let (ok, value, error) = match result {
        Ok(value) => (true, value, Value::None),
        Err(message) => {
            let tagged = message.split_once(':').map(|(code, _)| code);
            let code = match tagged {
                Some(
                    code @ ("validation_failed" | "timed_out" | "permission_denied" | "not_found"
                    | "conflict" | "invalid_encoding" | "invalid_json" | "invalid_csv"
                    | "invalid_toml" | "transport_error" | "http_status" | "resource_limit"
                    | "unsupported" | "invalid_argument" | "nonexistent_time"
                    | "ambiguous_time"),
                ) => code,
                _ if message.starts_with("path escapes sandbox root:")
                    || message.ends_with("denied by host policy") =>
                {
                    "permission_denied"
                }
                _ => "operation_failed",
            };
            (
                false,
                Value::None,
                Value::Map(vec![
                    ("code".into(), Value::Text(code.into())),
                    ("message".into(), Value::Text(message)),
                    ("operation".into(), Value::Text(operation.into())),
                ]),
            )
        }
    };
    Value::Map(vec![
        ("ok".into(), Value::Bool(ok)),
        ("value".into(), value),
        ("error".into(), error),
    ])
}
