use rusqlite::{Connection};
use colored::*;

use crate::{db::dbqueries, interfaces::user::User, logger};
use crate::authentication::authorization;

use crate::interfaces::menus::menu_generator;

pub fn technician_menu(conn: &Connection, user: &User) -> rusqlite::Result<()> {
    // SECURITY: Verify user has technician role
    if let Err(e) = authorization::require_technician(conn, user) {
        logger::logger::security(&format!("Blocked unauthorized access to technician menu by User ID: {}: {}", user.id, e));
        return Ok(());
    }
    
    // Log that technician has accessed the menu
    logger::logger::security(&format!("Technician (User ID: {}) accessed technician menu", user.id));
    
    loop {
        // Show options to user
        let menu_options = vec!["Games", "Statistics", "Security Logs", "Logout"];
        let user_input = menu_generator("═══ 🎰 Tech Menu 🎰 ═══", &menu_options);

        // println!("\n{}", "═══ 🎰 Tech Menu 🎰 ═══".bright_magenta().bold());
        // println!("{}. {}", "1".yellow(), "Games".white());
        // println!("{}. {}", "2".yellow(), "Statistics".white());
        // println!("{}. {}", "3".yellow(), "Security Logs".bright_cyan());
        // println!("{}. {}", "4".yellow(), "Logout".red());
        // print!("{} ", "Choose:".green().bold());
        // io::stdout().flush().ok();

        // let mut choice: String = String::new();
        // io::stdin().read_line(&mut choice).ok();

        match user_input.trim() {
            "Games" => {
                logger::logger::info(&format!("Technician (User ID: {}) accessed games menu", user.id));
                let _ = games_menu(conn, user);
            }
            "Statistics" => {
                logger::logger::info(&format!("Technician (User ID: {}) accessed statistics", user.id));
                technician_statistics(conn, user);
            }
            "Security" => {
                logger::logger::security(&format!("Technician (User ID: {}) accessed security logs", user.id));
                logger::verification::log_verification_menu(conn, user)?;
            }
            "Logout" => {
                logger::logger::info(&format!("Technician (User ID: {}) logged out", user.id));
                println!("Logging out...");
                break;
            }
            _ => {
                logger::logger::warning(&format!("Technician (User ID: {}) entered invalid menu choice", user.id));
                println!("Invalid choice. Please try again.");
            }
        }
    }
    Ok(())
}

/// Function to allow technician to change what games are available to the user - REQUIRES TECHNICIAN ROLE
fn games_menu(conn: &Connection, user: &User) -> rusqlite::Result<()>{
    // SECURITY: Double-check authorization
    if authorization::require_technician(conn, user).is_err() {
        return Ok(());
    }
    
    logger::logger::security(&format!("Technician (User ID: {}) accessing games control", user.id));
    loop {
        let games = dbqueries::get_games(conn)?;

        for game in games {
            let (name, active): (String, bool) = game;

            if active {
                println!("game: {} active: {}", name, active.to_string().green());
            } else {
                println!("game: {} active: {}", name, active.to_string().red());
            }
        }
        // Show options to user
        let menu_options = vec!["normal", "multi", "holding", "exit"];
        let user_input = menu_generator("═══ 🎰 Tech Games Control 🎰 ═══", &menu_options);

        // println!("\n{}", "═══ 🎰 Tech Games Control 🎰 ═══".bright_magenta().bold());
        // println!("Toggle game or type 'exit' to return");
        // print!("{} ", "Choose:".green().bold());
        // io::stdout().flush().ok();

        // let mut choice: String = String::new();
        // io::stdin().read_line(&mut choice).ok();
        // let choice = choice.trim();

        match user_input {
            "exit" => {
                break
            },
            _ => {
                if !user_input.is_empty() {
                    logger::logger::security(&format!("Game status toggle attempt for: {}", user_input));
                    dbqueries::toggle_game(conn, user_input)?;
                }
            }
        }
    }
    Ok(())
}

/// View game statistics - REQUIRES TECHNICIAN ROLE
fn technician_statistics(conn: &Connection, user: &User) {
    // SECURITY: Double-check authorization
    if authorization::require_technician(conn, user).is_err() {
        return;
    }
    
    logger::logger::info(&format!("Technician (User ID: {}) viewing game statistics", user.id));
    println!("Game Statistics:");
    let _ = dbqueries::get_game_statistics(conn);
}