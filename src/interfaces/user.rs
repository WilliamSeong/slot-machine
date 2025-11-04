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

use crate::db::dbqueries::{self, update_user_password};

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
                let menu_options = vec!["Deposit", "Withdraw", "Statistics", "Change Password", "Exit"];
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
                    "Change Password" => {
                        logger::info(&format!("User ID: {} attempt to change password", user.id));
                        change_password(conn, user);
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
            
            // Get confirmation from User
            let menu_options = vec!["Confirm", "Cancel"];
            let confirmation_message = format!("Confirm deposit of {}", amount);
            let user_input = menu_generator(confirmation_message.as_str(), &menu_options);

            match user_input.trim() {
                "Confirm" => {
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
                "Cancel" => {
                    logger::transaction(&format!("User ID: {} Cancel deposit of ${:.2}", user.id, amount));
                    return Ok(false);
                }
                _ => {
                    logger::transaction(&format!("User ID: {} Deposit confirmation failed", user.id));
                    return Ok(false);
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

            // Get confirmation from User
            let menu_options = vec!["Confirm", "Cancel"];
            let confirmation_message = format!("Confirm withdrawal of {}", amount);
            let user_input = menu_generator(confirmation_message.as_str(), &menu_options);

            match user_input.trim() {
                "Confirm" => {
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
                "Cancel" => {
                    logger::transaction(&format!("User ID: {} Cancel withdrawal of ${:.2}", user.id, amount));
                    return Ok(false);
                }
                _ => {
                    logger::transaction(&format!("User ID: {} Withdrawal confirmation failed", user.id));
                    return Ok(false);
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

fn change_password(conn: &Connection, user: &User) -> () {
    use crate::db::validator::{validate_password, display_validation_error};
    use dialoguer::Password;

    // Get password with secure input 
    let password = match Password::new()
        .with_prompt("Password")
        .interact() {
            Ok(pwd) => pwd,
            Err(_) => {
                println!("{}", "❌ Password input cancelled".red().bold());
                return;
            }
        };
    
    // Validate password
    if let Err(error) = validate_password(&password) {
        display_validation_error(&error);
        logger::warning(&format!("Failed login - invalid password format for username: {}", user.get_username(conn).unwrap()));
        return;
    }
    
    // Log login attempt
    logger::security(&format!("Login attempt for username: {}", user.get_username(conn).unwrap().as_str()));
    
    // Check if login credentials are valid
    let result = dbqueries::check_users(conn, user.get_username(conn).unwrap().as_str(), &password);

    match result {
        Ok(id) => {
            // Get password with secure input
            let password = match Password::new()
                .with_prompt("New Password (min 12 chars)")
                .interact() {
                    Ok(pwd) => pwd,
                    Err(_) => {
                        println!("{}", "❌ Password input cancelled".red().bold());
                        return;
                    }
                };
            
            // Validate password
            if let Err(error) = validate_password(&password) {
                display_validation_error(&error);
                logger::warning(&format!("Changing Password failed - invalid password for username: {}", user.get_username(conn).unwrap()));
                return ;
            }
            
            // Confirm password for security
            let password_confirm = match Password::new()
                .with_prompt("Confirm New Password")
                .interact() {
                    Ok(pwd) => pwd,
                    Err(_) => {
                        println!("{}", "❌ Password confirmation cancelled".red().bold());
                        return;
                    }
                };
            
            // Check if passwords match
            if password != password_confirm {
                println!("\n{}", "╔═══════════════════════════════════════════╗".red());
                println!("{}", "║        ❌ Passwords Don't Match!          ║".red().bold());
                println!("{}", "╠═══════════════════════════════════════════╣".red());
                println!("{}", "║  The passwords you entered don't match.   ║".red());
                println!("{}", "║  Please try again.                        ║".red());
                println!("{}", "╚═══════════════════════════════════════════╝".red());
                println!();
                logger::warning(&format!("Changing Password failed - password mismatch for username: {}", user.get_username(conn).unwrap()));
                return;
            }

            logger::security(&format!("Login Password attempt for username: {}",  user.get_username(conn).unwrap()));
            
            let result = update_user_password(conn, user.get_username(conn).unwrap().as_str(), password.as_str());

            match result {
                Ok(_) => {
                    println!("Successful password change");
                }
                Err(_) => {
                    println!("Bad password change!");
                }
            }
        }
        Err(e) => {            
            println!("Failed check");
            return;
        },
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::dbqueries::{self, change_balance, insert_users};

    // create a test database
    fn setup_test_db() -> Connection {
        // Initialize encryption BEFORE opening database
        crate::cryptography::crypto::initialize_encryption_key();

        let conn = Connection::open_in_memory().unwrap();
        
        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();

        // Use your actual initialization function
        crate::db::dbinitialize::initialize_dbs(&conn).unwrap();
        
        conn
    }

    // Create a test user in the database
    fn create_test_user(conn: &Connection, username: &str, password: &str) -> i32 {
        // You'll need to use your actual user creation function
        // This is a simplified version - adjust based on your actual API

        let _ = insert_users(conn, username, password);
        
        conn.last_insert_rowid() as i32
    }

    // Deposit balance for user
    fn deposit_balance(conn: &Connection, user: &User, deposit: f64) {
        let _ = change_balance(conn, user, deposit);
    }

    #[test]
    fn test_user_get_username() {
        let conn = setup_test_db();
        let user_id = create_test_user(&conn, "alice", "password1234");
        let user = User { id: user_id };
        
        
        let username = user.get_username(&conn).unwrap();
        println!("in caego test test_user_get_username the username retrieved was: {}", username);
        assert_eq!(username, "alice");
    }

    #[test]
    fn test_user_get_balance() {
        let conn = setup_test_db();
        let user_id = create_test_user(&conn, "bob", "password1234");
        let user = User { id: user_id };
        deposit_balance(&conn, &user, 250.50);
        
        let balance = user.get_balance(&conn).unwrap();
        assert_eq!(balance, 250.50);
    }

    #[test]
    fn test_user_get_role() {
        let conn = setup_test_db();
        let user_id = create_test_user(&conn, "charlie", "password1234");
        let user = User { id: user_id };
        
        let role = user.get_role(&conn).unwrap();
        assert_eq!(role, "user");
    }

    #[test]
    fn test_user_nonexistent() {
        let conn = setup_test_db();
        let user = User { id: 99999 }; // Non-existent user
        
        assert!(user.get_username(&conn).is_err());
        assert!(user.get_balance(&conn).is_err());
        assert!(user.get_role(&conn).is_err());
    }

    #[test]
    fn test_multiple_users() {
        let conn = setup_test_db();
        
        let user1_id = create_test_user(&conn, "user1", "password1234");
        let user2_id = create_test_user(&conn, "user2", "password1234");
        
        let user1 = User { id: user1_id };
        let user2 = User { id: user2_id };
        
        assert_eq!(user1.get_username(&conn).unwrap(), "user1");
        assert_eq!(user2.get_username(&conn).unwrap(), "user2");
        
        assert_eq!(user1.get_balance(&conn).unwrap(), 0.0);
        assert_eq!(user2.get_balance(&conn).unwrap(), 0.0);
    }

    #[test]
    fn test_user_balance_after_multiple_transaction() {
        let conn = setup_test_db();
        let user_id = create_test_user(&conn, "dave", "password1234");
        let user = User { id: user_id };
        
        // Perform a transaction (deposit)
        dbqueries::change_balance(&conn, &user, 50.0).unwrap();
        let original_balance = user.get_balance(&conn).unwrap();
        
        // Perform another transaction (deposit)
        dbqueries::change_balance(&conn, &user, 50.0).unwrap();
        let new_balance = user.get_balance(&conn).unwrap();
        assert_eq!(original_balance, 50.0);
        assert_eq!(new_balance, 100.0);
    }

    #[test]
    fn test_user_balance_after_withdrawal() {
        let conn = setup_test_db();
        let user_id = create_test_user(&conn, "eve", "password1234");
        let user = User { id: user_id };
        
        // Perform a deposit
        dbqueries::change_balance(&conn, &user, 50.0).unwrap();

        // Perform a withdrawal
        dbqueries::change_balance(&conn, &user, -30.0).unwrap();
        let new_balance = user.get_balance(&conn).unwrap();
        assert_eq!(new_balance, 20.0);
    }

    #[test]
    fn test_check_funds_sufficient() {
        let conn = setup_test_db();
        let user_id = create_test_user(&conn, "frank", "password1234");
        let user = User { id: user_id };

        // Perform a deposit
        dbqueries::change_balance(&conn, &user, 200.0).unwrap();

        
        assert!(dbqueries::check_funds(&conn, &user, 50.0));
        assert!(dbqueries::check_funds(&conn, &user, 100.0));
    }

    #[test]
    fn test_check_funds_insufficient() {
        let conn = setup_test_db();
        let user_id = create_test_user(&conn, "grace", "password1234");
        let user = User { id: user_id };

        // Perform a deposit
        dbqueries::change_balance(&conn, &user, 100.0).unwrap();
        assert!(!dbqueries::check_funds(&conn, &user, 150.0));
    }

    #[test]
    fn test_username_uniqueness() {
        let conn = setup_test_db();
        
        create_test_user(&conn, "unique_user", "password1234");
        
        // Attempt to create another user with the same username
        let result = conn.execute(
            "INSERT INTO users (username, password, balance, role) VALUES (?1, ?2, ?3, 'user')",
            ["unique_user", "password4321", "50.0"],
        );
        
        // Should fail due to UNIQUE constraint
        assert!(result.is_err());
    }

    #[test]
    fn test_default_role_is_user() {
        let conn = setup_test_db();
        let user_id = create_test_user(&conn, "default_role_user", "password1234");
        let user = User { id: user_id };
        
        let role = user.get_role(&conn).unwrap();
        assert_eq!(role, "user");
    }

    #[test]
    fn test_games_table_exists() {
        let conn = setup_test_db();
        
        // Query the games table to ensure it was created
        let result: Result<i32, _> = conn.query_row(
            "SELECT COUNT(*) FROM games",
            [],
            |row| row.get(0)
        );
        
        // Should succeed if table exists
        assert!(result.is_ok());
    }

    #[test]
    fn test_user_statistics_table_exists() {
        let conn = setup_test_db();
        
        let result: Result<i32, _> = conn.query_row(
            "SELECT COUNT(*) FROM user_statistics",
            [],
            |row| row.get(0)
        );
        
        assert!(result.is_ok());
    }
}
