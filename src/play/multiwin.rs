use rand::Rng;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use clearscreen;
use rusqlite::{Connection};
use crate::interfaces::user::User;
use crate::db::dbqueries;
use crate::logger::logger;

use crate::interfaces::menus::menu_generator;

const GRID_SIZE: usize = 5;
const SYMBOLS: [char; 6] = ['🍒', '🍊', '🍋', '🔔', '⭐', '💎'];

type Grid = [[char; GRID_SIZE]; GRID_SIZE];

struct WinCheckResults {
    win_descriptions: Vec<String>,
    has_horizontal_win: bool,
    has_four_corner_win: bool,
}

pub fn multi_win(conn: &Connection, user: &User, bet: f64) -> bool{
    let mut rng = rand::rng();
    println!("--- 🎰 Welcome to the 5x5 Rust Slot Machine! 🎰 ---");

    loop {
        // Check if player has the funds
        if !dbqueries::check_funds(conn, user, bet as f64) {
            logger::warning(&format!("User ID: {} has insufficient funds for bet: ${:.2}", user.id, bet));
            println!("Insufficient funds");
            return true;
        }

        //spinning animation
        run_spin_animation(&mut rng);

        //grid after the animation
        let grid = spin(&mut rng);

        //final result
        clearscreen::clear().expect("Failed to clear screen");
        println!("...And the result!\n");
        print_grid(&grid);

        //check dor wins
        let win_results = check_wins(&grid);

        //show to user for win or lose
        if win_results.win_descriptions.is_empty() {
            println!("\n--- No win this time. Try again! ---");
            println!("Current balance is {}", dbqueries::transaction(conn, user, -(bet)));
            let _ = dbqueries::add_loss(conn, "multi");
            let _ = dbqueries::add_user_loss(conn, user, "multi");
        } else {
            // check for Double Jackpot condition first
            if win_results.has_horizontal_win && win_results.has_four_corner_win {
                println!("\n💥💥💥 DOUBLE JACKPOT! 💥💥💥");
                println!("    > You hit a Horizontal AND Four Corners win!");
                println!("Current balance is {}", dbqueries::transaction(conn, user, bet * 4 as f64));
                let _ = dbqueries::add_win(conn, "multi");
                let _ = dbqueries::add_user_win(conn, user, "multi", bet * 4 as f64);
            } else {
                // print single row or diganol win
                println!("\n🎉 *** JACKPOT! *** 🎉");
                println!("Current balance is {}", dbqueries::transaction(conn, user, bet * 2 as f64));
                let _ = dbqueries::add_win(conn, "multi");
                let _ = dbqueries::add_user_win(conn, user, "multi", bet * 2 as f64);

            }

            //print wins if they won
            for win_line in win_results.win_descriptions {
                println!("    > {}", win_line);
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
fn run_spin_animation(rng: &mut impl Rng) {
    let animation_frames = 12; 
    let spin_delay_ms = 70; 

    for _ in 0..animation_frames {
        clearscreen::clear().expect("Failed to clear screen");
        let temp_grid = spin(rng);
        println!("Spinning...\n");
        print_grid(&temp_grid);
        thread::sleep(Duration::from_millis(spin_delay_ms));
    }
}

//creates 5 by 5 grid
fn spin(rng: &mut impl Rng) -> Grid {
    let mut grid = [[' '; GRID_SIZE]; GRID_SIZE];
    for r in 0..GRID_SIZE {
        for c in 0..GRID_SIZE {
            let symbol_index = rng.random_range(0..SYMBOLS.len());
            grid[r][c] = SYMBOLS[symbol_index];
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

    // check if slot is spinning

    #[test]
    fn test_spin_grid_dimensions() {
        let mut rng = rand::rng();
        let grid = spin(&mut rng);
        assert_eq!(grid.len(), GRID_SIZE, "Grid should have {} rows", GRID_SIZE);
        for row in grid {
            assert_eq!(row.len(), GRID_SIZE, "Each row should have {} columns", GRID_SIZE);
        }
    }

    #[test]
    fn test_spin_grid_symbols() {
        let mut rng = rand::rng();
        let grid = spin(&mut rng);
        let symbols_vec: Vec<char> = SYMBOLS.to_vec();

        for row in grid {
            for symbol in row {
                assert!(
                    symbols_vec.contains(&symbol),
                    "Grid symbol '{}' is not in the SYMBOLS constant", symbol
                );
            }
        }
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
