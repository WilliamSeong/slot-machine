use rusqlite::Connection;
use crate::interfaces::user::User;
use crate::logger::logger;
use colored::*;
// CRITICAL: copy paste it from your old lottery contract
// Authorization result type for cleaner error handling
pub type AuthResult<T> = Result<T, AuthError>;

// Authorization errors
#[derive(Debug)]
pub enum AuthError {
    // User doesn't have permission for this action
    InsufficientPrivileges(String),
    // Database error while checking authorization
    DatabaseError(String),
    // Role not found or invalid
    InvalidRole(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AuthError::InsufficientPrivileges(msg) => write!(f, "Access Denied: {}", msg),
            AuthError::DatabaseError(msg) => write!(f, "Authorization Error: {}", msg),
            AuthError::InvalidRole(msg) => write!(f, "Invalid Role: {}", msg),
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
// CRITICAL: remove if not needed
// We may need in future,Get user role safely with error handling
pub fn get_user_role_safe(conn: &Connection, user: &User) -> AuthResult<String> {
    user.get_role(conn)
        .map_err(|e| {
            logger::error(&format!(
                "Failed to retrieve role for User ID: {}: {}",
                user.id, e
            ));
            AuthError::DatabaseError(format!("Could not retrieve user role: {}", e))
        })
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_auth_error_display() {
//         let error = AuthError::InsufficientPrivileges("test".to_string());
//         assert_eq!(format!("{}", error), "Access Denied: test");
//     }
// }

