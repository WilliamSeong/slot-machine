use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2, Algorithm, Version, Params
};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm
};
use base64::{Engine as _, engine::general_purpose};
use crate::logger::logger;
use std::sync::OnceLock;
use std::fs;
use std::path::Path;
use std::env;

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
    
    // Create cipher
    let cipher = match Aes256Gcm::new_from_slice(key) {
        Ok(c) => c,
        Err(_) => {
            logger::error("Cipher initialization failed");
            return Err("Cryptographic operation failed".to_string());
        }
    };
    
    // Encrypt (nonce_bytes.as_ref() gives us &[u8] which coerces to Nonce)
    let ciphertext = match cipher.encrypt(nonce_bytes.as_ref().into(), data.as_bytes().as_ref()) {
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
    let ciphertext = &decoded[NONCE_SIZE..];
    
    // Create cipher
    let cipher = match Aes256Gcm::new_from_slice(key) {
        Ok(c) => c,
        Err(_) => {
            logger::error("Cipher initialization failed");
            return Err("Cryptographic operation failed".to_string());
        }
    };
    
    // Decrypt and verify (slice automatically coerces to Nonce)
    let plaintext = match cipher.decrypt((&decoded[..NONCE_SIZE]).into(), ciphertext.as_ref()) {
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

// ==================== Encryption Key Management ====================

// Global encryption key (initialized once at startup)
static ENCRYPTION_KEY: OnceLock<Vec<u8>> = OnceLock::new();

// Environment configuration
const ENV_FILE_PATH: &str = ".env";
const ENV_KEY_NAME: &str = "CASINO_ENCRYPTION_KEY";

/// Load encryption key from environment variable
fn load_key_from_env() -> Option<Vec<u8>> {
    match env::var(ENV_KEY_NAME) {
        Ok(key_hex) => {
            match hex::decode(&key_hex) {
                Ok(key) => {
                    if key.len() == 32 {
                        logger::security(&format!("Encryption key loaded from environment variable: {}", ENV_KEY_NAME));
                        Some(key)
                    } else {
                        logger::error(&format!("Environment variable {} contains invalid key length (expected 32 bytes, got {})", ENV_KEY_NAME, key.len()));
                        None
                    }
                }
                Err(e) => {
                    logger::error(&format!("Failed to decode hex key from environment variable: {}", e));
                    None
                }
            }
        }
        Err(_) => {
            logger::info(&format!("Environment variable {} not found", ENV_KEY_NAME));
            None
        }
    }
}

/// Save encryption key to .env file
fn save_key_to_env(key: &[u8]) -> Result<(), String> {
    let key_hex = hex::encode(key);
    let entry = format!("{}={}\n", ENV_KEY_NAME, key_hex);
    
    // Read existing content if file exists
    let existing_content = if Path::new(ENV_FILE_PATH).exists() {
        fs::read_to_string(ENV_FILE_PATH).unwrap_or_default()
    } else {
        String::new()
    };
    
    // Check if key already exists
    if existing_content.contains(&format!("{}=", ENV_KEY_NAME)) {
        logger::info(&format!("Encryption key already exists in {}", ENV_FILE_PATH));
        return Ok(());
    }
    
    // Append new entry
    match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ENV_FILE_PATH)
    {
        Ok(mut file) => {
            use std::io::Write;
            match file.write_all(entry.as_bytes()) {
                Ok(_) => {
                    logger::security(&format!("Encryption key saved to {}", ENV_FILE_PATH));
                    
                    // On Unix systems, set file permissions to read/write for owner only (600)
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if let Ok(metadata) = fs::metadata(ENV_FILE_PATH) {
                            let mut perms = metadata.permissions();
                            perms.set_mode(0o600);
                            if let Err(e) = fs::set_permissions(ENV_FILE_PATH, perms) {
                                logger::warning(&format!("Failed to set secure file permissions: {}", e));
                            } else {
                                logger::security("Secure file permissions set (read/write owner only)");
                            }
                        }
                    }
                    
                    Ok(())
                }
                Err(e) => {
                    let error_msg = format!("Failed to write encryption key: {}", e);
                    logger::error(&error_msg);
                    Err(error_msg)
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to open {}: {}", ENV_FILE_PATH, e);
            logger::error(&error_msg);
            Err(error_msg)
        }
    }
}

/// Initialize the encryption key for database operations.
/// Priority order:
/// 1. Load from environment variable (CASINO_ENCRYPTION_KEY in .env)
/// 2. Generate new key and save to .env file
pub fn initialize_encryption_key() {
    ENCRYPTION_KEY.get_or_init(|| {
        logger::info("Initializing encryption key for database operations");
        
        // Try to load from environment variable
        if let Some(key) = load_key_from_env() {
            logger::security("Using encryption key from .env file");
            return key;
        }
        
        // Generate new key and save to .env file
        logger::warning("No existing encryption key found - generating new key");
        let key = generate_encryption_key();
        
        match save_key_to_env(&key) {
            Ok(_) => {
                logger::security("New encryption key generated and saved to .env file");
                logger::info("Key will persist across application restarts via .env file");
            }
            Err(e) => {
                logger::error(&format!("Failed to save encryption key: {}", e));
                logger::warning("Key will NOT persist across restarts - data will be unrecoverable!");
            }
        }
        
        key
    });
}

/// Get the encryption key for database operations.
fn get_encryption_key() -> &'static [u8] {
    ENCRYPTION_KEY.get()
        .expect("Encryption key not initialized! Call initialize_encryption_key() first")
}

// ==================== Balance Encryption ====================

/// Encrypt a balance value for storage in the database
pub fn encrypt_balance(balance: f64) -> Result<String, String> {
    let balance_str = balance.to_string();
    let key = get_encryption_key();
    
    match encrypt_data(&balance_str, key) {
        Ok(encrypted) => {
            logger::info("Balance encrypted successfully");
            Ok(encrypted)
        }
        Err(e) => {
            logger::error(&format!("Failed to encrypt balance: {}", e));
            Err(e)
        }
    }
}

/// Decrypt a balance value from the database
pub fn decrypt_balance(encrypted_balance: &str) -> Result<f64, String> {
    let key = get_encryption_key();
    
    match decrypt_data(encrypted_balance, key) {
        Ok(decrypted_str) => {
            match decrypted_str.parse::<f64>() {
                Ok(balance) => {
                    logger::info("Balance decrypted successfully");
                    Ok(balance)
                }
                Err(e) => {
                    logger::error(&format!("Failed to parse decrypted balance: {}", e));
                    Err(format!("Invalid balance format: {}", e))
                }
            }
        }
        Err(e) => {
            logger::error(&format!("Failed to decrypt balance: {}", e));
            Err(e)
        }
    }
}