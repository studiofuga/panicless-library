use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use crate::errors::{AppError, AppResult};

/// TokenCrypto handles encryption and decryption of sensitive tokens
/// using ChaCha20-Poly1305 (AEAD cipher)
pub struct TokenCrypto {
    cipher: ChaCha20Poly1305,
}

impl TokenCrypto {
    /// Creates a new TokenCrypto instance from a base64-encoded key
    /// The key must be 32 bytes (44 characters when base64-encoded)
    pub fn new(key_base64: &str) -> AppResult<Self> {
        let key_bytes = base64::decode(key_base64)
            .map_err(|_| AppError::Internal("Invalid encryption key format".to_string()))?;

        if key_bytes.len() != 32 {
            return Err(AppError::Internal(
                "Encryption key must be exactly 32 bytes (44 base64 characters)".to_string(),
            ));
        }

        let key = Key::from_slice(&key_bytes);
        let cipher = ChaCha20Poly1305::new(key);

        Ok(Self { cipher })
    }

    /// Generates a new random encryption key and returns it as base64
    /// Use this once during setup and store the result in ENCRYPTION_KEY env var
    pub fn generate_key() -> String {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        base64::encode(key)
    }

    /// Encrypts plaintext and returns base64(nonce + ciphertext)
    /// Each call uses a random nonce for security
    pub fn encrypt(&self, plaintext: &str) -> AppResult<String> {
        // Generate 96-bit random nonce
        let nonce_bytes = rand::random::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the plaintext
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| AppError::Internal("Token encryption failed".to_string()))?;

        // Concatenate nonce + ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        // Return as base64
        Ok(base64::encode(result))
    }

    /// Decrypts a token encrypted by encrypt()
    /// Expects base64(nonce + ciphertext) format
    pub fn decrypt(&self, encrypted_base64: &str) -> AppResult<String> {
        // Decode from base64
        let encrypted_data = base64::decode(encrypted_base64)
            .map_err(|_| AppError::Internal("Invalid encrypted token format".to_string()))?;

        if encrypted_data.len() < 12 {
            return Err(AppError::Internal(
                "Encrypted token is too short (corrupted?)".to_string(),
            ));
        }

        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| AppError::Authentication(
                "Failed to decrypt token (corrupted or tampered?)".to_string(),
            ))?;

        // Convert to string
        String::from_utf8(plaintext).map_err(|_| {
            AppError::Internal("Decrypted token is not valid UTF-8".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_parse_key() {
        let key = TokenCrypto::generate_key();

        // Verify key is 44 base64 characters (32 bytes)
        assert_eq!(key.len(), 44);

        // Verify we can create a TokenCrypto with it
        let crypto = TokenCrypto::new(&key);
        assert!(crypto.is_ok());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = TokenCrypto::generate_key();
        let crypto = TokenCrypto::new(&key).unwrap();

        let original = "sk-ant-1234567890abcdef";
        let encrypted = crypto.encrypt(original).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted); // Should be encrypted (different from plaintext)
    }

    #[test]
    fn test_different_nonces_produce_different_ciphertexts() {
        let key = TokenCrypto::generate_key();
        let crypto = TokenCrypto::new(&key).unwrap();

        let original = "sk-same-token";
        let encrypted1 = crypto.encrypt(original).unwrap();
        let encrypted2 = crypto.encrypt(original).unwrap();

        // Same plaintext with different nonces produces different ciphertexts
        assert_ne!(encrypted1, encrypted2);

        // But both decrypt to the same plaintext
        assert_eq!(crypto.decrypt(&encrypted1).unwrap(), original);
        assert_eq!(crypto.decrypt(&encrypted2).unwrap(), original);
    }

    #[test]
    fn test_invalid_key_format() {
        let result = TokenCrypto::new("invalid-key");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = base64::encode(vec![0u8; 16]); // 16 bytes instead of 32
        let result = TokenCrypto::new(&short_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext() {
        let key = TokenCrypto::generate_key();
        let crypto = TokenCrypto::new(&key).unwrap();

        let encrypted = crypto.encrypt("secret").unwrap();

        // Tamper with the ciphertext
        let mut tampered = base64::decode(&encrypted).unwrap();
        tampered[15] ^= 0xFF; // Flip some bits
        let tampered_base64 = base64::encode(tampered);

        // Decryption should fail due to authentication tag verification
        let result = crypto.decrypt(&tampered_base64);
        assert!(result.is_err());
    }
}
