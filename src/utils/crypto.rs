use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    aead::rand_core::RngCore,
    Aes256Gcm, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use sha2::{Sha256, Digest};

const NONCE_SIZE: usize = 12;

fn derive_key() -> [u8; 32] {
    if let Ok(key) = std::env::var("ENCRYPTION_KEY") {
        if key.len() >= 32 {
            let mut derived = [0u8; 32];
            let bytes = key.as_bytes();
            derived.copy_from_slice(&bytes[..32]);
            return derived;
        }
        tracing::warn!("ENCRYPTION_KEY is less than 32 bytes, using derived fallback");
    } else {
        tracing::warn!("ENCRYPTION_KEY env var not set. Using derived key (less secure for production). Set ENCRYPTION_KEY (32+ chars) for production.");
    }

    let fallback = format!(
        "dipak_site_quiz_secret_{}",
        std::env::var("DATABASE_URL").unwrap_or_default()
    );
    let mut hasher = Sha256::new();
    hasher.update(fallback.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

pub fn encrypt(plaintext: &str) -> Result<String, String> {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

pub fn decrypt(encoded: &str) -> Result<String, String> {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;

    let combined = BASE64
        .decode(encoded)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    if combined.len() < NONCE_SIZE {
        return Err("Encrypted data too short".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext)
        .map_err(|e| format!("Failed to decode UTF-8: {}", e))
}
