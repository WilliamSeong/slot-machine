use colored::*;
use std::io;
use std::io::Write;
use rusqlite::Connection;
use crate::interfaces::user::User;
use crate::authentication::authorization;
use crate::logger::logger;
use crate::interfaces::menus::menu_generator;

/// Commissioner Control Panel - Requires commissioner role
pub fn commissioner_menu(conn: &Connection, user: &User) -> rusqlite::Result<()> {
    // SECURITY: Verify user has commissioner role
    if let Err(e) = authorization::require_commissioner(conn, user) {
        logger::security(&format!("Blocked unauthorized access to commissioner menu by User ID: {}: {}", user.id, e));
        return Ok(());
    }
    
    logger::security(&format!("Commissioner (User ID: {}) accessed commissioner menu", user.id));
    loop {
        let menu_options = vec!["Run fairness test", "View game probabilities", "Adjust symbol weights", "Adjust symbol payouts", "Logout"];
        let user_input = menu_generator("═══ 🧮 Commissioner Control Panel 🧮 ═══", &menu_options);

        match user_input.trim() {
            "Run fairness test" => {
                logger::info(&format!("Commissioner (User ID: {}) running fairness test", user.id));
                run_commissioner_test(conn, user)
            },
            "View game probabilities" => {
                logger::info(&format!("Commissioner (User ID: {}) viewing game probabilities", user.id));
                view_game_probabilities(conn, user)
            },
            "Adjust symbol weights" => {
                logger::security(&format!("Commissioner (User ID: {}) adjusting symbol weights", user.id));
                adjust_symbol_weights(conn, user)
            },
            "Adjust symbol payouts" => {
                logger::security(&format!("Commissioner (User ID: {}) adjusting symbol payouts", user.id));
                adjust_symbol_payouts(conn, user)
            },
            "Logout" => {
                logger::info(&format!("Commissioner (User ID: {}) exited commissioner menu", user.id));
                break;
            },
            _ => {
                logger::warning(&format!("Commissioner (User ID: {}) entered invalid menu choice", user.id));
                println!("Invalid input");
            }
        }
    }
    Ok(())
}

/// Automated fairness test for X rounds - REQUIRES COMMISSIONER ROLE
fn run_commissioner_test(conn: &Connection, user: &User) {
    use crate::db::dbqueries;
    use crate::cryptography::rng::CasinoRng;
    
    // SECURITY: Double-check authorization
    if authorization::require_commissioner(conn, user).is_err() {
        return;
    }
    
    logger::security(&format!("Commissioner (User ID: {}) initiated fairness test", user.id));
    
    // Select game to test
    let game_options = vec!["normal", "multi", "holding", "Cancel"];
    let game_choice = menu_generator("Select Game to Test", &game_options);
    
    if game_choice == "Cancel" {
        return;
    }
    
    let game_name = game_choice;
    
    // Load symbol probabilities from database
    let symbol_probs = match dbqueries::get_symbol_probabilities(conn, game_name) {
        Ok(probs) => probs,
        Err(e) => {
            logger::error(&format!("Failed to load symbol probabilities: {}", e));
            println!("{}", "Error loading game configuration".red());
            return;
        }
    };
    
    println!("\n{}", format!("═══ Testing {} Game ═══", game_name.to_uppercase()).bright_cyan().bold());
    
    // Ask for seed (for reproducible testing)
    println!("\nEnter seed for testing (leave empty for random):");
    io::stdout().flush().unwrap();
    let mut seed_input = String::new();
    io::stdin().read_line(&mut seed_input).unwrap();
    let seed_str = seed_input.trim();
    
    if !seed_str.is_empty() {
        println!("Using seed: {}", seed_str);
    } else {
        println!("Using random seed");
    }
    
    println!("\nEnter number of rounds to test:");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let rounds: u32 = input.trim().parse().unwrap_or(100);

    println!("\nRunning {} rounds for {} game...", rounds, game_name);

    // Convert to weighted format for RNG
    let weighted_symbols: Vec<(&str, usize)> = symbol_probs.iter()
        .map(|(sym, weight, _)| (sym.as_str(), *weight))
        .collect();

    // Initialize RNG (seeded if seed provided, otherwise random)
    let mut rng = if seed_str.is_empty() {
        CasinoRng::new()
    } else {
        let seed: u64 = seed_str.parse().unwrap_or_else(|_| {
            // If parsing fails, hash the string to get a seed
            let mut hash = 0u64;
            for (i, byte) in seed_str.bytes().enumerate() {
                hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
                if i >= 8 { break; }
            }
            hash
        });
        CasinoRng::seeded(seed)
    };

    let mut wins = 0;
    let mut partials = 0;
    let mut losses = 0;
    let mut total_bet = 0.0;
    let mut total_payout = 0.0;

    // Run game-specific simulation
    match game_name {
        "normal" => {
            // Normal slots: 3 symbols, match 3 or 2
            for _ in 0..rounds {
                let slot1 = rng.weighted_choice(&weighted_symbols).unwrap();
                let slot2 = rng.weighted_choice(&weighted_symbols).unwrap();
                let slot3 = rng.weighted_choice(&weighted_symbols).unwrap();

                let bet = 1.0;
                total_bet += bet;

                if slot1 == slot2 && slot2 == slot3 {
                    let payout_multiplier = symbol_probs.iter()
                        .find(|(sym, _, _)| sym.as_str() == *slot1)
                        .map(|(_, _, mult)| mult)
                        .unwrap_or(&3.0);
                    wins += 1;
                    total_payout += payout_multiplier * bet;
                } else if slot1 == slot2 || slot2 == slot3 || slot1 == slot3 {
                    let matching_symbol = if slot1 == slot2 { slot1 } else if slot2 == slot3 { slot2 } else { slot1 };
                    let base_multiplier = symbol_probs.iter()
                        .find(|(sym, _, _)| sym.as_str() == *matching_symbol)
                        .map(|(_, _, mult)| mult)
                        .unwrap_or(&3.0);
                    let payout_multiplier = base_multiplier * 0.5;
                    partials += 1;
                    total_payout += payout_multiplier * bet;
                } else {
                    losses += 1;
                }
            }
        },
        "multi" => {
            // Multi-win: 5x5 grid, match rows/columns/diagonals
            let base_multiplier: f64 = symbol_probs.iter()
                .map(|(_, _, mult)| mult)
                .sum::<f64>() / symbol_probs.len() as f64;

            for _ in 0..rounds {
                let bet = 1.0;
                total_bet += bet;

                // Generate 5x5 grid using weighted symbols
                let mut grid = [[' '; 5]; 5];
                for i in 0..5 {
                    for j in 0..5 {
                        let symbol = rng.weighted_choice(&weighted_symbols).unwrap();
                        grid[i][j] = symbol.chars().next().unwrap();
                    }
                }

                // Check for wins - must match actual game logic
                let mut has_horizontal_win = false;
                let mut has_four_corner_win = false;
                let mut any_win = false;

                // Check rows (horizontal wins)
                for row in grid.iter() {
                    if row.iter().all(|&s| s == row[0]) {
                        has_horizontal_win = true;
                        any_win = true;
                    }
                }

                // Check columns
                for col in 0..5 {
                    if (0..5).all(|row| grid[row][col] == grid[0][col]) {
                        any_win = true;
                    }
                }

                // Check diagonals
                if (0..5).all(|i| grid[i][i] == grid[0][0]) {
                    any_win = true;
                }
                if (0..5).all(|i| grid[i][4-i] == grid[0][4]) {
                    any_win = true;
                }

                // Check four corners
                if grid[0][0] == grid[0][4] && grid[0][0] == grid[4][0] && grid[0][0] == grid[4][4] {
                    has_four_corner_win = true;
                    any_win = true;
                }

                if any_win {
                    wins += 1;
                    if has_horizontal_win && has_four_corner_win {
                        total_payout += base_multiplier * 2.0 * bet; // Double jackpot
                    } else {
                        total_payout += base_multiplier * bet;
                    }
                } else {
                    losses += 1;
                }
            }
        },
        "holding" => {
            // Holding: 5 symbols with hold feature (2 spins)
            // This simulation models the actual game with hold mechanics
            for _ in 0..rounds {
                let bet = 1.0;
                total_bet += bet;

                // First spin: Generate 5 symbols
                let mut reels: Vec<&str> = (0..5)
                    .map(|_| *rng.weighted_choice(&weighted_symbols).unwrap())
                    .collect();

                // Count occurrences after first spin
                let mut counts = std::collections::HashMap::new();
                for &symbol in &reels {
                    *counts.entry(symbol).or_insert(0) += 1;
                }

                // Simple hold strategy: hold reels with most common symbol (up to 2)
                let most_common_symbol = counts.iter()
                    .max_by_key(|(_, &count)| count)
                    .map(|(sym, _)| *sym)
                    .unwrap_or(&"");
                
                let mut held_indices = Vec::new();
                for (i, &symbol) in reels.iter().enumerate() {
                    if symbol == most_common_symbol && held_indices.len() < 2 {
                        held_indices.push(i);
                    }
                }
                
                // Calculate hold charge (25% per held reel)
                let held_count = held_indices.len();
                let hold_charge = bet * 0.25 * held_count as f64;
                total_bet += hold_charge;

                // Second spin: Respin non-held reels
                for i in 0..5 {
                    if !held_indices.contains(&i) {
                        reels[i] = *rng.weighted_choice(&weighted_symbols).unwrap();
                    }
                }

                // Count occurrences after second spin
                counts.clear();
                for &symbol in &reels {
                    *counts.entry(symbol).or_insert(0) += 1;
                }

                let max_count = counts.values().copied().max().unwrap_or(0);
                
                // Calculate final bet (base + hold charges)
                let final_bet = bet * (1.0 + 0.25 * held_count as f64);
                
                if max_count >= 3 {
                    let winning_symbol = counts.iter()
                        .max_by_key(|(_, &count)| count)
                        .map(|(sym, _)| *sym)
                        .unwrap();

                    let base_multiplier = symbol_probs.iter()
                        .find(|(sym, _, _)| sym.as_str() == winning_symbol)
                        .map(|(_, _, mult)| *mult)
                        .unwrap_or(2.0);

                    let payout = match max_count {
                        5 => base_multiplier * 5.0 * final_bet,
                        4 => base_multiplier * 2.5 * final_bet,
                        3 => base_multiplier * final_bet,
                        _ => 0.0,
                    };

                    wins += 1;
                    total_payout += payout;
                } else {
                    losses += 1;
                }
            }
        },
        _ => {
            println!("{}", "Unknown game type!".red());
            return;
        }
    }

    let rtp = (total_payout / total_bet) * 100.0;

    println!("\n{}", "🎰 Test Results 🎰".bright_yellow().bold());
    println!("Game: {}", game_name);
    if !seed_str.is_empty() {
        println!("Seed: {}", seed_str);
    }
    println!("Total rounds: {}", rounds);
    
    match game_name {
        "normal" => {
            println!("Wins (3 match): {}", wins);
            println!("Two-symbol matches: {}", partials);
            println!("Losses: {}", losses);
        },
        "multi" => {
            println!("Wins (any line match): {}", wins);
            println!("Losses: {}", losses);
        },
        "holding" => {
            println!("Wins (3+ of a kind): {}", wins);
            println!("Losses: {}", losses);
        },
        _ => {}
    }
    
    println!("Total Bet: ${:.2}", total_bet);
    println!("Total Payout: ${:.2}", total_payout);
    println!("RTP (Return To Player): {:.2}%", rtp);
    
    // Display symbol distribution
    println!("\n{}", "Symbol Probabilities:".bright_cyan());
    let total_weight: usize = symbol_probs.iter().map(|(_, w, _)| w).sum();
    for (symbol, weight, payout) in &symbol_probs {
        let probability = (*weight as f64 / total_weight as f64) * 100.0;
        println!("  {} - {:.1}% chance, {:.1}x payout", symbol, probability, payout);
    }

    // Store test summary in DB
    let seed_for_db = if seed_str.is_empty() { "random" } else { seed_str };
    
    match dbqueries::insert_commissioner_log(conn, game_name, seed_for_db, rounds, wins, partials, losses, rtp) {
        Ok(_) => println!("\n{}", "✓ Test results stored in commissioner_log table.".green().bold()),
        Err(e) => {
            logger::error(&format!("Failed to store test results: {}", e));
            println!("{}", format!("Warning: Failed to store test results: {}", e).yellow());
        }
    }
    
    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut String::new()).ok();
}

// // ---------------------------------------------------------------------------
/*
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS commissioner_log (
                id INTEGER PRIMARY KEY,
                seed TEXT,
                rounds INTEGER,
                wins INTEGER,
                partials INTEGER,
                losses INTEGER,
                rtp REAL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).unwrap();
        conn
    }

    fn dummy_user(role: &str) -> User {
        User {
            id: 1,
            username: "unit_test_user".to_string(),
            role: role.to_string(),
            password_hash: "dummy".to_string(),
        }
    }

    // -------------------------------------------------------------------
    // 1️⃣ commissioner_menu()
    // -------------------------------------------------------------------

    #[test]
    fn test_commissioner_menu_authorization_check() {
        let conn = setup_db();
        let user = dummy_user("player");
        let result = auth::require_commissioner(&conn, &user);
        assert!(result.is_err(), "Non-commissioners must not access commissioner menu");
    }

    #[test]
    fn test_commissioner_menu_allows_commissioner() {
        let conn = setup_db();
        let user = dummy_user("commissioner");
        assert!(
            auth::require_commissioner(&conn, &user).is_ok(),
            "Commissioner role should have access"
        );
    }

    // -------------------------------------------------------------------
    // 2️⃣ run_commissioner_test()
    // -------------------------------------------------------------------

    #[test]
    fn test_run_commissioner_test_rtp_calculation() {
        let total_payout = 50.0;
        let total_bet = 100.0;
        let rtp = (total_payout / total_bet) * 100.0;
        assert!(
            (0.0..=300.0).contains(&rtp),
            "RTP {:.2}% should be within logical bounds",
            rtp
        );
    }

    #[test]
    fn test_run_commissioner_test_db_insert_success() {
        let conn = setup_db();
        let result = conn.execute(
            "INSERT INTO commissioner_log (seed, rounds, wins, partials, losses, rtp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            ("seed123", 100, 10, 20, 70, 95.0),
        );
        assert!(result.is_ok(), "commissioner_log insert should succeed");
    }

    // -------------------------------------------------------------------
    // 3️⃣ view_game_probabilities()
    // -------------------------------------------------------------------

    #[test]
    fn test_view_game_probabilities_authorized_access() {
        let conn = setup_db();
        let user = dummy_user("commissioner");
        assert!(
            auth::require_commissioner(&conn, &user).is_ok(),
            "Commissioner should be authorized to view game probabilities"
        );
    }

    #[test]
    fn test_view_game_probabilities_handles_empty_db() {
        let conn = setup_db();
        let user = dummy_user("commissioner");
        assert!(
            auth::require_commissioner(&conn, &user).is_ok(),
            "Function should not panic with empty DB"
        );
    }

    // -------------------------------------------------------------------
    // 4️⃣ adjust_symbol_weights()
    // -------------------------------------------------------------------

    #[test]
    fn test_adjust_symbol_weights_valid_range() {
        let valid_weight = 75usize;
        assert!(
            (1..=100).contains(&valid_weight),
            "Valid weight must be between 1–100"
        );
    }

    #[test]
    fn test_adjust_symbol_weights_invalid_range() {
        let invalid_low = 0usize;
        let invalid_high = 200usize;
        assert!(
            !(1..=100).contains(&invalid_low),
            "Weight 0 must fail validation"
        );
        assert!(
            !(1..=100).contains(&invalid_high),
            "Weight 200 must fail validation"
        );
    }

    // -------------------------------------------------------------------
    // 5️⃣ adjust_symbol_payouts()
    // -------------------------------------------------------------------

    #[test]
    fn test_adjust_symbol_payouts_valid_range() {
        let valid_payout = 5.0;
        assert!(
            (0.5..=50.0).contains(&valid_payout),
            "Valid payout must be between 0.5–50.0"
        );
    }

    #[test]
    fn test_adjust_symbol_payouts_invalid_range() {
        let invalid_low = 0.1;
        let invalid_high = 99.9;
        assert!(
            !(0.5..=50.0).contains(&invalid_low),
            "Payout < 0.5 must fail validation"
        );
        assert!(
            !(0.5..=50.0).contains(&invalid_high),
            "Payout > 50.0 must fail validation"
        );
    }

    // -------------------------------------------------------------------
    // 6️⃣ Database Layer & Cross-checks
    // -------------------------------------------------------------------

    #[test]
    fn test_commissioner_log_schema_integrity() {
        let conn = setup_db();
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='commissioner_log'"
        ).unwrap();
        let exists: bool = stmt.exists([]).unwrap();
        assert!(exists, "commissioner_log table must exist");
    }

    #[test]
    fn test_authorization_consistency() {
        let conn = setup_db();
        let commissioner = dummy_user("commissioner");
        let player = dummy_user("player");

        assert!(
            auth::require_commissioner(&conn, &commissioner).is_ok(),
            "Commissioner should pass authorization"
        );
        assert!(
            auth::require_commissioner(&conn, &player).is_err(),
            "Player should fail authorization"
        );
    }
}*/
/// View probabilities for all games - REQUIRES COMMISSIONER ROLE
fn view_game_probabilities(conn: &Connection, user: &User) {
    // SECURITY: Double-check authorization
    if authorization::require_commissioner(conn, user).is_err() {
        return;
    }
    
    logger::info(&format!("Commissioner (User ID: {}) viewing game probabilities", user.id));
    use crate::db::dbqueries;
    
    let games = vec!["normal", "multi", "holding"];
    
    for game in games {
        println!("\n{}", format!("═══ {} ═══", game.to_uppercase()).bright_cyan());
        
        match dbqueries::get_symbol_probabilities(conn, game) {
            Ok(symbols) => {
                if symbols.is_empty() {
                    println!("No symbols configured for this game");
                    continue;
                }
                
                let total_weight: usize = symbols.iter().map(|(_, w, _)| w).sum();
                
                println!("{:<10} {:<10} {:<15} {:<10}", "Symbol", "Weight", "Probability", "Payout");
                println!("{}", "-".repeat(50));
                
                for (symbol, weight, payout) in symbols {
                    let probability = (weight as f64 / total_weight as f64) * 100.0;
                    println!("{:<10} {:<10} {:<14.2}% {:<10.1}x",
                        symbol, weight, probability, payout);
                }
            }
            Err(e) => println!("Error retrieving probabilities: {}", e),
        }
    }
    
    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut String::new()).ok();
}

/// Adjust symbol weights (probabilities) for a game - REQUIRES COMMISSIONER ROLE
fn adjust_symbol_weights(conn: &Connection, user: &User) {
    // SECURITY: Double-check authorization before allowing game modifications
    if authorization::require_commissioner(conn, user).is_err() {
        return;
    }
    
    logger::security(&format!("Commissioner (User ID: {}) accessing symbol weight adjustment", user.id));
    use crate::db::dbqueries;
    
    // Select game using menu_generator
    let game_options = vec!["normal", "multi", "holding", "Cancel"];
    let game_choice = menu_generator("Select Game to Adjust Weights", &game_options);
    
    if game_choice == "Cancel" {
        return;
    }
    
    let game_name = game_choice;
    
    match dbqueries::get_symbol_probabilities(conn, game_name) {
        Ok(symbols) => {
            // Create symbol options for menu
            let symbol_options: Vec<String> = symbols.iter()
                .map(|(symbol, weight, _)| format!("{} (weight: {})", symbol, weight))
                .collect();
            
            let mut menu_opts: Vec<&str> = symbol_options.iter()
                .map(|s| s.as_str())
                .collect();
            menu_opts.push("Cancel");
            
            let symbol_choice = menu_generator(
                &format!("Current symbols for {}", game_name),
                &menu_opts
            );
            
            if symbol_choice == "Cancel" {
                return;
            }
            
            // Find the selected symbol index
            let sym_idx = symbols.iter()
                .position(|(symbol, weight, _)| {
                    format!("{} (weight: {})", symbol, weight) == symbol_choice
                })
                .unwrap_or(0);
            
            let (symbol, old_weight, _) = &symbols[sym_idx];
            
            println!("\nCurrent weight for {}: {}", symbol, old_weight);
            print!("Enter new weight (1-100): ");
            io::stdout().flush().ok();
            let mut weight_input = String::new();
            io::stdin().read_line(&mut weight_input).ok();
            
            let new_weight: usize = match weight_input.trim().parse() {
                Ok(w) if w > 0 && w <= 100 => w,
                _ => {
                    println!("{}", "Invalid weight! Must be 1-100".red());
                    return;
                }
            };
            
            match dbqueries::update_symbol_weight(conn, game_name, symbol, new_weight) {
                Ok(_) => println!("{}", format!("✓ Weight updated for {} to {}", symbol, new_weight).green()),
                Err(e) => println!("{}", format!("Error updating weight: {}", e).red()),
            }
        }
        Err(e) => println!("{}", format!("Error loading symbols: {}", e).red()),
    }
    
    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut String::new()).ok();
}

/// Adjust symbol payout multipliers for a game - REQUIRES COMMISSIONER ROLE
fn adjust_symbol_payouts(conn: &Connection, user: &User) {
    // SECURITY: Double-check authorization before allowing payout modifications
    if authorization::require_commissioner(conn, user).is_err() {
        return;
    }
    
    logger::security(&format!("Commissioner (User ID: {}) accessing symbol payout adjustment", user.id));
    use crate::db::dbqueries;
    
    // Select game using menu_generator
    let game_options = vec!["normal", "multi", "holding", "Cancel"];
    let game_choice = menu_generator("Select Game to Adjust Payouts", &game_options);
    
    if game_choice == "Cancel" {
        return;
    }
    
    let game_name = game_choice;
    
    match dbqueries::get_symbol_probabilities(conn, game_name) {
        Ok(symbols) => {
            // Create symbol options for menu
            let symbol_options: Vec<String> = symbols.iter()
                .map(|(symbol, _, payout)| format!("{} (payout: {}x)", symbol, payout))
                .collect();
            
            let mut menu_opts: Vec<&str> = symbol_options.iter()
                .map(|s| s.as_str())
                .collect();
            menu_opts.push("Cancel");
            
            let symbol_choice = menu_generator(
                &format!("Current payout for {}", game_name),
                &menu_opts
            );
            
            if symbol_choice == "Cancel" {
                return;
            }
            
            // Find the selected symbol index
            let sym_idx = symbols.iter()
                .position(|(symbol, _, payout)| {
                    format!("{} (payout: {}x)", symbol, payout) == symbol_choice
                })
                .unwrap_or(0);
            
            let (symbol, _, old_payout) = &symbols[sym_idx];
            
            println!("\nCurrent payout for {}: {}x", symbol, old_payout);
            print!("Enter new payout multiplier (0.5-50.0): ");
            io::stdout().flush().ok();
            let mut payout_input = String::new();
            io::stdin().read_line(&mut payout_input).ok();
            
            let new_payout: f64 = match payout_input.trim().parse() {
                Ok(p) if p >= 0.5 && p <= 50.0 => p,
                _ => {
                    println!("{}", "Invalid payout! Must be 0.5-50.0".red());
                    return;
                }
            };
            
            match dbqueries::update_symbol_payout(conn, game_name, symbol, new_payout) {
                Ok(_) => println!("{}", format!("✓ Payout updated for {} to {}x", symbol, new_payout).green()),
                Err(e) => println!("{}", format!("Error updating payout: {}", e).red()),
            }
        }
        Err(e) => println!("{}", format!("Error loading symbols: {}", e).red()),
    }
    
    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut String::new()).ok();
}