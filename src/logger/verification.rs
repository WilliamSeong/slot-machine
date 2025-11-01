use rusqlite::Connection;
use colored::*;
use std::io::{self, Write};
// use chrono::Local;

use crate::interfaces::user::User;
use crate::logger::logger;

pub fn log_verification_menu(conn: &Connection, user: &User) -> rusqlite::Result<()> {
    // First check if user has admin privileges
    let role = user.get_role(conn)?;
    if role != "technician" && role != "commissioner" {
        println!("{}", "Access denied: Insufficient privileges".red());
        logger::security(&format!("User ID: {} attempted to access log verification without proper permissions", user.id));
        return Ok(());
    }

    logger::security(&format!("User ID: {} accessed log verification menu", user.id));

    loop {
        println!("\n{}", "═══ 🔒 Log Verification Menu 🔒 ═══".bright_cyan().bold());
        println!("{}. {}", "1".yellow(), "View Recent Security Events".white());
        println!("{}. {}", "2".yellow(), "Check Login Attempts by Username".white());
        println!("{}. {}", "3".yellow(), "View User Transactions".white());
        println!("{}. {}", "4".yellow(), "Return to Main Menu".red());
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok();
        
        match choice.trim() {
            "1" => view_security_events(),
            "2" => check_login_attempts(),
            "3" => view_user_transactions(),
            "4" => break,
            _ => println!("Invalid choice"),
        }
    }
    
    Ok(())
}

fn view_security_events() {
    println!("\n{}", "═══ Recent Security Events ═══".cyan());
    print!("Time window in minutes (default 60): ");
    io::stdout().flush().ok();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    
    let minutes: u32 = input.trim().parse().unwrap_or(60);
    
    match logger::verify_security_events(minutes) {
        Ok(events) => {
            if events.is_empty() {
                println!("{}", "No security events found in the specified time window.".yellow());
            } else {
                println!("{} security events found:", events.len());
                for event in events {
                    println!("{}", event);
                }
            }
        },
        Err(e) => {
            println!("{}", format!("Error retrieving security events: {}", e).red());
        }
    }
}

fn check_login_attempts() {
    println!("\n{}", "═══ Check Login Attempts ═══".cyan());
    print!("Username to check: ");
    io::stdout().flush().ok();
    
    let mut username = String::new();
    io::stdin().read_line(&mut username).ok();
    let username = username.trim();
    
    print!("Time window in minutes (default 60): ");
    io::stdout().flush().ok();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    
    let minutes: u32 = input.trim().parse().unwrap_or(60);
    
    match logger::verify_login_attempts(username, minutes) {
        Ok((successful, failed)) => {
            println!("Login attempts for {} in the last {} minutes:", username, minutes);
            println!("Successful logins: {}", successful);
            println!("Failed logins: {}", failed);
            
            if failed > 3 {
                println!("{}", "WARNING: Multiple failed login attempts detected!".red().bold());
            }
        },
        Err(e) => {
            println!("{}", format!("Error retrieving login attempts: {}", e).red());
        }
    }
}

fn view_user_transactions() {
    println!("\n{}", "═══ View User Transactions ═══".cyan());
    print!("User ID to check: ");
    io::stdout().flush().ok();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    
    let user_id: i32 = match input.trim().parse() {
        Ok(id) => id,
        Err(_) => {
            println!("{}", "Invalid user ID".red());
            return;
        }
    };
    
    print!("Time window in minutes (default 60): ");
    io::stdout().flush().ok();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    
    let minutes: u32 = input.trim().parse().unwrap_or(60);
    
    match logger::verify_transactions(user_id, minutes) {
        Ok(transactions) => {
            if transactions.is_empty() {
                println!("{}", "No transactions found for this user in the specified time window.".yellow());
            } else {
                println!("{} transactions found:", transactions.len());
                for transaction in transactions {
                    println!("{}", transaction);
                }
            }
        },
        Err(e) => {
            println!("{}", format!("Error retrieving transactions: {}", e).red());
        }
    }
}