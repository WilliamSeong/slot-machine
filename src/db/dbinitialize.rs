use rusqlite::{Connection, Result};

pub fn initialize_dbs(conn: &Connection) -> Result<(), rusqlite::Error> {
    // Create User table
    conn.execute(
        "Create Table If Not Exists users (
            id Integer Primary Key,
            username Text Unique Not Null,
            password Text Not Null,
            balance Real Not Null Default 0.0,
            role Text Default 'user' Check(role In ('user', 'technician', 'commissioner'))
        )",
        [],
    )?;

    // Create Games table
    conn.execute(
        "Create Table If Not Exists games (
            id Integer Primary Key,
            name Text Unique Not Null,
            played Integer,
            win Integer,
            loss Integer,
            active Bool
        )",
        [],
    )?;

    // Create User Statistics
    conn.execute(
        "Create Table If Not Exists user_statistics (
            id Integer Primary Key,
            user_id Integer Not Null,
            game_id Integer Not Null,
            win Integer,
            loss Integer,
            highest_payout Real,
            last_played Text,
            Foreign Key (user_id) References users(id),
            Foreign Key (game_id) References games(id)
        )",
        [],
    )?;

    add_technician_commissioner(&conn)?;
    add_games(&conn)?;

    Ok(())
}

// Add technician and commissioner to user table
fn add_technician_commissioner(conn: &Connection) -> Result<(),rusqlite::Error> {
    conn.execute(
        "Insert Or Ignore Into users (username, password, role, balance) 
        Values ('technician', ?1, 'technician', 5000.0)",
        ["123"]
    )?;

    conn.execute(
        "Insert Or Ignore Into users (username, password, role, balance) 
        Values ('commissioner', ?1, 'commissioner', 10000.0)",
        ["123"]  // Change password after first login!
    )?;

    Ok(())
}

// Add game modes to games table
fn add_games(conn: &Connection) -> Result<(),rusqlite::Error> {
    conn.execute(
        "Insert Or Ignore Into games (name, played, win, loss, active) 
        Values ('normal', 0, 0, 0, true),
                ('multi', 0, 0, 0, true),
                ('holding', 0, 0, 0, true)",[]
    )?;

    Ok(())
}