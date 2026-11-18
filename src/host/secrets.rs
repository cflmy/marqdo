//! Cryptographic token helpers for Mid M3 (`lib/secrets`).

use crate::value::Value;

fn nbytes(n: Option<&Value>) -> Result<usize, String> {
    match n {
        None | Some(Value::None) => Ok(16),
        Some(Value::Int(i)) => {
            if *i <= 0 {
                return Err("n must be positive".into());
            }
            if *i > 1024 {
                return Err("n too large (max 1024)".into());
            }
            Ok(*i as usize)
        }
        Some(_) => Err("n needs int".into()),
    }
}

fn fill_random(buf: &mut [u8]) -> Result<(), String> {
    getrandom::getrandom(buf).map_err(|e| format!("secrets: getrandom failed: {e}"))
}

pub fn token_hex(n: Option<&Value>) -> Result<Value, String> {
    let len = nbytes(n)?;
    let mut buf = vec![0u8; len];
    fill_random(&mut buf)?;
    Ok(Value::Text(
        buf.iter().map(|b| format!("{b:02x}")).collect(),
    ))
}

const URLSAFE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub fn token_urlsafe(n: Option<&Value>) -> Result<Value, String> {
    let len = nbytes(n)?;
    let mut buf = vec![0u8; len];
    fill_random(&mut buf)?;
    // URL-safe Base64 without padding.
    let mut out = String::with_capacity((len + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= buf.len() {
        let n = ((buf[i] as u32) << 16) | ((buf[i + 1] as u32) << 8) | (buf[i + 2] as u32);
        out.push(URLSAFE[((n >> 18) & 63) as usize] as char);
        out.push(URLSAFE[((n >> 12) & 63) as usize] as char);
        out.push(URLSAFE[((n >> 6) & 63) as usize] as char);
        out.push(URLSAFE[(n & 63) as usize] as char);
        i += 3;
    }
    let rem = buf.len() - i;
    if rem == 1 {
        let n = (buf[i] as u32) << 16;
        out.push(URLSAFE[((n >> 18) & 63) as usize] as char);
        out.push(URLSAFE[((n >> 12) & 63) as usize] as char);
    } else if rem == 2 {
        let n = ((buf[i] as u32) << 16) | ((buf[i + 1] as u32) << 8);
        out.push(URLSAFE[((n >> 18) & 63) as usize] as char);
        out.push(URLSAFE[((n >> 12) & 63) as usize] as char);
        out.push(URLSAFE[((n >> 6) & 63) as usize] as char);
    }
    Ok(Value::Text(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_hex_len() {
        let v = token_hex(Some(&Value::Int(8))).unwrap();
        match v {
            Value::Text(s) => assert_eq!(s.len(), 16),
            _ => panic!("expected text"),
        }
    }
}
