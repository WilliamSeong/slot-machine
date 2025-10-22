use anyhow::{anyhow, Result};
use std::fs::File;
use csv::ReaderBuilder;
use argon2::{Argon2, PasswordHash, PasswordVerifier};

pub fn verify_login(db_file: &str, username: &str, password: &str) -> Result<bool> {
    // Open the CSV database file. The ? operator handles the error.
    let file = File::open(db_file)?;

    // Create a CSV reader.
    let mut rdr = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    // Iterate over records in the CSV.
    for result in rdr.records() {
        // Propagate CSV parse errors immediately.
        let record = result?;

        // Skip malformed rows with fewer than 2 columns.
        if record.len() < 2 {
            continue;
        }

        // Check if the username matches.
        if record.get(0).unwrap_or("") == username {
            // Attempt to parse the stored Argon2 password hash.
            let parsed_hash = match PasswordHash::new(record.get(1).unwrap_or("")) {
                Ok(hash) => hash,

                // incaseparsing the password hash fails, create the error with context.
                
                Err(e) => return Err(anyhow!("Failed to parse password hash: {}", e)),
            };

            // Verify the supplied password against the hash.
            let is_match = Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok();

            // Return the result of the verification.
            return Ok(is_match);
        }
    }
    // If the loop completes without finding the username, return false.
    Ok(false)
}
