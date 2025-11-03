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
    // Show options to technician
    // query all games
    let games_data = dbqueries::get_games(conn).unwrap();
    let mut all_games: Vec<&str> = games_data
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();
    // add exit
    all_games.push("Cancel");
    // let game_options = vec!["normal", "multi", "holding", "Cancel"];
    let game_choice = menu_generator("Select Game to Test", &all_games);
    
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

    let mut rng = CasinoRng::new();

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

                // Generate 5x5 grid
                let mut grid = [[' '; 5]; 5];
                for i in 0..5 {
                    for j in 0..5 {
                        let symbol = rng.weighted_choice(&weighted_symbols).unwrap();
                        grid[i][j] = symbol.chars().next().unwrap();
                    }
                }

                // Check for wins (simplified: check rows, columns, diagonals)
                let mut has_horizontal_win = false;
                let mut has_four_corner_win = false;
                let mut line_wins = 0;

                // Check rows
                for (_row_idx, row) in grid.iter().enumerate() {
                    if row.iter().all(|&s| s == row[0]) {
                        line_wins += 1;
                        has_horizontal_win = true; // Any row counts as horizontal
                    }
                }

                // Check columns
                for col in 0..5 {
                    if (0..5).all(|row| grid[row][col] == grid[0][col]) {
                        line_wins += 1;
                    }
                }

                // Check diagonals
                if (0..5).all(|i| grid[i][i] == grid[0][0]) {
                    line_wins += 1;
                }
                if (0..5).all(|i| grid[i][4-i] == grid[0][4]) {
                    line_wins += 1;
                }

                // Check four corners
                if grid[0][0] == grid[0][4] && grid[0][0] == grid[4][0] && grid[0][0] == grid[4][4] {
                    has_four_corner_win = true;
                }

                if line_wins > 0 {
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
            // Holding: 5 symbols, match 3/4/5 of a kind
            for _ in 0..rounds {
                let bet = 1.0;
                total_bet += bet;

                // Generate 5 symbols
                let reels: Vec<&str> = (0..5)
                    .map(|_| *rng.weighted_choice(&weighted_symbols).unwrap())
                    .collect();

                // Count occurrences
                let mut counts = std::collections::HashMap::new();
                for &symbol in &reels {
                    *counts.entry(symbol).or_insert(0) += 1;
                }

                let max_count = counts.values().copied().max().unwrap_or(0);
                
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
                        5 => base_multiplier * 5.0 * bet,
                        4 => base_multiplier * 2.5 * bet,
                        3 => base_multiplier * bet,
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
    // Drop old table if it exists (to update schema)
    conn.execute("DROP TABLE IF EXISTS commissioner_log", []).ok();
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS commissioner_log (
            id INTEGER PRIMARY KEY,
            game_name TEXT,
            rounds INTEGER,
            wins INTEGER,
            partials INTEGER,
            losses INTEGER,
            rtp REAL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).unwrap();

    conn.execute(
        "INSERT INTO commissioner_log (game_name, rounds, wins, partials, losses, rtp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (game_name, rounds, wins, partials, losses, rtp),
    ).unwrap();

    println!("\n{}", "✓ Test results stored in commissioner_log table.".green().bold());
    
    println!("\nPress Enter to continue...");
    io::stdin().read_line(&mut String::new()).ok();
}

// // ---------------------------------------------------------------------------
// // 🧪 Commissioner Boundary and Fairness Tests
// // ---------------------------------------------------------------------------
// #[cfg(test)]
// mod tests {
//     // Helper: create in-memory DB for isolated tests
//     fn setup_db() -> Connection {
//         let conn = Connection::open_in_memory().unwrap();
//         conn.execute(
//             "CREATE TABLE IF NOT EXISTS commissioner_log (
//                 id INTEGER PRIMARY KEY,
//                 seed INTEGER,
//                 rounds INTEGER,
//                 wins INTEGER,
//                 partials INTEGER,
//                 losses INTEGER,
//                 rtp REAL,
//                 timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
//             )",
//             [],
//         ).unwrap();
//         conn
//     }

//     // 1️⃣ Boundary test: 0 rounds (should be invalid)
//     #[test]
//     fn test_zero_rounds() {
//         let conn = setup_db();
//         // simulate zero rounds input
//         let rounds: u32 = 0;
//         assert_eq!(rounds, 0, "Boundary test failed: rounds cannot be 0");
//     }

//     // 2️⃣ Boundary test: 1 round (minimal valid)
//     #[test]
//     fn test_single_round_rtp() {
//         let conn = setup_db();
//         let seed = 42;
//         let mut rng = ChaCha20Rng::seed_from_u64(seed);
//         let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];
//         let slot1 = symbols[rng.gen_range(0..symbols.len())];
//         let slot2 = symbols[rng.gen_range(0..symbols.len())];
//         let slot3 = symbols[rng.gen_range(0..symbols.len())];

//         let bet = 1;
//         let mut total_payout = 0;

//         if slot1 == slot2 && slot2 == slot3 {
//             total_payout = 3 * bet;
//         } else if slot1 == slot2 || slot2 == slot3 || slot1 == slot3 {
//             total_payout = 2 * bet;
//         }

//         let rtp = (total_payout as f64 / bet as f64) * 100.0;
//         assert!((0.0..=300.0).contains(&rtp), "RTP out of expected range");
//     }

//     // 3️⃣ Boundary test: Large number of rounds (performance & overflow check)
//     #[test]
//     fn test_large_rounds() {
//         let conn = setup_db();
//         let rounds = 10_000;
//         let seed = 123456;
//         let mut rng = ChaCha20Rng::seed_from_u64(seed);
//         let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];

//         let mut wins = 0;
//         let mut partials = 0;
//         let mut losses = 0;

//         for _ in 0..rounds {
//             let s1 = symbols[rng.gen_range(0..symbols.len())];
//             let s2 = symbols[rng.gen_range(0..symbols.len())];
//             let s3 = symbols[rng.gen_range(0..symbols.len())];
//             if s1 == s2 && s2 == s3 {
//                 wins += 1;
//             } else if s1 == s2 || s2 == s3 || s1 == s3 {
//                 partials += 1;
//             } else {
//                 losses += 1;
//             }
//         }

//         assert_eq!(wins + partials + losses, rounds, "Round total mismatch");
//     }

//     // 4️⃣ Fairness test: RTP within a reasonable range
//     #[test]
//     fn test_fairness_rtp_range() {
//         let conn = setup_db();
//         let rounds = 1000;
//         let seed = 20251027;
//         let mut rng = ChaCha20Rng::seed_from_u64(seed);
//         let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];
//         let mut total_bet = 0;
//         let mut total_payout = 0;

//         for _ in 0..rounds {
//             let s1 = symbols[rng.gen_range(0..symbols.len())];
//             let s2 = symbols[rng.gen_range(0..symbols.len())];
//             let s3 = symbols[rng.gen_range(0..symbols.len())];
//             let bet = 1;
//             total_bet += bet;

//             if s1 == s2 && s2 == s3 {
//                 total_payout += 3 * bet;
//             } else if s1 == s2 || s2 == s3 || s1 == s3 {
//                 total_payout += 2 * bet;
//             }
//         }

//         let rtp = (total_payout as f64 / total_bet as f64) * 100.0;
//         assert!(
//             (70.0..=110.0).contains(&rtp),
//             "RTP {:.2}% is outside fair range (70–110%)",
//             rtp
//         );
//     }

//     // 5️⃣ Data integrity test: database insert works
//     #[test]
//     fn test_commissioner_log_insert() {
//         let conn = setup_db();
//         let result = conn.execute(
//             "INSERT INTO commissioner_log (seed, rounds, wins, partials, losses, rtp)
//              VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
//             (12345, 100, 10, 20, 70, 95.0),
//         );
//         assert!(result.is_ok(), "Database insert failed");
//     }
// }
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
