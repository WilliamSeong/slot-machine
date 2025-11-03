use argon2::password_hash::rand_core::{OsRng, RngCore as ArgonRngCore};
use rand::{RngCore as RandRngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use crate::logger::logger;

enum RngType {
    Secure(OsRng),
    Seeded(ChaCha20Rng),
}

pub struct CasinoRng {
    // OS-provided cryptographically secure RNG or seeded RNG for testing
    rng: RngType,
}

impl CasinoRng {
    // Create a new cryptographically secure random number generator.
    pub fn new() -> Self {
        logger::info("Cryptographically secure RNG initialized");
        CasinoRng { rng: RngType::Secure(OsRng) }
    }
    
    // Create a seeded RNG for reproducible testing (commissioner use only)
    pub fn seeded(seed: u64) -> Self {
        logger::info(&format!("Seeded RNG initialized with seed: {}", seed));
        CasinoRng { rng: RngType::Seeded(ChaCha20Rng::seed_from_u64(seed)) }
    }

    // Generate a random number in the range [min, max).
    pub fn gen_range(&mut self, min: usize, max: usize) -> usize {
        assert!(min < max, "min must be less than max");
        
        let range = max - min;
        
        // Use rejection sampling for uniform distribution
        // This ensures no bias in the random numbers
        loop {
            let mut bytes = [0u8; 8];
            match &mut self.rng {
                RngType::Secure(rng) => ArgonRngCore::fill_bytes(rng, &mut bytes),
                RngType::Seeded(rng) => RandRngCore::fill_bytes(rng, &mut bytes),
            }
            let random_value = u64::from_le_bytes(bytes);
            
            // Use the random value if it's within our usable range
            // This prevents modulo bias
            let max_usable = u64::MAX - (u64::MAX % range as u64);
            if random_value < max_usable {
                return min + (random_value % range as u64) as usize;
            }
            // If outside usable range, try again (rejection sampling)
        }
    }

    // Select a random element from a weighted list.
    pub fn weighted_choice<'a, T>(&mut self, weights: &'a [(T, usize)]) -> Option<&'a T> {
        if weights.is_empty() {
            return None;
        }

        // Calculate total weight
        let total_weight: usize = weights.iter().map(|(_, w)| w).sum();
        
        if total_weight == 0 {
            return None;
        }

        // Generate random number in range [0, total_weight)
        let mut random = self.gen_range(0, total_weight);

        // Find the selected item
        for (item, weight) in weights.iter() {
            if random < *weight {
                return Some(item);
            }
            random -= weight;
        }

        // Should never reach here, but return last item as fallback
        weights.last().map(|(item, _)| item)
    }
}

// Default implementation for CasinoRng.
impl Default for CasinoRng {
    fn default() -> Self {
        Self::new()
    }
}

