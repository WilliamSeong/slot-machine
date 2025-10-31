use rusqlite::{Connection};
use colored::*;
use std::io::{self, Write};
use crate::play;

pub struct User {
    pub id: i32,
}

impl User {
    pub fn get_username(&self, conn: &Connection) -> rusqlite::Result<String> {
        dbqueries::user_get_username(conn, self.id)
    }

    pub fn get_balance(&self, conn: &Connection) -> rusqlite::Result<f64> {
        dbqueries::user_get_balance(conn, self.id)
    }

    pub fn get_role(&self, conn: &Connection) -> rusqlite::Result<String> {
        dbqueries::user_get_role(conn, self.id)
    }
}

use crate::db::dbqueries;

pub fn user_menu(conn: &Connection, user: &User) -> rusqlite::Result<()> {
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
                play_menu(conn, user)?;
            }
            "2" => {
                user_account(conn, user);
            }
            "3" => {
                break;
            }
            _ => {
                println!("{}", "Invalid input".red().bold());
            }
        }
    }
    Ok(())
}

fn play_menu(conn: &Connection, user: &User) -> rusqlite::Result<()>{
    loop {
        // The available games are queried through the get_games function that scans the games table and checks which games were made available by the technician
        let all_games: Vec<(String, bool)> = dbqueries::get_games(conn)?;

        // Initialize a new Vec
        let mut active_games: Vec<(String, bool)> = vec![];

        // Go through all existing games and just get the active games
        for game in all_games {
            let (_, active): (String, bool) = game;
            if active {
                active_games.push(game);
            }
        }

        // print playable games according to the active games vec
        println!("\n{}", "═══ 🎰 Modes 🎰 ═══".bright_cyan().bold());
        for (index, (name, _)) in active_games.iter().enumerate(){
            println!("{}: {}", (index+1).to_string().yellow(), name);
        }
        println!("{}. Back", (active_games.len()+1).to_string().yellow());

        print!("{} ", "Choose:".green());
        io::stdout().flush().ok();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok();

        // In the case the user selects to go back
        if choice.trim() == (active_games.len()+1).to_string() {
            println!("Go back");
            break;
        }

        // else get the number to index the active games Vec
        let num_choice: usize = choice.trim().parse().unwrap();
        let index_choice: usize = num_choice - 1;

        // Get the name of the game user selected
        let (name_choice, _) = &active_games[index_choice];

        // match to that name
        match name_choice.trim() {
            "normal" => {
                loop{ 
                    // Get the bed amount
                    let bet = bet();
                    // If valid bet is chosen then play normal slots
                    if  bet != 0.0 {
                        if !play::slots::normal_slots(conn, bet, user) {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            "multi" => {
                println!("Entering Multi hit");
            }
            "holding" => {
                println!("Entering holding");
            }
            _ => {
                println!("Let's type something valid buddy");
            }
        }
    }
    Ok(())
}

fn bet()-> f64 {
    loop {
        println!("\n{}", "🎰 PLACE BET 🎰".bright_red().bold());
        println!("{}. $1", "1".green());
        println!("{}. $5", "2".green());
        println!("{}. $10", "3".green());
        println!("{}. $20", "4".green());
        println!("{}. Back", "5".red());
        print!("{} ", "Choose:".yellow());
        io::stdout().flush().ok();

        let mut input: String = String::new();
        io::stdin().read_line(&mut input).ok();
        
        match input.trim() {
            "1" => return 1.0,
            "2" => return 5.0,
            "3" => return 10.0,
            "4" => return 20.0,
            "5" => return 0.0,
            _ => println!("Invalid Input")
        }
    }
}

fn user_account(conn: &Connection, user: &User) {
    loop {
        match (user.get_username(conn), user.get_balance(conn)) {
            (Ok(username), Ok(balance)) => {
                println!("{}", "═══ 🎰 User Information 🎰 ═══".bright_cyan().bold());
                println!("{}: {}", "Id".yellow(), user.id);
                println!("{}: {}", "Username".yellow(), username);
                println!("{}: {}", "Balance".yellow(), format!("${:.2}", balance).green());
                println!();

                println!("{}", "═══ 🎰 User Options 🎰 ═══".bright_cyan().bold());
                println!("{}. Deposit", "1".yellow());
                println!("{}. Withdraw", "2".yellow());
                println!("{}. Statistics", "3".yellow());
                println!("{}. Settings", "4".yellow());
                println!("{}. Exit", "5".yellow());
                io::stdout().flush().ok();

                let mut choice = String::new();
                io::stdin().read_line(&mut choice).ok();

                match choice.trim() {
                    "1" => {
                        if deposit(conn, user).unwrap() {
                            println!("Deposit Successful");
                        } else {
                            println!("Deposit Fail");
                        }
                    }
                    "2" => {
                        if withdraw(conn, user).unwrap() {
                            println!("Withdraw Successful");
                        } else {
                            println!("Withdraw Fail");
                        }
                    }
                    "3" => {
                        user_statistics(conn, user);
                    }
                    "4" => {
                        println!("settings");
                    }
                    "5" => {
                        println!("Exit");
                        break;
                    }
                    _ => {
                        println!("Let's type something valid buddy");
                    }
                }
            }
            _ => {println!("User not found!")}
        }
    }
}

// Function to let users deposit funds
fn deposit(conn: &Connection, user: &User) -> rusqlite::Result<bool>{
    println!("{}", "═══ 🎰 Deposit 🎰 ═══".bright_cyan().bold());
    print!("{}", "How much would you like to deposit: ".green());
    io::stdout().flush().ok();

    let mut choice: String = String::new();
    io::stdin().read_line(&mut choice).ok();

    let deposit_amount: Result<f64, std::num::ParseFloatError> = choice.trim().parse();
    if let Ok(amount) = deposit_amount && amount > 0.0 {
        println!("{}", amount);
        dbqueries::change_balance(conn, user, amount)
    } else {
        println!("Invalid input");
        Ok(false)
    }
}

// Function to let user withdraw funds
fn withdraw(conn: &Connection, user: &User) -> rusqlite::Result<bool>{
    println!("{}", "═══ 🎰 Deposit 🎰 ═══".bright_cyan().bold());
    print!("{}", "How much would you like to withdraw: ".green());
    io::stdout().flush().ok();

    let mut choice: String = String::new();
    io::stdin().read_line(&mut choice).ok();

    let deposit_amount: Result<f64, std::num::ParseFloatError> = choice.trim().parse();
    println!(" parsed!");
    if let Ok(amount) = deposit_amount && amount > 0.0 {
        println!("{}", amount);
        dbqueries::change_balance(conn, user, -1.0 * amount)
    } else {
        println!("Invalid input");
        Ok(false)
    }
}

fn user_statistics(conn: &Connection, user: &User) {
    let _ = dbqueries::query_user_statistics(conn, user);
}