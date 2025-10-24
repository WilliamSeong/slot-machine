use rusqlite::Connection;

use crate::interfaces::user::User;

pub fn transaction (conn: &Connection, user: &User, amount: i32) -> f64 {
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "Update users Set balance = balance + ?1 Where id = ?2"
    ).unwrap();

    let result = stmt.execute([amount, user.id]);

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
        "Update games Set win = win + 1 Where name = ?1",
        [game]
    )?;

    Ok(())
}