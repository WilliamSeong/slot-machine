use rusqlite::{Connection, Result};
use colored::*;
use std::io::{self, Write};

use crate::interfaces;
use crate::interfaces::user::User;
use crate::db::dbqueries;
use crate::logger::logger;

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
            "1" => {
                logger::info("User selected registration option");
                register(conn)?
            },
            "2" => {
                logger::info("User selected sign-in option");
                sign_in(conn)?
            },
            "3" => {
                logger::info("User selected exit option");
                break
            },
            _ => {
                logger::warning("Invalid menu choice selected");
                println!("Invalid choice");None
            },
        };

        // if registration or sign in was successful and returned the User struct
        if let Some(user) = user {
            match user.get_role(&conn).unwrap().as_str() {
                "user" => {
                    logger::info(&format!("User ID: {} logged in as regular user", user.id));
                    interfaces::user::user_menu(conn, &user)?
                },
                "technician" => {
                    logger::info(&format!("User ID: {} logged in as technician", user.id));
                    interfaces::technician::technician_menu(conn, &user)?
                }, 
                _ => {
                    logger::warning(&format!("User ID: {} has unknown role", user.id));
                    println!("User not found")
                }
            }
        }
    }
    logger::info("Application shutting down");
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

    // Log registration attempt (without the password)
    logger::security(&format!("Registration attempt for username: {}", username));

    // Insert user and check if insert was successful
    match dbqueries::insert_users(conn, username, password)
    {
        Ok(_) => {
            // Create user statistics functions
            dbqueries::initialize_user_statistics(conn, username, password)?;
            logger::security(&format!("Registration successful for username: {}", username));
            println!("Registration Complete!");
            let user = dbqueries::get_user(&username, &password, conn);
            if let Some(ref u) = user {
                logger::security(&format!("New user created with ID: {}", u.id));
            }
            Ok(user)
        }
        Err(e) => {
            logger::error(&format!("Registration failed for username: {}. Error: {}", username, e));
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
    
    // Log login attempt (without the password)
    logger::security(&format!("Login attempt for username: {}", username));
    
    // Check if login credentials are valid
    let result = dbqueries::check_users(conn, username, password);

    match result {
        Ok(id) => {
            logger::security(&format!("Successful login for username: {} (User ID: {})", username, id));
            println!("{}", "Login successful!".green());
            return Ok(Some(User { id: id}))
        }
        Err(e) => {
            logger::security(&format!("Failed login for username: {}. Error: {}", username, e));
            println!("{}", "Invalid credentials".red());
            
            // Check for multiple failed login attempts
            match logger::verify_login_attempts(username, 10) {
                Ok((_, failed_attempts)) => {
                    if failed_attempts >= 3 {
                        logger::warning(&format!("Multiple failed login attempts ({}) for username: {}", failed_attempts, username));
                        println!("{}", "Multiple failed login attempts detected. Account may be locked soon.".yellow());
                    }
                },
                Err(_) => {}
            }
            
            return Ok(None)
        },
    }
}