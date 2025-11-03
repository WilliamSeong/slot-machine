use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2, Algorithm, Version, Params
};
use aes_gcm::{
    aead::{Aead, KeyInit, generic_array::GenericArray},
    Aes256Gcm
};
use base64::{Engine as _, engine::general_purpose};
use crate::logger::logger;

// Constants for encryption
const NONCE_SIZE: usize = 12; // 96 bits as recommended for AES-GCM
// MANDATORY: Argon2id parameters
// Argon2 configuration parameters
// based on owasp recommendations for password hashing
const ARGON2_MEM_COST: u32 = 19456; // 19 MiB memory (increased from default 12 MiB)
const ARGON2_TIME_COST: u32 = 2;     // 2 iterations (OWASP minimum)
const ARGON2_PARALLELISM: u32 = 1;   // 1 thread 

/// Get configured Argon2 instance with explicit security parameters
fn get_argon2() -> Result<Argon2<'static>, String> {
    // Explicitly configure Argon2id parameters
    let params = Params::new(
        ARGON2_MEM_COST,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
        None
    ).map_err(|e| format!("Failed to configure Argon2: {}", e))?;
    
    Ok(Argon2::new(
        Algorithm::Argon2id,  // Argon2id: resistant to both GPU and side-channel attacks
        Version::V0x13,       // Latest version
        params
    ))
}

/// Hash a password using Argon2id with explicit security parameters
pub fn hash_password(password: &str) -> Result<String, String> {
    // Generate a cryptographically secure random salt
    let salt = SaltString::generate(&mut OsRng);
    
    // Get configured Argon2id instance
    let argon2 = get_argon2()?;
    
    // Hash the password with the generated salt
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => {
            logger::info("Password hashed successfully with Argon2id");
            Ok(hash.to_string())
        },
        Err(_e) => {
            logger::error("Password hashing failed");
            // Generic error message to prevent information disclosure
            Err("Cryptographic operation failed".to_string())
        }
    }
}

/// Verify a password against a stored hash using constant-time comparison
/// SECURITY: Uses constant-time comparison to prevent timing attacks
pub fn verify_password(password: &str, hash: &str) -> bool {
    // Parse the stored hash string
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(_) => {
            // Generic error - don't reveal details
            logger::error("Hash parsing failed");
            return false;
        }
    };
    
    // Get configured Argon2 instance
    let argon2 = match get_argon2() {
        Ok(a) => a,
        Err(_) => {
            logger::error("Argon2 configuration failed");
            return false;
        }
    };
    
    // Verify using constant-time comparison
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            logger::info("Password verification successful");
            true
        },
        Err(_) => {
            // Generic logging - dont reveal specifics
            logger::info("Password verification failed");
            false
        }
    }
}

/// Generate a cryptographically secure random encryption key
pub fn generate_encryption_key() -> Vec<u8> {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    logger::security("New encryption key generated");
    key.to_vec()
}

/// Encrypt sensitive data using AES-256-GCM authenticated encryption
/// Returns generic error messages to prevent information disclosure
pub fn encrypt_data(data: &str, key: &[u8]) -> Result<String, String> {
    // Generate unique nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = GenericArray::from_slice(&nonce_bytes);
    
    // Create cipher
    let cipher = match Aes256Gcm::new_from_slice(key) {
        Ok(c) => c,
        Err(_) => {
            logger::error("Cipher initialization failed");
            return Err("Cryptographic operation failed".to_string());
        }
    };
    
    // Encrypt
    let ciphertext = match cipher.encrypt(nonce, data.as_bytes().as_ref()) {
        Ok(c) => c,
        Err(_) => {
            logger::error("Encryption operation failed");
            return Err("Cryptographic operation failed".to_string());
        }
    };
    
    // Combine nonce and ciphertext
    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    
    Ok(general_purpose::STANDARD.encode(result))
}

/// Decrypt data that was encrypted with `encrypt_data()`
/// Returns generic error messages to prevent information disclosure
pub fn decrypt_data(encrypted_data: &str, key: &[u8]) -> Result<String, String> {
    // Decode from base64
    let decoded = match general_purpose::STANDARD.decode(encrypted_data) {
        Ok(d) => d,
        Err(_) => {
            logger::error("Base64 decode failed");
            return Err("Cryptographic operation failed".to_string());
        }
    };
    
    // Validate length
    if decoded.len() < NONCE_SIZE {
        logger::error("Invalid encrypted data length");
        return Err("Cryptographic operation failed".to_string());
    }
    
    // Extract nonce and ciphertext
    let nonce = GenericArray::from_slice(&decoded[..NONCE_SIZE]);
    let ciphertext = &decoded[NONCE_SIZE..];
    
    // Create cipher
    let cipher = match Aes256Gcm::new_from_slice(key) {
        Ok(c) => c,
        Err(_) => {
            logger::error("Cipher initialization failed");
            return Err("Cryptographic operation failed".to_string());
        }
    };
    
    // Decrypt and verify
    let plaintext = match cipher.decrypt(nonce, ciphertext.as_ref()) {
        Ok(p) => p,
        Err(_) => {
            logger::error("Decryption or authentication failed");
            return Err("Cryptographic operation failed".to_string());
        }
    };
    
    // Convert to string
    match String::from_utf8(plaintext) {
        Ok(s) => Ok(s),
        Err(_) => {
            logger::error("UTF-8 conversion failed");
            Err("Cryptographic operation failed".to_string())
        }
    }
}