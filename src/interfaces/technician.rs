use rusqlite::{Connection};
use colored::*;
use std::io::{self, Write};

use crate::{db::dbqueries, interfaces::user::User, logger};

pub fn technician_menu(conn: &Connection, user: &User) -> rusqlite::Result<()> {
    // Log that technician has accessed the menu
    logger::logger::security(&format!("Technician (User ID: {}) accessed technician menu", user.id));
    
    loop {
        println!("\n{}", "═══ 🎰 Tech Menu 🎰 ═══".bright_magenta().bold());
        println!("{}. {}", "1".yellow(), "Games".white());
        println!("{}. {}", "2".yellow(), "Statistics".white());
        println!("{}. {}", "3".yellow(), "Security Logs".bright_cyan());
        println!("{}. {}", "4".yellow(), "Logout".red());
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();

        let mut choice: String = String::new();
        io::stdin().read_line(&mut choice).ok();

        match choice.trim() {
            "1" => {
                logger::logger::info(&format!("Technician (User ID: {}) accessed games menu", user.id));
                let _ = games_menu(conn);
            }
            "2" => {
                logger::logger::info(&format!("Technician (User ID: {}) accessed statistics", user.id));
                technician_statistics(conn);
            }
            "3" => {
                logger::logger::security(&format!("Technician (User ID: {}) accessed security logs", user.id));
                logger::verification::log_verification_menu(conn, user)?;
            }
            "4" => {
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

// Function to allow technician to change what games are available to the user
fn games_menu(conn: &Connection) -> rusqlite::Result<()>{
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

        println!("\n{}", "═══ 🎰 Tech Games Control 🎰 ═══".bright_magenta().bold());
        println!("Toggle game or type 'exit' to return");
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();

        let mut choice: String = String::new();
        io::stdin().read_line(&mut choice).ok();
        let choice = choice.trim();

        match choice {
            "exit" => {
                break
            },
            _ => {
                if !choice.is_empty() {
                    logger::logger::security(&format!("Game status toggle attempt for: {}", choice));
                    dbqueries::toggle_game(conn, choice)?;
                }
            }
        }
    }
    Ok(())
}

fn technician_statistics(conn: &Connection) {
    println!("Game Statistics:");
    let _ = dbqueries::get_game_statistics(conn);
}