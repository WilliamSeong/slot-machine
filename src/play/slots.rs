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

    // Logging player attempt
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

        // Convert to weighted format for RNG
        let weighted_symbols: Vec<(&str, usize)> = symbol_probs.iter()
            .map(|(sym, weight, _)| (sym.as_str(), *weight))
            .collect();

        // Create cryptographically secure RNG
        let mut rng = CasinoRng::new();

        // Log game activity
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

        if check_three_of_kind(slot1, slot2, slot3) { // jackpot(match three)
            let base_multiplier = *get_base_multiplier(&symbol_probs, &slot1);
            // Calculate the winnings
            let winnings = calculate_three_match_payout(bet, base_multiplier);
            // Log the results
            logger::transaction(&format!("User ID: {} won ${:.2} with three {}s in normal slots", user.id, winnings, slot1));
            // Deposit winnings
            let final_balance = dbqueries::transaction(conn, user, winnings);
            print_jackpot_message(slot1, bet, base_multiplier, winnings, final_balance);
            // Collect statistics
            let _ = dbqueries::add_win(conn, "normal");
            let _ = dbqueries::add_user_win(conn, user, "normal", winnings);
        } else if let Some(symbol) = check_two_match(slot1, slot2, slot3) { // match two
            let base_multiplier = *get_base_multiplier(&symbol_probs, symbol);
            // Calculate the winnings
            let winnings = calculate_two_match_payout(bet, base_multiplier);
            // Log the results
            logger::transaction(&format!("User ID: {} won ${:.2} with three {}s in normal slots", user.id, winnings, slot1));
            // Deposit winnings
            let final_balance = dbqueries::transaction(conn, user, winnings);
            print_semi_jackpot_message(symbol, bet, base_multiplier, winnings, final_balance);
            // Collect statistics
            let _ = dbqueries::add_win(conn, "normal");
            let _ = dbqueries::add_user_win(conn, user, "normal", winnings);
        } else { // Lose
            // Log the loss
            logger::transaction(&format!("User ID: {} lost ${:.2} in normal slots", user.id, bet));
            print_losing_message(bet, balance_after_bet);
            // Collect statistics
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

// Normal Slots helper functions
fn calculate_three_match_payout(bet: f64, multiplier: f64) -> f64 {
    bet * multiplier
}

fn calculate_two_match_payout(bet: f64, base_multiplier: f64) -> f64 {
    bet * base_multiplier * 0.5
}

fn check_three_of_kind(slot1: &str, slot2: &str, slot3: &str) -> bool {
    slot1 == slot2 && slot2 == slot3
}

fn check_two_match<'a>(slot1: &'a str, slot2: &'a str, slot3: &'a str) -> Option<&'a str> {
    if slot1 == slot2 {
        Some(slot1)
    } else if slot2 == slot3 {
        Some(slot2)
    } else if slot1 == slot3 {
        Some(slot1)
    } else {
        None
    }
}

fn get_base_multiplier<'a>(symbol_probs: &'a[(String, usize, f64)], symbol: &str) -> &'a f64 {
    symbol_probs.iter()
        .find(|(sym, _, _)| sym == symbol)
        .map(|(_, _, mult)| mult)
        .unwrap_or(&3.0)
}

fn print_jackpot_message(symbol: &str, bet: f64, base_multiplier: f64, winnings: f64, final_balance: f64) {
    println!("\n{}", "═══════════════════════════════════════".green().bold());
    println!("{}", "    🎉 JACKPOT! THREE OF A KIND! 🎉    ".green().bold());
    println!("{}", "═══════════════════════════════════════".green().bold());
    println!("\n{}  {} {} {}", "Result:".bright_white().bold(), symbol, symbol, symbol);
    println!("{} ${:.2} × {:.1}x = ${:.2}", "Payout:".bright_white().bold(), bet, base_multiplier, winnings);
    println!("{} ${:.2}", "Balance:".bright_white().bold(), final_balance);
    println!();
}

fn print_semi_jackpot_message(matching_symbol:&str, bet: f64, base_multiplier: f64, winnings: f64, final_balance: f64) {
            println!("\n{}", "═══════════════════════════════════════".yellow().bold());
            println!("{}", "      ✨ TWO MATCHING SYMBOLS! ✨       ".yellow().bold());
            println!("{}", "═══════════════════════════════════════".yellow().bold());
            println!("\n{}  Two {}s matched!", "Result:".bright_white().bold(), matching_symbol);
            println!("{} ${:.2} × {:.1}x = ${:.2}", "Payout:".bright_white().bold(), bet, base_multiplier, winnings);
            println!("{} ${:.2}", "Balance:".bright_white().bold(), final_balance);
            println!();
}

fn print_losing_message(bet: f64, final_balance: f64) {
    println!("\n{}", "═══════════════════════════════════════".red());
    println!("{}", "           ❌ NO MATCH ❌               ".red().bold());
    println!("{}", "═══════════════════════════════════════".red());
    println!("\n{}  No matching symbols", "Result:".bright_white().bold());
    println!("{} ${:.2}", "Lost:".bright_white().bold(), bet);
    println!("{} ${:.2}", "Balance:".bright_white().bold(), final_balance);
    println!();

}

#[cfg(test)]
mod tests {
    use super::*;

    // Test payout helper functions
    #[test]
    fn test_three_match_payout_basic() {
        let bet = 10.0;
        let multiplier = 2.5;
        let result = calculate_three_match_payout(bet, multiplier);
        assert_eq!(result, 25.0);
    }

    #[test]
    fn test_two_match_payout_is_half() {
        let bet = 10.0;
        let base_multiplier = 4.0;
        let result = calculate_two_match_payout(bet, base_multiplier);
        assert_eq!(result, 20.0); // 10 * 4.0 * 0.5 = 20
    }

    #[test]
    fn test_payout_float() {
        let bet = 2.50;
        let multiplier = 1.5;
        let result = calculate_three_match_payout(bet, multiplier);
        assert!((result - 3.75).abs() < 0.01);
    }
    
    // Test win detection
    #[test]
    fn test_three_of_kind_all_match() {
        assert!(check_three_of_kind("🍒", "🍒", "🍒"));
    }

    #[test]
    fn test_three_of_kind_no_match() {
        assert!(!check_three_of_kind("🍒", "🍋", "🍊"));
    }

    #[test]
    fn test_three_of_kind_two_match() {
        assert!(!check_three_of_kind("🍒", "🍒", "🍋"));
    }

    #[test]
    fn test_two_match_first_two() {
        let result = check_two_match("🍒", "🍒", "🍋");
        assert_eq!(result, Some("🍒"));
    }

    #[test]
    fn test_two_match_last_two() {
        let result = check_two_match("🍒", "🍋", "🍋");
        assert_eq!(result, Some("🍋"));
    }

    #[test]
    fn test_two_match_first_and_third() {
        let result = check_two_match("🍒", "🍋", "🍒");
        assert_eq!(result, Some("🍒"));
    }

    #[test]
    fn test_two_match_no_match() {
        let result = check_two_match("🍒", "🍋", "🍊");
        assert_eq!(result, None);
    }

    #[test]
    fn test_two_match_all_three() {
        // When all three match, should still return the symbol
        let result = check_two_match("🍒", "🍒", "🍒");
        assert_eq!(result, Some("🍒"));
    }
}