use rand_chacha::ChaCha20Rng;
use rand::{SeedableRng, Rng};
use colored::*;
use std::io;
use std::io::Write;
use rusqlite::Connection;

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

/// automated fairness test for X rounds
fn run_commissioner_test(conn: &Connection) {
    println!("\nEnter number of rounds to test:");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let rounds: u32 = input.trim().parse().unwrap_or(100);

    println!("Enter RNG seed (any number): ");
    io::stdout().flush().unwrap();
    let mut seed_input = String::new();
    io::stdin().read_line(&mut seed_input).unwrap();
    let seed: u64 = seed_input.trim().parse().unwrap_or(20251027);

    println!("\nRunning {} rounds with seed {} ...", rounds, seed);

    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];

    let mut wins = 0;
    let mut partials = 0;
    let mut losses = 0;
    let mut total_bet = 0;
    let mut total_payout = 0;

    for _ in 0..rounds {
        let slot1 = symbols[rng.gen_range(0..symbols.len())];
        let slot2 = symbols[rng.gen_range(0..symbols.len())];
        let slot3 = symbols[rng.gen_range(0..symbols.len())];

        let bet = 1;
        total_bet += bet;

        if slot1 == slot2 && slot2 == slot3 {
            wins += 1;
            total_payout += 3 * bet;
        } else if slot1 == slot2 || slot2 == slot3 || slot1 == slot3 {
            partials += 1;
            total_payout += 2 * bet;
        } else {
            losses += 1;
        }
    }

    let rtp = (total_payout as f64 / total_bet as f64) * 100.0;

    println!("\n{}", "🎰 Test Results 🎰".bright_yellow().bold());
    println!("Total rounds: {}", rounds);
    println!("Wins (3 match): {}", wins);
    println!("Two-symbol matches: {}", partials);
    println!("Losses: {}", losses);
    println!("Total Bet: ${}", total_bet);
    println!("Total Payout: ${}", total_payout);
    println!("RTP (Return To Player): {:.2}%", rtp);
    println!("RNG Seed Used: {}", seed);

    // Store test summary in DB
    conn.execute(
        "CREATE TABLE IF NOT EXISTS commissioner_log (
            id INTEGER PRIMARY KEY,
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
        "INSERT INTO commissioner_log (seed, rounds, wins, partials, losses, rtp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (seed, rounds, wins, partials, losses, rtp),
    ).unwrap();

    println!("{}", " Test results stored in commissioner_log table.".green().bold());
}
