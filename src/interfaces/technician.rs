use rusqlite::{Connection};
use colored::*;
use std::io::{self, Write};

use crate::interfaces::user::User;

pub fn technician_menu(_conn: &Connection, _user: &User) {
    loop {
        println!("\n{}", "═══ 🎰 777 🎰 ═══".bright_magenta().bold());
        println!("{}. {}", "1".yellow(), "Games".white());
        println!("{}. {}", "2".yellow(), "Statistics".white());
        println!("{}. {}", "3".yellow(), "Logout".red());
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();

        let mut choice: String = String::new();
        io::stdin().read_line(&mut choice).ok();

        // match choice.trim() {
        //     "1" => {
        //         play_menu(conn, user)
        //     }
        //     "2" => {
        //         user_account(conn, user);
        //     }
        //     "3" => {
        //         println!("Let's logout");
        //         break;
        //     }
        //     _ => {
        //         println!("Let's type something valid buddy");
        //     }
        // }

        break;

    }
}
