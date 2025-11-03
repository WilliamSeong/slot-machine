use rusqlite::Connection;
use chrono;

use crate::interfaces::user::User;
use crate::logger::logger;

/*  ---------------------------------------------------------------------------------------------------------------------------------- */
// db queries for registering and signing in users


// Inserts a new user into the database with a securely hashed password.
pub fn insert_users(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<usize> {
    use crate::cryptography::crypto::hash_password;
    use crate::db::encryption::encrypt_balance;
    
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

// Verifies user credentials by checking username and password hash.
pub fn check_users(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<i32> {
    use crate::cryptography::crypto::verify_password;
    
    logger::info(&format!("Checking credentials for user: {}", username));
    
    // CRITICAL: statement check
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
    use crate::db::encryption::decrypt_balance;
    
    // Retrieve encrypted balance from database
    let encrypted_balance: String = conn.query_row(
        "Select balance From users Where id = ?1",
        [id],
        |row| row.get(0)
    )?;
    // CRITICAL: look at any sensetive data
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

// CRITICAL: learn this should be logged or not
// CRITICAL: check williams sql and make it secure
// Execute a financial transaction (deposit or withdrawal) for a user.
pub fn transaction(conn: &Connection, user: &User, amount: f64) -> f64 {
    use crate::db::encryption::encrypt_balance;
    
    // SECURITY: Log all transaction attempts for audit trail
    // This helps detect fraud, errors, and suspicious activity
    logger::transaction(&format!("User ID: {} transaction attempt for amount: {:.2}", user.id, amount));
    
    // Retrieve current encrypted balance
    let current_balance = match user.get_balance(conn) {
        Ok(bal) => bal,
        Err(e) => {
            logger::error(&format!("Failed to retrieve balance for User ID: {}. Error: {}", user.id, e));
            eprintln!("Transaction Failed - Cannot retrieve balance");
            return 0.0;
        }
    };
    
    // Calculate new balance
    let new_balance = current_balance + amount;
    
    // Encrypt the new balance
    let encrypted_balance = match encrypt_balance(new_balance) {
        Ok(enc) => enc,
        Err(e) => {
            logger::error(&format!("Failed to encrypt new balance for User ID: {}. Error: {}", user.id, e));
            eprintln!("Transaction Failed - Encryption error");
            return current_balance;
        }
    };
    
    // Update database with encrypted balance
    let result = conn.execute(
        "Update users Set balance = ?1 Where id = ?2",
        rusqlite::params![encrypted_balance, user.id]
    );

    // CRITICAL: you removed one thing by mistake find it and fix it
    match result {
        Ok(_) => {
            logger::transaction(&format!("User ID: {} transaction completed successfully for amount: {:.2}", user.id, amount));
            logger::info(&format!("User ID: {} new balance: {:.2}", user.id, new_balance));
        }
        Err(e) => {
            logger::error(&format!("User ID: {} transaction failed for amount: {:.2}. Error: {}", user.id, amount, e));
            eprintln!("Transaction Failed");
            return current_balance;
        },
    }

    return new_balance;
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
            // CRITICAL: make this fail safe
            // Fail-safe: if we can't check funds, don't allow the transaction
            logger::error(&format!("Failed to check funds for User ID: {}. Error: {}", user.id, e));
            println!("Unable to check funds");
            return false;
        }
    }
}

// Change user balance with encrypted storage.ly
pub fn change_balance(conn: &Connection, user: &User, deposit: f64) -> rusqlite::Result<bool> {
    use crate::db::encryption::encrypt_balance;
    
    logger::transaction(&format!("Balance change for User ID: {} amount: {:.2}", user.id, deposit));
    
    // Get current balance
    let current_balance = user.get_balance(conn)?;
    
    // Calculate new balance
    let new_balance = current_balance + deposit;
    
    // Encrypt the new balance
    let encrypted_balance = encrypt_balance(new_balance)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
    
    // Update database
    conn.execute(
        "Update users Set balance = ?1 where id = ?2",
        rusqlite::params![encrypted_balance, user.id]
    )?;

    // Log the new balance
    logger::transaction(&format!("User ID: {} new balance after change: {:.2}", user.id, new_balance));

    Ok(true)
}

// Retrieves and authenticates a user from the database.
pub fn get_user(username: &str, password: &str, conn: &Connection) -> Option<User> {
    use crate::cryptography::crypto::verify_password;
    
    logger::info(&format!("Retrieving user data for: {}", username));
    // CRITICAL: no sql always statements test here
    // Prepare statement to fetch user data by username only
    let mut stmt = conn.prepare("Select id, password From users Where username = ?1");
    
    match stmt {
        Ok(mut prepared_stmt) => {
            let user_result = prepared_stmt.query_row([username], |row| {
                let id: i32 = row.get(0)?;
                let stored_hash: String = row.get(1)?;
                
                // Verify password against stored hash
                // This is done in application code (not SQL) for security
                if verify_password(password, &stored_hash) {
                    Ok(User { id })
                } else {
                    // Return same error as "user not found" to prevent username enumeration
                    Err(rusqlite::Error::QueryReturnedNoRows)
                }
            });
            
            match user_result {
                Ok(user) => {
                    logger::info(&format!("Successfully retrieved user data for ID: {}", user.id));
                    println!("Login successful!");
                    Some(user)
                }
                Err(e) => {
                    logger::warning(&format!("Failed to retrieve user data for: {}. Error: {}", username, e));
                    println!("Invalid credentials");
                    None
                },
            }
        },
        Err(e) => {
            logger::error(&format!("Failed to prepare statement: {}", e));
            None
        }
    }
}

// CRITICAL: check are they safe
// add_played, add_win, add_loss record game statistics
pub fn add_played(conn: &Connection, game: &str) -> rusqlite::Result<()>{
    logger::info(&format!("Recording game played: {}", game));
    conn.execute(
        "Update games Set played = played + 1 Where name = ?1",
        [game]
    )?;

    Ok(())
}

pub fn add_win(conn: &Connection, game: &str) -> rusqlite::Result<()> {
    logger::info(&format!("Recording game win: {}", game));
    add_played(conn, game)?;
    
    conn.execute(
        "Update games Set win = win + 1 Where name = ?1",
        [game]
    )?;

    Ok(())
}

pub fn add_loss(conn: &Connection, game: &str) -> rusqlite::Result<()> {
    logger::info(&format!("Recording game loss: {}", game));
    add_played(conn, game)?;
    
    conn.execute(
        "Update games Set loss = loss + 1 Where name = ?1",
        [game]
    )?;

    Ok(())
}

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
        "Update user_statistics Set loss = loss + 1, last_played = ?2 Where user_id = ?3 And game_id = ?4",
        rusqlite::params![chrono::Local::now().to_rfc3339(), user.id, game_id],
    )?;

    Ok(())
}

pub fn get_games(conn: &Connection) -> rusqlite::Result<Vec<(String, bool)>> {
    logger::info("Retrieving list of games");
    let mut stmt = conn.prepare("Select * From games")?;

    let games = stmt.query_map([], |row| {
        Ok((row.get::<_,String>(1)?, row.get::<_,bool>(5)?))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(games)
}

pub fn toggle_game(conn: &Connection, name: &str) -> rusqlite::Result<()> {
    logger::security(&format!("Game status toggle attempt for: {}", name));

    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "Update games Set active = Not active Where name = ?1"
    ).unwrap();

    let result = stmt.execute([name]);

    match result {
        Ok(_) => {
            logger::security(&format!("Game status successfully toggled for: {}", name));
            println!("{} toggled", name);
        }
        Err(e) => {
            logger::error(&format!("Failed to toggle game status for: {}. Error: {}", name, e));
            println!("Toggle failed");
        }
    }

    Ok(())
}

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

pub fn query_user_statistics(conn: &Connection, user: &User) -> rusqlite::Result<()> {
    logger::info(&format!("Retrieving statistics for User ID: {}", user.id));
    let mut stmt = conn.prepare("Select * From user_statistics Where user_id = ?1")?;
    let stats = stmt.query_map([user.id], |row| {
        Ok((row.get(2)?,row.get(3)?, row.get(4)?, row.get(5)?))
    })?;

    for stat in stats {
        let (game_id, win, loss, high): (i32, i32, i32, f64) = stat?;
        let mut stmt = conn.prepare("Select name From games Where id = ?1")?;
        let game_name = stmt.query_row([game_id], |row| row.get::<_, String>(0))?;
        println!("game: {} played: {} win: {} loss:{} highest:{}", game_name, win+loss, win, loss, high);
    }
    Ok(())
}

/// Initializes game statistics for a newly registered user.
pub fn initialize_user_statistics(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<()> {
    use crate::cryptography::crypto::verify_password;
    
    logger::info(&format!("Initializing statistics for new user: {}", username));
    
    // Query to get the user_id using secure password verification
    // We need to verify the password hash instead of comparing plain text
    let mut stmt = conn.prepare("Select id, password From users Where username = ?1")?;
    
    let user_id: i32 = stmt.query_row([username], |row| {
        let id: i32 = row.get(0)?;
        let stored_hash: String = row.get(1)?;
        
        // Verify password before returning user_id
        if verify_password(password, &stored_hash) {
            Ok(id)
        } else {
            Err(rusqlite::Error::QueryReturnedNoRows)
        }
    })?;

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

// Update the weight (probability) of a specific symbol in a game.
pub fn update_symbol_weight(conn: &Connection, game_name: &str, symbol: &str, new_weight: usize) -> rusqlite::Result<()> {
    logger::security(&format!("Updating symbol weight for game: {}, symbol: {}, new weight: {}", game_name, symbol, new_weight));
    
    // Get game ID
    let game_id: i32 = conn.query_row(
        "Select id From games Where name = ?1",
        [game_name],
        |row| row.get(0)
    )?;

    // Update the weight
    conn.execute(
        "Update symbol_probabilities Set weight = ?1 Where game_id = ?2 And symbol = ?3",
        [&new_weight.to_string(), &game_id.to_string(), symbol]
    )?;

    logger::security(&format!("Symbol weight updated successfully for {} in {}", symbol, game_name));
    Ok(())
}

// Update the payout multiplier of a specific symbol in a game.
pub fn update_symbol_payout(conn: &Connection, game_name: &str, symbol: &str, new_multiplier: f64) -> rusqlite::Result<()> {
    logger::security(&format!("Updating payout multiplier for game: {}, symbol: {}, new multiplier: {}", game_name, symbol, new_multiplier));
    
    // Get game ID
    let game_id: i32 = conn.query_row(
        "Select id From games Where name = ?1",
        [game_name],
        |row| row.get(0)
    )?;

    // Update the payout multiplier
    conn.execute(
        "Update symbol_probabilities Set payout_multiplier = ?1 Where game_id = ?2 And symbol = ?3",
        [&new_multiplier.to_string(), &game_id.to_string(), symbol]
    )?;

    logger::security(&format!("Payout multiplier updated successfully for {} in {}", symbol, game_name));
    Ok(())
}