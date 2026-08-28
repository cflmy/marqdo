//! Password hashing (argon2id) for admin users and auth tables.

use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use getrandom::getrandom;

/// True when `stored` looks like an argon2 PHC string.
pub fn is_hashed(stored: &str) -> bool {
    stored.starts_with("$argon2")
}

/// Hash a plaintext password (argon2id, random salt).
pub fn hash_password(plain: &str) -> Result<String, String> {
    if plain.is_empty() {
        return Err("password must not be empty".into());
    }
    let mut salt_bytes = [0u8; 16];
    getrandom(&mut salt_bytes).map_err(|e| format!("getrandom: {e}"))?;
    let salt = SaltString::encode_b64(&salt_bytes).map_err(|e| e.to_string())?;
    let argon2 = Argon2::default();
    argon2
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

/// Verify plaintext against stored hash or legacy plaintext (dev / gold tests).
pub fn verify_password(plain: &str, stored: &str) -> bool {
    if stored.is_empty() || plain.is_empty() {
        return false;
    }
    if is_hashed(stored) {
        let parsed = match PasswordHash::new(stored) {
            Ok(h) => h,
            Err(_) => return false,
        };
        Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok()
    } else {
        plain == stored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("secret").unwrap();
        assert!(is_hashed(&hash));
        assert!(verify_password("secret", &hash));
        assert!(!verify_password("wrong", &hash));
    }

    #[test]
    fn legacy_plaintext_still_works() {
        assert!(verify_password("secret", "secret"));
        assert!(!verify_password("other", "secret"));
    }
}
