// DONT TRUST ANYONE, EVEN COMMISSIONER
// Game validation result
pub type GameValidationResult = Result<(), String>;

// Whitelist of allowed game names
const ALLOWED_GAMES: &[&str] = &["normal", "multi", "holding", "wheel of fortune"];

// Whitelist of allowed symbols
const ALLOWED_SYMBOLS: &[&str] = &["🍒", "🍋", "🍊", "🍇", "💎", "7️⃣"];

// Validation limits for game parameters
const MIN_SYMBOL_WEIGHT: usize = 1;
const MAX_SYMBOL_WEIGHT: usize = 100;
const MIN_PAYOUT_MULTIPLIER: f64 = 0.5;
const MAX_PAYOUT_MULTIPLIER: f64 = 50.0;

// Validate game name against whitelist
// SECURITY: Prevents SQL injection and invalid game access
pub fn validate_game_name(game_name: &str) -> GameValidationResult {
    // Check if empty
    if game_name.is_empty() {
        return Err("❌ Game name cannot be empty!".to_string());
    }
    
    // Check if only whitespace
    if game_name.trim().is_empty() {
        return Err("❌ Game name cannot be only whitespace!".to_string());
    }
    
    // Normalize to lowercase for comparison
    let game_name_lower = game_name.trim().to_lowercase();
    
    // Check against whitelist
    if !ALLOWED_GAMES.contains(&game_name_lower.as_str()) {
        return Err(format!(
            "❌ Invalid game name! Allowed games: {}",
            ALLOWED_GAMES.join(", ")
        ));
    }
    
    Ok(())
}

// Validate symbol against whitelist
// SECURITY: Prevents SQL injection and invalid symbol manipulation
pub fn validate_symbol(symbol: &str) -> GameValidationResult {
    // Check if empty
    if symbol.is_empty() {
        return Err("❌ Symbol cannot be empty!".to_string());
    }
    
    // Check if only whitespace
    if symbol.trim().is_empty() {
        return Err("❌ Symbol cannot be only whitespace!".to_string());
    }
    
    // Check against whitelist
    if !ALLOWED_SYMBOLS.contains(&symbol) {
        return Err(format!(
            "❌ Invalid symbol! Allowed symbols: {}",
            ALLOWED_SYMBOLS.join(", ")
        ));
    }
    
    Ok(())
}

// Validate symbol weight
pub fn validate_symbol_weight(weight: usize) -> GameValidationResult {
    if weight < MIN_SYMBOL_WEIGHT {
        return Err(format!("❌ Symbol weight must be at least {}!", MIN_SYMBOL_WEIGHT));
    }
    
    if weight > MAX_SYMBOL_WEIGHT {
        return Err(format!("❌ Symbol weight cannot exceed {}!", MAX_SYMBOL_WEIGHT));
    }
    
    Ok(())
}

// Validate payout multiplier
pub fn validate_payout_multiplier(multiplier: f64) -> GameValidationResult {
    // Check if valid number
    if multiplier.is_nan() || multiplier.is_infinite() {
        return Err("❌ Invalid payout multiplier!".to_string());
    }
    
    if multiplier < MIN_PAYOUT_MULTIPLIER {
        return Err(format!("❌ Payout multiplier must be at least {}!", MIN_PAYOUT_MULTIPLIER));
    }
    
    if multiplier > MAX_PAYOUT_MULTIPLIER {
        return Err(format!("❌ Payout multiplier cannot exceed {}!", MAX_PAYOUT_MULTIPLIER));
    }
    
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_validate_game_name_valid() {
//         assert!(validate_game_name("normal").is_ok());
//         assert!(validate_game_name("multi").is_ok());
//         assert!(validate_game_name("holding").is_ok());
//         assert!(validate_game_name("wheel of fortune").is_ok());
//     }

//     #[test]
//     fn test_validate_game_name_invalid() {
//         assert!(validate_game_name("").is_err());
//         assert!(validate_game_name("   ").is_err());
//         assert!(validate_game_name("invalid_game").is_err());
//         assert!(validate_game_name("DROP TABLE").is_err());
//     }

//     #[test]
//     fn test_validate_symbol_valid() {
//         assert!(validate_symbol("🍒").is_ok());
//         assert!(validate_symbol("💎").is_ok());
//     }

//     #[test]
//     fn test_validate_symbol_invalid() {
//         assert!(validate_symbol("").is_err());
//         assert!(validate_symbol("X").is_err());
//         assert!(validate_symbol("'; DROP TABLE--").is_err());
//     }

//     #[test]
//     fn test_validate_weight() {
//         assert!(validate_symbol_weight(1).is_ok());
//         assert!(validate_symbol_weight(50).is_ok());
//         assert!(validate_symbol_weight(100).is_ok());
//         assert!(validate_symbol_weight(0).is_err());
//         assert!(validate_symbol_weight(101).is_err());
//     }

//     #[test]
//     fn test_validate_multiplier() {
//         assert!(validate_payout_multiplier(1.0).is_ok());
//         assert!(validate_payout_multiplier(10.0).is_ok());
//         assert!(validate_payout_multiplier(0.4).is_err());
//         assert!(validate_payout_multiplier(51.0).is_err());
//     }
// }

