use rand::Rng;
use std::io;
use std::thread;
use std::time::Duration;

const GRID_SIZE: usize = 5;
const SYMBOLS: [char; 6] = ['🍒', '🍊', '🍋', '🔔', '⭐', '💎'];


type Grid = [[char; GRID_SIZE]; GRID_SIZE];


struct WinCheckResults {
    win_descriptions: Vec<String>,
    has_horizontal_win: bool,
    has_four_corner_win: bool,
}

pub fn multi_win(){
    let mut rng = rand::thread_rng();
    println!("--- 🎰 Welcome to the 5x5 Rust Slot Machine! 🎰 ---");

    loop {
        //Get user input 
        println!("\nPress ENTER to spin (or 'q' to quit)...");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        if input.trim().eq_ignore_ascii_case("q") {
            break; // Exit the game loop
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
        } else {
            // check for Double Jackpot condition first
            if win_results.has_horizontal_win && win_results.has_four_corner_win {
                println!("\n💥💥💥 DOUBLE JACKPOT! 💥💥💥");
                println!("    > You hit a Horizontal AND Four Corners win!");
            } else {
                // print single row or diganol win
                println!("\n🎉 *** JACKPOT! *** 🎉");
            }

            //print wins if they won
            for win_line in win_results.win_descriptions {
                println!("    > {}", win_line);
            }
        }
    }

    println!("Thanks for playing!");
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
            let symbol_index = rng.gen_range(0..SYMBOLS.len());
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
