//! Base64 / hex encoding for Mid M1 (`lib/encoding`).
//! No third-party crates — standard alphabet + padding.

use crate::value::Value;

fn as_text<'a>(v: &'a Value, label: &str) -> Result<&'a str, String> {
    match v {
        Value::Text(s) => Ok(s.as_str()),
        _ => Err(format!("{label} needs text")),
    }
}

const B64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(text: &Value) -> Result<Value, String> {
    let s = as_text(text, "text")?;
    Ok(Value::Text(encode_base64(s.as_bytes())))
}

pub fn base64_decode(text: &Value) -> Result<Value, String> {
    let s = as_text(text, "text")?;
    let bytes = decode_base64(s)?;
    let out = String::from_utf8(bytes).map_err(|e| format!("base64_decode: invalid utf-8: {e}"))?;
    Ok(Value::Text(out))
}

pub fn hex_encode(text: &Value) -> Result<Value, String> {
    let s = as_text(text, "text")?;
    let mut out = String::with_capacity(s.len() * 2);
    for b in s.as_bytes() {
        out.push_str(&format!("{b:02x}"));
    }
    Ok(Value::Text(out))
}

pub fn hex_decode(text: &Value) -> Result<Value, String> {
    let s = as_text(text, "text")?;
    if s.len() % 2 != 0 {
        return Err("hex_decode: length must be even".into());
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    // Require ASCII hex digits only (byte-oriented).
    let raw = s.as_bytes();
    if raw.len() != s.len() {
        return Err("hex_decode: needs ASCII hex".into());
    }
    for i in (0..raw.len()).step_by(2) {
        let hi = from_hex(raw[i])?;
        let lo = from_hex(raw[i + 1])?;
        bytes.push((hi << 4) | lo);
    }
    let out = String::from_utf8(bytes).map_err(|e| format!("hex_decode: invalid utf-8: {e}"))?;
    Ok(Value::Text(out))
}

const B32: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub fn base32_encode(text: &Value) -> Result<Value, String> {
    let s = as_text(text, "text")?;
    Ok(Value::Text(encode_base32(s.as_bytes())))
}

pub fn base32_decode(text: &Value) -> Result<Value, String> {
    let s = as_text(text, "text")?;
    let bytes = decode_base32(s)?;
    let out = String::from_utf8(bytes).map_err(|e| format!("base32_decode: invalid utf-8: {e}"))?;
    Ok(Value::Text(out))
}

fn encode_base32(input: &[u8]) -> String {
    let mut out = String::new();
    let mut buffer: u64 = 0;
    let mut bits = 0u32;
    for &b in input {
        buffer = (buffer << 8) | (b as u64);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((buffer >> bits) & 31) as usize;
            out.push(B32[idx] as char);
        }
    }
    if bits > 0 {
        let idx = ((buffer << (5 - bits)) & 31) as usize;
        out.push(B32[idx] as char);
    }
    while out.len() % 8 != 0 {
        out.push('=');
    }
    out
}

fn decode_base32(input: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if cleaned.len() % 8 != 0 {
        return Err("base32_decode: length must be multiple of 8".into());
    }
    let mut out = Vec::new();
    let mut buffer: u64 = 0;
    let mut bits = 0u32;
    for c in cleaned.bytes() {
        if c == b'=' {
            break;
        }
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'2'..=b'7' => c - b'2' + 26,
            _ => return Err(format!("base32_decode: invalid character {:?}", c as char)),
        } as u64;
        buffer = (buffer << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

fn from_hex(b: u8) -> Result<u8, String> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(format!("hex_decode: invalid digit {:?}", b as char)),
    }
}

fn encode_base64(input: &[u8]) -> String {
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= input.len() {
        let n = ((input[i] as u32) << 16) | ((input[i + 1] as u32) << 8) | (input[i + 2] as u32);
        out.push(B64[((n >> 18) & 63) as usize] as char);
        out.push(B64[((n >> 12) & 63) as usize] as char);
        out.push(B64[((n >> 6) & 63) as usize] as char);
        out.push(B64[(n & 63) as usize] as char);
        i += 3;
    }
    let rem = input.len() - i;
    if rem == 1 {
        let n = (input[i] as u32) << 16;
        out.push(B64[((n >> 18) & 63) as usize] as char);
        out.push(B64[((n >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = ((input[i] as u32) << 16) | ((input[i + 1] as u32) << 8);
        out.push(B64[((n >> 18) & 63) as usize] as char);
        out.push(B64[((n >> 12) & 63) as usize] as char);
        out.push(B64[((n >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}

fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let raw = input.as_bytes();
    if raw.len() % 4 != 0 {
        return Err("base64_decode: length must be multiple of 4".into());
    }
    let mut out = Vec::with_capacity(raw.len() / 4 * 3);
    let mut i = 0;
    while i < raw.len() {
        let a = decode_b64_char(raw[i])?;
        let b = decode_b64_char(raw[i + 1])?;
        let c = raw[i + 2];
        let d = raw[i + 3];
        let c_val = if c == b'=' {
            0
        } else {
            decode_b64_char(c)?
        };
        let d_val = if d == b'=' {
            0
        } else {
            decode_b64_char(d)?
        };
        let n = (a << 18) | (b << 12) | (c_val << 6) | d_val;
        out.push(((n >> 16) & 0xff) as u8);
        if c != b'=' {
            out.push(((n >> 8) & 0xff) as u8);
        }
        if d != b'=' {
            out.push((n & 0xff) as u8);
        }
        if c == b'=' && d != b'=' {
            return Err("base64_decode: invalid padding".into());
        }
        i += 4;
    }
    Ok(out)
}

fn decode_b64_char(b: u8) -> Result<u32, String> {
    let v = match b {
        b'A'..=b'Z' => b - b'A',
        b'a'..=b'z' => b - b'a' + 26,
        b'0'..=b'9' => b - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => return Err(format!("base64_decode: invalid character {:?}", b as char)),
    };
    Ok(v as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_hello() {
        let enc = base64_encode(&Value::Text("hello".into())).unwrap();
        assert_eq!(enc, Value::Text("aGVsbG8=".into()));
        let dec = base64_decode(&enc).unwrap();
        assert_eq!(dec, Value::Text("hello".into()));
        let hx = hex_encode(&Value::Text("hi".into())).unwrap();
        assert_eq!(hx, Value::Text("6869".into()));
        assert_eq!(hex_decode(&hx).unwrap(), Value::Text("hi".into()));
        let b32 = base32_encode(&Value::Text("hello".into())).unwrap();
        assert_eq!(b32, Value::Text("NBSWY3DP".into()));
        assert_eq!(base32_decode(&b32).unwrap(), Value::Text("hello".into()));
    }
}
