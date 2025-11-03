use colored::*;

// Validation result with error message
pub type ValidationResult = Result<(), String>;

// Constants for validation limits - Authentication
const MIN_USERNAME_LENGTH: usize = 3;
const MAX_USERNAME_LENGTH: usize = 30;
const MIN_PASSWORD_LENGTH: usize = 12;
const MAX_PASSWORD_LENGTH: usize = 128;

// Constants for validation limits - Financial
const MIN_DEPOSIT: f64 = 0.01;
const MAX_DEPOSIT: f64 = 1_000_000.0;
const MIN_WITHDRAWAL: f64 = 0.01;
const MAX_WITHDRAWAL: f64 = 100_000.0;

// ==================== Authentication Validation ====================

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

// ==================== Financial Validation ====================

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

// ==================== Shared Utilities ====================

// Display validation error in a formatted box
pub fn display_validation_error(error: &str) {
    println!("\n{}", "╔═══════════════════════════════════════════╗".red());
    println!("{}", "║        ⚠️  VALIDATION ERROR ⚠️             ║".red().bold());
    println!("{}", "╠═══════════════════════════════════════════╣".red());
    println!("║  {}  ║", error.red());
    println!("{}", "╚═══════════════════════════════════════════╝".red());
    println!();
}

