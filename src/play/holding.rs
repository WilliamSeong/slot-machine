use colored::*;
use std::io::{self, Write};
use rusqlite::Connection;

use crate::logger::logger;
use crate::interfaces::menus::{menu_generator, menu_generator_multi};
use crate::cryptography::rng::CasinoRng;

use crate::interfaces::user::User;
use crate::db::dbqueries;
/// Hold 5x3 slot game - allows up to 2 reels to be held for next spin
pub fn hold_game(conn: &Connection, user: &User, bet: f64) -> bool {
    // Load symbol probabilities from database
    let symbol_probs = match dbqueries::get_symbol_probabilities(conn, "holding") {
        Ok(probs) => probs,
        Err(e) => {
            logger::error(&format!("Failed to load symbol probabilities: {}", e));
            println!("{}", "Error loading game configuration".red());
            return true;
        }
    };
    
    // Extract symbols
    let symbols: Vec<&str> = symbol_probs.iter()
        .map(|(sym, _, _)| sym.as_str())
        .collect();
    
    // Convert to weighted format for RNG
    let weighted_symbols: Vec<(&str, usize)> = symbol_probs.iter()
        .map(|(sym, weight, _)| (sym.as_str(), *weight))
        .collect();
    
    let mut rng = CasinoRng::new();
    
    println!("\n{}", "═══ 🎰 Welcome to Hold Slots! 🎰 ═══".bright_yellow().bold());
    println!("{}", "Hold up to 2 reels for a second spin!".bright_cyan());
    println!("{} ${:.2}\n", "Your bet:".bright_white().bold(), bet);
    
    loop {
        // Check if player has the funds for base bet
        if !dbqueries::check_funds(conn, user, bet) {
            logger::warning(&format!("User ID: {} has insufficient funds for bet: ${:.2}", user.id, bet));
            println!("{}", "Insufficient funds!".red().bold());
            return true;
        }
        
        // CHARGE BASE BET FIRST
        logger::transaction(&format!("User ID: {} placing bet of ${:.2} for holding slots", user.id, bet));
        let mut current_balance = dbqueries::transaction(conn, user, -bet);
        
        if current_balance < 0.0 {
            println!("{}", "Transaction failed!".red().bold());
            return true;
        }
        
        println!("{}", format!("Bet placed: ${:.2}", bet).yellow());
        
        // Use cryptographically secure weighted selection
        let mut reels: [&str; 5] = [
            rng.weighted_choice(&weighted_symbols).unwrap(),
            rng.weighted_choice(&weighted_symbols).unwrap(),
            rng.weighted_choice(&weighted_symbols).unwrap(),
            rng.weighted_choice(&weighted_symbols).unwrap(),
            rng.weighted_choice(&weighted_symbols).unwrap()
        ];
        let mut held = [false; 5]; 

        // First result
        println!("\n{}", "🎰 First Spin 🎰".bright_yellow().bold());
        // Animate
        for _ in 0..30 {
            print!("\r{} | {} | {} | {} | {} ANIMATION", 
                rng.weighted_choice(&weighted_symbols).unwrap(),
                rng.weighted_choice(&weighted_symbols).unwrap(),
                rng.weighted_choice(&weighted_symbols).unwrap(),
                rng.weighted_choice(&weighted_symbols).unwrap(),
                rng.weighted_choice(&weighted_symbols).unwrap()
            );
            io::stdout().flush().ok();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        print!("\r{} | {} | {} | {} | {}", reels[0], reels[1], reels[2], reels[3], reels[4]);
        io::stdout().flush().ok();
        logger::info(&format!("User ID: {} slot result: {} | {} | {} | {} | {}", user.id, reels[0], reels[1], reels[2], reels[3], reels[4]));
        
        
        let menu_options = vec!["1", "2", "3", "4", "5"];
        let user_input = menu_generator_multi("Select up to 2 slots to hold (space to select)", &menu_options);
        
        // let mut input = String::new();
        // io::stdin().read_line(&mut input).unwrap();

        for num in user_input {
            // println!("selected index {}", num);
            held[num] = true;
        }

        let held_count = held.iter().filter(|&&h| h).count();
        
        // If user holds reels, charge additional bet (each held reel = +25% extra bet)
        if held_count > 0 {
            let hold_charge = bet * 0.25 * held_count as f64;
            
            // Check if they can afford the hold charge
            if current_balance < hold_charge {
                println!("{}", format!("⚠️ Cannot afford to hold {} reels (costs ${:.2})", held_count, hold_charge).red());
                println!("Continuing without holds...");
                held = [false; 5]; // Reset holds
            } else {
                // Charge for holding reels
                current_balance = dbqueries::transaction(conn, user, -hold_charge);
                println!("{}", format!("Hold charge: ${:.2} for {} reel(s)", hold_charge, held_count).yellow());
                println!("{}", format!("Balance: ${:.2}", current_balance).bright_white());
            }
        }
        
        // Calculate total bet (for payout calculation)
        let multiplier = 1.0 + 0.25 * held_count as f64;
        let final_bet = bet * multiplier;


        // Show first spin again
        // println!("\n{}", "🎰 First Spin 🎰".bright_yellow().bold());
        // print!("\r{} | {} | {} | {} | {}", reels[0], reels[1], reels[2], reels[3], reels[4]);
        
        println!("\n{}", "🎰 Second Spin 🎰".bright_cyan().bold());
        // Check if user holds then animate if so
        if held.iter().any(|&h| h) {
            
            for _ in 0..30 {
                print!("\r{} | {} | {} | {} | {}", 
                    if held[0] { reels[0] } else { rng.weighted_choice(&weighted_symbols).unwrap() },
                    if held[1] { reels[1] } else { rng.weighted_choice(&weighted_symbols).unwrap() },
                    if held[2] { reels[2] } else { rng.weighted_choice(&weighted_symbols).unwrap() },
                    if held[3] { reels[3] } else { rng.weighted_choice(&weighted_symbols).unwrap() },
                    if held[4] { reels[4] } else { rng.weighted_choice(&weighted_symbols).unwrap() }
                );
                io::stdout().flush().ok();
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        // Second spin - use cryptographic RNG for non-held reels
        for i in 0..5 {
            if !held[i] {
                reels[i] = rng.weighted_choice(&weighted_symbols).unwrap();
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
            // WIN - deposit winnings (bets already deducted)
            let final_balance = dbqueries::transaction(conn, user, payout);
            
            println!("\n{}", "═══════════════════════════════════════".green().bold());
            println!("{}", "         🎉 YOU WIN! 🎉                ".green().bold());
            println!("{}", "═══════════════════════════════════════".green().bold());
            println!("\n{} {} of a kind!", "Result:".bright_white().bold(), max_count);
            println!("{} ${:.2}", "Payout:".bright_white().bold(), payout);
            println!("{} ${:.2}", "Balance:".bright_white().bold(), final_balance);
            println!();
            let _ = dbqueries::add_win(conn, "holding");
            let _ = dbqueries::add_user_win(conn, user, "holding", payout);

        } else {
            // LOSS - bets already deducted, no winnings
            println!("\n{}", "═══════════════════════════════════════".red());
            println!("{}", "           ❌ NO WIN ❌                 ".red().bold());
            println!("{}", "═══════════════════════════════════════".red());
            println!("\n{}  No matching symbols", "Result:".bright_white().bold());
            println!("{} ${:.2}", "Lost:".bright_white().bold(), final_bet);
            println!("{} ${:.2}", "Balance:".bright_white().bold(), current_balance);
            println!();
            let _ = dbqueries::add_loss(conn, "holding");
            let _ = dbqueries::add_user_loss(conn, user, "holding");

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
