use rusqlite::Connection;
use crate::interfaces::user::User;
use crate::logger::logger;
use colored::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use lazy_static::lazy_static;

// Authorization result type for cleaner error handling
pub type AuthResult<T> = Result<T, AuthError>;

// Authorization errors
#[derive(Debug)]
pub enum AuthError {
    // User doesn't have permission for this action
    InsufficientPrivileges(String),
    // Database error while checking authorization
    DatabaseError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AuthError::InsufficientPrivileges(msg) => write!(f, "Access Denied: {}", msg),
            AuthError::DatabaseError(msg) => write!(f, "Authorization Error: {}", msg),
        }
    }
}

// Check if user has a specific role
pub fn has_role(conn: &Connection, user: &User, required_role: &str) -> AuthResult<bool> {
    match user.get_role(conn) {
        Ok(role) => {
            let has_permission = role == required_role;
            if !has_permission {
                logger::security(&format!(
                    "Authorization denied: User ID: {} (role: {}) attempted to access {} functionality",
                    user.id, role, required_role
                ));
            }
            Ok(has_permission)
        }
        Err(e) => {
            logger::error(&format!(
                "Failed to retrieve role for User ID: {}: {}",
                user.id, e
            ));
            Err(AuthError::DatabaseError(format!("Could not verify user role: {}", e)))
        }
    }
}

// Verify user is a commissioner, log and return error if not
pub fn require_commissioner(conn: &Connection, user: &User) -> AuthResult<()> {
    match has_role(conn, user, "commissioner")? {
        true => {
            logger::security(&format!("Commissioner access granted for User ID: {}", user.id));
            Ok(())
        }
        false => {
            let role = user.get_role(conn).unwrap_or_else(|_| "unknown".to_string());
            logger::security(&format!(
                "SECURITY ALERT: User ID: {} (role: {}) attempted unauthorized commissioner access",
                user.id, role
            ));
            println!("\n{}", "╔═══════════════════════════════════════════╗".red());
            println!("{}", "║        🚫 ACCESS DENIED 🚫                ║".red().bold());
            println!("{}", "╠═══════════════════════════════════════════╣".red());
            println!("{}", "║  You do not have permission to access    ║".red());
            println!("{}", "║  this functionality.                     ║".red());
            println!("{}", "║  Commissioner privileges required.        ║".red());
            println!("{}", "╚═══════════════════════════════════════════╝".red());
            println!();
            Err(AuthError::InsufficientPrivileges(
                "Commissioner role required".to_string()
            ))
        }
    }
}

// Verify user is a technician, log and return error if not
pub fn require_technician(conn: &Connection, user: &User) -> AuthResult<()> {
    match has_role(conn, user, "technician")? {
        true => {
            logger::security(&format!("Technician access granted for User ID: {}", user.id));
            Ok(())
        }
        false => {
            let role = user.get_role(conn).unwrap_or_else(|_| "unknown".to_string());
            logger::security(&format!(
                "SECURITY ALERT: User ID: {} (role: {}) attempted unauthorized technician access",
                user.id, role
            ));
            println!("\n{}", "╔═══════════════════════════════════════════╗".red());
            println!("{}", "║        🚫 ACCESS DENIED 🚫                ║".red().bold());
            println!("{}", "╠═══════════════════════════════════════════╣".red());
            println!("{}", "║  You do not have permission to access    ║".red());
            println!("{}", "║  this functionality.                     ║".red());
            println!("{}", "║  Technician privileges required.          ║".red());
            println!("{}", "╚═══════════════════════════════════════════╝".red());
            println!();
            Err(AuthError::InsufficientPrivileges(
                "Technician role required".to_string()
            ))
        }
    }
}

// ==================== Brute Force Protection ====================

// Brute force protection constants
const MAX_FAILED_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION_SECONDS: u64 = 900;  // 15 minutes lockout
const ATTEMPT_WINDOW_SECONDS: u64 = 300;  // 5 minutes window for attempts

// Structure to track login attempts for a user
#[derive(Debug, Clone)]
struct LoginAttempts {
    failed_count: u32,
    last_attempt_time: u64,
    lockout_until: Option<u64>,
}

// Track login attempts per username
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

// Check if an account is currently locked out due to failed login attempts
pub fn is_account_locked(username: &str) -> bool {
    let mut attempts = LOGIN_ATTEMPTS.lock().unwrap();
    let current_time = current_timestamp();
    
    if let Some(user_attempts) = attempts.get(username) {
        if let Some(lockout_until) = user_attempts.lockout_until {
            if current_time < lockout_until {
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

// Get remaining lockout time in seconds
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

// Record a failed login attempt
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
        
        logger::security(&format!(
            "ACCOUNT LOCKED: {} due to {} failed attempts. Duration: {} seconds ({:.1} minutes)", 
            username, user_attempts.failed_count,
            LOCKOUT_DURATION_SECONDS, LOCKOUT_DURATION_SECONDS as f64 / 60.0
        ));
    }
}

// Record a successful login attempt
pub fn record_successful_login(username: &str) {
    let mut attempts = LOGIN_ATTEMPTS.lock().unwrap();
    
    // Clear failed attempts on successful login
    if attempts.remove(username).is_some() {
        logger::info(&format!("Cleared failed login attempts for {}", username));
    }
}

// Get current failed attempt count for a username
pub fn get_failed_attempts(username: &str) -> u32 {
    let attempts = LOGIN_ATTEMPTS.lock().unwrap();
    
    if let Some(user_attempts) = attempts.get(username) {
        user_attempts.failed_count
    } else {
        0
    }
}
