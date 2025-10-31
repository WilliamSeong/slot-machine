use rusqlite::Connection;
use chrono;

use crate::interfaces::user::User;

/*  ---------------------------------------------------------------------------------------------------------------------------------- */
// db queries for registering and signing in users

// inserts a user into the users db given a username and password string
pub fn insert_users(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<usize> {
    conn.execute(
        "Insert Into users (username, password) Values (?1, ?2)",
        rusqlite::params![username, password],
    )
}

// checks and returns the id if a record in the users db matches the input username and password
pub fn check_users(conn: &Connection, username: &str, password: &str) -> rusqlite::Result<i32> {
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

/*  ---------------------------------------------------------------------------------------------------------------------------------- */
// User struct queries, queries the user's details with the user id

pub fn user_get_username(conn: &Connection, id: i32) -> rusqlite::Result<String> {
    conn.query_row(
    "Select username From users Where id = ?1",
    [id],
    |row| row.get(0)
    )
}

pub fn user_get_balance(conn: &Connection, id: i32) -> rusqlite::Result<f64> {
    conn.query_row(
        "Select balance From users Where id = ?1",
        [id],
        |row| row.get(0)
    )
}

pub fn user_get_role(conn: &Connection, id: i32) -> rusqlite::Result<String> {
    conn.query_row(
        "Select role From users Where id = ?1",
        [id],
        |row| row.get(0)
    )
}

/*  ---------------------------------------------------------------------------------------------------------------------------------- */

// queries the games db to see what games exist, returns a Vec of tuples (name of game, active status)
pub fn get_games(conn: &Connection) -> rusqlite::Result<Vec<(String, bool)>> {
    let mut stmt = conn.prepare("Select * From games")?;

    let games = stmt.query_map([], |row| {
        Ok((row.get::<_,String>(1)?, row.get::<_,bool>(5)?))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(games)
}


pub fn transaction (conn: &Connection, user: &User, amount: f64) -> f64 {
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "Update users Set balance = balance + ?1 Where id = ?2"
    ).unwrap();

    let result = stmt.execute(rusqlite::params![amount, user.id]);

    match result {
        Ok(_) => {
            eprintln!("Transaction Complete!");
        }
        Err(_) => {
            eprintln!("Transaction Failed");
        },
    }

    let balance = user.get_balance(conn);

    match balance {
        Ok(_) => {}
        Err(_) => {println!("No balance found!")}
    }

    return balance.unwrap();
}

pub fn check_funds(conn: &Connection, user: &User, limit: f64) -> bool {
    // Query the users funds
    match user.get_balance(conn) {
        Ok(balance) => {
            if balance >= limit {
                return true;
            } else {
                return false;
            }
        }
        Err(_) => {
            println!("Unable to check funds");
            return false;
        }
    }
}

pub fn change_balance(conn: &Connection, user: &User, deposit: f64) -> rusqlite::Result<bool> {
    conn.execute(
        "Update users Set balance = balance + ?1 where id = ?2",
        rusqlite::params![deposit, user.id]
    )?;

    Ok(true)
}

pub fn get_user(username: &str, password: &str, conn: &Connection) -> Option<User> {
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
            println!("Login successful!");
            Some(User{id: id})
        }
        Err(_) => {
            println!("Invalid credentials");
            None
        },
    }
}

// add_played, add_win, add_loss record game statistics
pub fn add_played(conn: &Connection, game: &str) -> rusqlite::Result<()>{
    conn.execute(
        "Update games Set played = played + 1 Where name = ?1",
        [game]
    )?;

    Ok(())
}

pub fn add_win(conn: &Connection, game: &str) -> rusqlite::Result<()> {
    add_played(conn, game)?;
    
    conn.execute(
        "Update games Set win = win + 1 Where name = ?1",
        [game]
    )?;

    Ok(())
}

pub fn add_loss(conn: &Connection, game: &str) -> rusqlite::Result<()> {
    add_played(conn, game)?;
    
    conn.execute(
        "Update games Set loss = loss + 1 Where name = ?1",
        [game]
    )?;

    Ok(())
}

pub fn add_user_win(conn: &Connection, user: &User, game: &str, winnings: f64) -> rusqlite::Result<()> {
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

pub fn toggle_game(conn: &Connection, name: &str) -> rusqlite::Result<()> {

    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "Update games Set active = Not active Where name = ?1"
    ).unwrap();

    let result = stmt.execute([name]);

    match result {
        Ok(_) => {
            println!("{} toggled", name);
        }
        Err(_) => {
            println!("Toggle failed");
        }
    }

    Ok(())
}

pub fn get_game_statistics(conn: &Connection) -> rusqlite::Result<()>{
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
    println!("User statistics registered");
    Ok(())
}