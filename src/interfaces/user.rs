use rusqlite::{Connection};
use colored::*;
use std::io::{self, Write};
use rand::Rng;

pub struct User {
    pub id: i32,
}

impl User {
    pub fn get_username(&self, conn: &Connection) -> Result<String, rusqlite::Error> {
        conn.query_row(
        "SELECT username FROM users WHERE id = ?1",
        [self.id],
        |row| row.get(0)
        )
    }


    pub fn get_balance(&self, conn: &Connection) -> Result<f64, rusqlite::Error> {
        conn.query_row(
            "SELECT balance FROM users WHERE id = ?1",
            [self.id],
            |row| row.get(0)
        )
    }

    pub fn get_role(&self, conn: &Connection) -> Result<String, rusqlite::Error> {
        conn.query_row(
            "SELECT role FROM users WHERE id = ?1",
            [self.id],
            |row| row.get(0)
        )
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
                println!("Let's logout");
                break;
            }
            _ => {
                println!("Let's type something valid buddy");
            }
        }
    }
    Ok(())
}

fn play_menu(conn: &Connection, user: &User) -> rusqlite::Result<()>{
    loop {

        let all_games: Vec<(String, bool)> = dbqueries::get_games(conn)?;

        let mut active_games: Vec<(String, bool)> = vec![];

        for game in all_games {
            let (_, active): (String, bool) = game;

            if active {
                active_games.push(game);
            }
        }

        println!("\n{}", "═══ 🎰 Modes 🎰 ═══".bright_cyan().bold());
        for (index, (name, _)) in active_games.iter().enumerate(){
            println!("{}: {}", (index+1).to_string().yellow(), name);
        }
        println!("{}. Back", (active_games.len()+1).to_string().yellow());

        print!("{} ", "Choose:".green());
        io::stdout().flush().ok();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok();

        if choice.trim() == (active_games.len()+1).to_string() {
            println!("Go back");
            break;
        }

        let num_choice: usize = choice.trim().parse().unwrap();
        let index_choice: usize = num_choice - 1;

        let (name_choice, _) = &active_games[index_choice];

        match name_choice.trim() {
            "normal" => {
                loop{ 
                    let bet = bet();
                    if  bet != 0 {
                        if !normal_slots(conn, bet, user) {
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
                        println!("statistics");
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

fn normal_slots(conn: &Connection, bet: i32, user: &User) -> bool {
    loop {
        if !dbqueries::check_funds(conn, user, bet as f64) {
            println!("Insufficient more funds");
            return true;
        }

        let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];
        let mut rng = rand::rng();
        
        println!("\n{}", "🎰 SLOT MACHINE 🎰".bright_yellow().bold());
                
        // Spin the slots
        let slot1 = symbols[rng.random_range(0..symbols.len())];
        let slot2 = symbols[rng.random_range(0..symbols.len())];
        let slot3 = symbols[rng.random_range(0..symbols.len())];
        // let slot1 = symbols[0];
        // let slot2 = symbols[1];
        // let slot3 = symbols[2];

        // Animate
        for _ in 0..30 {
            print!("\r{} | {} | {}", 
                symbols[rng.random_range(0..symbols.len())],
                symbols[rng.random_range(0..symbols.len())],
                symbols[rng.random_range(0..symbols.len())]
            );
            io::stdout().flush().ok();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        // Final result
        println!("\r{} | {} | {}", slot1, slot2, slot3);

        std::thread::sleep(std::time::Duration::from_millis(500));

        // Check win (adjustable probability via symbol frequency)
        if slot1 == slot2 && slot2 == slot3 {
            println!("\n{}", "🎉 JACKPOT! YOU WIN! 🎉".green().bold());
            println!("You win {}", 3 * bet);
            println!("Current balance is {}", dbqueries::transaction(conn, user, 3*bet));
            let _ = dbqueries::add_win(conn, "normal");
        } else if slot1 == slot2 || slot2 == slot3 || slot1 == slot3 {
            println!("\n{}", "Nice! Two matching!".yellow());
            println!("Current balance is {}", dbqueries::transaction(conn, user, 2*bet));
            let _ = dbqueries::add_win(conn, "normal");
        } else {
            println!("\n{}", "YOU LOSE!".red());
            println!("You lose {}", &bet);
            println!("Current balance is {}", dbqueries::transaction(conn, user, -(bet as i32)));
            let _ = dbqueries::add_loss(conn, "normal");
        }

        println!();

        println!("Play Again?");
        println!("Press Enter to continue");
        println!("Press 1 to change bet");
        println!("Press 2 to exit");
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();

        match input.trim() {
            "" => {continue;}
            "1" => {return true;}
            "2" => {return false;}
            _ => {println!("Playing again..."); continue;}
        }
    }
}

fn bet()-> i32 {

    loop {
        println!("\n{}", "🎰 PLACE BET 🎰".bright_red().bold());
        println!("{}. $1", "1".green());
        println!("{}. $5", "2".green());
        println!("{}. $10", "3".green());
        println!("{}. $20", "4".green());
        println!("{}. Back", "4".red());
        print!("{} ", "Choose:".yellow());
        io::stdout().flush().ok();

        let mut input: String = String::new();
        io::stdin().read_line(&mut input).ok();
        
        match input.trim() {
            "1" => {
                println!("Betting $1");
                return 1
            }
            "2" => {
                println!("Betting $5");
                return 5
            }
            "3" => {
                println!("Betting $10");
                return 10
            }
            "4" => {
                println!("Betting $20");
                return 20
            }
            "5" => {
                break;
            }
            _ => {println!("Invalid Input");}
        }
    }
}
