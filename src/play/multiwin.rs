use std::thread;
use std::time::Duration;
use clearscreen;
use rusqlite::{Connection};
use crate::interfaces::user::User;
use crate::db::dbqueries;
use crate::logger::logger;
use crate::cryptography::rng::CasinoRng;
use colored::*;

use crate::interfaces::menus;

// Display payout table to user before playing
fn display_payout_table(symbol_probs: &[(String, usize, f64)], bet: f64) {
    // Calculate average multiplier
    let base_multiplier: f64 = symbol_probs.iter()
        .map(|(_, _, mult)| mult)
        .sum::<f64>() / symbol_probs.len() as f64;
    
    let single_win = bet * base_multiplier;
    let double_jackpot = bet * base_multiplier * 2.0;
    
    menus::print_box_top(50);
    menus::print_box_line("💰 PAYOUT TABLE 💰", 48);
    menus::print_box_line("Match any ROW, COLUMN, or DIAGONAL:", 50);
    menus::print_box_line(&format!("  Regular Win:     ${:<6.2} ({:.1}x)", single_win, base_multiplier), 50);
    menus::print_box_separator(50);
    menus::print_box_line("Match ANY ROW + FOUR CORNERS:", 50);
    menus::print_box_line(&format!("  Double Jackpot:  ${:<6.2} ({:.1}x)", double_jackpot, base_multiplier * 2.0), 50);
    menus::print_box_separator(50);
    menus::print_box_line("Symbols in play:", 50);
    
    // Calculate total weight for probability display
    let total_weight: usize = symbol_probs.iter().map(|(_, w, _)| w).sum();
    
    for (symbol, weight, payout) in symbol_probs {
        let probability = (*weight as f64 / total_weight as f64) * 100.0;

        menus::print_box_line(&format!("{} - {:.1}x multiplier [{:.1}% chance]", 
            symbol, payout, probability), 49);
    }
    
    menus::print_box_bottom(50);
    println!();
}

// CRITICAL: check grid size is used or not and adjust and fix it
// CRITICAL: implement rng here
const GRID_SIZE: usize = 5;

type Grid = [[char; GRID_SIZE]; GRID_SIZE];

struct WinCheckResults {
    win_descriptions: Vec<String>,
    has_horizontal_win: bool,
    has_four_corner_win: bool,
}

pub fn multi_win(conn: &Connection, user: &User, bet: f64) -> bool{
    // Load symbol probabilities from database
    let symbol_probs = match dbqueries::get_symbol_probabilities(conn, "multi") {
        Ok(probs) => probs,
        Err(e) => {
            logger::error(&format!("Failed to load symbol probabilities: {}", e));
            println!("{}", "Error loading game configuration".red());
            return true;
        }
    };
    
    // Extract symbols for grid
    let symbols: Vec<char> = symbol_probs.iter()
        .map(|(sym, _, _)| sym.chars().next().unwrap())
        .collect();
    
    let mut rng = CasinoRng::new();
    
    println!("\n{}", "═══ 🎰 Welcome to 5x5 Multi-Win Slots! 🎰 ═══".bright_yellow().bold());
    println!("{}", "Win by matching any row, column, or diagonal!".bright_cyan());
    println!("{} ${:.2}\n", "Your bet:".bright_white().bold(), bet);
    

    loop {
        // Check if player has the funds
        if !dbqueries::check_funds(conn, user, bet as f64) {
            logger::warning(&format!("User ID: {} has insufficient funds for bet: ${:.2}", user.id, bet));
            println!("{}", "Insufficient funds!".red().bold());
            return true;
        }

        // CHARGE BET FIRST before playing
        logger::transaction(&format!("User ID: {} placing bet of ${:.2} for multi-win slots", user.id, bet));
        let balance_after_bet = dbqueries::transaction(conn, user, -bet);
        
        if balance_after_bet < 0.0 {
            println!("{}", "Transaction failed!".red().bold());
            return true;
        }
        
        println!("{}", format!("Bet placed: ${:.2}", bet).yellow());
        println!("{}", format!("Balance: ${:.2}", balance_after_bet).bright_white());

        //spinning animation
        run_spin_animation(&mut rng, &symbols);
        // CRITICAL: double check here  
        //grid after the animation
        let grid = spin(&mut rng, &symbols);

        //final result
        clearscreen::clear().expect("Failed to clear screen");
        println!("...And the result!\n");
        print_grid(&grid);

        // Display payout table to user
        display_payout_table(&symbol_probs, bet);

        //check dor wins
        let win_results = check_wins(&grid);

        //show to user for win or lose
        if win_results.win_descriptions.is_empty() {
            // Loss - bet already deducted, no winnings
            println!("\n{}", "═══════════════════════════════════════".red());
            println!("{}", "           ❌ NO WIN ❌                 ".red().bold());
            println!("{}", "═══════════════════════════════════════".red());
            println!("\n{}  No matching lines found", "Result:".bright_white().bold());
            println!("{} ${:.2}", "Lost:".bright_white().bold(), bet);
            println!("{} ${:.2}", "Balance:".bright_white().bold(), balance_after_bet);
            println!();
            let _ = dbqueries::add_loss(conn, "holding");
            let _ = dbqueries::add_user_loss(conn, user, "multi");
        } else {
            // ADDED BY SUCA
            // Calculate base multiplier from database (average of all symbols)
            let base_multiplier: f64 = symbol_probs.iter()
                .map(|(_, _, mult)| mult)
                .sum::<f64>() / symbol_probs.len() as f64;
            //////////////////////////////////////////////////////////////
            
            // check for Double Jackpot condition first
            if win_results.has_horizontal_win && win_results.has_four_corner_win {
                // REMOVED BY SUCA
                //let winnings = bet * 4.0;
                //////////////////////////////////////////////////////////////
                // ADDED BY SUCA INSTEAD
                let payout_multiplier = base_multiplier * 2.0; // Double jackpot
                let winnings = bet * payout_multiplier;
                //////////////////////////////////////////////////////////////
                
                // DEPOSIT WINNINGS
                let final_balance = dbqueries::transaction(conn, user, winnings);
                
                println!("\n{}", "═══════════════════════════════════════".green().bold());
                println!("{}", "      💥 DOUBLE JACKPOT! 💥            ".green().bold());
                println!("{}", "═══════════════════════════════════════".green().bold());
                println!("\n{}  Horizontal + Four Corners!", "Result:".bright_white().bold());
                println!("{} ${:.2} × {:.1}x = ${:.2}", "Payout:".bright_white().bold(), bet, payout_multiplier, winnings);
                println!("{} ${:.2}", "Balance:".bright_white().bold(), final_balance);
                println!();
                let _ = dbqueries::add_win(conn, "multi");
                let _ = dbqueries::add_user_win(conn, user, "multi", winnings);
            } else {
                let payout_multiplier = base_multiplier;
                let winnings = bet * payout_multiplier;
                
                // DEPOSIT WINNINGS
                let final_balance = dbqueries::transaction(conn, user, winnings);
                
                println!("\n{}", "═══════════════════════════════════════".green().bold());
                println!("{}", "         🎉 JACKPOT! 🎉                ".green().bold());
                println!("{}", "═══════════════════════════════════════".green().bold());
                println!();
                for win_line in &win_results.win_descriptions {
                    println!("  ✓ {}", win_line.bright_cyan());
                }
                println!("\n{} ${:.2} × {:.1}x = ${:.2}", "Payout:".bright_white().bold(), bet, payout_multiplier, winnings);
                println!("{} ${:.2}", "Balance:".bright_white().bold(), final_balance);
                println!();
                let _ = dbqueries::add_win(conn, "holding");
                let _ = dbqueries::add_user_win(conn, user, "holding", winnings);
            }
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

 //spining animation
fn run_spin_animation(rng: &mut CasinoRng, symbols: &[char]) {
    let animation_frames = 12; 
    let spin_delay_ms = 70; 

    for _ in 0..animation_frames {
        clearscreen::clear().expect("Failed to clear screen");
        let temp_grid = spin(rng, symbols);
        println!("Spinning...\n");
        print_grid(&temp_grid);
        thread::sleep(Duration::from_millis(spin_delay_ms));
    }
}
// CRITICAL:  using cryptographically secure RNG
//creates 5 by 5 grid
fn spin(rng: &mut CasinoRng, symbols: &[char]) -> Grid {
    let mut grid = [[' '; GRID_SIZE]; GRID_SIZE];
    for r in 0..GRID_SIZE {
        for c in 0..GRID_SIZE {
            let symbol_index = rng.gen_range(0, symbols.len());
            grid[r][c] = symbols[symbol_index];
        }
    }
    grid
}

//boarder around the slot
fn print_grid(grid: &Grid) {
    let border = "+---------------------+"; 
    println!("{}", border);
    for row in grid {
        print!("| ");
        for &symbol in row {
            print!(" {} |", symbol);
        }
        println!();
    }
    println!("{}", border);
}
//checks all win conditions
fn check_wins(grid: &Grid) -> WinCheckResults {
    let mut wins = Vec::<String>::new(); 
    let mut has_horizontal = false;
    let mut has_four_corner = false;
    
    let last_idx = GRID_SIZE - 1;

    //check if row is a win
    for r in 0..GRID_SIZE {
        let first = grid[r][0];
        if (1..GRID_SIZE).all(|c| grid[r][c] == first) {
            wins.push(format!(
                "Row {} win: {}",
                r + 1,
                first.to_string().repeat(GRID_SIZE)
            ));
            has_horizontal = true; // NEW: Set the horizontal flag
        }
    }

    //check if column is a wil
    for c in 0..GRID_SIZE {
        let first = grid[0][c];
        if (1..GRID_SIZE).all(|r| grid[r][c] == first) {
            wins.push(format!(
                "Column {} win: {}",
                c + 1,
                first.to_string().repeat(GRID_SIZE)
            ));
        }
    }

    //Diagonal win check TL - BR
    let first_diag1 = grid[0][0];
    if (1..GRID_SIZE).all(|i| grid[i][i] == first_diag1) {
        wins.push(format!(
            "Main Diagonal win: {}",
            first_diag1.to_string().repeat(GRID_SIZE)
        ));
    }

    //Diagonal TR - BL
    let first_diag2 = grid[0][last_idx];
    if (1..GRID_SIZE).all(|i| grid[i][last_idx - i] == first_diag2) {
        wins.push(format!(
            "Anti-Diagonal win: {}",
            first_diag2.to_string().repeat(GRID_SIZE)
        ));
    }

    //check 4 corners
    let top_left = grid[0][0];
    let top_right = grid[0][last_idx];
    let bottom_left = grid[last_idx][0];
    let bottom_right = grid[last_idx][last_idx];

    if top_left == top_right && top_left == bottom_left && top_left == bottom_right {
        wins.push(format!("Four Corners win: {}", top_left));
        has_four_corner = true; // NEW: Set the four corner flag
    }

    // win struct
    WinCheckResults {
        win_descriptions: wins,
        has_horizontal_win: has_horizontal,
        has_four_corner_win: has_four_corner,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand; // to use the rand crate 

    // Helper function to create a grid from a 2D vector of chars.
    fn grid_from_vec(vec: Vec<Vec<char>>) -> Grid {
        let mut grid = [[' '; GRID_SIZE]; GRID_SIZE];
        for (r_idx, row) in vec.iter().enumerate().take(GRID_SIZE) {
            for (c_idx, &col) in row.iter().enumerate().take(GRID_SIZE) {
                grid[r_idx][c_idx] = col;
            }
        }
        grid
    }

    // Test to check for wins

    #[test]
    fn test_check_wins_no_win() {
        let grid = grid_from_vec(vec![
            vec!['🍒', '🍊', '🍋', '🔔', '⭐'],
            vec!['🍊', '🍋', '🔔', '⭐', '💎'],
            vec!['🍋', '🔔', '⭐', '💎', '🍒'],
            vec!['🔔', '⭐', '💎', '🍒', '🍊'],
            vec!['⭐', '💎', '🍒', '🍊', '🍋'],
        ]);
        let results = check_wins(&grid);
        assert!(results.win_descriptions.is_empty(), "Should be no wins");
        assert!(!results.has_horizontal_win, "Should not have horizontal win");
        assert!(!results.has_four_corner_win, "Should not have corner win");
    }

    #[test]
    fn test_check_wins_row_win() {
        let grid = grid_from_vec(vec![
            vec!['🍒', '🍊', '🍋', '🔔', '⭐'],
            vec!['💎', '💎', '💎', '💎', '💎'], // Winning row
            vec!['🍋', '🔔', '⭐', '💎', '🍒'],
            vec!['🔔', '⭐', '💎', '🍒', '🍊'],
            vec!['⭐', '💎', '🍒', '🍊', '🍋'],
        ]);
        let results = check_wins(&grid);
        assert_eq!(results.win_descriptions.len(), 1, "Should have 1 win");
        assert!(results.has_horizontal_win, "Should have horizontal win flag set");
        assert!(!results.has_four_corner_win, "Should not have corner win");
        assert!(results.win_descriptions[0].contains("Row 2 win"));
    }

    #[test]
    fn test_check_wins_col_win() {
        let grid = grid_from_vec(vec![
            vec!['🍒', '🍊', '🍋', '🔔', '⭐'],
            vec!['🍊', '🍋', '🍋', '⭐', '💎'],
            vec!['🍋', '🔔', '🍋', '💎', '🍒'],
            vec!['🔔', '⭐', '🍋', '🍒', '🍊'],
            vec!['⭐', '💎', '🍋', '🍊', '🍋'],
        ]);
        let results = check_wins(&grid);
        assert_eq!(results.win_descriptions.len(), 1, "Should have 1 win");
        assert!(!results.has_horizontal_win, "Should not have horizontal win");
        assert!(!results.has_four_corner_win, "Should not have corner win");
        assert!(results.win_descriptions[0].contains("Column 3 win"));
    }

    #[test]
    fn test_check_wins_main_diag_win() {
        let grid = grid_from_vec(vec![
            vec!['⭐', '🍊', '🍋', '🔔', '🍒'],
            vec!['🍊', '⭐', '🔔', '🍒', '💎'],
            vec!['🍋', '🔔', '⭐', '💎', '🍒'],
            vec!['🔔', '💎', '💎', '⭐', '🍊'],
            vec!['💎', '💎', '🍒', '🍊', '⭐'],
        ]);
        let results = check_wins(&grid);
        assert_eq!(results.win_descriptions.len(), 1, "Should have 1 win");
        assert!(!results.has_horizontal_win, "Should not have horizontal win");
        assert!(!results.has_four_corner_win, "Should not have corner win");
        assert!(results.win_descriptions[0].contains("Main Diagonal win"));
    }

    #[test]
    fn test_check_wins_anti_diag_win() {
        let grid = grid_from_vec(vec![
            vec!['🍒', '🍊', '🍋', '🔔', '🔔'],
            vec!['🍊', '🍋', '🔔', '🔔', '💎'],
            vec!['🍋', '🔔', '🔔', '💎', '🍒'],
            vec!['🔔', '🔔', '💎', '🍒', '🍊'],
            vec!['🔔', '💎', '🍒', '🍊', '🍋'],
        ]);
        let results = check_wins(&grid);
        assert_eq!(results.win_descriptions.len(), 1, "Should have 1 win");
        assert!(!results.has_horizontal_win, "Should not have horizontal win");
        assert!(!results.has_four_corner_win, "Should not have corner win");
        assert!(results.win_descriptions[0].contains("Anti-Diagonal win"));
    }

    #[test]
    fn test_check_wins_four_corners_win() {
        let grid = grid_from_vec(vec![
            vec!['💎', '🍊', '🍋', '🔔', '💎'],
            vec!['🍊', '🍋', '🔔', '⭐', '🍒'],
            vec!['🍋', '🔔', '⭐', '💎', '🍒'],
            vec!['🔔', '⭐', '💎', '🍒', '🍊'],
            vec!['💎', '💎', '🍒', '🍊', '💎'],
        ]);
        let results = check_wins(&grid);
        assert_eq!(results.win_descriptions.len(), 1, "Should have 1 win");
        assert!(!results.has_horizontal_win, "Should not have horizontal win");
        assert!(results.has_four_corner_win, "Should have corner win flag set");
        assert!(results.win_descriptions[0].contains("Four Corners win"));
    }

    #[test]
    fn test_check_wins_double_jackpot() {
        let grid = grid_from_vec(vec![
            vec!['💎', '🍊', '🍋', '🔔', '💎'], // Corner wins
            vec!['🍒', '🍒', '🍒', '🍒', '🍒'], // Horizontal win
            vec!['🍋', '🔔', '⭐', '💎', '🍒'],
            vec!['🔔', '⭐', '💎', '🍒', '🍊'],
            vec!['💎', '💎', '🍒', '🍊', '💎'], // Corner win 
        ]);
        let results = check_wins(&grid);
        assert_eq!(results.win_descriptions.len(), 2, "Should have 2 wins");
        assert!(results.has_horizontal_win, "Should have horizontal win flag set");
        assert!(results.has_four_corner_win, "Should have corner win flag set");
        assert!(results.win_descriptions.iter().any(|s| s.contains("Row 2 win")));
        assert!(results.win_descriptions.iter().any(|s| s.contains("Four Corners win")));
    }
}
