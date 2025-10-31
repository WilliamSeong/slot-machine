use rusqlite::{Connection};
use crate::db::dbqueries;
use crate::interfaces::user::User;
use colored::*;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use rand::RngCore;
use std::io::{self, Write};

// function to run the normal slots game, returns a bool to indiciate whether to change bet (true) or to exit the game (false)
pub fn normal_slots(conn: &Connection, bet: f64, user: &User) -> bool {
    loop {
        // Check if player has the funds
        if !dbqueries::check_funds(conn, user, bet as f64) {
            println!("Insufficient more funds");
            return true;
        }

        let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];
        let mut rng = ChaCha20Rng::from_seed(Default::default());

        
        println!("\n{}", "🎰 SLOT MACHINE 🎰".bright_yellow().bold());
                
        // Spin the slots
        let slot1 = symbols[rng.next_u32() as usize % symbols.len()];
        let slot2 = symbols[rng.next_u32() as usize % symbols.len()];
        let slot3 = symbols[rng.next_u32() as usize % symbols.len()];
        // let slot1 = symbols[0];
        // let slot2 = symbols[1];
        // let slot3 = symbols[2];

        // Animate
        for _ in 0..30 {
            print!("\r{} | {} | {}", 
                symbols[rng.next_u32() as usize % symbols.len()],
                symbols[rng.next_u32() as usize % symbols.len()],
                symbols[rng.next_u32() as usize % symbols.len()]
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
            println!("You win {}", 3.0 * bet);
            println!("Current balance is {}", dbqueries::transaction(conn, user, 3.0 * bet));
            let _ = dbqueries::add_win(conn, "normal");
            let _ = dbqueries::add_user_win(conn, user, "normal", 3.0 * bet);
        } else if slot1 == slot2 || slot2 == slot3 || slot1 == slot3 {
            println!("\n{}", "Nice! Two matching!".green());
            println!("Current balance is {}", dbqueries::transaction(conn, user, 2.0 * bet));
            let _ = dbqueries::add_win(conn, "normal");
            let _ = dbqueries::add_user_win(conn, user, "normal", 2.0 * bet);
        } else {
            println!("\n{}", "YOU LOSE!".red());
            println!("You lose {}", &bet);
            println!("Current balance is {}", dbqueries::transaction(conn, user, -(bet)));
            let _ = dbqueries::add_loss(conn, "normal");
            let _ = dbqueries::add_user_loss(conn, user, "normal");
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
