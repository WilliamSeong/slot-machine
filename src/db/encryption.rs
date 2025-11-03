use crate::cryptography::crypto::{encrypt_data, decrypt_data, generate_encryption_key};
use crate::logger::logger;
use std::sync::OnceLock;
use std::fs;
use std::path::Path;
use std::env;

// Global encryption key (initialized once at startup)
static ENCRYPTION_KEY: OnceLock<Vec<u8>> = OnceLock::new();

// Path to the encryption key file
const KEY_FILE_PATH: &str = ".encryption_key";
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

/// Load encryption key from secure file
fn load_key_from_file() -> Option<Vec<u8>> {
    let path = Path::new(KEY_FILE_PATH);
    
    if !path.exists() {
        logger::info(&format!("Encryption key file not found at: {}", KEY_FILE_PATH));
        return None;
    }
    
    match fs::read_to_string(path) {
        Ok(key_hex) => {
            let key_hex = key_hex.trim();
            match hex::decode(key_hex) {
                Ok(key) => {
                    if key.len() == 32 {
                        logger::security(&format!("Encryption key loaded from file: {}", KEY_FILE_PATH));
                        Some(key)
                    } else {
                        logger::error(&format!("Key file contains invalid key length (expected 32 bytes, got {})", key.len()));
                        None
                    }
                }
                Err(e) => {
                    logger::error(&format!("Failed to decode hex key from file: {}", e));
                    None
                }
            }
        }
        Err(e) => {
            logger::error(&format!("Failed to read encryption key file: {}", e));
            None
        }
    }
}

/// Save encryption key to secure file
fn save_key_to_file(key: &[u8]) -> Result<(), String> {
    let key_hex = hex::encode(key);
    
    match fs::write(KEY_FILE_PATH, key_hex) {
        Ok(_) => {
            logger::security(&format!("Encryption key saved to file: {}", KEY_FILE_PATH));
            
            // On Unix systems, set file permissions to read/write for owner only (600)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(KEY_FILE_PATH) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o600);
                    if let Err(e) = fs::set_permissions(KEY_FILE_PATH, perms) {
                        logger::warning(&format!("Failed to set secure file permissions: {}", e));
                    } else {
                        logger::security("Secure file permissions set (read/write owner only)");
                    }
                }
            }
            
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to save encryption key to file: {}", e);
            logger::error(&error_msg);
            Err(error_msg)
        }
    }
}

/// Initialize the encryption key for database operations.
/// Priority order:
/// 1. Load from environment variable (CASINO_ENCRYPTION_KEY)
/// 2. Load from secure file (.encryption_key)
/// 3. Generate new key and save to file
pub fn initialize_encryption_key() {
    ENCRYPTION_KEY.get_or_init(|| {
        logger::info("Initializing encryption key for database operations");
        
        // Try to load from environment variable first
        if let Some(key) = load_key_from_env() {
            logger::security("Using encryption key from environment variable (recommended for production)");
            return key;
        }
        
        // Try to load from file
        if let Some(key) = load_key_from_file() {
            logger::security("Using persistent encryption key from file");
            return key;
        }
        
        // Generate new key and save to file
        logger::warning("No existing encryption key found - generating new key");
        let key = generate_encryption_key();
        
        match save_key_to_file(&key) {
            Ok(_) => {
                logger::security("New encryption key generated and saved successfully");
                logger::info(&format!("Key will persist across application restarts via file: {}", KEY_FILE_PATH));
                logger::info(&format!("For production, consider using environment variable: {}", ENV_KEY_NAME));
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

// Encrypt a balance value for storage in the database.
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

// Decrypt a balance value from the database.
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
/// Encrypt a generic string value for database storage.
pub fn encrypt_value(value: &str) -> Result<String, String> {
    let key = get_encryption_key();
    encrypt_data(value, key)
}

/// Decrypt a generic string value from the database.
pub fn decrypt_value(encrypted_value: &str) -> Result<String, String> {
    let key = get_encryption_key();
    decrypt_data(encrypted_value, key)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_balance_encryption_decryption() {
//         initialize_encryption_key();
        
//         let original_balance = 1234.56;
//         let encrypted = encrypt_balance(original_balance).unwrap();
//         let decrypted = decrypt_balance(&encrypted).unwrap();
        
//         assert!((original_balance - decrypted).abs() < 0.001);
//     }

//     #[test]
//     fn test_tampering_detection() {
//         initialize_encryption_key();
        
//         let balance = 1000.00;
//         let mut encrypted = encrypt_balance(balance).unwrap();
        
//         // Tamper with the encrypted data
//         encrypted.push('X');
        
//         // Decryption should fail
//         assert!(decrypt_balance(&encrypted).is_err());
//     }

//     #[test]
//     fn test_value_encryption() {
//         initialize_encryption_key();
        
//         let original = "sensitive data";
//         let encrypted = encrypt_value(original).unwrap();
//         let decrypted = decrypt_value(&encrypted).unwrap();
        
//         assert_eq!(original, decrypted);
//     }
// }

