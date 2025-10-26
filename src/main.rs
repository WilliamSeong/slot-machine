use rusqlite::{Connection, Result};

mod interfaces;
mod db;
mod authentication;

fn main() -> Result<()> {
    let conn = Connection::open("casino.db")?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    // Create users table
    db::dbinitialize::initialize_dbs(&conn)?;

    authentication::auth::login(&conn)
}