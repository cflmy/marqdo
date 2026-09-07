//! UUID v4 for Mid M6 (`lib/uuid`).

use crate::value::Value;

pub fn v4() -> Result<Value, String> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|e| format!("uuid.v4: {e}"))?;
    // RFC 4122 version 4 + variant 10xx
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Ok(Value::Text(format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v4_shape() {
        let v = v4().unwrap();
        let Value::Text(s) = v else { panic!("text") };
        assert_eq!(s.len(), 36);
        assert_eq!(&s[14..15], "4");
        let variant = u8::from_str_radix(&s[19..20], 16).unwrap();
        assert!(variant == 8 || variant == 9 || variant == 0xa || variant == 0xb);
    }
}
