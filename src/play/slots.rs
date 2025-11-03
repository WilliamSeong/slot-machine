use rusqlite::{Connection};
use crate::db::dbqueries;
use crate::interfaces::user::User;
use crate::logger::logger;
use crate::cryptography::rng::CasinoRng;
use colored::*;
use std::io::{self, Write};

use crate::interfaces::menus;
// Display payout table to user before playing
fn display_payout_table(symbol_probs: &[(String, usize, f64)], bet: f64) {
    menus::print_box_top(50);
    menus::print_box_line("💰 PAYOUT TABLE 💰", 48);
    menus::print_box_separator(50);
    menus::print_box_line("Three of a kind pays:", 50);
    menus::print_box_separator(50);
    
    // Calculate total weight for probability display
    let total_weight: usize = symbol_probs.iter().map(|(_, w, _)| w).sum();
    
    for (symbol, weight, payout) in symbol_probs {
        let probability = (*weight as f64 / total_weight as f64) * 100.0;
        let winnings = payout * bet;
        menus::print_box_line(&format!("{} {} {} = ${:<6.2} ({}x) [{:.1}% chance]", 
            symbol, symbol, symbol, 
            winnings, 
            payout,
            probability), 47);
    }
    
    menus::print_box_separator(50);
    menus::print_box_line("Two matching pays:", 50);
    menus::print_box_line("Any two symbols = 50% of three-match payout", 50);
    menus::print_box_bottom(50);
    println!();
}

// function to run the normal slots game, returns a bool to indiciate whether to change bet (true) or to exit the game (false)
pub fn normal_slots(conn: &Connection, bet: f64, user: &User) -> bool {
    logger::info(&format!("User ID: {} started normal slots game with bet: ${:.2}", user.id, bet));
    
    // Load symbol probabilities from database once (commissioner-configured)
    let symbol_probs = match dbqueries::get_symbol_probabilities(conn, "normal") {
        Ok(probs) => probs,
        Err(e) => {
            logger::error(&format!("Failed to load symbol probabilities: {}", e));
            println!("Error loading game configuration");
            return true;
        }
    };

    
    loop {

        // Check if player has the funds
        if !dbqueries::check_funds(conn, user, bet as f64) {
            logger::warning(&format!("User ID: {} has insufficient funds for bet: ${:.2}", user.id, bet));
            println!("{}", "Insufficient funds!".red().bold());
            return true;
        }

        // Convert to weighted format for RNG
        let weighted_symbols: Vec<(&str, usize)> = symbol_probs.iter()
            .map(|(sym, weight, _)| (sym.as_str(), *weight))
            .collect();

        // CHARGE BET FIRST before playing
        logger::transaction(&format!("User ID: {} placing bet of ${:.2} for normal slots", user.id, bet));
        let balance_after_bet = dbqueries::transaction(conn, user, -bet);
        
        if balance_after_bet < 0.0 {
            // This shouldn't happen due to check_funds, but safety check
            println!("{}", "Transaction failed!".red().bold());
            return true;
        }
        
        println!("{}", format!("Bet placed: ${:.2}", bet).yellow());
        println!("{}", format!("Balance: ${:.2}", balance_after_bet).bright_white());

        // Create cryptographically secure RNG
        let mut rng = CasinoRng::new();

        logger::info(&format!("User ID: {} spinning slots with bet: ${:.2}", user.id, bet));
        println!("\n{}", "🎰 SLOT MACHINE 🎰".bright_yellow().bold());
        
        // Display payout table to user
        display_payout_table(&symbol_probs, bet);

                
        // Spin the slots using cryptographically secure weighted random selection
        let slot1 = rng.weighted_choice(&weighted_symbols).unwrap();
        let slot2 = rng.weighted_choice(&weighted_symbols).unwrap();
        let slot3 = rng.weighted_choice(&weighted_symbols).unwrap();

        // Animate
        for _ in 0..30 {
            let anim1 = rng.weighted_choice(&weighted_symbols).unwrap();
            let anim2 = rng.weighted_choice(&weighted_symbols).unwrap();
            let anim3 = rng.weighted_choice(&weighted_symbols).unwrap();
            print!("\r{} | {} | {}", anim1, anim2, anim3);
            io::stdout().flush().ok();
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        
        // Final result
        println!("\r{} | {} | {}", slot1, slot2, slot3);
        logger::info(&format!("User ID: {} slot result: {} | {} | {}", user.id, slot1, slot2, slot3));

        std::thread::sleep(std::time::Duration::from_millis(500));

        // Check win and calculate payout using database-configured multipliers
        if slot1 == slot2 && slot2 == slot3 {
            // Three of a kind! Get the payout multiplier for this symbol
            let payout_multiplier = symbol_probs.iter()
                .find(|(sym, _, _)| sym == slot1)
                .map(|(_, _, mult)| mult)
                .unwrap_or(&3.0);
            
            let winnings = payout_multiplier * bet;
            logger::transaction(&format!("User ID: {} won ${:.2} with three {}s in normal slots", user.id, winnings, slot1));
            
            // DEPOSIT WINNINGS
            let final_balance = dbqueries::transaction(conn, user, winnings);
            
            println!("\n{}", "═══════════════════════════════════════".green().bold());
            println!("{}", "    🎉 JACKPOT! THREE OF A KIND! 🎉    ".green().bold());
            println!("{}", "═══════════════════════════════════════".green().bold());
            println!("\n{}  {} {} {}", "Result:".bright_white().bold(), slot1, slot1, slot1);
            println!("{} ${:.2} × {:.1}x = ${:.2}", "Payout:".bright_white().bold(), bet, payout_multiplier, winnings);
            println!("{} ${:.2}", "Balance:".bright_white().bold(), final_balance);
            println!();
            let _ = dbqueries::add_win(conn, "normal");
            let _ = dbqueries::add_user_win(conn, user, "normal", winnings);
        } else if slot1 == slot2 || slot2 == slot3 || slot1 == slot3 {
            // Two matching symbols - use half multiplier
            let matching_symbol = if slot1 == slot2 { slot1 } else if slot2 == slot3 { slot2 } else { slot1 };
            let base_multiplier = symbol_probs.iter()
                .find(|(sym, _, _)| sym == matching_symbol)
                .map(|(_, _, mult)| mult)
                .unwrap_or(&3.0);
            let payout_multiplier = base_multiplier * 0.5; // Half payout for two symbols
            let winnings = payout_multiplier * bet;
            logger::transaction(&format!("User ID: {} won ${:.2} with two matching symbols in normal slots", user.id, winnings));
            
            // DEPOSIT WINNINGS
            let final_balance = dbqueries::transaction(conn, user, winnings);
            
            println!("\n{}", "═══════════════════════════════════════".yellow().bold());
            println!("{}", "      ✨ TWO MATCHING SYMBOLS! ✨       ".yellow().bold());
            println!("{}", "═══════════════════════════════════════".yellow().bold());
            println!("\n{}  Two {}s matched!", "Result:".bright_white().bold(), matching_symbol);
            println!("{} ${:.2} × {:.1}x = ${:.2}", "Payout:".bright_white().bold(), bet, payout_multiplier, winnings);
            println!("{} ${:.2}", "Balance:".bright_white().bold(), final_balance);
            println!();
            let _ = dbqueries::add_win(conn, "normal");
            let _ = dbqueries::add_user_win(conn, user, "normal", winnings);
        } else {
            // Loss - bet already deducted, no winnings to add
            logger::transaction(&format!("User ID: {} lost ${:.2} in normal slots", user.id, bet));
            
            println!("\n{}", "═══════════════════════════════════════".red());
            println!("{}", "           ❌ NO MATCH ❌               ".red().bold());
            println!("{}", "═══════════════════════════════════════".red());
            println!("\n{}  No matching symbols", "Result:".bright_white().bold());
            println!("{} ${:.2}", "Lost:".bright_white().bold(), bet);
            println!("{} ${:.2}", "Balance:".bright_white().bold(), balance_after_bet);
            println!();
            let _ = dbqueries::add_loss(conn, "normal");
            let _ = dbqueries::add_user_loss(conn, user, "normal");
        }

        // Show options to user
        let menu_options = vec!["Spin Again", "Change Bet", "Exit"];
        let user_input = menus::menu_generator("═══ 🎰 Play Again? 🎰 ═══", &menu_options);

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