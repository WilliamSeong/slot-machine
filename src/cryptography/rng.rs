use argon2::password_hash::rand_core::{OsRng, RngCore};
use crate::logger::logger;
// CRITICAL: clean unneceseary files
// CRITICAL: clean chatgpt silly mistakes
// CRITICAL: chatgpt generated read and fix it
pub struct CasinoRng {
    // OS-provided cryptographically secure RNG
    rng: OsRng,
}

impl CasinoRng {
    // Create a new cryptographically secure random number generator.
    pub fn new() -> Self {
        logger::info("Cryptographically secure RNG initialized");
        CasinoRng { rng: OsRng }
    }

    // Generate a random number in the range [min, max).
    pub fn gen_range(&mut self, min: usize, max: usize) -> usize {
        assert!(min < max, "min must be less than max");
        
        let range = max - min;
        
        // Use rejection sampling for uniform distribution
        // This ensures no bias in the random numbers
        loop {
            let mut bytes = [0u8; 8];
            self.rng.fill_bytes(&mut bytes);
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


    pub fn gen_bool(&mut self) -> bool {
        let mut byte = [0u8; 1];
        self.rng.fill_bytes(&mut byte);
        byte[0] & 1 == 1
    }

    // Generate a random floating point number in the range [0.0, 1.0).
    pub fn gen_f64(&mut self) -> f64 {
        let mut bytes = [0u8; 8];
        self.rng.fill_bytes(&mut bytes);
        let random_value = u64::from_le_bytes(bytes);
        
        // Convert to [0.0, 1.0) range
        // Use 53 bits of precision (standard for f64)
        (random_value >> 11) as f64 / (1u64 << 53) as f64
    }

    // Shuffle a slice in place using the Fisher-Yates algorithm.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let len = slice.len();
        for i in 0..len {
            let j = self.gen_range(i, len);
            slice.swap(i, j);
        }
    }

    // CRITICAL: do math and adjust this
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
// CRITICAL: examine it
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_gen_range() {
//         let mut rng = CasinoRng::new();
//         for _ in 0..100 {
//             let val = rng.gen_range(0, 10);
//             assert!(val < 10);
//         }
//     }

//     #[test]
//     fn test_weighted_choice() {
//         let mut rng = CasinoRng::new();
//         let items = [("a", 10), ("b", 20), ("c", 30)];
        
//         for _ in 0..100 {
//             let choice = rng.weighted_choice(&items);
//             assert!(choice.is_some());
//         }
//     }
// }

