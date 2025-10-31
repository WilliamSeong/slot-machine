use rusqlite::Connection;
use chrono;

use crate::interfaces::user::User;
use crate::logger::logger;

// db queries for registering and signing in users
pub fn insert_users(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<usize> {
    logger::info(&format!("Attempting to insert new user: {}", username));
    conn.execute(
        "Insert Into users (username, password) Values (?1, ?2)",
        rusqlite::params![username, password],
    )
}

pub fn check_users(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<i32> {
    logger::info(&format!("Checking credentials for user: {}", username));
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "Select id, username From users Where username = ?1 And password = ?2"
    )?;
    
    let result: rusqlite::Result<i32> = stmt.query_row([username, password], |row| {
        Ok(
            row.get::<_, i32>(0)?,
        )
    });

    result
}

// ----------------------------------------------------------------------------------------------------------------------------------

pub fn transaction(conn: &Connection, user: &User, amount: f64) -> f64 {
    // Log the transaction attempt
    logger::transaction(&format!("User ID: {} transaction attempt for amount: {:.2}", user.id, amount));
    
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "Update users Set balance = balance + ?1 Where id = ?2"
    ).unwrap();

    let result = stmt.execute(rusqlite::params![amount, user.id]);

    match result {
        Ok(_) => {
            logger::transaction(&format!("User ID: {} transaction completed successfully for amount: {:.2}", user.id, amount));
            eprintln!("Transaction Complete!");
        }
        Err(e) => {
            logger::error(&format!("User ID: {} transaction failed for amount: {:.2}. Error: {}", user.id, amount, e));
            eprintln!("Transaction Failed");
        },
    }

    let balance = user.get_balance(conn);

    match &balance {  // Use a reference here with &balance
        Ok(bal) => {
            logger::info(&format!("User ID: {} new balance: {:.2}", user.id, bal));
        }
        Err(ref e) => {  // Use ref to borrow the error instead of moving it
            logger::error(&format!("Failed to retrieve balance for User ID: {}. Error: {}", user.id, e));
            println!("No balance found!");
        }
    }

    return balance.unwrap_or(0.0);
}


pub fn check_funds(conn: &Connection, user: &User, limit: f64) -> bool {
    logger::info(&format!("Checking funds for User ID: {} against limit: {:.2}", user.id, limit));
    // Query the users funds
    match user.get_balance(conn) {
        Ok(balance) => {
            let has_funds = balance >= limit;
            if !has_funds {
                logger::warning(&format!("Insufficient funds for User ID: {}. Balance: {:.2}, Required: {:.2}", 
                    user.id, balance, limit));
            }
            return has_funds;
        }
        Err(e) => {
            logger::error(&format!("Failed to check funds for User ID: {}. Error: {}", user.id, e));
            println!("Unable to check funds");
            return false;
        }
    }
}

pub fn change_balance(conn: &Connection, user: &User, deposit: f64) -> rusqlite::Result<bool> {
    logger::transaction(&format!("Balance change for User ID: {} amount: {:.2}", user.id, deposit));
    
    conn.execute(
        "Update users Set balance = balance + ?1 where id = ?2",
        rusqlite::params![deposit, user.id]
    )?;

    // Log the new balance
    if let Ok(new_balance) = user.get_balance(conn) {
        logger::transaction(&format!("User ID: {} new balance after change: {:.2}", user.id, new_balance));
    }

    Ok(true)
}

pub fn get_user(username: &str, password: &str, conn: &Connection) -> Option<User> {
    logger::info(&format!("Retrieving user data for: {}", username));
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "Select id, username, balance From users Where username = ?1 And password = ?2"
    ).unwrap();
    
    let result: std::result::Result<i32, rusqlite::Error> = stmt.query_row([username, password], |row| {
        Ok(
            row.get::<_, i32>(0)?
        )
    });

    match result {
        Ok(id) => {
            logger::info(&format!("Successfully retrieved user data for ID: {}", id));
            println!("Login successful!");
            Some(User{id: id})
        }
        Err(e) => {
            logger::warning(&format!("Failed to retrieve user data for: {}. Error: {}", username, e));
            println!("Invalid credentials");
            None
        },
    }
}

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

pub fn initialize_user_statistics(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<()> {
    logger::info(&format!("Initializing statistics for new user: {}", username));
    // Query to get the user_id
    let user_id: i32 = conn.query_row(
        "Select id From users Where username = ?1 And password = ?2",
        rusqlite::params![username, password],
        |row| row.get(0),
    )?;

    // Query to get all game_ids
    let mut stmt = conn.prepare("Select id From games")?;
    let game_ids: Vec<i32> = stmt.query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<i32>>>()?;

    // Insert a new entry for each game
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