use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;
use crate::logger::logger;

// Brute force protection constants
const MAX_FAILED_ATTEMPTS: u32 = 5;
// CRITICAL: dont forget adjust
const LOCKOUT_DURATION_SECONDS: u64 = 1; 
const ATTEMPT_WINDOW_SECONDS: u64 = 786;

// Structure to track login attempts for a user
#[derive(Debug, Clone)]
struct LoginAttempts {
    failed_count: u32,
    last_attempt_time: u64,
    lockout_until: Option<u64>,
}

// CRITICAL: research is there any way to check from trusted source
// Follow user interactions 
lazy_static! {
    static ref LOGIN_ATTEMPTS: Mutex<HashMap<String, LoginAttempts>> = Mutex::new(HashMap::new());
}

// Get current Unix timestamp in seconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Check if an account is currently locked out due to failed login attempts.
pub fn is_account_locked(username: &str) -> bool {
    let mut attempts = LOGIN_ATTEMPTS.lock().unwrap();
    let current_time = current_timestamp();
    
    if let Some(user_attempts) = attempts.get(username) {
        // Check if account has a lockout
        if let Some(lockout_until) = user_attempts.lockout_until {
            if current_time < lockout_until {
                // Account is still locked
                let remaining = lockout_until - current_time;
                logger::security(&format!("Account {} is locked. {} seconds remaining", username, remaining));
                return true;
            } else {
                // Lockout expired, clear it
                logger::security(&format!("Lockout expired for account {}", username));
                attempts.remove(username);
                return false;
            }
        }
    }
    
    false
}

// Get remaining lockout time in seconds.
pub fn get_lockout_remaining(username: &str) -> Option<u64> {
    let attempts = LOGIN_ATTEMPTS.lock().unwrap();
    let current_time = current_timestamp();
    
    if let Some(user_attempts) = attempts.get(username) {
        if let Some(lockout_until) = user_attempts.lockout_until {
            if current_time < lockout_until {
                return Some(lockout_until - current_time);
            }
        }
    }
    
    None
}

// Record a failed login attempt.
pub fn record_failed_attempt(username: &str) {
    let mut attempts = LOGIN_ATTEMPTS.lock().unwrap();
    let current_time = current_timestamp();
    
    let user_attempts = attempts.entry(username.to_string())
        .or_insert(LoginAttempts {
            failed_count: 0,
            last_attempt_time: current_time,
            lockout_until: None,
        });
    
    // Check if this is within the tracking window
    // CRITICAL: check your old blockchain code (you did samething before it was totally secure)
    if current_time - user_attempts.last_attempt_time > ATTEMPT_WINDOW_SECONDS {
        // Outside window, reset counter
        user_attempts.failed_count = 1;
        user_attempts.last_attempt_time = current_time;
        logger::info(&format!("Failed login attempt for {}: 1/{}", username, MAX_FAILED_ATTEMPTS));
    } else {
        // Within window, increment counter
        user_attempts.failed_count += 1;
        user_attempts.last_attempt_time = current_time;
        logger::warning(&format!("Failed login attempt for {}: {}/{}", 
                                 username, user_attempts.failed_count, MAX_FAILED_ATTEMPTS));
    }
    
    // Check if threshold exceeded
    if user_attempts.failed_count >= MAX_FAILED_ATTEMPTS {
        let lockout_until = current_time + LOCKOUT_DURATION_SECONDS;
        user_attempts.lockout_until = Some(lockout_until);
        
        logger::security(&format!("ACCOUNT LOCKED: {} due to {} failed attempts. Locked for {} seconds", 
                                  username, user_attempts.failed_count, LOCKOUT_DURATION_SECONDS));
    }
}

// Record a successful login attempt.
pub fn record_successful_login(username: &str) {
    let mut attempts = LOGIN_ATTEMPTS.lock().unwrap();
    
    // Clear failed attempts on successful login
    if attempts.remove(username).is_some() {
        logger::info(&format!("Cleared failed login attempts for {}", username));
    }
}

// Get current failed attempt count for a username.
pub fn get_failed_attempts(username: &str) -> u32 {
    let attempts = LOGIN_ATTEMPTS.lock().unwrap();
    
    if let Some(user_attempts) = attempts.get(username) {
        user_attempts.failed_count
    } else {
        0
    }
}
// CRITICAL: remove this because this is from chatgpt
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_account_not_locked_initially() {
//         assert!(!is_account_locked("testuser"));
//     }

//     #[test]
//     fn test_failed_attempts_increment() {
//         let username = "test_fail_user";
        
//         for i in 1..=3 {
//             record_failed_attempt(username);
//             assert_eq!(get_failed_attempts(username), i);
//         }
//     }

//     #[test]
//     fn test_account_locks_after_max_attempts() {
//         let username = "test_lock_user";
        
//         // Record max attempts
//         for _ in 0..MAX_FAILED_ATTEMPTS {
//             record_failed_attempt(username);
//         }
        
//         // Should be locked now
//         assert!(is_account_locked(username));
//     }

//     #[test]
//     fn test_successful_login_clears_attempts() {
//         let username = "test_clear_user";
        
//         // Record some failed attempts
//         record_failed_attempt(username);
//         record_failed_attempt(username);
//         assert_eq!(get_failed_attempts(username), 2);
        
//         // Successful login
//         record_successful_login(username);
        
//         // Attempts should be cleared
//         assert_eq!(get_failed_attempts(username), 0);
//     }
// }

