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
    
    // Display status of each file
    print!("Checking configuration file (.env)... ");
    if env_exists {
        println!("{}", "✓ FOUND".green());
    } else {
        println!("{}", "✗ MISSING".red());
    }
    
    print!("Checking log file (casino_logs.log)... ");
    if log_exists {
        println!("{}", "✓ FOUND".green());
    } else {
        println!("{}", "✗ MISSING".red());
    }
    
    print!("Checking database file (casino.db)... ");
    if db_exists {
        println!("{}", "✓ FOUND".green());
    } else {
        println!("{}", "✗ MISSING".red());
    }
    
    println!();
    
    // If ANY file is missing, reinitialize ALL from scratch
    let all_files_exist = env_exists && db_exists && log_exists;
    
    if !all_files_exist {
        println!("{}", "⚠️  INCOMPLETE SYSTEM DETECTED ⚠️".yellow().bold());
        println!("One or more critical files are missing.");
        println!("Reinitializing entire system from scratch...");
        println!();
        
        // Delete existing files to ensure clean state
        if env_exists {
            print!("  → Removing existing .env file... ");
            if let Err(e) = std::fs::remove_file(".env") {
                println!("{}", format!("Failed: {}", e).red());
            } else {
                println!("{}", "✓".green());
            }
        }
        
        if db_exists {
            print!("  → Removing existing database file... ");
            if let Err(e) = std::fs::remove_file("casino.db") {
                println!("{}", format!("Failed: {}", e).red());
            } else {
                println!("{}", "✓".green());
            }
        }
        
        if log_exists {
            print!("  → Removing existing log file... ");
            if let Err(e) = std::fs::remove_file("casino_logs.log") {
                println!("{}", format!("Failed: {}", e).red());
            } else {
                println!("{}", "✓".green());
            }
        }
        
        println!();
        println!("{}", "Creating fresh system files...".bright_cyan().bold());
        println!();
    } else {
        println!("{}", "✓ All system files present".green().bold());
        println!("Continuing with existing configuration...");
        println!();
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
    print!("Initializing encryption system... ");
    cryptography::crypto::initialize_encryption_key();
    logger::logger::info("Database encryption initialized");
    println!("{}", "✓ DONE".green());
    
    println!();
    
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
    
    if is_fresh_init {
        println!("  → Database created");
    } else {
        println!("Database connection established");
    }

    // Allows casino.db to utilize foreign_keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    logger::logger::info("Foreign keys enabled");

    // Initializes db with all the tables (users, games, user_statistics) and adds records if needed
    if is_fresh_init {
        print!("  → Initializing database schema... ");
    } else {
        print!("Initializing database tables and default data... ");
    }
    
    db::dbinitialize::initialize_dbs(&conn)?;
    logger::logger::info("Database tables initialized");
    println!("{}", "✓ DONE".green());
    
    println!();
    println!("{}", "═══════════════════════════════════════════════".bright_cyan().bold());
    println!("{}", "   ✓ System Ready!".bright_green().bold());
    println!("{}", "═══════════════════════════════════════════════".bright_cyan().bold());
    println!();
    
    // If fresh initialization, inform user about credentials
    if is_fresh_init {
        // Check if .env has admin credentials
        if std::env::var("CASINO_TECH_PASSWORD").is_ok() || std::env::var("CASINO_COMM_PASSWORD").is_ok() {
            println!("{}", "🔐 IMPORTANT: NEW SYSTEM INITIALIZED 🔐".yellow().bold());
            println!();
            println!("  ✓ Fresh database created");
            println!("  ✓ New admin credentials generated");
            println!("  ✓ Log file initialized");
            println!();
            println!("{}", "Admin credentials saved to '.env' file".bright_yellow());
            println!("Please check '.env' for technician and commissioner passwords");
            println!();
            std::thread::sleep(std::time::Duration::from_secs(4));
        }
    }
    
    // Starts the application via the login function
    logger::logger::info("Starting application login flow");
    authentication::auth::login(&conn)
}