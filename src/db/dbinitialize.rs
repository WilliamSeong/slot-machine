use rusqlite::{Connection, Result};

// CRITICAL: solve problems here and check security
// initialize all database tables for the casino application.
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

// Create default administrator accounts with secure password hashing.
fn add_technician_commissioner(conn: &Connection) -> Result<(),rusqlite::Error> {
    use crate::cryptography::crypto::hash_password;
    use crate::db::encryption::encrypt_balance;
    
    // Default password (MUST be changed after first login!)
    let default_password = "123";
    
    // Hash passwords securely with Argon2id
    // Each gets a unique salt, so hashes will be different even for same password
    let hashed_tech_password = hash_password(default_password)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
    
    let hashed_comm_password = hash_password(default_password)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
    
    // Encrypt initial balances for security
    let encrypted_tech_balance = encrypt_balance(5000.0)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
    
    let encrypted_comm_balance = encrypt_balance(10000.0)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e))?;
    
    // Create technician account (if it doesn't already exist)
    conn.execute(
        "Insert Or Ignore Into users (username, password, role, balance) 
        Values ('technician', ?1, 'technician', ?2)",
        [&hashed_tech_password, &encrypted_tech_balance]
    )?;

    // Create commissioner account (if it doesn't already exist)
    conn.execute(
        "Insert Or Ignore Into users (username, password, role, balance) 
        Values ('commissioner', ?1, 'commissioner', ?2)",
        [&hashed_comm_password, &encrypted_comm_balance]
    )?;

    Ok(())
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

// CRITICAL: learn real values and implement here and check ascii suitibility
// Populate default symbol probabilities for all games.
fn add_default_symbols(conn: &Connection) -> Result<(),rusqlite::Error> {
    // Get all game IDs
    let mut stmt = conn.prepare("Select id, name From games")?;
    let games: Vec<(i32, String)> = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    })?.collect::<Result<Vec<_>, _>>()?;

    // Default symbols with weights and payout multipliers
    // Format: (symbol, weight, payout_multiplier)
    let default_symbols = [
        ("🍒", 25, 2.0),   // Cherry: 25% chance, 2x payout
        ("🍋", 25, 2.0),   // Lemon: 25% chance, 2x payout
        ("🍊", 20, 3.0),   // Orange: 20% chance, 3x payout
        ("🍇", 15, 5.0),   // Grape: 15% chance, 5x payout
        ("💎", 10, 10.0),  // Diamond: 10% chance, 10x payout
        ("7️⃣", 5, 20.0),   // Seven: 5% chance, 20x payout (jackpot!)
    ];

    // Add symbols for each game
    for (game_id, _game_name) in games {
        for (symbol, weight, multiplier) in &default_symbols {
            conn.execute(
                "Insert Or Ignore Into symbol_probabilities (game_id, symbol, weight, payout_multiplier)
                Values (?1, ?2, ?3, ?4)",
                [&game_id.to_string(), &symbol.to_string(), &weight.to_string(), &multiplier.to_string()]
            )?;
        }
    }

    Ok(())
}