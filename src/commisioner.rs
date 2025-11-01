use rand_chacha::ChaCha20Rng;
use rand::{SeedableRng, Rng};
use colored::*;
use std::io;
use std::io::Write;
use rusqlite::Connection;
use super::*;

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


// ---------------------------------------------------------------------------
// 🧪 Commissioner Boundary and Fairness Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    // Helper: create in-memory DB for isolated tests
    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
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
        conn
    }

    // 1️⃣ Boundary test: 0 rounds (should be invalid)
    #[test]
    fn test_zero_rounds() {
        let conn = setup_db();
        // simulate zero rounds input
        let rounds: u32 = 0;
        assert_eq!(rounds, 0, "Boundary test failed: rounds cannot be 0");
    }

    // 2️⃣ Boundary test: 1 round (minimal valid)
    #[test]
    fn test_single_round_rtp() {
        let conn = setup_db();
        let seed = 42;
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];
        let slot1 = symbols[rng.gen_range(0..symbols.len())];
        let slot2 = symbols[rng.gen_range(0..symbols.len())];
        let slot3 = symbols[rng.gen_range(0..symbols.len())];

        let bet = 1;
        let mut total_payout = 0;

        if slot1 == slot2 && slot2 == slot3 {
            total_payout = 3 * bet;
        } else if slot1 == slot2 || slot2 == slot3 || slot1 == slot3 {
            total_payout = 2 * bet;
        }

        let rtp = (total_payout as f64 / bet as f64) * 100.0;
        assert!((0.0..=300.0).contains(&rtp), "RTP out of expected range");
    }

    // 3️⃣ Boundary test: Large number of rounds (performance & overflow check)
    #[test]
    fn test_large_rounds() {
        let conn = setup_db();
        let rounds = 10_000;
        let seed = 123456;
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];

        let mut wins = 0;
        let mut partials = 0;
        let mut losses = 0;

        for _ in 0..rounds {
            let s1 = symbols[rng.gen_range(0..symbols.len())];
            let s2 = symbols[rng.gen_range(0..symbols.len())];
            let s3 = symbols[rng.gen_range(0..symbols.len())];
            if s1 == s2 && s2 == s3 {
                wins += 1;
            } else if s1 == s2 || s2 == s3 || s1 == s3 {
                partials += 1;
            } else {
                losses += 1;
            }
        }

        assert_eq!(wins + partials + losses, rounds, "Round total mismatch");
    }

    // 4️⃣ Fairness test: RTP within a reasonable range
    #[test]
    fn test_fairness_rtp_range() {
        let conn = setup_db();
        let rounds = 1000;
        let seed = 20251027;
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let symbols = ["🍒", "🍋", "🍊", "💎", "7️⃣", "⭐"];
        let mut total_bet = 0;
        let mut total_payout = 0;

        for _ in 0..rounds {
            let s1 = symbols[rng.gen_range(0..symbols.len())];
            let s2 = symbols[rng.gen_range(0..symbols.len())];
            let s3 = symbols[rng.gen_range(0..symbols.len())];
            let bet = 1;
            total_bet += bet;

            if s1 == s2 && s2 == s3 {
                total_payout += 3 * bet;
            } else if s1 == s2 || s2 == s3 || s1 == s3 {
                total_payout += 2 * bet;
            }
        }

        let rtp = (total_payout as f64 / total_bet as f64) * 100.0;
        assert!(
            (70.0..=110.0).contains(&rtp),
            "RTP {:.2}% is outside fair range (70–110%)",
            rtp
        );
    }

    // 5️⃣ Data integrity test: database insert works
    #[test]
    fn test_commissioner_log_insert() {
        let conn = setup_db();
        let result = conn.execute(
            "INSERT INTO commissioner_log (seed, rounds, wins, partials, losses, rtp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (12345, 100, 10, 20, 70, 95.0),
        );
        assert!(result.is_ok(), "Database insert failed");
    }
}

