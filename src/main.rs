use rusqlite::{Connection, Result};
use clearscreen;
use colored::*;
use std::path::Path;

mod interfaces;
mod db;
mod authentication;
mod play;
mod logger;
mod cryptography;

// Check and initialize all required files at startup
// If ANY file is missing, reinitialize ALL from scratch
fn initialize_system() -> bool {
    println!("{}", "═══════════════════════════════════════════════".bright_cyan().bold());
    println!("{}", "   🎰 Casino System Initialization 🎰".bright_yellow().bold());
    println!("{}", "═══════════════════════════════════════════════".bright_cyan().bold());
    println!();
    
    // Check if ALL critical files exist
    let env_exists = Path::new(".env").exists();
    let db_exists = Path::new("casino.db").exists();
    let log_exists = Path::new("casino_logs.log").exists();
    let all_files_exist = env_exists && db_exists && log_exists;
    
    // If ANY file is missing, reinitialize ALL from scratch
    if !all_files_exist {
        println!("{}", "⚠️  Incomplete system detected - reinitializing...".yellow());
        
        // Delete existing files silently to ensure clean state
        if env_exists {
            let _ = std::fs::remove_file(".env");
        }
        if db_exists {
            let _ = std::fs::remove_file("casino.db");
        }
        if log_exists {
            let _ = std::fs::remove_file("casino_logs.log");
        }
    }
    
    // Load .env file (will be created if needed during admin account setup)
    dotenvy::dotenv().ok();
    
    // Initialize logger (will create file if needed)
    logger::logger::info("═══════════════════════════════════════");
    logger::logger::info("Application is starting");
    if !all_files_exist {
        logger::logger::info("System reinitialized from scratch");
    } else {
        logger::logger::info("System continuing with existing files");
    }
    
    // Initialize encryption system
    cryptography::crypto::initialize_encryption_key();
    logger::logger::info("Database encryption initialized");
    
    // Return whether this is a fresh initialization
    !all_files_exist
}

// Main function, creates and connects to db, casino.db
fn main() -> Result<()> {
    clearscreen::clear().expect("Failed clearscreen");
    
    // Initialize all system files and components
    // Returns true if this is a fresh initialization
    let is_fresh_init = initialize_system();
    
    // Connect to database (creates if doesn't exist)
    let conn = Connection::open("casino.db")?;
    logger::logger::info("Database connection established");

    // Allows casino.db to utilize foreign_keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    logger::logger::info("Foreign keys enabled");

    // Initializes db with all the tables (users, games, user_statistics) and adds records if needed
    db::dbinitialize::initialize_dbs(&conn)?;
    logger::logger::info("Database tables initialized");
    
    println!("{}", "✓ System Ready!".bright_green().bold());
    println!();
    
    // If fresh initialization, inform user about credentials
    if is_fresh_init {
        if std::env::var("CASINO_TECH_PASSWORD").is_ok() || std::env::var("CASINO_COMM_PASSWORD").is_ok() {
            println!("{}", "⚠️  NEW CREDENTIALS GENERATED".yellow().bold());
            println!("Check '.env' file for admin passwords");
            println!();
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }
    
    // Starts the application via the login function
    logger::logger::info("Starting application login flow");
    authentication::auth::login(&conn)
}