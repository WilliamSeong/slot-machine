use rusqlite::{Connection, Result};
use colored::*;
use std::io::{self, Write};

use crate::interfaces;
use crate::interfaces::user::User;
use crate::db::dbqueries;

// main entry point into the application
pub fn login(conn: &Connection) -> rusqlite::Result<()> {
    loop {
        // Show options to user
        println!("\n{}", "═══ 🎰 Casino Login 🎰 ═══".bright_cyan().bold());
        println!("{}. {}", "1".yellow(), "Register".white());
        println!("{}. {}", "2".yellow(), "Sign In".white());
        println!("{}. {}", "3".yellow(), "Exit".red());
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();
        
        // Get user input
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok();
        
        // check user input
        let user = match choice.trim() {
            "1" => register(conn)?,
            "2" => sign_in(conn)?,
            "3" => break,
            _ => {println!("Invalid choice"); None},
        };

        // if registration or sign in was successful and returned the User struct
        if let Some(user) = user {
            match user.get_role(&conn).unwrap().as_str() {
                "user" => interfaces::user::user_menu(conn, &user)?,
                "technician" => interfaces::technician::technician_menu(conn, &user)?, 
                _ => println!("User not found")
            }
        }
    }
    Ok(())
}

// Function to handle a new user trying to register their account, returns a user struct
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


    // Insert user and check if insert was successful
    match dbqueries::insert_users(conn, username, password)
    {
        Ok(_) => {
            // Create user statistics functions
            dbqueries::initialize_user_statistics(conn, username, password)?;
            println!("Registration Complete!");
            Ok(dbqueries::get_user(&username, &password, conn))
        }
        Err(_) => {
            Ok(None)
        },
    }
}

// function to handle user signing in with existing account in users db, returns a user struct wrapped in a Option wrapped in a Result
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
    
    // insert new user
    let result = dbqueries::check_users(conn, username, password);

    match result {
        Ok(id) => {
            println!("{}", "Login successful!".green());
            return Ok(Some(User { id: id}))
        }
        Err(_) => {
            println!("{}", "Invalid credentials".red());
            return Ok(None)
        },
    }
}
