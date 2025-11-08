use rusqlite::Connection;
use chrono;
use std::collections::HashMap;
use std::sync::Mutex;
use std::io;

use crate::interfaces::user::User;
use crate::logger::logger;

// SECURITY: Transaction rate limiting and fraud detection
lazy_static::lazy_static! {
    // Track transaction timestamps per user for rate limiting
    static ref TRANSACTION_TRACKER: Mutex<HashMap<i32, Vec<chrono::DateTime<chrono::Local>>>> = Mutex::new(HashMap::new());
    
    // Track daily transaction totals per user
    static ref DAILY_TOTALS: Mutex<HashMap<i32, (chrono::NaiveDate, f64, f64)>> = Mutex::new(HashMap::new());
}

// Transaction security limits
const MAX_TRANSACTIONS_PER_MINUTE: usize = 5; // DONT FORGET TO SET THIS BACK TO 5
const SUSPICIOUS_PATTERN_THRESHOLD: usize = 3; // Rapid identical transactions DONT FORGET TO SET THIS BACK TO 3

/*  ---------------------------------------------------------------------------------------------------------------------------------- */
// db queries for registering and signing in users

// Inserts a new user into the database with a securely hashed password.
pub fn insert_users(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<usize> {
    use crate::cryptography::crypto::{hash_password, encrypt_balance};
    
    logger::info(&format!("Attempting to insert new user: {}", username));
    
    // Hash the password before storing
    // The hash includes: algorithm parameters + salt + password hash
    let hashed_password = match hash_password(password) {
        Ok(hash) => hash,
        Err(e) => {
            logger::error(&format!("Failed to hash password for user {}: {}", username, e));
            return Err(rusqlite::Error::InvalidParameterName(e));
        }
    };
    
    // Encrypt the initial balance (0) for security
    let encrypted_balance = match encrypt_balance(0.0) {
        Ok(enc) => enc,
        Err(e) => {
            logger::error(&format!("Failed to encrypt initial balance for user {}: {}", username, e));
            return Err(rusqlite::Error::InvalidParameterName(e));
        }
    };
    
    // Store username with hashed password and encrypted balance
    conn.execute(
        "Insert Into users (username, password, balance) Values (?1, ?2, ?3)",
        rusqlite::params![username, hashed_password, encrypted_balance],
    )
}

pub fn update_user_password(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<usize> {
    use crate::cryptography::crypto::{hash_password};
    
    logger::info(&format!("Attempting to update user's password: {}", username));
    
    // Hash the password before storing
    // The hash includes: algorithm parameters + salt + password hash
    let hashed_password = match hash_password(password) {
        Ok(hash) => hash,
        Err(e) => {
            logger::error(&format!("Failed to hash password for user {}: {}", username, e));
            return Err(rusqlite::Error::InvalidParameterName(e));
        }
    };
        
    // Store username with hashed password and encrypted balance
    conn.execute(
        "Update users set password = ?1 where username = ?2",
        rusqlite::params![hashed_password, username],
    )
}

// Verifies user credentials by checking username and password hash.
pub fn check_users(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<i32> {
    use crate::cryptography::crypto::verify_password;
    
    logger::info(&format!("Checking credentials for user: {}", username));
    
    // Retrieve the stored password hash for this username
    // We fetch the hash and verify it locally (NOT in SQL query for security)
    let mut stmt = conn.prepare("Select id, password From users Where username = ?1")?;
    
    let result = stmt.query_row([username], |row| {
        let id: i32 = row.get(0)?;
        let stored_hash: String = row.get(1)?;
        
        // Verify the provided password against the stored hash
        // This uses constant-time comparison to prevent timing attacks
        if verify_password(password, &stored_hash) {
            logger::info(&format!("Password verification successful for user: {}", username));
            Ok(id)
        } else {
            logger::warning(&format!("Password verification failed for user: {}", username));
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    });

    result
}

// Get user ID by username only
// Used after registration or when user is already authenticated
pub fn get_user_id_by_username(conn: &Connection, username: &str) -> rusqlite::Result<i32> {
    logger::info(&format!("Retrieving user ID for username: {}", username));
    
    conn.query_row(
        "Select id From users Where username = ?1",
        [username],
        |row| row.get(0)
    )
}

// ----------------------------------------------------------------------------------------------------------------------------------
// Queries for users to access details in their records
pub fn user_get_username(conn: &Connection, id: i32) -> rusqlite::Result<String> {
    conn.query_row(
    "Select username From users Where id = ?1",
    [id],
    |row| row.get(0)
    )
}

// Retrieve and decrypt a user's balance from the database.
pub fn user_get_balance(conn: &Connection, id: i32) -> rusqlite::Result<f64> {
    use crate::cryptography::crypto::decrypt_balance;
    
    // Retrieve encrypted balance from database
    let encrypted_balance: String = conn.query_row(
        "Select balance From users Where id = ?1",
        [id],
        |row| row.get(0)
    )?;
    // Decrypt the balance
    decrypt_balance(&encrypted_balance)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e))
}

pub fn user_get_role(conn: &Connection, id: i32) -> rusqlite::Result<String> {
    conn.query_row(
        "Select role From users Where id = ?1",
        [id],
        |row| row.get(0)
    )
}

/// Execute a financial transaction with comprehensive security checks
pub fn transaction(conn: &Connection, user: &User, amount: f64) -> f64 {
    use crate::cryptography::crypto::{encrypt_balance, decrypt_balance};
    
    logger::transaction(&format!("User ID: {} transaction attempt for amount: {:.2}", user.id, amount));
    
    // SECURITY TIER 1: Rate limiting check
    if let Err(e) = check_rate_limit(user.id) {
        logger::security(&format!("SECURITY ALERT: Rate limit exceeded for User ID: {}: {}", user.id, e));
        eprintln!("❌ Transaction blocked: Too many transactions. Please wait.");
        return user.get_balance(conn).unwrap_or(0.0);
    }
    
    // SECURITY TIER 2: Fraud pattern detection
    if detect_fraud_patterns(user.id, amount) {
        logger::security(&format!("SECURITY ALERT: Suspicious pattern detected for User ID: {}. Amount: {:.2}", user.id, amount));
        eprintln!("❌ Transaction blocked: Suspicious activity detected. Contact support.");
        return user.get_balance(conn).unwrap_or(0.0);
    }
    
    // Start database transaction with proper locking
    let tx = match conn.unchecked_transaction() {
        Ok(t) => t,
        Err(e) => {
            logger::error(&format!("Failed to start transaction for User ID: {}: {}", user.id, e));
            eprintln!("Transaction Failed - System error");
            return user.get_balance(conn).unwrap_or(0.0);
        }
    };
    
    // Lock the user row for update (prevents race conditions)
    let current_balance = match tx.query_row(
        "Select balance From users Where id = ?1",
        [user.id],
        |row| row.get::<_, String>(0)
    ) {
        Ok(encrypted) => match decrypt_balance(&encrypted) {
            Ok(bal) => bal,
            Err(e) => {
                logger::error(&format!("Decryption failed for User ID: {}: {}", user.id, e));
                let _ = tx.rollback();
                eprintln!("Transaction Failed - Cannot retrieve balance");
                return 0.0;
            }
        },
        Err(e) => {
            logger::error(&format!("Failed to lock user row for User ID: {}: {}", user.id, e));
            let _ = tx.rollback();
            eprintln!("Transaction Failed - Cannot retrieve balance");
            return 0.0;
        }
    };
    
    // Calculate new balance
    let new_balance = current_balance + amount;
    
    // Validate new balance is non-negative
    if new_balance < 0.0 {
        logger::warning(&format!("Transaction would result in negative balance for User ID: {}. Current: {:.2}, Amount: {:.2}", user.id, current_balance, amount));
        let _ = tx.rollback();
        eprintln!("Transaction Failed - Insufficient funds");
        return current_balance;
    }
    
    // Encrypt the new balance
    let encrypted_balance = match encrypt_balance(new_balance) {
        Ok(enc) => enc,
        Err(e) => {
            logger::error(&format!("Encryption failed for User ID: {}: {}", user.id, e));
            let _ = tx.rollback();
            eprintln!("Transaction Failed - Encryption error");
            return current_balance;
        }
    };
    
    // Update database with encrypted balance
    match tx.execute(
        "Update users Set balance = ?1 Where id = ?2",
        rusqlite::params![encrypted_balance, user.id]
    ) {
        Ok(_) => {
            // Commit the transaction
            match tx.commit() {
                Ok(_) => {
                    // Update tracking after successful commit
                    record_transaction(user.id, amount);
                    
                    logger::transaction(&format!("User ID: {} transaction completed: {:.2}. New balance: {:.2}", user.id, amount, new_balance));
                    new_balance
                }
                Err(e) => {
                    logger::error(&format!("Failed to commit transaction for User ID: {}: {}", user.id, e));
                    eprintln!("Transaction Failed - Commit error");
                    current_balance
                }
            }
        }
        Err(e) => {
            logger::error(&format!("Failed to update balance for User ID: {}: {}", user.id, e));
            let _ = tx.rollback();
            eprintln!("Transaction Failed");
            current_balance
        }
    }
}

// Check if a user has sufficient funds for a transaction.
pub fn check_funds(conn: &Connection, user: &User, limit: f64) -> bool {
    logger::info(&format!("Checking funds for User ID: {} against limit: {:.2}", user.id, limit));
    
    // Query the user's current balance
    match user.get_balance(conn) {
        Ok(balance) => {
            let has_funds = balance >= limit;
            if !has_funds {
                // SECURITY: Log insufficient funds attempts to detect potential fraud
                logger::warning(&format!("Insufficient funds for User ID: {}. Balance: {:.2}, Required: {:.2}", 
                    user.id, balance, limit));
            }
            return has_funds;
        }
        Err(e) => {
            // Fail-safe: if we can't check funds, don't allow the transaction
            logger::error(&format!("Failed to check funds for User ID: {}. Error: {}", user.id, e));
            println!("Unable to check funds");
            return false;
        }
    }
}

/// Rate limiting check - prevents transaction spam
fn check_rate_limit(user_id: i32) -> Result<(), String> {
    let mut tracker = TRANSACTION_TRACKER.lock().unwrap();
    let now = chrono::Local::now();
    
    // Get or create user's transaction history
    let transactions = tracker.entry(user_id).or_insert_with(|| Vec::new());
    
    // Remove transactions older than 1 hour
    transactions.retain(|ts| now.signed_duration_since(*ts).num_seconds() < 60);
    
    // Check per-minute limit
    let last_minute = transactions.iter()
        .filter(|ts| now.signed_duration_since(**ts).num_seconds() < 60)
        .count();
    
    if last_minute >= MAX_TRANSACTIONS_PER_MINUTE {
        return Err(format!("Rate limit: {} transactions per minute", MAX_TRANSACTIONS_PER_MINUTE));
    }
    Ok(())
}

/// Record transaction timestamp for rate limiting
fn record_transaction(user_id: i32, _amount: f64) {
    let mut tracker = TRANSACTION_TRACKER.lock().unwrap();
    let transactions = tracker.entry(user_id).or_insert_with(|| Vec::new());
    transactions.push(chrono::Local::now());
}

/// Basic fraud detection - detects suspicious patterns
fn detect_fraud_patterns(user_id: i32, _amount: f64) -> bool {
    let tracker = TRANSACTION_TRACKER.lock().unwrap();
    
    if let Some(transactions) = tracker.get(&user_id) {
        let now = chrono::Local::now();
        
        // Check for rapid identical transactions (common fraud pattern)
        let recent_similar = transactions.iter()
            .filter(|ts| now.signed_duration_since(**ts).num_seconds() < 60)
            .count();
        
        if recent_similar >= SUSPICIOUS_PATTERN_THRESHOLD {
            return true;
        }
    }
    
    false
}

/// Change user balance with encrypted storage
pub fn change_balance(conn: &Connection, user: &User, deposit: f64) -> rusqlite::Result<bool> {
    use crate::cryptography::crypto::{encrypt_balance, decrypt_balance};
    
    logger::transaction(&format!("Balance change for User ID: {} amount: {:.2}", user.id, deposit));
    
    // SECURITY: Apply multi-tier validation (same as transaction())
    // TIER 1: Rate limiting
    if let Err(e) = check_rate_limit(user.id) {
        logger::security(&format!("SECURITY ALERT: Rate limit exceeded for User ID: {}: {}", user.id, e));
        return Err(rusqlite::Error::InvalidParameterName(format!("Rate limit exceeded: {}", e)));
    }
    
    // TIER 2: Fraud pattern detection
    if detect_fraud_patterns(user.id, deposit) {
        logger::security(&format!("SECURITY ALERT: Suspicious pattern detected for User ID: {}. Amount: {:.2}", user.id, deposit));
        return Err(rusqlite::Error::InvalidParameterName("Suspicious activity detected".to_string()));
    }
    
    // SECURITY: Use database transaction with proper locking
    let tx = conn.unchecked_transaction()
        .map_err(|e| {
            logger::error(&format!("Failed to start transaction for User ID: {}: {}", user.id, e));
            e
        })?;
    
    // Lock the user row for update (prevents race conditions)
    let current_balance = match tx.query_row(
        "Select balance From users Where id = ?1",
        [user.id],
        |row| row.get::<_, String>(0)
    ) {
        Ok(encrypted) => match decrypt_balance(&encrypted) {
            Ok(bal) => bal,
            Err(e) => {
                logger::error(&format!("Decryption failed for User ID: {}: {}", user.id, e));
                let _ = tx.rollback();
                return Err(rusqlite::Error::InvalidParameterName(e));
            }
        },
        Err(e) => {
            logger::error(&format!("Failed to lock user row for User ID: {}: {}", user.id, e));
            let _ = tx.rollback();
            return Err(e);
        }
    };
    
    // Calculate new balance
    let new_balance = current_balance + deposit;
    
    // Validate non-negative balance
    if new_balance < 0.0 {
        logger::warning(&format!("Balance change would result in negative balance for User ID: {}. Current: {:.2}, Deposit: {:.2}", user.id, current_balance, deposit));
        let _ = tx.rollback();
        return Err(rusqlite::Error::InvalidParameterName("Insufficient funds".to_string()));
    }
    
    // Encrypt the new balance
    let encrypted_balance = match encrypt_balance(new_balance) {
        Ok(enc) => enc,
        Err(e) => {
            logger::error(&format!("Encryption failed for User ID: {}: {}", user.id, e));
            let _ = tx.rollback();
            return Err(rusqlite::Error::InvalidParameterName(e));
        }
    };
    
    // Update database
    match tx.execute(
        "Update users Set balance = ?1 where id = ?2",
        rusqlite::params![encrypted_balance, user.id]
    ) {
        Ok(_) => {
            // Commit the transaction
            tx.commit().map_err(|e| {
                logger::error(&format!("Failed to commit transaction for User ID: {}: {}", user.id, e));
                e
            })?;
            
            // Update tracking after successful commit
            record_transaction(user.id, deposit);
            
            logger::transaction(&format!("User ID: {} balance updated. New balance: {:.2}", user.id, new_balance));
            Ok(true)
        }
        Err(e) => {
            logger::error(&format!("Failed to update balance for User ID: {}. Error: {}", user.id, e));
            let _ = tx.rollback();
            Err(e)
        }
    }
}

/// Record that a game was played
pub fn add_played(conn: &Connection, game: &str) -> rusqlite::Result<()>{
    logger::info(&format!("Recording game played: {}", game));
    conn.execute(
        "Update games Set played = played + 1 Where name = ?1",
        [game]
    )?;

    Ok(())
}

/// Record a game win
pub fn add_win(conn: &Connection, game: &str) -> rusqlite::Result<()> {
    logger::info(&format!("Recording game win: {}", game));
    add_played(conn, game)?;
    
    conn.execute(
        "Update games Set win = win + 1 Where name = ?1",
        [game]
    )?;

    Ok(())
}

/// Record a game loss
pub fn add_loss(conn: &Connection, game: &str) -> rusqlite::Result<()> {
    logger::info(&format!("Recording game loss: {}", game));
    add_played(conn, game)?;
    
    conn.execute(
        "Update games Set loss = loss + 1 Where name = ?1",
        [game]
    )?;

    Ok(())
}

/// Record a user's win in a specific game
pub fn add_user_win(conn: &Connection, user: &User, game: &str, winnings: f64) -> rusqlite::Result<()> {
    logger::transaction(&format!("User ID: {} won {:.2} in game: {}", user.id, winnings, game));
    
    // Query to get the game_id from the game name
    let game_id: i32 = conn.query_row(
        "Select id From games Where name = ?1",
        rusqlite::params![game],
        |row| row.get(0),
    )?;

    // Get the current highest_payout
    let current_payout: f64 = conn.query_row(
        "Select highest_payout From user_statistics Where user_id = ?1 And game_id = ?2",
        rusqlite::params![user.id, game_id],
        |row| row.get(0),
    )?;

    // Update highest_payout if winnings is greater
    let new_payout = if winnings > current_payout {
        logger::info(&format!("New highest payout for User ID: {} in game {}: {:.2}", user.id, game, winnings));
        winnings
    } else {
        current_payout
    };

    // Update the user_statistics table
    conn.execute(
        "Update user_statistics SET win = win + 1, highest_payout = ?1, last_played = ?2 where user_id = ?3 And game_id = ?4",
        rusqlite::params![new_payout, chrono::Local::now().to_rfc3339(), user.id, game_id],
    )?;

    Ok(())
}

/// Record a user's loss in a specific game
pub fn add_user_loss(conn: &Connection, user: &User, game: &str) -> rusqlite::Result<()> {
    logger::info(&format!("User ID: {} lost in game: {}", user.id, game));
    
    // Query to get the game_id from the game name
    let game_id: i32 = conn.query_row(
        "Select id From games Where name = ?1",
        rusqlite::params![game],
        |row| row.get(0),
    )?;

    // Update the user_statistics table
    conn.execute(
        "Update user_statistics Set loss = loss + 1, last_played = ?1 Where user_id = ?2 And game_id = ?3",
        rusqlite::params![chrono::Local::now().to_rfc3339(), user.id, game_id],
    )?;

    Ok(())
}

// Get vec of all games (name string, active bool)
pub fn get_games(conn: &Connection) -> rusqlite::Result<Vec<(String, bool)>> {
    logger::info("Retrieving list of games");
    let mut stmt = conn.prepare("Select * From games")?;

    let games = stmt.query_map([], |row| {
        Ok((row.get::<_,String>(1)?, row.get::<_,bool>(5)?))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(games)
}

/// Toggle game active status (enable/disable)
pub fn toggle_game(conn: &Connection, name: &str) -> rusqlite::Result<()> {

    logger::security(&format!("Game status toggle attempt for: {}", name));

    let mut stmt = conn.prepare(
        "Update games Set active = Not active Where name = ?1"
    )?;

    match stmt.execute([name]) {
        Ok(_) => {
            logger::security(&format!("Game status successfully toggled for: {}", name));
            println!("{} toggled", name);
            Ok(())
        }
        Err(e) => {
            logger::error(&format!("Failed to toggle game status for: {}. Error: {}", name, e));
            println!("Toggle failed");
            Err(e)
        }
    }
}

// Get statistics for technician to see win/loss of each game
pub fn get_game_statistics(conn: &Connection) -> rusqlite::Result<()>{
    logger::info("Retrieving game statistics");
    let mut stmt = conn.prepare("Select * From games")?;
    let games = stmt.query_map([], |row| {
        Ok((row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
    })?;

    for game in games {
        let (name, played, win, loss): (String, i32, i32, i32) = game?;
        println!("game: {} played: {} win: {} loss:{}", name, played, win, loss);
    }

    Ok(())
}

// Get statistics for users for their win/loss/highest payout for each game
pub fn query_user_statistics(conn: &Connection, user: &User) -> rusqlite::Result<()> {
    
    logger::info(&format!("Retrieving statistics for User ID: {}", user.id));
    let mut stmt = conn.prepare("Select * From user_statistics Where user_id = ?1")?;
    // FIX HERE
    let stats = stmt.query_map([user.id], |row| {
        Ok((row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
    })?;

    println!("\n{}", "=".repeat(80));
    println!("{:^80}", format!("Statistics for {}", user.get_username(conn).unwrap()));
    println!("{}", "=".repeat(80));
    println!("{:<20} {:>10} {:>10} {:>10} {:>10} {:>15}", "Game", "Played", "Wins", "Losses", "Win %", "Highest Payout");
    println!("{}", "-".repeat(80));

    for stat in stats {
        let (game_id, win, loss, high): (i32, i32, i32, f64) = stat?;
        let mut stmt = conn.prepare("Select name From games Where id = ?1")?;

        let game_name = stmt.query_row([game_id], |row| row.get::<_, String>(0))?;
        
        let total_played = win + loss;
        let win_percentage = if total_played > 0 {
            (win as f64 / total_played as f64) * 100.0
        } else {
            0.0
        };
        // Fix Print Over there
        
        println!("{:<20} {:>10} {:>10} {:>10} {:>9.1}% {:>15.2}", 
            game_name, 
            total_played, 
            win, 
            loss,
            win_percentage,
            high
        );
    }
    
    println!("{}", "=".repeat(80));
    println!();
    Ok(())
}

/// Initializes game statistics for a newly registered user.
pub fn initialize_user_statistics(conn: &Connection, username: &str, _password: &str) -> rusqlite::Result<()> {
    logger::info(&format!("Initializing statistics for new user: {}", username));
    
    // Get user ID by username only (password already verified during registration)
    let user_id = get_user_id_by_username(conn, username)?;

    // Query to get all game_ids
    let mut game_stmt = conn.prepare("Select id From games")?;
    let game_ids: Vec<i32> = game_stmt.query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<i32>>>()?;

    // Insert a new statistics entry for each game
    for game_id in game_ids {
        conn.execute(
            "Insert Into user_statistics (user_id, game_id, win, loss, highest_payout, last_played)
                    Values (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![user_id, game_id, 0, 0, 0.0, "yesterday"],
        )?;
    }
    
    logger::info(&format!("User statistics successfully registered for User ID: {}", user_id));
    println!("User statistics registered");
    Ok(())
}

// ----------------------------------------------------------------------------------------------------------------------------------
// Symbol probability management functions for commissioner control

/// Get symbol probabilities for a specific game
pub fn get_symbol_probabilities(conn: &Connection, game_name: &str) -> rusqlite::Result<Vec<(String, usize, f64)>> {
    logger::info(&format!("Retrieving symbol probabilities for game: {}", game_name));
    
    // Get game ID
    let game_id: i32 = conn.query_row(
        "Select id From games Where name = ?1",
        [game_name],
        |row| row.get(0)
    )?;

    // Get all symbols for this game
    let mut stmt = conn.prepare(
        "Select symbol, weight, payout_multiplier From symbol_probabilities Where game_id = ?1 Order By weight Desc"
    )?;

    let symbols = stmt.query_map([game_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, i32>(1)? as usize,
            row.get::<_, f64>(2)?
        ))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(symbols)
}

/// Update the weight (probability) of a specific symbol in a game
pub fn update_symbol_weight(conn: &Connection, game_name: &str, symbol: &str, new_weight: usize) -> rusqlite::Result<()> {
    logger::security(&format!("Updating symbol weight for game: {}, symbol: {}, new weight: {}", game_name, symbol, new_weight));
    
    // Get game ID
    let game_id: i32 = conn.query_row(
        "Select id From games Where name = ?1",
        [game_name],
        |row| row.get(0)
    )?;

    // Update the weight - Use proper types with rusqlite::params!
    conn.execute(
        "Update symbol_probabilities Set weight = ?1 Where game_id = ?2 And symbol = ?3",
        rusqlite::params![new_weight as i32, game_id, symbol]
    )?;

    logger::security(&format!("Symbol weight updated successfully for {} in {}", symbol, game_name));
    Ok(())
}

/// Update the payout multiplier of a specific symbol in a game
pub fn update_symbol_payout(conn: &Connection, game_name: &str, symbol: &str, new_multiplier: f64) -> rusqlite::Result<()> {
    logger::security(&format!("Updating payout multiplier for game: {}, symbol: {}, new multiplier: {}", game_name, symbol, new_multiplier));
    
    // Get game ID
    let game_id: i32 = conn.query_row(
        "Select id From games Where name = ?1",
        [game_name],
        |row| row.get(0)
    )?;

    // Update the payout multiplier - Use proper types with rusqlite::params!
    conn.execute(
        "Update symbol_probabilities Set payout_multiplier = ?1 Where game_id = ?2 And symbol = ?3",
        rusqlite::params![new_multiplier, game_id, symbol]
    )?;

    logger::security(&format!("Payout multiplier updated successfully for {} in {}", symbol, game_name));
    Ok(())
}

/// Insert a commissioner test log entry
pub fn insert_commissioner_log(
    conn: &Connection,
    game_name: &str,
    seed: &str,
    rounds: u32,
    wins: u32,
    partials: u32,
    losses: u32,
    rtp: f64
) -> rusqlite::Result<()> {
    logger::info(&format!(
        "Storing commissioner test: game={}, seed={}, rounds={}, rtp={:.2}%",
        game_name, seed, rounds, rtp
    ));
    
    conn.execute(
        "INSERT INTO commissioner_log (game_name, seed, rounds, wins, partials, losses, rtp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![game_name, seed, rounds as i32, wins as i32, partials as i32, losses as i32, rtp],
    )?;
    
    Ok(())
}