use rusqlite::{Connection, Result};

mod interfaces;
mod db;
mod authentication;
mod play;
mod logger;

// Main function, creates and connects to db, casino.db
fn main() -> Result<()> {
    // Initialize logger first thing
    logger::logger::info("Application is starting");
    
    let conn = Connection::open("casino.db")?;
    logger::logger::info("Database connection established");

    // Allows casino.db to utilize foreign_keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    logger::logger::info("Foreign keys enabled");

    // Initializes db with all the tables (users, games, user_statistics) and adds records if needed
    db::dbinitialize::initialize_dbs(&conn)?;
    logger::logger::info("Database is initialized");

    // Starts the application via the login function
    logger::logger::info("starting application login flow");
    authentication::auth::login(&conn)
}