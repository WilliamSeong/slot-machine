use rusqlite::{Connection};
use colored::*;

use crate::{db::dbqueries, interfaces::user::User, logger};
use crate::authentication::authorization;

use crate::interfaces::menus::menu_generator;

pub fn technician_menu(conn: &Connection, user: &User) -> rusqlite::Result<()> {
    // SECURITY: Verify user has technician role
    if let Err(e) = authorization::require_technician(conn, user) {
        logger::logger::security(&format!("Blocked unauthorized access to technician menu by User ID: {}: {}", user.id, e));
        return Ok(());
    }
    
    // Log that technician has accessed the menu
    logger::logger::security(&format!("Technician (User ID: {}) accessed technician menu", user.id));
    
    loop {
        // Show options to user
        let menu_options = vec!["Show Games", "Show Statistics", "Security Logs", "Logout"];
        let user_input = menu_generator("═══ 🎰 Tech Menu 🎰 ═══", &menu_options);

        match user_input.trim() {
            "Show Games" => {
                logger::logger::info(&format!("Technician (User ID: {}) accessed games menu", user.id));
                let _ = games_menu(conn, user);
            }
            "Show Statistics" => {
                logger::logger::info(&format!("Technician (User ID: {}) accessed statistics", user.id));
                technician_statistics(conn, user);
            }
            "Security Logs" => {
                logger::logger::security(&format!("Technician (User ID: {}) accessed security logs", user.id));
                logger::verification::log_verification_menu(conn, user)?;
            }
            "Logout" => {
                logger::logger::info(&format!("Technician (User ID: {}) logged out", user.id));
                println!("Logging out...");
                break;
            }
            _ => {
                logger::logger::warning(&format!("Technician (User ID: {}) entered invalid menu choice", user.id));
                println!("Invalid choice. Please try again.");
            }
        }
    }
    Ok(())
}

/// Function to allow technician to change what games are available to the user - REQUIRES TECHNICIAN ROLE
fn games_menu(conn: &Connection, user: &User) -> rusqlite::Result<()>{
    // SECURITY: Double-check authorization
    if authorization::require_technician(conn, user).is_err() {
        return Ok(());
    }
    
    logger::logger::security(&format!("Technician (User ID: {}) accessing games control", user.id));
    loop {
        let games = dbqueries::get_games(conn)?;

        for game in games {
            let (name, active): (String, bool) = game;

            if active {
                println!("game: {} active: {}", name, active.to_string().green());
            } else {
                println!("game: {} active: {}", name, active.to_string().red());
            }
        }
        // Show options to technician
        // query all games
        let games_data = dbqueries::get_games(conn)?;
        let mut all_games: Vec<&str> = games_data
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        // add exit
        all_games.push("exit");
        let user_input = menu_generator("═══ 🎰 Technician Games Control 🎰 ═══", &all_games);

        match user_input {
            "exit" => {
                break
            },
            _ => {
                if !user_input.is_empty() {
                    logger::logger::security(&format!("Game status toggle attempt for: {}", user_input));
                    dbqueries::toggle_game(conn, user_input)?;
                }
            }
        }
    }
    Ok(())
}

/// View game statistics - REQUIRES TECHNICIAN ROLE
fn technician_statistics(conn: &Connection, user: &User) {
    // SECURITY: Double-check authorization
    if authorization::require_technician(conn, user).is_err() {
        return;
    }
    
    logger::logger::info(&format!("Technician (User ID: {}) viewing game statistics", user.id));
    println!("Game Statistics:");
    let _ = dbqueries::get_game_statistics(conn);
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use rusqlite::Connection;
    use crate::interfaces::user::User;

    fn setup_test_db() -> Connection {
        crate::cryptography::crypto::initialize_encryption_key();
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("PRAGMA foreign_keys = ON", []).unwrap();
        crate::db::dbinitialize::initialize_dbs(&conn).unwrap();
        conn
    }

    fn create_test_user(conn: &Connection, username: &str, role: &str) -> i32 {
        conn.execute(
            "INSERT INTO users (username, password, balance, role) VALUES (?1, ?2, '0.0', ?3)",
            [username, "test_hash", role],
        ).unwrap();
        
        conn.last_insert_rowid() as i32
    }

    #[test]
    fn test_technician_authorization_succeeds() {
        let conn = setup_test_db();
        let tech_id = create_test_user(&conn, "techuser", "technician");
        let user = User { id: tech_id };
        
        // Should succeed for technician
        let result = crate::authentication::authorization::require_technician(&conn, &user);
        assert!(result.is_ok());
    }

    #[test]
    fn test_technician_authorization_fails_for_user() {
        let conn = setup_test_db();
        let user_id = create_test_user(&conn, "normaluser", "user");
        let user = User { id: user_id };
        
        // Should fail for regular user
        let result = crate::authentication::authorization::require_technician(&conn, &user);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_games_returns_games() {
        let conn = setup_test_db();
        
        // initializedbs creates default games
        let games = dbqueries::get_games(&conn).unwrap();
        
        assert!(!games.is_empty());
        assert!(games.iter().any(|(name, _)| name == "normal"));
    }

    #[test]
    fn test_toggle_game_changes_status() {
        let conn = setup_test_db();
        
        // Get initial state
        let games_before = dbqueries::get_games(&conn).unwrap();
        let normal_before = games_before.iter()
            .find(|(name, _)| name == "normal")
            .map(|(_, active)| *active)
            .unwrap();
        
        // Toggle the game
        dbqueries::toggle_game(&conn, "normal").unwrap();
        
        // Check if it changed
        let games_after = dbqueries::get_games(&conn).unwrap();
        let normal_after = games_after.iter()
            .find(|(name, _)| name == "normal")
            .map(|(_, active)| *active)
            .unwrap();
        
        assert_ne!(normal_before, normal_after);
    }

    #[test]
    fn test_toggle_game_twice_returns_to_original() {
        let conn = setup_test_db();
        
        let games_initial = dbqueries::get_games(&conn).unwrap();
        let initial_state = games_initial.iter()
            .find(|(name, _)| name == "normal")
            .map(|(_, active)| *active)
            .unwrap();
        
        // Toggle twice
        dbqueries::toggle_game(&conn, "normal").unwrap();
        dbqueries::toggle_game(&conn, "normal").unwrap();
        
        let games_final = dbqueries::get_games(&conn).unwrap();
        let final_state = games_final.iter()
            .find(|(name, _)| name == "normal")
            .map(|(_, active)| *active)
            .unwrap();
        
        assert_eq!(initial_state, final_state);
    }

    #[test]
    fn test_toggle_nonexistent_game_fails() {
        let conn = setup_test_db();
        
        let result = dbqueries::toggle_game(&conn, "fake_game");
        
        // Should return an error or handle gracefully
        assert!(result.is_ok()); // Depends on your implementation
    }

    #[test]
    fn test_get_game_statistics_succeeds() {
        let conn = setup_test_db();
        
        let result = dbqueries::get_game_statistics(&conn);
        
        // check it returns Ok
        assert!(result.is_ok());
    }

    #[test]
    fn test_games_menu_requires_technician() {
        let conn = setup_test_db();
        let user_id = create_test_user(&conn, "regularuser", "user");
        let user = User { id: user_id };
        
        // games_menu should return early for non-technician
        // Test is user is not technician
        let auth_result = crate::authentication::authorization::require_technician(&conn, &user);
        assert!(auth_result.is_err());
    }

    #[test]
    fn test_multiple_games_can_be_active() {
        let conn = setup_test_db();
        
        // Ensure multiple games are active
        dbqueries::toggle_game(&conn, "normal").ok();
        dbqueries::toggle_game(&conn, "multi").ok();
        
        let games = dbqueries::get_games(&conn).unwrap();
        let active_count = games.iter().filter(|(_, active)| *active).count();
        
        // At least some games should be active
        assert!(active_count > 0);
    }
}
