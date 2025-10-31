use rusqlite::{Connection};
use crate::db::dbqueries;
use crate::interfaces::user::User;
use crate::logger::logger;
use colored::*;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use rand::RngCore;
use std::io::{self, Write};

// function to run the normal slots game, returns a bool to indiciate whether to change bet (true) or to exit the game (false)
pub fn normal_slots(conn: &Connection, bet: f64, user: &User) -> bool {
    logger::info(&format!("User ID: {} started normal slots game with bet: ${:.2}", user.id, bet));
    
    loop {
        // Check if player has the funds
        if !dbqueries::check_funds(conn, user, bet as f64) {
            logger::warning(&format!("User ID: {} has insufficient funds for bet: ${:.2}", user.id, bet));
            println!("Insufficient funds");
            return true;
        }

        let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];
        let mut rng = ChaCha20Rng::from_seed(Default::default());

        logger::info(&format!("User ID: {} spinning slots with bet: ${:.2}", user.id, bet));
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
        logger::info(&format!("User ID: {} slot result: {} | {} | {}", user.id, slot1, slot2, slot3));

        std::thread::sleep(std::time::Duration::from_millis(500));

        // Check win (adjustable probability via symbol frequency)
        if slot1 == slot2 && slot2 == slot3 {
            // Jackpot win
            let winnings = 3.0 * bet;
            logger::transaction(&format!("User ID: {} won jackpot of ${:.2} in normal slots", user.id, winnings));
            
            println!("\n{}", "🎉 JACKPOT! YOU WIN! 🎉".green().bold());
            println!("You win {}", winnings);
            println!("Current balance is {}", dbqueries::transaction(conn, user, winnings));
            let _ = dbqueries::add_win(conn, "normal");
            let _ = dbqueries::add_user_win(conn, user, "normal", winnings);
        } else if slot1 == slot2 || slot2 == slot3 || slot1 == slot3 {
            // Two matching win
            let winnings = 2.0 * bet;
            logger::transaction(&format!("User ID: {} won ${:.2} with two matching symbols in normal slots", user.id, winnings));
            
            println!("\n{}", "Nice! Two matching!".green());
            println!("Current balance is {}", dbqueries::transaction(conn, user, winnings));
            let _ = dbqueries::add_win(conn, "normal");
            let _ = dbqueries::add_user_win(conn, user, "normal", winnings);
        } else {
            // Loss
            logger::transaction(&format!("User ID: {} lost ${:.2} in normal slots", user.id, bet));
            
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
            "" => {
                logger::info(&format!("User ID: {} continuing with same bet", user.id));
                continue;
            }
            "1" => {
                logger::info(&format!("User ID: {} changing bet", user.id));
                return true;
            }
            "2" => {
                logger::info(&format!("User ID: {} exiting slots game", user.id));
                return false;
            }
            _ => {
                logger::info(&format!("User ID: {} made invalid selection, continuing game", user.id));
                println!("Playing again..."); 
                continue;
            }
        }
    }
}