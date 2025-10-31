pub fn initialize_db(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "Create Table If Not Exists users (
            id Integer Primary Key,
            username Text Unique Not Null,
            password Text Not Null,
            user_type Text Not Null Default 'player',
            balance Real Not Null Default 0.0,
            role Text Default 'user' Check(role In ('user', 'technician', 'commissioner'))
        )",
        [],
    )?;

    Ok(())
}

pub fn add_technician_comissioner(conn: &Connection) -> Result<(),rusqlite::Error> {
    conn.execute(
        "Insert Or Ignore Into users (username, password, role, balance) 
        VALUES ('technician', ?1, 'technician', 5000.0)",
        ["123"]
    )?;

    conn.execute(
        "Insert Or Ignore Into users (username, password, role, balance) 
        VALUES ('commissioner', ?1, 'commissioner', 10000.0)",
        ["123"]  // Change password after first login!
    )?;

    Ok(())
}
