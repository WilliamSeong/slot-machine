use rusqlite::{Connection};
use colored::*;
use std::io::{self, Write};

use crate::{db::dbqueries, interfaces::user::User};

pub fn technician_menu(conn: &Connection, user: &User) {
    loop {
        println!("\n{}", "═══ 🎰 Tech Menu 🎰 ═══".bright_magenta().bold());
        println!("{}. {}", "1".yellow(), "Games".white());
        println!("{}. {}", "2".yellow(), "Statistics".white());
        println!("{}. {}", "3".yellow(), "Logout".red());
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();

        let mut choice: String = String::new();
        io::stdin().read_line(&mut choice).ok();

        match choice.trim() {
            "1" => {
                let _ = games_menu(conn);
            }
            "2" => {
                technician_statistics(conn);
            }
            "3" => {
                println!("Let's logout");
                break;
            }
            _ => {
                println!("Let's type something valid buddy");
            }
        }

    }
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
        println!("Name to toggle game or exit");
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();

        let mut choice: String = String::new();
        io::stdin().read_line(&mut choice).ok();

        // match choice.trim() {

        // }
    }
}

fn technician_statistics(conn: &Connection) {
    println!("Printing stats");
    let _ = dbqueries::get_game_statistics(conn);
}