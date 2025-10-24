use rusqlite::{Connection, Result};

mod interfaces;
mod db;
mod authentication;

fn main() -> Result<()> {
    let conn = Connection::open("casino.db")?;
    
    // Create users table
    db::dbinitialize::initialize_dbs(&conn)?;

    authentication::auth::login(&conn)
}