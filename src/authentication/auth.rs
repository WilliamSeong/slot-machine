use rusqlite::{Connection, Result};
use colored::*;
use std::io::{self, Write};

use crate::interfaces;
use crate::interfaces::user::User;
use crate::db::dbqueries;
use crate::logger::logger;

use crate::interfaces::menus::menu_generator;

use clearscreen;

// main entry point into the application
pub fn login(conn: &Connection) -> rusqlite::Result<()> {
    loop {
        // Show options to user
        let menu_options = vec!["Register", "Sign In", "Exit"];
        let user_input = menu_generator("═══ 🎰 Casino Login 🎰 ═══", &menu_options);
        
        // check user input
        let user = match user_input.trim() {
            "Register" => {
                logger::info("User selected registration option");
                register(conn)?
            },
            "Sign In" => {
                logger::info("User selected sign-in option");
                sign_in(conn)?
            },
            "Exit" => {
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
            // Get user role with proper error handling
            match user.get_role(&conn) {
                Ok(role) => {
                    match role.as_str() {
                        "user" => {
                            logger::info(&format!("User ID: {} logged in as regular user", user.id));
                            interfaces::user::user_menu(conn, &user)?;
                        },
                        "technician" => {
                            logger::info(&format!("User ID: {} logged in as technician", user.id));
                            interfaces::technician::technician_menu(conn, &user)?;
                        },
                        "commissioner" => {
                            logger::info(&format!("User ID: {} logged in as commissioner", user.id));
                            interfaces::commisioner::commissioner_menu(conn, &user)?;
                        }, 
                        _ => {
                            logger::warning(&format!("User ID: {} has unknown role: {}", user.id, role));
                            println!("{}", "Error: Unknown user role. Please contact administrator.".red().bold());
                        }
                    }
                }
                Err(e) => {
                    logger::error(&format!("Failed to retrieve role for User ID: {}. Error: {}", user.id, e));
                    println!("{}", "Error: Could not verify user role. Please try again.".red().bold());
                }
            }
        }
    }
    logger::info("Application shutting down");
    Ok(())
}

// Function to handle a new user trying to register their account, returns a user struct
pub fn register(conn: &Connection) -> Result<Option<User>> {
    use crate::db::validator::{validate_username, validate_password, display_validation_error};
    use dialoguer::Password;
    
    println!("\n{}", "═══ 📝 User Registration 📝 ═══".bright_cyan().bold());
    
    // Get username
    print!("{} ", "Username (3-30 chars):".bright_white().bold());
    if io::stdout().flush().is_err() {
        logger::error("Failed to flush stdout during registration");
        println!("{}", "❌ System error occurred".red().bold());
        return Ok(None);
    }
    
    let mut username = String::new();
    if io::stdin().read_line(&mut username).is_err() {
        logger::error("Failed to read username input during registration");
        println!("{}", "❌ Input error occurred".red().bold());
        return Ok(None);
    }
    let username = username.trim();
    
    // Validate username
    if let Err(error) = validate_username(username) {
        display_validation_error(&error);
        logger::warning(&format!("Registration failed - invalid username: {}", username));
        return Ok(None);
    }
    
    // Get password with secure input
    let password = match Password::new()
        .with_prompt("Password (min 12 chars)")
        .interact() {
            Ok(pwd) => pwd,
            Err(_) => {
                println!("{}", "❌ Password input cancelled".red().bold());
                return Ok(None);
            }
        };
    
    // Validate password (comprehensive validation)
    if let Err(error) = validate_password(&password) {
        display_validation_error(&error);
        logger::warning(&format!("Registration failed - invalid password for username: {}", username));
        return Ok(None);
    }
    
    // Confirm password for security
    let password_confirm = match Password::new()
        .with_prompt("Confirm Password")
        .interact() {
            Ok(pwd) => pwd,
            Err(_) => {
                println!("{}", "❌ Password confirmation cancelled".red().bold());
                return Ok(None);
            }
        };
    
    // Check if passwords match
    if password != password_confirm {
        println!("\n{}", "╔═══════════════════════════════════════════╗".red());
        println!("{}", "║        ❌ Passwords Don't Match!          ║".red().bold());
        println!("{}", "╠═══════════════════════════════════════════╣".red());
        println!("{}", "║  The passwords you entered don't match.   ║".red());
        println!("{}", "║  Please try again.                        ║".red());
        println!("{}", "╚═══════════════════════════════════════════╝".red());
        println!();
        logger::warning(&format!("Registration failed - password mismatch for username: {}", username));
        return Ok(None);
    }

    // Log registration attempt (without the password)
    logger::security(&format!("Registration attempt for username: {}", username));

    // Insert user and check if insert was successful
    match dbqueries::insert_users(conn, username, &password)
    {
        Ok(_) => {
            // Create user statistics (no password verification needed - user was just created)
            dbqueries::initialize_user_statistics(conn, username, &password)?;
            logger::security(&format!("Registration successful for username: {}", username));
            println!("{}", "✓ Registration successful!".green().bold());
            clearscreen::clear().expect("Failed clearscreen");
            
            // Get user ID directly (no password verification needed after registration)
            match dbqueries::get_user_id_by_username(conn, username) {
                Ok(id) => {
                    logger::security(&format!("New user created with ID: {}", id));
                    Ok(Some(User { id }))
                }
                Err(e) => {
                    logger::error(&format!("Failed to retrieve user ID after registration: {}", e));
                    Ok(None)
                }
            }
        }
        Err(e) => {
            logger::error(&format!("Registration failed for username: {}. Error: {}", username, e));
            
            // Check if error is due to duplicate username
            let error_msg = format!("{}", e);
            if error_msg.contains("UNIQUE constraint failed") || error_msg.contains("username") {
                println!("\n{}", "╔═══════════════════════════════════════════╗".red());
                println!("{}", "║   ❌ Username Already Exists!            ║".red().bold());
                println!("{}", "╠═══════════════════════════════════════════╣".red());
                println!("{}", "║  That username is already taken.          ║".red());
                println!("{}", "║  Please choose a different username.      ║".red());
                println!("{}", "╚═══════════════════════════════════════════╝".red());
                println!();
            } else {
                println!("{}", "Registration failed. Please try again.".red().bold());
            }
            
            Ok(None)
        },
    }
}

// function to handle user signing in with existing account in users db, returns a user struct wrapped in a Option wrapped in a Result
pub fn sign_in(conn: &Connection) -> Result<Option<User>> {
    use crate::db::validator::{validate_username, validate_password, display_validation_error};
    use crate::authentication::authorization::{is_account_locked, get_lockout_remaining, record_failed_attempt, record_successful_login, get_failed_attempts};
    use dialoguer::Password;
    
    const MAX_FAILED_ATTEMPTS: u32 = 5; 
    
    println!("\n{}", "═══ 🔐 User Login 🔐 ═══".bright_cyan().bold());
    
    // Get username
    print!("{} ", "Username:".bright_white().bold());
    if io::stdout().flush().is_err() {
        logger::error("Failed to flush stdout during sign-in");
        println!("{}", "❌ System error occurred".red().bold());
        return Ok(None);
    }
    
    let mut username = String::new();
    if io::stdin().read_line(&mut username).is_err() {
        logger::error("Failed to read username input during sign-in");
        println!("{}", "❌ Input error occurred".red().bold());
        return Ok(None);
    }
    let username = username.trim();
    
    // Validate username
    if let Err(error) = validate_username(username) {
        display_validation_error(&error);
        logger::warning(&format!("Login failed - invalid username format: {}", username));
        return Ok(None);
    }
    
    // BRUTE FORCE PROTECTION: Check if account is locked
    if is_account_locked(username) {
        if let Some(remaining) = get_lockout_remaining(username) {
            println!("\n{}", "╔═══════════════════════════════════════════╗".red());
            println!("{}", "║        🔒 ACCOUNT LOCKED 🔒               ║".red().bold());
            println!("{}", "╠═══════════════════════════════════════════╣".red());
            println!("{}", format!("║  Too many failed login attempts!          ║").red());
            println!("{}", format!("║  Account locked for {} seconds.           ║", remaining).red());
            println!("{}", format!("║  Try again later.                         ║").red());
            println!("{}", "╚═══════════════════════════════════════════╝".red());
            println!();
            logger::security(&format!("Blocked login attempt for locked account: {}", username));
            return Ok(None);
        }
    }
    
    // Get password with secure input 
    let password = match Password::new()
        .with_prompt("Password")
        .interact() {
            Ok(pwd) => pwd,
            Err(_) => {
                println!("{}", "❌ Password input cancelled".red().bold());
                return Ok(None);
            }
        };
    
    // Validate password
    if let Err(error) = validate_password(&password) {
        display_validation_error(&error);
        logger::warning(&format!("Failed login - invalid password format for username: {}", username));
        return Ok(None);
    }
    
    // Log login attempt
    logger::security(&format!("Login attempt for username: {}", username));
    
    // Check if login credentials are valid
    let result = dbqueries::check_users(conn, username, &password);

    match result {
        Ok(id) => {
            // BRUTE FORCE PROTECTION: Clear failed attempts on successful login
            record_successful_login(username);
            
            logger::security(&format!("Successful login for username: {} (User ID: {})", username, id));
            println!("{}", "✓ Login successful!".green().bold());
            clearscreen::clear().expect("Failed clearscreen");
            return Ok(Some(User { id: id}))
        }
        Err(e) => {
            // BRUTE FORCE PROTECTION: Record failed attempt
            record_failed_attempt(username);
            let failed_count = get_failed_attempts(username);
            
            logger::security(&format!("Failed login for username: {}. Error: {}. Failed attempts: {}", 
                                    username, e, failed_count));
            
            println!("\n{}", "╔═══════════════════════════════════════════╗".red());
            println!("{}", "║        ❌ Invalid Credentials!            ║".red().bold());
            println!("{}", "╠═══════════════════════════════════════════╣".red());
            println!("{}", "║  Username or password incorrect.          ║".red());
            
            // Warn about approaching lockout
            if failed_count >= 3 {
                println!("{}", format!("║  ⚠️  Warning: {} failed attempts!          ║", failed_count).yellow().bold());
                println!("{}", format!("║  Account will lock after {} attempts.      ║", MAX_FAILED_ATTEMPTS).yellow());
            } else {
                println!("{}", format!("║  Attempt {}/{}                              ║", failed_count, MAX_FAILED_ATTEMPTS).red());
            }
            
            println!("{}", "╚═══════════════════════════════════════════╝".red());
            println!();
            
            return Ok(None)
        },
    }
}