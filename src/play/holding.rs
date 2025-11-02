use rand::{rng, Rng};
use colored::*;
use std::io::{self};
use rusqlite::Connection;

use crate::logger::logger;
use crate::interfaces::menus::menu_generator;

use crate::interfaces::user::User;

/// Hold 5x3 slot game - allows up to 2 reels to be held for next spin
pub fn hold_game(conn: &Connection, user: &User, bet: f64) -> bool {
    loop {
        
        let symbols = ["🍒", "🍋", "🍊", "💎", "🔔", "⭐"];
        let mut rng = rng();
        let mut reels = ["", "", "", "", ""];
        let mut held = [false; 5];

        // First spin
        for i in 0..5 {
            reels[i] = symbols[rng.random_range(0..symbols.len())];
        }

        println!("\n{}", "🎰 First Spin 🎰".bright_yellow().bold());
        println!("{:?}", reels);
        println!("\nEnter up to 2 reel numbers to hold (1-5), separated by spaces (e.g. '2 4'): ");

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        for num in input.split_whitespace() {
            if let Ok(n) = num.parse::<usize>() {
                if n >= 1 && n <= 5 {
                    held[n - 1] = true;
                }
            }
        }

        let held_count = held.iter().filter(|&&h| h).count();
        if held_count > 2 {
            println!("⚠️ You can only hold up to 2 reels. Only the first 2 will be held.");
        }

        // // Player places bet
        // println!("\nEnter your bet amount: ");
        // let mut bet_input = String::new();
        // io::stdin().read_line(&mut bet_input).unwrap();
        // let base_bet: f64 = bet_input.trim().parse().unwrap_or(1.0);

        // Adjust bet for fairness (each held reel = +25%)
        let multiplier = 1.0 + 0.25 * held_count as f64;
        let final_bet = (bet * multiplier).round();
        println!(
            "Held {} reel(s) → Adjusted Bet: ${:.2} (x{:.2})",
            held_count, final_bet, multiplier
        );

        // Second spin
        for i in 0..5 {
            if !held[i] {
                reels[i] = symbols[rng.random_range(0..symbols.len())];
            }
        }

        println!("\n{}", "🎰 Second Spin 🎰".bright_cyan().bold());
        println!("{:?}", reels);

        // Simple win check (3+ of a kind)
        let mut win_map = std::collections::HashMap::new();
        for &symbol in &reels {
            *win_map.entry(symbol).or_insert(0) += 1;
        }
        let max_count = win_map.values().copied().max().unwrap_or(1);
        let payout = match max_count {
            5 => 10.0 * final_bet,
            4 => 5.0 * final_bet,
            3 => 2.0 * final_bet,
            _ => 0.0,
        };

        if payout > 0.0 {
            println!("{}", format!("🎉 You won ${:.2}!", payout).green().bold());
        } else {
            println!("{}", "❌ No win this time!".red().bold());
        }

        // Show options to user
        let menu_options = vec!["Spin Again", "Change Bet", "Exit"];
        let user_input = menu_generator("═══ 🎰 Play Again? 🎰 ═══", &menu_options);

        match user_input.trim() {
            "Spin Again" => {
                logger::info(&format!("User ID: {} continuing with same bet", user.id));
                continue;
            }
            "Change Bet" => {
                logger::info(&format!("User ID: {} changing bet", user.id));
                return true;
            }
            "Exit" => {
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


    // Store results in DB
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS spins_hold (
    //         id INTEGER PRIMARY KEY,
    //         user_id INTEGER,
    //         reels TEXT,
    //         held TEXT,
    //         bet REAL,
    //         payout REAL,
    //         ts DATETIME DEFAULT CURRENT_TIMESTAMP
    //     )",
    //     [],
    // ).unwrap();

    // conn.execute(
    //     "INSERT INTO spins_hold (user_id, reels, held, bet, payout)
    //     VALUES (?1, ?2, ?3, ?4, ?5)",
    //     (
    //         user_id,
    //         format!("{:?}", reels),
    //         format!("{:?}", held),
    //         final_bet,
    //         payout,
    //     ),
    // ).unwrap();
}
