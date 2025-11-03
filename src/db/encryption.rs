// CRITICAL: check unneceserry things
use crate::cryptography::crypto::{encrypt_data, decrypt_data, generate_encryption_key};
use crate::logger::logger;
use std::sync::OnceLock;

// CRITICAL: make this global not only startuop
// Global encryption key (initialized once at startup)
static ENCRYPTION_KEY: OnceLock<Vec<u8>> = OnceLock::new();

// Initialize the encryption key for database operations.
pub fn initialize_encryption_key() {
    ENCRYPTION_KEY.get_or_init(|| {
        // PRODUCTION WARNING: Replace this with secure key loading!
        // For now, we generate a random key at startup
        let key = generate_encryption_key();
        logger::security("Database encryption key initialized (TEMPORARY - will not persist across restarts)");
        logger::warning("PRODUCTION WARNING: Use persistent key storage for production environments!");
        key
    });
}
// CRITICAL: implement panics well
// Get the encryption key for database operations.
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
// CRITICAL: check this if it is safe or not
// Encrypt a generic string value for database storage.
pub fn encrypt_value(value: &str) -> Result<String, String> {
    let key = get_encryption_key();
    encrypt_data(value, key)
}

// CRITICAL: check this if it is safe or not
// Decrypt a generic string value from the database.
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

