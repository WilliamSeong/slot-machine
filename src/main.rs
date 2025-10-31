use rusqlite::{Connection, Result};

mod interfaces;
mod db;
mod authentication;
mod play;

// Main function, creates and connects to db, casino.db
fn main() -> Result<()> {
    let conn = Connection::open("casino.db")?;

    // Allows casino.db to utilize foreign_keys
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Initializes db with all the tables (users, games, user_statistics) and adds records if needed
    db::dbinitialize::initialize_dbs(&conn)?;

    // Starts the application via the login function
    authentication::auth::login(&conn)
}