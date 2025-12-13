use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

/// Encryption service for securing sensitive data like integration credentials.
/// Uses AES-256-GCM which provides authenticated encryption (confidentiality + integrity).
#[derive(Clone)]
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    /// Create a new encryption service from a 32-byte (256-bit) key.
    /// The key should be provided as a hex-encoded string (64 characters).
    pub fn new(key_hex: &str) -> Result<Self, EncryptionError> {
        let key_bytes = hex_decode(key_hex)?;
        if key_bytes.len() != 32 {
            return Err(EncryptionError::InvalidKeyLength(key_bytes.len()));
        }

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self { cipher })
    }

    /// Generate a new random encryption key (hex-encoded).
    pub fn generate_key() -> String {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        hex_encode(&key)
    }

    /// Encrypt plaintext and return base64-encoded ciphertext.
    /// The nonce is prepended to the ciphertext.
    pub fn encrypt(&self, plaintext: &str) -> Result<String, EncryptionError> {
        // Generate a random 96-bit (12-byte) nonce
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the plaintext
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| EncryptionError::EncryptionFailed)?;

        // Prepend nonce to ciphertext and base64 encode
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(BASE64.encode(&result))
    }

    /// Decrypt base64-encoded ciphertext and return plaintext.
    pub fn decrypt(&self, ciphertext_b64: &str) -> Result<String, EncryptionError> {
        // Decode from base64
        let data = BASE64
            .decode(ciphertext_b64)
            .map_err(|_| EncryptionError::InvalidBase64)?;

        // Extract nonce (first 12 bytes) and ciphertext
        if data.len() < 12 {
            return Err(EncryptionError::InvalidCiphertext);
        }

        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| EncryptionError::DecryptionFailed)?;

        String::from_utf8(plaintext).map_err(|_| EncryptionError::InvalidUtf8)
    }

    /// Encrypt a JSON value, encrypting only the sensitive fields.
    /// Fields are considered sensitive if their key contains: secret, password, token, key, credential
    pub fn encrypt_config(&self, config: &serde_json::Value) -> Result<serde_json::Value, EncryptionError> {
        match config {
            serde_json::Value::Object(map) => {
                let mut encrypted_map = serde_json::Map::new();
                for (key, value) in map {
                    let encrypted_value = if is_sensitive_field(key) {
                        match value {
                            serde_json::Value::String(s) if !s.is_empty() => {
                                let encrypted = self.encrypt(s)?;
                                serde_json::Value::String(format!("enc:{}", encrypted))
                            }
                            _ => value.clone(),
                        }
                    } else {
                        value.clone()
                    };
                    encrypted_map.insert(key.clone(), encrypted_value);
                }
                Ok(serde_json::Value::Object(encrypted_map))
            }
            _ => Ok(config.clone()),
        }
    }

    /// Decrypt a JSON value, decrypting only the encrypted fields.
    /// Encrypted fields are prefixed with "enc:"
    pub fn decrypt_config(&self, config: &serde_json::Value) -> Result<serde_json::Value, EncryptionError> {
        match config {
            serde_json::Value::Object(map) => {
                let mut decrypted_map = serde_json::Map::new();
                for (key, value) in map {
                    let decrypted_value = match value {
                        serde_json::Value::String(s) if s.starts_with("enc:") => {
                            let encrypted = &s[4..]; // Remove "enc:" prefix
                            let decrypted = self.decrypt(encrypted)?;
                            serde_json::Value::String(decrypted)
                        }
                        _ => value.clone(),
                    };
                    decrypted_map.insert(key.clone(), decrypted_value);
                }
                Ok(serde_json::Value::Object(decrypted_map))
            }
            _ => Ok(config.clone()),
        }
    }
}

/// Check if a field name indicates sensitive data
fn is_sensitive_field(field_name: &str) -> bool {
    let lower = field_name.to_lowercase();
    lower.contains("secret")
        || lower.contains("password")
        || lower.contains("token")
        || lower.contains("key")
        || lower.contains("credential")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("private")
}

/// Decode hex string to bytes
fn hex_decode(hex: &str) -> Result<Vec<u8>, EncryptionError> {
    if hex.len() % 2 != 0 {
        return Err(EncryptionError::InvalidHex);
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| EncryptionError::InvalidHex)
        })
        .collect()
}

/// Encode bytes to hex string
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Invalid key length: expected 32 bytes, got {0}")]
    InvalidKeyLength(usize),

    #[error("Invalid hex encoding")]
    InvalidHex,

    #[error("Invalid base64 encoding")]
    InvalidBase64,

    #[error("Invalid ciphertext")]
    InvalidCiphertext,

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed - data may be corrupted or wrong key")]
    DecryptionFailed,

    #[error("Invalid UTF-8 in decrypted data")]
    InvalidUtf8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = EncryptionService::generate_key();
        assert_eq!(key.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = EncryptionService::generate_key();
        let service = EncryptionService::new(&key).unwrap();

        let plaintext = "my-super-secret-api-key-12345";
        let ciphertext = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted);
        assert_ne!(plaintext, ciphertext); // Should be encrypted
    }

    #[test]
    fn test_encrypt_config() {
        let key = EncryptionService::generate_key();
        let service = EncryptionService::new(&key).unwrap();

        let config = serde_json::json!({
            "region": "us-east-1",
            "access_key_id": "AKIAIOSFODNN7EXAMPLE",
            "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "api_token": "token123"
        });

        let encrypted = service.encrypt_config(&config).unwrap();

        // Non-sensitive fields should be unchanged
        assert_eq!(encrypted["region"], "us-east-1");

        // Sensitive fields should be encrypted (prefixed with "enc:")
        assert!(encrypted["secret_access_key"].as_str().unwrap().starts_with("enc:"));
        assert!(encrypted["api_token"].as_str().unwrap().starts_with("enc:"));

        // access_key_id contains "key" so it should be encrypted
        assert!(encrypted["access_key_id"].as_str().unwrap().starts_with("enc:"));

        // Decrypt and verify
        let decrypted = service.decrypt_config(&encrypted).unwrap();
        assert_eq!(decrypted["region"], "us-east-1");
        assert_eq!(decrypted["secret_access_key"], "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY");
        assert_eq!(decrypted["api_token"], "token123");
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = EncryptionService::generate_key();
        let key2 = EncryptionService::generate_key();

        let service1 = EncryptionService::new(&key1).unwrap();
        let service2 = EncryptionService::new(&key2).unwrap();

        let plaintext = "secret";
        let ciphertext = service1.encrypt(plaintext).unwrap();

        // Decryption with wrong key should fail
        let result = service2.decrypt(&ciphertext);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_length() {
        // Valid hex but wrong length (16 bytes instead of 32)
        let result = EncryptionService::new("0123456789abcdef0123456789abcdef");
        assert!(matches!(result, Err(EncryptionError::InvalidKeyLength(_))));
    }

    #[test]
    fn test_invalid_hex() {
        // Not valid hex
        let result = EncryptionService::new("not-valid-hex");
        assert!(matches!(result, Err(EncryptionError::InvalidHex)));
    }
}
