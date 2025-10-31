use rusqlite::{Connection};
use colored::*;
use std::io::{self, Write};

use crate::User;


pub fn user_menu(conn: &Connection, user: &User) {
    loop {
        println!("\n{}", "═══ 🎰 777 🎰 ═══".bright_magenta().bold());
        println!("{}. {}", "1".yellow(), "Play".white());
        println!("{}. {}", "2".yellow(), "Account".white());
        println!("{}. {}", "3".yellow(), "Logout".red());
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok();

        match choice.trim() {
            "1" => {
                play_menu(conn, user)
            }
            "2" => {
                user_account(conn, user);
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

fn play_menu(conn: &Connection, user: &User) {
    loop {
        println!("\n{}", "═══ 🎰 Modes 🎰 ═══".bright_cyan().bold());
        println!("{}. Normal", "1".yellow());
        println!("{}. Multi Hit", "2".yellow());
        println!("{}. Holding", "3".yellow());
        println!("{}. Back", "4".yellow());
        print!("{} ", "Choose:".green());
        io::stdout().flush().ok();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok();

        match choice.trim() {
            "1" => {
                loop{ 
                    let bet = bet();
                    if !normal_slots(conn, bet, user) {
                        break;
                    }
                }
            }
            "2" => {
                println!("Entering Multi hit");
            }
            "3" => {
                println!("Entering holding");
            }
            "4" => {
                println!("Go back");
                break;
            }
            _ => {
                println!("Let's type something valid buddy");
            }
        }
    }
}

fn user_account(conn: &Connection, user: &User) {

    match (user.get_username(conn), user.get_balance(conn)) {
        (Ok(username), Ok(balance)) => {
            println!("{}", "═══ 🎰 User Information 🎰 ═══".bright_cyan().bold());
            println!("{}: {}", "Id".yellow(), user.id);
            println!("{}: {}", "Username".yellow(), username);
            println!("{}: {}", "Balance".yellow(), format!("${:.2}", balance).green());
                
            println!("\n{}", "Press Enter to continue...".dimmed());
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
        }
        _ => {println!("User not found!")}
    }
}
