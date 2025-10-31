use rusqlite::{Connection, Result};
use std::io::{self, Write};
use colored::*;
use rand::Rng;

struct User {
    id: i32,
}

impl User {
    fn get_username(&self, conn: &Connection) -> Result<String, rusqlite::Error> {
        conn.query_row(
        "SELECT username FROM users WHERE id = ?1",
        [self.id],
        |row| row.get(0)
        )
    }


    fn get_balance(&self, conn: &Connection) -> Result<f64, rusqlite::Error> {
        conn.query_row(
            "SELECT balance FROM users WHERE id = ?1",
            [self.id],
            |row| row.get(0)
        )
    }

    fn get_role(&self, conn: &Connection) -> Result<String, rusqlite::Error> {
        conn.query_row(
            "SELECT role FROM users WHERE id = ?1",
            [self.id],
            |row| row.get(0)
        )
    }
}

mod user;
mod technician;
mod db;

fn main() -> Result<()> {
    let conn = Connection::open("casino.db")?;
    
    // Create users table
    db::initialize_db(&conn)?;

    // Add technician account and commissioner account
    db::add_technician_comissioner(&conn)?;

    login(&conn)
}

fn login(conn: &Connection) -> Result<(), rusqlite::Error> {
    loop {
        println!("\n{}", "═══ 🎰 Casino Login 🎰 ═══".bright_cyan().bold());
        println!("{}. {}", "1".yellow(), "Register".white());
        println!("{}. {}", "2".yellow(), "Sign In".white());
        println!("{}. {}", "3".yellow(), "Exit".red());
        print!("{} ", "Choose:".green().bold());
        io::stdout().flush().ok();
        
        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok();
        
        let user = match choice.trim() {
            "1" => register(&conn)?,
            "2" => sign_in(&conn)?,
            "3" => break,
            _ => {println!("Invalid choice"); None},
        };


        if let Some(user) = user {
            match user.get_role(&conn).unwrap().as_str() {
                "user" => user::user_menu(conn, &user),
                "technician" => technician::technician_menu(conn, &user), 
                _ => println!("User not found")
            }
        }
    }
    Ok(())
}

fn bet()-> i32 {

    loop {
        println!("\n{}", "🎰 PLACE BET 🎰".bright_red().bold());
        println!("{}. $1", "1".green());
        println!("{}. $5", "2".green());
        println!("{}. $10", "3".green());
        println!("{}. $20", "4".green());
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
            _ => {println!("Invalid Input");}
        }
    }
}

fn normal_slots(conn: &Connection, bet: i32, user: &User) -> bool {
    loop {
        if !check_funds(conn, user, bet as f64) {
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
            println!("Current balance is {}", transaction(conn, user, 3*bet));
        } else if slot1 == slot2 || slot2 == slot3 || slot1 == slot3 {
            println!("\n{}", "Nice! Two matching!".yellow());
            println!("Current balance is {}", transaction(conn, user, 2*bet));
        } else {
            println!("\n{}", "YOU LOSE!".red());
            println!("You lose {}", &bet);
            println!("Current balance is {}", transaction(conn, user, -(bet as i32)));
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

fn transaction (conn: &Connection, user: &User, amount: i32) -> f64 {
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "update users set balance = balance + ?1 where id = ?2"
    ).unwrap();

    let result = stmt.execute([amount, user.id]);

    match result {
        Ok(_) => {
            eprintln!("Transaction Complete!");
        }
        Err(_) => {
            eprintln!("Transaction Failed");
        },
    }

    let balance = user.get_balance(conn);

    match balance {
        Ok(_) => {}
        Err(_) => {println!("No balance found!")}
    }

    return balance.unwrap();
}

fn check_funds(conn: &Connection, user: &User, limit: f64) -> bool {
    // Query the users funds
    match user.get_balance(conn) {
        Ok(balance) => {
            if balance >= limit {
                return true;
            } else {
                return false;
            }
        }
        Err(_) => {
            println!("Unable to check funds");
            return false;
        }
    }
}

fn register(conn: &Connection) -> Result<Option<User>> {
    // Get username
    print!("Username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();
    
    // Get password
    print!("Password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();
    
    // Insert user
    match conn.execute(
        "INSERT INTO users (username, password) VALUES (?1, ?2)",
        [username, password],
    ) {
        Ok(_) => {
            println!("Registration Complete!");
            Ok(get_user(username, password, conn))
        }
        Err(_) => {
            Ok(None)
        },
    }
}

fn sign_in(conn: &Connection) -> Result<Option<User>> {
    println!();
    // Get username
    print!("Username: ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();
    
    // Get password
    print!("Password: ");
    io::stdout().flush().unwrap();
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();
    
    // Prepared query
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "SELECT id, username FROM users WHERE username = ?1 AND password = ?2"
    )?;
    
    let result: std::result::Result<i32, rusqlite::Error> = stmt.query_row([username, password], |row| {
        Ok(
            row.get::<_, i32>(0)?,
        )
    });
    
    match result {
        Ok(id) => {
            println!("✓ Login successful!");
            return Ok(Some(User { id: id}))
        }
        Err(_) => {
            println!("✗ Invalid credentials");
            return Ok(None)
        },
    }
}

fn get_user(username: &str, password: &str, conn: &Connection) -> Option<User> {
    let mut stmt: rusqlite::Statement<'_> = conn.prepare(
        "SELECT id, username, balance FROM users WHERE username = ?1 AND password = ?2"
    ).unwrap();
    
    let result: std::result::Result<i32, rusqlite::Error> = stmt.query_row([username, password], |row| {
        Ok(
            row.get::<_, i32>(0)?
        )
    });

    match result {
        Ok(id) => {
            println!("Login successful!");
            Some(User{id: id})
        }
        Err(_) => {
            println!("Invalid credentials");
            None
        },
    }
}