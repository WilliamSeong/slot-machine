use rusqlite::{Connection, Result};

mod interfaces;
mod db;
mod authentication;

fn main() -> Result<()> {
    let conn = Connection::open("casino.db")?;
    
    // Create users table
    db::dbqueries::initialize_db(&conn)?;

    // Add technician account and commissioner account
    db::dbqueries::add_technician_comissioner(&conn)?;

    authentication::auth::login(&conn)
}