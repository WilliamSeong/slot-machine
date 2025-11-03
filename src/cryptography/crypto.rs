use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use aes_gcm::{
    aead::{Aead, KeyInit, generic_array::GenericArray},
    Aes256Gcm
};
use base64::{Engine as _, engine::general_purpose};
use crate::logger::logger;

// CRITICAL: check this from your old smart contract
// Constants for encryption
const NONCE_SIZE: usize = 12; // 96 bits as recommended for AES-GCM

// Hash a password using Argon2id algorithm with automatic salt generation.
pub fn hash_password(password: &str) -> Result<String, String> {
    // Generate a cryptographically secure random salt
    // Salt ensures that identical passwords have different hashes
    let salt = SaltString::generate(&mut OsRng);
    
    // Configure Argon2id with secure default parameters
    // Argon2id combines resistance to both side-channel and GPU attacks
    let argon2 = Argon2::default();
    
    // Hash the password with the generated salt
    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(hash) => {
            logger::info("Password hashed successfully");
            // Return the complete hash string (includes salt and parameters)
            Ok(hash.to_string())
        },
        Err(e) => {
            logger::error(&format!("Failed to hash password: {}", e));
            Err(format!("Password hashing error: {}", e))
        }
    }
}

// CRITICAL: timing atack check
// Verify a password against a stored hash using constant-time comparison.
pub fn verify_password(password: &str, hash: &str) -> bool {
    // Parse the stored hash string into a PasswordHash structure
    // This extracts the salt, algorithm parameters, and hash value
    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(e) => {
            // Don't reveal parsing errors to potential attackers
            logger::error(&format!("Failed to parse password hash: {}", e));
            return false;
        }
    };
    
    // Verify the password against the hash using constant-time comparison
    // This is CRITICAL for security - never use == to compare passwords!
    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            logger::info("Password verification successful");
            true
        },
        Err(_) => {
            // Don't log the specific error to avoid information leakage
            logger::info("Password verification failed");
            false
        }
    }
}

// CRITICAL: coopy paste it from your old lottery contract
// Generate a cryptographically secure random encryption key.
pub fn generate_encryption_key() -> Vec<u8> {
    // Generate a random 32-byte (256-bit) key
    // This provides maximum security for AES-256 encryption
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key.to_vec()
}
// CRITICAL: coppied pasted dont forget check because this one had one issues
// Encrypt sensitive data using AES-256-GCM authenticated encryption.
pub fn encrypt_data(data: &str, key: &[u8]) -> Result<String, String> {
    // Generate a random nonce (number used once)
    // A unique nonce is CRITICAL - never reuse a nonce with the same key!
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = GenericArray::from_slice(&nonce_bytes);
    
    // Create the AES-256-GCM cipher with the provided key
    let cipher = match Aes256Gcm::new_from_slice(key) {
        Ok(c) => c,
        Err(e) => return Err(format!("Key error: {}", e))
    };
    
    // Encrypt the data
    // GCM mode also generates an authentication tag to ensure integrity
    let ciphertext = match cipher.encrypt(nonce, data.as_bytes().as_ref()) {
        Ok(c) => c,
        Err(e) => return Err(format!("Encryption error: {}", e))
    };
    
    // Combine nonce and ciphertext for storage
    // The nonce is prepended to the ciphertext (nonce doesn't need to be secret)
    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    
    // Encode as base64 for easy storage in databases or text files
    Ok(general_purpose::STANDARD.encode(result))
}

// CRITICAL: check your old project
// Decrypt data that was encrypted with `encrypt_data()`.
pub fn decrypt_data(encrypted_data: &str, key: &[u8]) -> Result<String, String> {
    // Decode from base64
    let decoded = match general_purpose::STANDARD.decode(encrypted_data) {
        Ok(d) => d,
        Err(e) => return Err(format!("Base64 decode error: {}", e))
    };
    
    // Validate minimum length (must have nonce + at least some ciphertext)
    if decoded.len() < NONCE_SIZE {
        return Err("Invalid encrypted data".to_string());
    }
    
    // Split nonce and ciphertext
    // The nonce was prepended during encryption
    let nonce = GenericArray::from_slice(&decoded[..NONCE_SIZE]);
    let ciphertext = &decoded[NONCE_SIZE..];
    
    // Create the AES-256-GCM cipher with the provided key
    let cipher = match Aes256Gcm::new_from_slice(key) {
        Ok(c) => c,
        Err(e) => return Err(format!("Key error: {}", e))
    };
    
    // Decrypt and verify authentication tag
    // This will FAIL if the data has been tampered with
    let plaintext = match cipher.decrypt(nonce, ciphertext.as_ref()) {
        Ok(p) => p,
        Err(e) => return Err(format!("Decryption error: {}", e))
    };
    
    // Convert decrypted bytes back to UTF-8 string
    match String::from_utf8(plaintext) {
        Ok(s) => Ok(s),
        Err(e) => Err(format!("UTF-8 conversion error: {}", e))
    }
}