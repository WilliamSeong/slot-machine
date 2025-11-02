use rand::{rng, Rng};
use colored::*;
use std::io::{self, Write};
use rusqlite::Connection;

use crate::logger::logger;
use crate::interfaces::menus::{menu_generator, menu_generator_multi};

use crate::interfaces::user::User;

/// Hold 5x3 slot game - allows up to 2 reels to be held for next spin
pub fn hold_game(conn: &Connection, user: &User, bet: f64) -> bool {
    loop {
        
        let symbols = ["🍒", "🍋", "🍊", "💎", "🔔", "⭐"];
        let mut rng = rng();
        let mut reels = [
                                    symbols[rng.random_range(0..symbols.len())],
                                    symbols[rng.random_range(0..symbols.len())],
                                    symbols[rng.random_range(0..symbols.len())],
                                    symbols[rng.random_range(0..symbols.len())],
                                    symbols[rng.random_range(0..symbols.len())]
                                    ];
        let mut held = [false; 5];    

        // First result
        println!("\n{}", "🎰 First Spin 🎰".bright_yellow().bold());
        // Animate
        for _ in 0..30 {
            print!("\r{} | {} | {} | {} | {}", 
                symbols[rng.random_range(0..symbols.len())],
                symbols[rng.random_range(0..symbols.len())],
                symbols[rng.random_range(0..symbols.len())],
                symbols[rng.random_range(0..symbols.len())],
                symbols[rng.random_range(0..symbols.len())]
            );
            io::stdout().flush().ok();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        print!("\r{} | {} | {} | {} | {}", reels[0], reels[1], reels[2], reels[3], reels[4]);
        logger::info(&format!("User ID: {} slot result: {} | {} | {} | {} | {}", user.id, reels[0], reels[1], reels[2], reels[3], reels[4]));
        
        
        let menu_options = vec!["1", "2", "3", "4", "5"];
        let user_input = menu_generator_multi("Select up to 2 slots to hold", &menu_options);
        
        // let mut input = String::new();
        // io::stdin().read_line(&mut input).unwrap();

        for num in user_input {
            // println!("selected index {}", num);
            held[num] = true;
        }

        let held_count = held.iter().filter(|&&h| h).count();
        // if held_count > 2 {
        //     println!("⚠️ You can only hold up to 2 reels. Only the first 2 will be held.");
        // }

        // Adjust bet for fairness (each held reel = +25%)
        let multiplier = 1.0 + 0.25 * held_count as f64;
        let final_bet = (bet * multiplier).round();
        // println!(
        //     "Held {} reel(s) → Adjusted Bet: ${:.2} (x{:.2})",
        //     held_count, final_bet, multiplier
        // );


        // Show first spin again
        // println!("\n{}", "🎰 First Spin 🎰".bright_yellow().bold());
        // print!("\r{} | {} | {} | {} | {}", reels[0], reels[1], reels[2], reels[3], reels[4]);
        
        // Second spin
        for i in 0..5 {
            if !held[i] {
                reels[i] = symbols[rng.random_range(0..symbols.len())];
            }
        }

        println!("\n{}", "🎰 Second Spin 🎰".bright_cyan().bold());
        // Check if user holds then animate if so
        if held.iter().any(|&h| h) {
            
            for _ in 0..30 {
                print!("\r{} | {} | {} | {} | {}", 
                    if held[0] { reels[0] } else { symbols[rng.random_range(0..symbols.len())] },
                    if held[1] { reels[1] } else { symbols[rng.random_range(0..symbols.len())] },
                    if held[2] { reels[2] } else { symbols[rng.random_range(0..symbols.len())] },
                    if held[3] { reels[3] } else { symbols[rng.random_range(0..symbols.len())] },
                    if held[4] { reels[4] } else { symbols[rng.random_range(0..symbols.len())] }
                );
                io::stdout().flush().ok();
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        // Show second spin
        println!("\r{} | {} | {} | {} | {}", reels[0], reels[1], reels[2], reels[3], reels[4]);

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
}
