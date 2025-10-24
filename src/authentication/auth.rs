use rusqlite::{Connection, Result};
use colored::*;
use std::io::{self, Write};

use crate::interfaces;
use crate::interfaces::user::User;
use crate::db::dbqueries;

pub fn login(conn: &Connection) -> Result<(), rusqlite::Error> {
    loop {
        println!("\n{}", "═══ 🎰 Casino Login 🎰 ═══".bright_cyan().bold());
        println!("{}. {}", "1".yellow(), "Register".white());
        println!("{}. {}", "2".yellow(), "Sign In".white());
        println!("{}. {}", "3".yellow(), "Exit".red());
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok();
        
        let user = match choice.trim() {
            "1" => register(&conn)?,
            "2" => sign_in(&conn)?,
            "3" => break,
            _ => {println!("Invalid choice"); None},
        };


        if let Some(user) = user {
            match user.get_role(&conn).unwrap().as_str() {
                "user" => interfaces::user::user_menu(conn, &user),
                "technician" => interfaces::technician::technician_menu(conn, &user), 
                _ => println!("User not found")
            }
        }
    }
    Ok(())
}

pub fn register(conn: &Connection) -> Result<Option<User>> {
    // Get username
    print!("Username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();
    
    // Get password
    print!("Password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();
    
    // Insert user
    match conn.execute(
        "INSERT INTO users (username, password) VALUES (?1, ?2)",
        [username, password],
    ) {
        Ok(_) => {
            println!("Registration Complete!");
            Ok(dbqueries::get_user(username, password, conn))
        }
        Err(_) => {
            Ok(None)
        },
    }
}

pub fn sign_in(conn: &Connection) -> Result<Option<User>> {
    println!();
    // Get username
    print!("Username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();
    
    // Get password
    print!("Password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();
    
    // Prepared query
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "SELECT id, username FROM users WHERE username = ?1 AND password = ?2"
    )?;
    
    let result: std::result::Result<i32, rusqlite::Error> = stmt.query_row([username, password], |row| {
        Ok(
            row.get::<_, i32>(0)?,
        )
    });
    
    match result {
        Ok(id) => {
            println!("Login successful!");
            return Ok(Some(User { id: id}))
        }
        Err(_) => {
            println!("Invalid credentials");
            return Ok(None)
        },
    }
}
