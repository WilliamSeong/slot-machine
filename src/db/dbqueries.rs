use rusqlite::Connection;
use chrono;

use crate::interfaces::user::User;

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
        "SELECT id FROM games WHERE name = ?1",
        rusqlite::params![game],
        |row| row.get(0),
    )?;

    // Get the current highest_payout
    let current_payout: f64 = conn.query_row(
        "SELECT highest_payout FROM user_statistics WHERE user_id = ?1 AND game_id = ?2",
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
        "UPDATE user_statistics SET win = win + 1, highest_payout = ?1, last_played = ?2 WHERE user_id = ?3 AND game_id = ?4",
        rusqlite::params![new_payout, chrono::Local::now().to_rfc3339(), user.id, game_id],
    )?;

    Ok(())
}

pub fn add_user_loss(conn: &Connection, user: &User, game: &str) -> rusqlite::Result<()> {
    // Query to get the game_id from the game name
    let game_id: i32 = conn.query_row(
        "SELECT id FROM games WHERE name = ?1",
        rusqlite::params![game],
        |row| row.get(0),
    )?;

    // Update the user_statistics table
    conn.execute(
        "UPDATE user_statistics SET loss = loss + 1, last_played = ?2 WHERE user_id = ?3 AND game_id = ?4",
        rusqlite::params![chrono::Local::now().to_rfc3339(), user.id, game_id],
    )?;

    Ok(())
}

pub fn get_games(conn: &Connection) -> rusqlite::Result<Vec<(String, bool)>> {
    let mut stmt = conn.prepare("Select * From games")?;

    let games = stmt.query_map([], |row| {
        Ok((row.get::<_,String>(1)?, row.get::<_,bool>(5)?))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(games)
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

