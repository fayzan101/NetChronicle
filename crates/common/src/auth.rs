use sha2::{Digest, Sha256};

/// Hash a secret with a random salt: `salt_hex:hash_hex`.
pub fn hash_secret(secret: &str) -> String {
    let salt = uuid::Uuid::new_v4().to_string();
    let hash = hex_sha256(&format!("{salt}:{secret}"));
    format!("{salt}:{hash}")
}

pub fn verify_secret(secret: &str, stored: &str) -> bool {
    let Some((salt, expected)) = stored.split_once(':') else {
        return false;
    };
    let actual = hex_sha256(&format!("{salt}:{secret}"));
    constant_time_eq(actual.as_bytes(), expected.as_bytes())
}

pub fn hash_token(token: &str) -> String {
    hex_sha256(token)
}

pub fn generate_api_key() -> String {
    format!("nck_{}", uuid::Uuid::new_v4().simple())
}

pub fn generate_bearer_token() -> String {
    format!("nct_{}", uuid::Uuid::new_v4().simple())
}

pub fn api_key_prefix(key: &str) -> String {
    key.chars().take(12).collect()
}

fn hex_sha256(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_roundtrip() {
        let stored = hash_secret("hunter2");
        assert!(verify_secret("hunter2", &stored));
        assert!(!verify_secret("wrong", &stored));
    }

    #[test]
    fn api_key_prefix_length() {
        let key = generate_api_key();
        assert!(key.starts_with("nck_"));
        assert_eq!(api_key_prefix(&key).len(), 12);
    }
}
