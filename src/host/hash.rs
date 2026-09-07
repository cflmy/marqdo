//! Digest / HMAC host primitives for Mid M3 (`lib/hash`).

use hmac::{Hmac, Mac};
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest, Sha256};

use crate::value::Value;

type HmacSha256 = Hmac<Sha256>;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} needs text")),
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn sha256(text: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let mut h = Sha256::new();
    h.update(t.as_bytes());
    Ok(Value::Text(to_hex(&h.finalize())))
}

pub fn sha1(text: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let mut h = Sha1::new();
    h.update(t.as_bytes());
    Ok(Value::Text(to_hex(&h.finalize())))
}

pub fn md5(text: &Value) -> Result<Value, String> {
    let t = as_text(text, "text")?;
    let mut h = Md5::new();
    h.update(t.as_bytes());
    Ok(Value::Text(to_hex(&h.finalize())))
}

pub fn hmac_sha256(key: &Value, text: &Value) -> Result<Value, String> {
    let k = as_text(key, "key")?;
    let t = as_text(text, "text")?;
    let mut mac =
        HmacSha256::new_from_slice(k.as_bytes()).map_err(|e| format!("hmac_sha256: {e}"))?;
    mac.update(t.as_bytes());
    Ok(Value::Text(to_hex(&mac.finalize().into_bytes())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_empty() {
        let v = sha256(&Value::Text("".into())).unwrap();
        assert_eq!(
            v,
            Value::Text(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into()
            )
        );
    }
}
