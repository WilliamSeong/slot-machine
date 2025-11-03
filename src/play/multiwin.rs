use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use clearscreen;
use rusqlite::{Connection};
use crate::interfaces::user::User;
use crate::db::dbqueries;
use crate::logger::logger;
use crate::cryptography::rng::CasinoRng;
use colored::*;

use crate::interfaces::menus::menu_generator;
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
            let _ = dbqueries::add_loss(conn, "multi");
            let _ = dbqueries::add_user_loss(conn, user, "multi");
        } else {
            // check for Double Jackpot condition first
            if win_results.has_horizontal_win && win_results.has_four_corner_win {
                let winnings = bet * 4.0;
                
                // DEPOSIT WINNINGS
                let final_balance = dbqueries::transaction(conn, user, winnings);
                
                println!("\n{}", "═══════════════════════════════════════".green().bold());
                println!("{}", "      💥 DOUBLE JACKPOT! 💥            ".green().bold());
                println!("{}", "═══════════════════════════════════════".green().bold());
                println!("\n{}  Horizontal + Four Corners!", "Result:".bright_white().bold());
                println!("{} ${:.2} × 4x = ${:.2}", "Payout:".bright_white().bold(), bet, winnings);
                println!("{} ${:.2}", "Balance:".bright_white().bold(), final_balance);
                println!();
                let _ = dbqueries::add_win(conn, "multi");
                let _ = dbqueries::add_user_win(conn, user, "multi", winnings);
            } else {
                let winnings = bet * 2.0;
                
                // DEPOSIT WINNINGS
                let final_balance = dbqueries::transaction(conn, user, winnings);
                
                println!("\n{}", "═══════════════════════════════════════".green().bold());
                println!("{}", "         🎉 JACKPOT! 🎉                ".green().bold());
                println!("{}", "═══════════════════════════════════════".green().bold());
                println!();
                for win_line in &win_results.win_descriptions {
                    println!("  ✓ {}", win_line.bright_cyan());
                }
                println!("\n{} ${:.2} × 2x = ${:.2}", "Payout:".bright_white().bold(), bet, winnings);
                println!("{} ${:.2}", "Balance:".bright_white().bold(), final_balance);
                println!();
                let _ = dbqueries::add_win(conn, "multi");
                let _ = dbqueries::add_user_win(conn, user, "multi", winnings);
            }
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
// CRITICAL: check logic here 
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
