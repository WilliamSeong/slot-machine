use colored::*;

// Validation result with error message
pub type ValidationResult = Result<(), String>;
// CRITICAL: instead of accepting string look internet find a way to do for floats
// CRITICAL: instead of float look at integer.integer type like you did in blockchain 

// Constants for validation limits
const MIN_USERNAME_LENGTH: usize = 3;
const MAX_USERNAME_LENGTH: usize = 30;
const MIN_PASSWORD_LENGTH: usize = 3;
const MAX_PASSWORD_LENGTH: usize = 128;
const MIN_DEPOSIT: f64 = 0.01;
const MAX_DEPOSIT: f64 = 1_000_000.0;
const MIN_WITHDRAWAL: f64 = 0.01;
const MAX_WITHDRAWAL: f64 = 100_000.0;
const MIN_BET: f64 = 0.01;
const MAX_BET: f64 = 10_000.0;

// Validate username for registration and login.
pub fn validate_username(username: &str) -> ValidationResult {
    // Check if empty
    if username.is_empty() {
        return Err("❌ Username cannot be empty!".to_string());
    }
    
    // Check if only whitespace
    if username.trim().is_empty() {
        return Err("❌ Username cannot be only whitespace!".to_string());
    }
    
    // Check minimum length
    if username.len() < MIN_USERNAME_LENGTH {
        return Err(format!("❌ Username must be at least {} characters long!", MIN_USERNAME_LENGTH));
    }
    
    // Check maximum length
    if username.len() > MAX_USERNAME_LENGTH {
        return Err(format!("❌ Username cannot exceed {} characters!", MAX_USERNAME_LENGTH));
    }
    
    // Check for dangerous characters (SQL injection prevention)
    let dangerous_chars = ['\'', '"', ';', '-', '\\', '/', '<', '>', '=', '(', ')'];
    for c in dangerous_chars.iter() {
        if username.contains(*c) {
            return Err(format!("❌ Username cannot contain special character: '{}'", c));
        }
    }
    
    // Check for SQL keywords (case-insensitive)
    let username_upper = username.to_uppercase();
    let sql_keywords = ["SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "CREATE", 
                        "ALTER", "TABLE", "WHERE", "FROM", "UNION", "OR", "AND"];
    for keyword in sql_keywords.iter() {
        if username_upper.contains(keyword) {
            return Err("❌ Username contains invalid keyword!".to_string());
        }
    }
    
    Ok(())
}

// Validate password for registration and login.
pub fn validate_password(password: &str) -> ValidationResult {
    // Check if empty
    if password.is_empty() {
        return Err("❌ Password cannot be empty!".to_string());
    }
    
    // Check if only whitespace
    if password.trim().is_empty() {
        return Err("❌ Password cannot be only whitespace!".to_string());
    }
    
    // Check minimum length
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(format!("❌ Password must be at least {} characters long!", MIN_PASSWORD_LENGTH));
    }
    
    // Check maximum length (prevent DOS attacks)
    if password.len() > MAX_PASSWORD_LENGTH {
        return Err(format!("❌ Password cannot exceed {} characters!", MAX_PASSWORD_LENGTH));
    }
    
    Ok(())
}
// CRITICAL: copy paste from your stacking smart contract
// Validate deposit amount.
pub fn validate_deposit(amount: f64) -> ValidationResult {
    // Check if valid number
    if amount.is_nan() || amount.is_infinite() {
        return Err("❌ Invalid deposit amount!".to_string());
    }
    
    // Check if positive
    if amount <= 0.0 {
        return Err("❌ Deposit amount must be greater than zero!".to_string());
    }
    
    // Check minimum
    if amount < MIN_DEPOSIT {
        return Err(format!("❌ Minimum deposit is ${:.2}!", MIN_DEPOSIT));
    }
    
    // Check maximum (prevent overflow and unrealistic values)
    if amount > MAX_DEPOSIT {
        return Err(format!("❌ Maximum deposit is ${:.2} per transaction!", MAX_DEPOSIT));
    }
    
    // Check for reasonable precision (max 2 decimal places)
    let rounded = (amount * 100.0).round() / 100.0;
    if (amount - rounded).abs() > 0.001 {
        return Err("❌ Deposit amount can have at most 2 decimal places!".to_string());
    }
    
    Ok(())
}
// CRITICAL: copy paste from your stacking smart contract
// Validate withdrawal amount.
pub fn validate_withdrawal(amount: f64, current_balance: f64) -> ValidationResult {
    // Check if valid number
    if amount.is_nan() || amount.is_infinite() {
        return Err("❌ Invalid withdrawal amount!".to_string());
    }
    
    // Check if positive
    if amount <= 0.0 {
        return Err("❌ Withdrawal amount must be greater than zero!".to_string());
    }
    
    // Check minimum
    if amount < MIN_WITHDRAWAL {
        return Err(format!("❌ Minimum withdrawal is ${:.2}!", MIN_WITHDRAWAL));
    }
    
    // Check maximum
    if amount > MAX_WITHDRAWAL {
        return Err(format!("❌ Maximum withdrawal is ${:.2} per transaction!", MAX_WITHDRAWAL));
    }
    
    // Check if user has sufficient funds
    if amount > current_balance {
        return Err(format!("❌ Insufficient funds! You have ${:.2}, trying to withdraw ${:.2}", 
                          current_balance, amount));
    }
    
    // Check for reasonable precision (max 2 decimal places)
    let rounded = (amount * 100.0).round() / 100.0;
    if (amount - rounded).abs() > 0.001 {
        return Err("❌ Withdrawal amount can have at most 2 decimal places!".to_string());
    }
    
    Ok(())
}
// CRITICAL: copy paste from your stacking smart contract
// Validate bet amount.
pub fn validate_bet(amount: f64, current_balance: f64) -> ValidationResult {
    // Check if valid number
    if amount.is_nan() || amount.is_infinite() {
        return Err("❌ Invalid bet amount!".to_string());
    }
    
    // Check if positive
    if amount <= 0.0 {
        return Err("❌ Bet amount must be greater than zero!".to_string());
    }
    
    // Check minimum
    if amount < MIN_BET {
        return Err(format!("❌ Minimum bet is ${:.2}!", MIN_BET));
    }
    
    // Check maximum (responsible gambling)
    if amount > MAX_BET {
        return Err(format!("❌ Maximum bet is ${:.2}!", MAX_BET));
    }
    
    // Check if user has sufficient funds
    if amount > current_balance {
        return Err(format!("❌ Insufficient funds! You have ${:.2}, trying to bet ${:.2}", 
                          current_balance, amount));
    }
    
    Ok(())
}

/// Display a validation error to the user with proper formatting.
/// 
/// # Arguments
/// * `error` - The error message to display
pub fn display_validation_error(error: &str) {
    println!("\n{}", "╔═══════════════════════════════════════════╗".red());
    println!("{}", "║        ⚠️  VALIDATION ERROR ⚠️             ║".red().bold());
    println!("{}", "╠═══════════════════════════════════════════╣".red());
    println!("║  {}  ║", error.red());
    println!("{}", "╚═══════════════════════════════════════════╝".red());
    println!();
}
// CRITICAL: copy paste from your stacking smart contract
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_valid_username() {
//         assert!(validate_username("alice").is_ok());
//         assert!(validate_username("bob123").is_ok());
//         assert!(validate_username("user_name").is_ok());
//     }

//     #[test]
//     fn test_invalid_username() {
//         assert!(validate_username("").is_err());
//         assert!(validate_username("ab").is_err()); // Too short
//         assert!(validate_username("a' OR '1'='1").is_err()); // SQL injection
//         assert!(validate_username("user;DROP TABLE").is_err()); // SQL injection
//     }

//     #[test]
//     fn test_valid_password() {
//         assert!(validate_password("pass123").is_ok());
//         assert!(validate_password("my_secure_password").is_ok());
//     }

//     #[test]
//     fn test_invalid_password() {
//         assert!(validate_password("").is_err());
//         assert!(validate_password("ab").is_err()); // Too short
//     }

//     #[test]
//     fn test_valid_deposit() {
//         assert!(validate_deposit(10.0).is_ok());
//         assert!(validate_deposit(100.50).is_ok());
//     }

//     #[test]
//     fn test_invalid_deposit() {
//         assert!(validate_deposit(0.0).is_err());
//         assert!(validate_deposit(-10.0).is_err());
//         assert!(validate_deposit(2_000_000.0).is_err()); // Too large
//     }

//     #[test]
//     fn test_valid_withdrawal() {
//         assert!(validate_withdrawal(10.0, 100.0).is_ok());
//         assert!(validate_withdrawal(50.0, 100.0).is_ok());
//     }

//     #[test]
//     fn test_invalid_withdrawal() {
//         assert!(validate_withdrawal(0.0, 100.0).is_err());
//         assert!(validate_withdrawal(-10.0, 100.0).is_err());
//         assert!(validate_withdrawal(150.0, 100.0).is_err()); // More than balance
//     }

//     #[test]
//     fn test_valid_bet() {
//         assert!(validate_bet(10.0, 100.0).is_ok());
//         assert!(validate_bet(50.0, 100.0).is_ok());
//     }

//     #[test]
//     fn test_invalid_bet() {
//         assert!(validate_bet(0.0, 100.0).is_err());
//         assert!(validate_bet(150.0, 100.0).is_err()); // More than balance
//         assert!(validate_bet(15_000.0, 20_000.0).is_err()); // Exceeds max bet
//     }
// }

