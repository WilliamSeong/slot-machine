use rusqlite::{Connection, Result};

use crate::interfaces::user::User;

pub fn initialize_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "Create Table If Not Exists users (
            id Integer Primary Key,
            username Text Unique Not Null,
            password Text Not Null,
            user_type Text Not Null Default 'player',
            balance Real Not Null Default 0.0,
            role Text Default 'user' Check(role In ('user', 'technician', 'commissioner'))
        )",
        [],
    )?;

    Ok(())
}

pub fn add_technician_comissioner(conn: &Connection) -> Result<(),rusqlite::Error> {
    conn.execute(
        "Insert Or Ignore Into users (username, password, role, balance) 
        VALUES ('technician', ?1, 'technician', 5000.0)",
        ["123"]
    )?;

    conn.execute(
        "Insert Or Ignore Into users (username, password, role, balance) 
        VALUES ('commissioner', ?1, 'commissioner', 10000.0)",
        ["123"]  // Change password after first login!
    )?;

    Ok(())
}

pub fn transaction (conn: &Connection, user: &User, amount: i32) -> f64 {
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "update users set balance = balance + ?1 where id = ?2"
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
        "SELECT id, username, balance FROM users WHERE username = ?1 AND password = ?2"
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