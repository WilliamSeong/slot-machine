use rand::Rng;

/// Centralized win-checking logic shared by all games.
/// Returns (is_full_win, is_partial_win)
pub fn is_win(symbols: &[&str]) -> (bool, bool) {
    if symbols.len() < 3 {
        return (false, false);
    }

    // 3-reel games
    if symbols.len() == 3 {
        if symbols[0] == symbols[1] && symbols[1] == symbols[2] {
            (true, false)
        } else if symbols[0] == symbols[1] || symbols[1] == symbols[2] || symbols[0] == symbols[2] {
            (false, true)
        } else {
            (false, false)
        }
    } else {
        // ≥5-reel games (e.g., Hold 5×3)
        let mut freq = std::collections::HashMap::new();
        for &s in symbols {
            *freq.entry(s).or_insert(0) += 1;
        }
        let max = *freq.values().max().unwrap_or(&1);
        match max {
            5 => (true, false), // jackpot
            4 | 3 => (false, true),
            _ => (false, false),
        }
    }
}

/// Default probability expectations (used by Commissioner)
pub fn default_probabilities(game: &str) -> (f64, f64) {
    match game {
        "Hold5x3" => (0.15, 0.25), // win%, partial%
        "Classic3x3" => (0.10, 0.20),
        _ => (0.10, 0.20),
    }
}

/// Helper for symbol generation
pub fn spin_symbols<'a, R: Rng + ?Sized>(rng: &mut R, symbols: &'a [&'a str], reels: usize) -> Vec<&'a str> {
    (0..reels).map(|_| symbols[rng.gen_range(0..symbols.len())]).collect()
}
