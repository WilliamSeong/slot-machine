use rusqlite::{Connection};
use colored::*;
use std::io::{self, Write};

use crate::{db::dbqueries, interfaces::user::User};

pub fn technician_menu(conn: &Connection, user: &User) {
    loop {
        println!("\n{}", "═══ 🎰 777 🎰 ═══".bright_magenta().bold());
        println!("{}. {}", "1".yellow(), "Games".white());
        println!("{}. {}", "2".yellow(), "Statistics".white());
        println!("{}. {}", "3".yellow(), "Logout".red());
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();

        let mut choice: String = String::new();
        io::stdin().read_line(&mut choice).ok();

        match choice.trim() {
            "1" => {
                games_menu(conn, user)
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

fn games_menu(conn: &Connection, user: &User) {

}

fn technician_statistics(conn: &Connection) {
    println!("Printing stats");
    let _ = dbqueries::get_game_statistics(conn);
}