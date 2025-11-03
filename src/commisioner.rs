use rand_chacha::ChaCha20Rng;
use rand::{SeedableRng};
use colored::*;
use std::io::{self, Write};
use rusqlite::Connection;
use crate::game_logic::{is_win, spin_symbols, default_probabilities};

/// Commissioner-only fairness testing menu
pub fn commissioner_menu(conn: &Connection) {
    loop {
        println!("\n{}", "═══ 🧮 Commissioner Testing Mode 🧮 ═══".bright_blue().bold());
        println!("{}. Run fairness test", "1".yellow());
        println!("{}. Back", "2".yellow());
        print!("{} ", "Choose:".green());
        io::stdout().flush().ok();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).ok();

        match choice.trim() {
            "1" => run_commissioner_test(conn),
            "2" => break,
            _ => println!("Invalid input"),
        }
    }
}

/// Automated fairness test for X rounds with RNG seed
fn run_commissioner_test(conn: &Connection) {
    // --- Game selection ---
    println!("\nSelect game to test:");
    println!("1. Classic 3x3");
    println!("2. Hold 5x3");
    print!("Enter choice: ");
    io::stdout().flush().unwrap();

    let mut game_choice = String::new();
    io::stdin().read_line(&mut game_choice).unwrap();
    let (reels, game_name) = match game_choice.trim() {
        "2" => (5, "Hold5x3"),
        _ => (3, "Classic3x3"),
    };

    // --- Rounds input ---
    println!("\nEnter number of rounds to test:");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let rounds: u32 = input.trim().parse().unwrap_or(100);

    // --- RNG seed input ---
    println!("Enter RNG seed (any number): ");
    io::stdout().flush().unwrap();
    let mut seed_input = String::new();
    io::stdin().read_line(&mut seed_input).unwrap();
    let seed: u64 = seed_input.trim().parse().unwrap_or(20251027);

    println!("\nRunning {} ({reels} reels) for {} rounds with seed {} ...",
             game_name, rounds, seed);

    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];

    // --- Expected probabilities for comparison ---
    let (expected_full, expected_partial) = default_probabilities(game_name);
    println!(
        "Expected Win% ≈ {:.1}%, Partial Win% ≈ {:.1}%",
        expected_full * 100.0,
        expected_partial * 100.0
    );

    // --- Counters ---
    let mut wins = 0;
    let mut partials = 0;
    let mut losses = 0;
    let mut total_bet = 0;
    let mut total_payout = 0;

    // --- Main test loop ---
    for _ in 0..rounds {
        let current_spin = spin_symbols(&mut rng, &symbols, reels);
        let (full_win, partial_win) = is_win(&current_spin);

        let bet = 1;
        total_bet += bet;

        if full_win {
            wins += 1;
            total_payout += 3 * bet;
        } else if partial_win {
            partials += 1;
            total_payout += 2 * bet;
        } else {
            losses += 1;
        }
    }

    // --- Results ---
    let rtp = (total_payout as f64 / total_bet as f64) * 100.0;
    println!("\n{}", "🎰 Test Results 🎰".bright_yellow().bold());
    println!("Game Type: {}", game_name);
    println!("Total rounds: {}", rounds);
    println!("Wins (full match): {}", wins);
    println!("Partial wins (two+ match): {}", partials);
    println!("Losses: {}", losses);
    println!("Total Bet: ${}", total_bet);
    println!("Total Payout: ${}", total_payout);
    println!("RTP (Return To Player): {:.2}%", rtp);
    println!("RNG Seed Used: {}", seed);

    if rtp < 70.0 || rtp > 110.0 {
        println!("{}", "⚠️ WARNING: RTP outside fair range!".red().bold());
    }

    // --- Database logging ---
    conn.execute(
        "CREATE TABLE IF NOT EXISTS commissioner_log (
            id INTEGER PRIMARY KEY,
            game_type TEXT,
            seed INTEGER,
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
        "INSERT INTO commissioner_log (game_type, seed, rounds, wins, partials, losses, rtp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        (game_name, seed, rounds, wins, partials, losses, rtp),
    ).unwrap();

    println!("{}", "✅ Test results stored in commissioner_log table.".green().bold());
}
