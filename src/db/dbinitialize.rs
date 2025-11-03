use rusqlite::{Connection, Result};
use colored::*;

// Initialize all database tables for the casino application
pub fn initialize_dbs(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Create Users table with security constraints
    // Note: balance is stored as TEXT to hold encrypted data
    conn.execute(
        "Create Table If Not Exists users (
            id Integer Primary Key,
            username Text Unique Not Null,
            password Text Not Null,
            balance Text Not Null Default '0.0',
            role Text Default 'user' Check(role In ('user', 'technician', 'commissioner'))
        )",
        [],
    )?;

    // Create Games table
    conn.execute(
        "Create Table If Not Exists games (
            id Integer Primary Key,
            name Text Unique Not Null,
            played Integer,
            win Integer,
            loss Integer,
            active Bool
        )",
        [],
    )?;

    // Create User Statistics
    conn.execute(
        "Create Table If Not Exists user_statistics (
            id Integer Primary Key,
            user_id Integer Not Null,
            game_id Integer Not Null,
            win Integer,
            loss Integer,
            highest_payout Real,
            last_played Text,
            Foreign Key (user_id) References users(id),
            Foreign Key (game_id) References games(id)
        )",
        [],
    )?;

    // Create Symbol Probabilities table for commissioner control
    conn.execute(
        "Create Table If Not Exists symbol_probabilities (
            id Integer Primary Key,
            game_id Integer Not Null,
            symbol Text Not Null,
            weight Integer Not Null Default 10,
            payout_multiplier Real Not Null Default 1.0,
            Foreign Key (game_id) References games(id),
            Unique(game_id, symbol)
        )",
        [],
    )?;

    add_technician_commissioner(&conn)?;
    add_games(&conn)?;
    add_default_symbols(&conn)?;

    Ok(())
}

/// Create default administrator accounts with secure password setup
/// SECURITY: Credentials loaded from environment or generated and saved to .env
fn add_technician_commissioner(conn: &Connection) -> Result<(),rusqlite::Error> {
    use crate::cryptography::crypto::{hash_password, encrypt_balance};
    use crate::logger::logger;
    use std::env;
    
    const ENV_FILE: &str = ".env";
    
    // Load or generate technician credentials
    let tech_username = env::var("CASINO_TECH_USERNAME")
        .unwrap_or_else(|_| "technician".to_string());
    
    let tech_password = match env::var("CASINO_TECH_PASSWORD") {
        Ok(pwd) => pwd,
        Err(_) => {
            let pwd = generate_secure_password();
            save_to_env_file(ENV_FILE, "CASINO_TECH_USERNAME", &tech_username);
            save_to_env_file(ENV_FILE, "CASINO_TECH_PASSWORD", &pwd);
            pwd
        }
    };
    
    // Load or generate commissioner credentials
    let comm_username = env::var("CASINO_COMM_USERNAME")
        .unwrap_or_else(|_| "commissioner".to_string());
    
    let comm_password = match env::var("CASINO_COMM_PASSWORD") {
        Ok(pwd) => pwd,
        Err(_) => {
            let pwd = generate_secure_password();
            save_to_env_file(ENV_FILE, "CASINO_COMM_USERNAME", &comm_username);
            save_to_env_file(ENV_FILE, "CASINO_COMM_PASSWORD", &pwd);
            pwd
        }
    };
    
    // Hash passwords securely with Argon2id
    let hashed_tech_password = hash_password(&tech_password)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
    
    let hashed_comm_password = hash_password(&comm_password)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
    
    // Encrypt initial balances
    let encrypted_tech_balance = encrypt_balance(0.0)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
    
    let encrypted_comm_balance = encrypt_balance(0.0)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
    
    // Create technician account
    conn.execute(
        "Insert Or Ignore Into users (username, password, role, balance) 
        Values (?1, ?2, 'technician', ?3)",
        [&tech_username, &hashed_tech_password, &encrypted_tech_balance]
    )?;

    // Create commissioner account
    conn.execute(
        "Insert Or Ignore Into users (username, password, role, balance) 
        Values (?1, ?2, 'commissioner', ?3)",
        [&comm_username, &hashed_comm_password, &encrypted_comm_balance]
    )?;

    Ok(())
}

/// Save a key-value pair to .env file
fn save_to_env_file(file_path: &str, key: &str, value: &str) {
    use std::fs;
    use std::path::Path;
    use std::io::Write;
    
    let path = Path::new(file_path);
    let entry = format!("{}={}\n", key, value);
    
    // Read existing content if file exists
    let existing_content = if path.exists() {
        fs::read_to_string(path).unwrap_or_default()
    } else {
        String::new()
    };
    
    // Check if key already exists
    if existing_content.contains(&format!("{}=", key)) {
        return; // Don't overwrite existing entries
    }
    
    // Append new entry
    match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(entry.as_bytes()) {
                eprintln!("Failed to write to {}: {}", file_path, e);
            }
        }
        Err(e) => {
            eprintln!("Failed to open {}: {}", file_path, e);
        }
    }
}

// MANDATORY: Generate a secure random password for default accounts
fn generate_secure_password() -> String {
    use rand::Rng;
    
    // Character set for password generation
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%";
    
    // Generate 16-character random password
    let mut rng = rand::rng();
    let password: String = (0..16)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    
    password
}

// Populate the games table with available casino game modes.
fn add_games(conn: &Connection) -> Result<(),rusqlite::Error> {
    conn.execute(
        "Insert Or Ignore Into games (name, played, win, loss, active) 
        Values ('normal', 0, 0, 0, true),
                ('multi', 0, 0, 0, true),
                ('holding', 0, 0, 0, true),
                ('wheel of fortune', 0, 0, 0, true)", []
    )?;

    Ok(())
}

// Populate default symbol probabilities for all games.
fn add_default_symbols(conn: &Connection) -> Result<(),rusqlite::Error> {
    // Get all game IDs
    let mut stmt = conn.prepare("Select id, name From games")?;
    let games: Vec<(i32, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?.collect::<Result<Vec<_>, _>>()?;

    // Default symbols with weights and payout multipliers
    // NOT FAIR
    let default_symbols = [
        ("🍒", 20, 2.0),   // Cherry: 20% chance, 2x payout
        ("🍋", 20, 2.0),   // Lemon: 20% chance, 2x payout
        ("🍊", 15, 3.0),   // Orange: 15% chance, 3x payout
        ("🍇", 10, 5.0),   // Grape: 10% chance, 5x payout
        ("💎", 5, 10.0),  // Diamond: 5% chance, 10x payout
        ("🔔", 1, 20.0),   // Seven: 1% chance, 20x payout
    ];

    // Add symbols for each game - Use proper types with rusqlite::params!
    for (game_id, _game_name) in games {
        for (symbol, weight, multiplier) in &default_symbols {
            conn.execute(
                "Insert Or Ignore Into symbol_probabilities (game_id, symbol, weight, payout_multiplier)
                Values (?1, ?2, ?3, ?4)",
                rusqlite::params![game_id, symbol, *weight as i32, multiplier]
            )?;
        }
    }

    Ok(())
}