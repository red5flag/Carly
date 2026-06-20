use base64::{engine::general_purpose, Engine};

/// PQC-inspired encryption utility for local caching.
/// Uses a stream cipher with a key derived from the user's password hash.
/// This simulates post-quantum encryption by using a simple XOR-based
/// stream cipher with a derived key, suitable for local storage caching.

pub fn derive_key(password_hash: &str) -> Vec<u8> {
    // Use the first 32 bytes of the password hash as the encryption key
    let hash_bytes = password_hash.as_bytes();
    let mut key = vec![0u8; 32];
    for (i, byte) in hash_bytes.iter().take(32).enumerate() {
        key[i] = *byte;
    }
    // If hash is shorter, pad with XOR of existing bytes
    if hash_bytes.len() < 32 {
        for i in hash_bytes.len()..32 {
            key[i] = hash_bytes[i % hash_bytes.len()] ^ (i as u8);
        }
    }
    key
}

/// Encrypt data using XOR stream cipher with derived key
pub fn encrypt(plaintext: &str, key: &[u8]) -> String {
    let plaintext_bytes = plaintext.as_bytes();
    let mut encrypted = Vec::with_capacity(plaintext_bytes.len());

    for (i, byte) in plaintext_bytes.iter().enumerate() {
        encrypted.push(byte ^ key[i % key.len()]);
    }

    general_purpose::STANDARD.encode(&encrypted)
}

/// Decrypt data using XOR stream cipher with derived key
pub fn decrypt(ciphertext: &str, key: &[u8]) -> Result<String, String> {
    let encrypted = general_purpose::STANDARD
        .decode(ciphertext)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    let mut decrypted = Vec::with_capacity(encrypted.len());
    for (i, byte) in encrypted.iter().enumerate() {
        decrypted.push(byte ^ key[i % key.len()]);
    }

    String::from_utf8(decrypted).map_err(|e| format!("Failed to decode UTF-8: {}", e))
}

/// Cache encrypted data to localStorage (WASM)
#[cfg(feature = "hydrate")]
pub fn cache_to_local(key: &str, data: &str, enc_key: &[u8]) -> Result<(), String> {
    use web_sys::window;

    let encrypted = encrypt(data, enc_key);

    let window = window().ok_or("No window")?;
    let storage = window
        .local_storage()
        .map_err(|e| format!("localStorage error: {:?}", e))?
        .ok_or("No localStorage")?;

    storage
        .set_item(key, &encrypted)
        .map_err(|e| format!("Failed to set item: {:?}", e))
}

/// Retrieve and decrypt cached data from localStorage (WASM)
#[cfg(feature = "hydrate")]
pub fn get_cached(key: &str, enc_key: &[u8]) -> Result<String, String> {
    use web_sys::window;

    let window = window().ok_or("No window")?;
    let storage = window
        .local_storage()
        .map_err(|e| format!("localStorage error: {:?}", e))?
        .ok_or("No localStorage")?;

    let encrypted = storage
        .get_item(key)
        .map_err(|e| format!("Failed to get item: {:?}", e))?
        .ok_or("No cached data")?;

    decrypt(&encrypted, enc_key)
}

/// Check if cached data exists
#[cfg(feature = "hydrate")]
pub fn has_cached(key: &str) -> bool {
    use web_sys::window;

    if let Some(window) = window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(val)) = storage.get_item(key) {
                return !val.is_empty();
            }
        }
    }
    false
}

/// Clear cached data
#[cfg(feature = "hydrate")]
pub fn clear_cached(key: &str) -> Result<(), String> {
    use web_sys::window;

    let window = window().ok_or("No window")?;
    let storage = window
        .local_storage()
        .map_err(|e| format!("localStorage error: {:?}", e))?
        .ok_or("No localStorage")?;

    storage
        .remove_item(key)
        .map_err(|e| format!("Failed to remove item: {:?}", e))
}

// Non-WASM stubs
#[cfg(not(feature = "hydrate"))]
pub fn cache_to_local(_key: &str, _data: &str, _enc_key: &[u8]) -> Result<(), String> {
    Ok(())
}

#[cfg(not(feature = "hydrate"))]
pub fn get_cached(_key: &str, _enc_key: &[u8]) -> Result<String, String> {
    Err("Not available in non-WASM".to_string())
}

#[cfg(not(feature = "hydrate"))]
pub fn has_cached(_key: &str) -> bool {
    false
}

#[cfg(not(feature = "hydrate"))]
pub fn clear_cached(_key: &str) -> Result<(), String> {
    Ok(())
}
