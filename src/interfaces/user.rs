use rusqlite::{Connection};
use colored::*;
use std::io::{self, Write};
use crate::play;
use crate::logger::logger;
use crate::interfaces::menus::menu_generator;
// User struct to hold the id of the user
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
        // print user menu
        let menu_options = vec!["Play", "Account", "Logout"];
        let user_input = menu_generator("═══ 🎰 777 🎰 ═══", &menu_options);

        // println!("\n{}", "═══ 🎰 777 🎰 ═══".bright_magenta().bold());
        // println!("{}. {}", "1".yellow(), "Play".white());
        // println!("{}. {}", "2".yellow(), "Account".white());
        // println!("{}. {}", "3".yellow(), "Logout".red());
        // print!("{} ", "Choose:".green().bold());
        // io::stdout().flush().ok();

        // let mut choice = String::new();
        // io::stdin().read_line(&mut choice).ok();

        match user_input.trim() {
            "Play" => {
                logger::info(&format!("User ID: {} selected Play option", user.id));
                play_menu(conn, user)?;
            }
            "Account" => {
                logger::info(&format!("User ID: {} selected Account option", user.id));
                user_account(conn, user);
            }
            "Logout" => {
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

        // Extract the names from active_games for menus
        let game_names: Vec<String> = active_games.iter()
            .map(|(name, _)| name.clone())
            .collect();
        // with Back option
        let mut all_options = game_names.clone();
        all_options.push("Back".to_string());

        // Convert to &str
        let menu_options: Vec<&str> = all_options.iter()
            .map(|s| s.as_str())
            .collect();

        let user_input = menu_generator("Select a game", &menu_options);

        match user_input.trim() {
            "normal" => {
                loop{ 
                    // Get the bet amount
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
                loop{
                    // Get the bet amount
                    let bet = bet();
                    if bet != 0.0 {
                        logger::transaction(&format!("User ID: {} placed bet of ${:.2} on multiwin slots", user.id, bet));

                        // Check if user has sufficient funds
                        if !dbqueries::check_funds(conn, user, bet) {
                            logger::warning(&format!("User ID: {} attempted to bet ${:.2} with insufficient funds", user.id, bet));
                            println!("{}", "Insufficient funds for this bet".red());
                            break;
                        }
                        if !play::multiwin::multi_win(conn, user, bet) {
                            break;
                        }
                    } else {
                        logger::info(&format!("User ID: {} cancelled betting", user.id));
                        break;
                    }
                }
            }
            "holding" => {
                loop{
                    // Get the bet amount
                    let bet = bet();
                    if bet != 0.0 {
                        logger::transaction(&format!("User ID: {} placed bet of ${:.2} on holding slots", user.id, bet));

                        // Check if user has sufficient funds
                        if !dbqueries::check_funds(conn, user, bet) {
                            logger::warning(&format!("User ID: {} attempted to bet ${:.2} with insufficient funds", user.id, bet));
                            println!("{}", "Insufficient funds for this bet".red());
                            break;
                        }
                        if !play::holding::hold_game(conn, user, bet) {
                            break;
                        }
                    } else {
                        logger::info(&format!("User ID: {} cancelled betting", user.id));
                        break;
                    }
                }
            }
            "wheel of fortune" => {
                loop{
                    // Get the bet amount
                    let bet = bet();
                    if bet != 0.0 {
                        logger::transaction(&format!("User ID: {} placed bet of ${:.2} on holding slots", user.id, bet));

                        // Check if user has sufficient funds
                        if !dbqueries::check_funds(conn, user, bet) {
                            logger::warning(&format!("User ID: {} attempted to bet ${:.2} with insufficient funds", user.id, bet));
                            println!("{}", "Insufficient funds for this bet".red());
                            break;
                        }
                        if !play::wheelOfFortune::gameplay_wheel(conn, user, bet) {
                            break;
                        }
                    } else {
                        logger::info(&format!("User ID: {} cancelled betting", user.id));
                        break;
                    }
                }
            }
            "Back" => {
                logger::info(&format!("User ID: {} selected holding game (not implemented)", user.id));
                break;
            }
            _ => {
                logger::warning(&format!("User ID: {} selected unknown game type: {}", user.id, user_input));
            }
        }
    }
    Ok(())
}

fn bet()-> f64 {
    loop {

        let menu_options = vec!["$1", "$5", "$10", "$20", "Back"];
        let user_input = menu_generator("How much will you bet?", &menu_options);
        
        match user_input.trim() {
            "$1" => return 1.0,
            "$5" => return 5.0,
            "$10" => return 10.0,
            "$20" => return 20.0,
            "Back" => return 0.0,
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

                // Show options to user
                let menu_options = vec!["Deposit", "Withdraw", "Statistics", "Settings", "Exit"];
                let user_input = menu_generator("═══ 🎰 User Options 🎰 ═══", &menu_options);

                match user_input.trim() {
                    "Deposit" => {
                        logger::info(&format!("User ID: {} selected deposit option", user.id));
                        match deposit(conn, user) {
                            Ok(true) => println!("Deposit Successful"),
                            Ok(false) => println!("Deposit Failed"),
                            Err(e) => {
                                logger::error(&format!("Deposit error for User ID: {}: {}", user.id, e));
                                println!("{}", "System error during deposit".red().bold());
                            }
                        }
                    }
                    "Withdraw" => {
                        logger::info(&format!("User ID: {} selected withdraw option", user.id));
                        match withdraw(conn, user) {
                            Ok(true) => {},
                            Ok(false) => println!("Withdraw Failed"),
                            Err(e) => {
                                logger::error(&format!("Withdraw error for User ID: {}: {}", user.id, e));
                                println!("{}", "System error during withdrawal".red().bold());
                            }
                        }
                    }
                    "Statistics" => {
                        logger::info(&format!("User ID: {} accessed statistics", user.id));
                        user_statistics(conn, user);
                    }
                    "Settings" => {
                        logger::info(&format!("User ID: {} accessed settings (not implemented)", user.id));
                    }
                    "Exit" => {
                        logger::info(&format!("User ID: {} exited account menu", user.id));
                        break;
                    }
                    _ => {
                        logger::warning(&format!("User ID: {} made invalid account menu selection", user.id));
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
    use crate::db::validator::{validate_deposit, display_validation_error};
    
    println!("\n{}", "═══ 💵 Deposit Funds 💵 ═══".bright_cyan().bold());
    println!("{}", "Enter amount to deposit ($0.01 - $1,000,000)".bright_white());
    print!("{} $", "Amount:".bright_white().bold());
    io::stdout().flush().ok();

    let mut choice: String = String::new();
    io::stdin().read_line(&mut choice).ok();

    // Parse the input
    let deposit_amount: Result<f64, std::num::ParseFloatError> = choice.trim().parse();
    
    match deposit_amount {
        Ok(amount) => {
            // Validate deposit amount
            if let Err(error) = validate_deposit(amount) {
                display_validation_error(&error);
                logger::warning(&format!("User ID: {} attempted invalid deposit: ${:.2}", user.id, amount));
                return Ok(false);
            }
            
            // Amount is valid, process deposit
            logger::transaction(&format!("User ID: {} depositing ${:.2}", user.id, amount));
            
            match dbqueries::change_balance(conn, user, amount) {
                Ok(_) => {
                    println!("\n{}", "✅ Deposit successful!".green().bold());
                    println!("{} ${:.2}", "Deposited:".bright_white().bold(), amount);
                    if let Ok(balance) = user.get_balance(conn) {
                        println!("{} ${:.2}", "New Balance:".bright_white().bold(), balance);
                    }
                    println!();
                    Ok(true)
                }
                Err(e) => {
                    println!("{}", "❌ Deposit failed!".red().bold());
                    logger::error(&format!("Deposit failed for User ID: {}: {}", user.id, e));
                    Ok(false)
                }
            }
        }
        Err(_) => {
            display_validation_error("❌ Invalid input! Please enter a valid number.");
            logger::warning(&format!("User ID: {} provided invalid deposit input: {}", user.id, choice.trim()));
            Ok(false)
        }
    }
}

// Function to let user withdraw funds
fn withdraw(conn: &Connection, user: &User) -> rusqlite::Result<bool>{
    use crate::db::validator::{validate_withdrawal, display_validation_error};
    
    println!("\n{}", "═══ 💰 Withdraw Funds 💰 ═══".bright_cyan().bold());
    
    // Show current balance first
    match user.get_balance(conn) {
        Ok(balance) => {
            println!("{} ${:.2}", "Current Balance:".bright_white().bold(), balance);
        }
        Err(_) => {
            println!("{}", "❌ Cannot retrieve balance!".red().bold());
            return Ok(false);
        }
    }
    
    println!("{}", "Enter amount to withdraw ($0.01 - $100,000)".bright_white());
    print!("{} $", "Amount:".bright_white().bold());
    io::stdout().flush().ok();

    let mut choice: String = String::new();
    io::stdin().read_line(&mut choice).ok();
    // Parse the input
    let withdraw_amount: Result<f64, std::num::ParseFloatError> = choice.trim().parse();
    
    match withdraw_amount {
        Ok(amount) => {
            // Get current balance for validation
            let current_balance = match user.get_balance(conn) {
                Ok(bal) => bal,
                Err(e) => {
                    logger::error(&format!("Failed to retrieve balance for withdrawal: {}", e));
                    println!("{}", "❌ Cannot process withdrawal!".red().bold());
                    return Ok(false);
                }
            };
            
            // Validate withdrawal amount
            if let Err(error) = validate_withdrawal(amount, current_balance) {
                display_validation_error(&error);
                logger::warning(&format!("User ID: {} attempted invalid withdrawal: ${:.2} (balance: ${:.2})", 
                                        user.id, amount, current_balance));
                return Ok(false);
            }
            
            // Amount is valid process withdrawal
            logger::transaction(&format!("User ID: {} withdrawing ${:.2}", user.id, amount));
            
            match dbqueries::change_balance(conn, user, -amount) {
                Ok(_) => {
                    println!("\n{}", "✅ Withdrawal successful!".green().bold());
                    println!("{} ${:.2}", "Withdrawn:".bright_white().bold(), amount);
                    if let Ok(balance) = user.get_balance(conn) {
                        println!("{} ${:.2}", "New Balance:".bright_white().bold(), balance);
                    }
                    println!();
                    Ok(true)
                }
                Err(e) => {
                    println!("{}", "❌ Withdrawal failed!".red().bold());
                    logger::error(&format!("Withdrawal failed for User ID: {}: {}", user.id, e));
                    Ok(false)
                }
            }
        }
        Err(_) => {
            display_validation_error("❌ Invalid input! Please enter a valid number.");
            logger::warning(&format!("User ID: {} provided invalid withdraw input: {}", user.id, choice.trim()));
            Ok(false)
        }
    }
}

fn user_statistics(conn: &Connection, user: &User) {
    logger::info(&format!("User ID: {} viewed their game statistics", user.id));
    let _ = dbqueries::query_user_statistics(conn, user);
}