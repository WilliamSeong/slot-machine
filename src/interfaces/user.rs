use rusqlite::{Connection};
use colored::*;
use std::io::{self, Write};
use crate::play;
use crate::logger::logger;

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
    // Log user accessing the menu
    if let Ok(username) = user.get_username(conn) {
        logger::info(&format!("User ID: {} ({}) accessed the main menu", user.id, username));
    } else {
        logger::info(&format!("User ID: {} accessed the main menu", user.id));
    }
    
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
                logger::info(&format!("User ID: {} selected Play option", user.id));
                play_menu(conn, user)?;
            }
            "2" => {
                logger::info(&format!("User ID: {} selected Account option", user.id));
                user_account(conn, user);
            }
            "3" => {
                logger::info(&format!("User ID: {} logged out", user.id));
                break;
            }
            _ => {
                logger::warning(&format!("User ID: {} made invalid menu selection", user.id));
                println!("{}", "Invalid input".red().bold());
            }
        }
    }
    Ok(())
}

fn play_menu(conn: &Connection, user: &User) -> rusqlite::Result<()>{
    logger::info(&format!("User ID: {} entered game selection menu", user.id));
    
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
            logger::info(&format!("User ID: {} exited game selection menu", user.id));
            println!("Go back");
            break;
        }

        let num_choice: usize = match choice.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                logger::warning(&format!("User ID: {} made invalid game selection", user.id));
                println!("Invalid selection");
                continue;
            }
        };
        
        if num_choice < 1 || num_choice > active_games.len() {
            logger::warning(&format!("User ID: {} selected out of range game number: {}", user.id, num_choice));
            println!("Invalid game number");
            continue;
        }
        
        let index_choice: usize = num_choice - 1;
        let (name_choice, _) = &active_games[index_choice];

        match name_choice.trim() {
            "normal" => {
                loop{ 
                    // Get the bed amount
                    let bet = bet();
                    if bet != 0.0 {
                        logger::transaction(&format!("User ID: {} placed bet of ${:.2} on normal slots", user.id, bet));
                        
                        // Check if user has sufficient funds
                        if !dbqueries::check_funds(conn, user, bet) {
                            logger::warning(&format!("User ID: {} attempted to bet ${:.2} with insufficient funds", user.id, bet));
                            println!("{}", "Insufficient funds for this bet".red());
                            break;
                        }
                        
                        if !play::slots::normal_slots(conn, bet, user) {
                            break;
                        }
                    } else {
                        logger::info(&format!("User ID: {} cancelled betting", user.id));
                        break;
                    }
                }
            }
            "multi" => {
                logger::info(&format!("User ID: {} selected multi hit game (not implemented)", user.id));
                println!("Entering Multi hit");
            }
            "holding" => {
                logger::info(&format!("User ID: {} selected holding game (not implemented)", user.id));
                println!("Entering holding");
            }
            _ => {
                logger::warning(&format!("User ID: {} selected unknown game type: {}", user.id, name_choice));
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
    logger::info(&format!("User ID: {} accessed account information", user.id));
    
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
                        logger::info(&format!("User ID: {} selected deposit option", user.id));
                        if deposit(conn, user).unwrap() {
                            println!("Deposit Successful");
                        } else {
                            println!("Deposit Failed");
                        }
                    }
                    "2" => {
                        logger::info(&format!("User ID: {} selected withdraw option", user.id));
                        if withdraw(conn, user).unwrap() {
                            println!("Withdraw Successful");
                        } else {
                            println!("Withdraw Failed");
                        }
                    }
                    "3" => {
                        logger::info(&format!("User ID: {} accessed statistics", user.id));
                        user_statistics(conn, user);
                    }
                    "4" => {
                        logger::info(&format!("User ID: {} accessed settings (not implemented)", user.id));
                        println!("settings");
                    }
                    "5" => {
                        logger::info(&format!("User ID: {} exited account menu", user.id));
                        println!("Exit");
                        break;
                    }
                    _ => {
                        logger::warning(&format!("User ID: {} made invalid account menu selection", user.id));
                        println!("Let's type something valid buddy");
                    }
                }
            }
            _ => {
                logger::error(&format!("Failed to retrieve user information for User ID: {}", user.id));
                println!("User not found!");
            }
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
    if let Ok(amount) = deposit_amount {
        if amount > 0.0 {
            logger::transaction(&format!("User ID: {} deposited ${:.2}", user.id, amount));
            println!("{}", amount);
            dbqueries::change_balance(conn, user, amount)
        } else {
            logger::warning(&format!("User ID: {} attempted to deposit invalid amount: ${:.2}", user.id, amount));
            println!("Amount must be greater than zero");
            Ok(false)
        }
    } else {
        logger::warning(&format!("User ID: {} provided invalid deposit input", user.id));
        println!("Invalid input");
        Ok(false)
    }
}

// Function to let user withdraw funds
fn withdraw(conn: &Connection, user: &User) -> rusqlite::Result<bool>{
    println!("{}", "═══ 🎰 Withdraw 🎰 ═══".bright_cyan().bold());
    print!("{}", "How much would you like to withdraw: ".green());
    io::stdout().flush().ok();

    let mut choice: String = String::new();
    io::stdin().read_line(&mut choice).ok();

    let withdraw_amount: Result<f64, std::num::ParseFloatError> = choice.trim().parse();
    
    if let Ok(amount) = withdraw_amount {
        if amount > 0.0 {
            // Check if user has sufficient funds
            if !dbqueries::check_funds(conn, user, amount) {
                logger::warning(&format!("User ID: {} attempted to withdraw ${:.2} with insufficient funds", user.id, amount));
                println!("Insufficient funds");
                return Ok(false);
            }
            
            logger::transaction(&format!("User ID: {} withdrew ${:.2}", user.id, amount));
            println!("{}", amount);
            dbqueries::change_balance(conn, user, -1.0 * amount)
        } else {
            logger::warning(&format!("User ID: {} attempted to withdraw invalid amount: ${:.2}", user.id, amount));
            println!("Amount must be greater than zero");
            Ok(false)
        }
    } else {
        logger::warning(&format!("User ID: {} provided invalid withdraw input", user.id));
        println!("Invalid input");
        Ok(false)
    }
}

fn user_statistics(conn: &Connection, user: &User) {
    logger::info(&format!("User ID: {} viewed their game statistics", user.id));
    let _ = dbqueries::query_user_statistics(conn, user);
}