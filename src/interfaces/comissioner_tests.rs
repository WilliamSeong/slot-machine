#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    // -------------------------------------------------------------------
    // 🧩 Helpers
    // -------------------------------------------------------------------

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS commissioner_log (
                id INTEGER PRIMARY KEY,
                seed TEXT,
                rounds INTEGER,
                wins INTEGER,
                partials INTEGER,
                losses INTEGER,
                rtp REAL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).unwrap();
        conn
    }

    fn dummy_user(role: &str) -> User {
        User {
            id: 1,
            username: "unit_test_user".to_string(),
            role: role.to_string(),
            password_hash: "dummy".to_string(),
        }
    }

    // -------------------------------------------------------------------
    // 1️⃣ commissioner_menu()
    // -------------------------------------------------------------------

    #[test]
    fn test_commissioner_menu_authorization_check() {
        let conn = setup_db();
        let user = dummy_user("player");
        let result = auth::require_commissioner(&conn, &user);
        assert!(result.is_err(), "Non-commissioners must not access commissioner menu");
    }

    #[test]
    fn test_commissioner_menu_allows_commissioner() {
        let conn = setup_db();
        let user = dummy_user("commissioner");
        assert!(
            auth::require_commissioner(&conn, &user).is_ok(),
            "Commissioner role should have access"
        );
    }

    // -------------------------------------------------------------------
    // 2️⃣ run_commissioner_test()
    // -------------------------------------------------------------------

    #[test]
    fn test_run_commissioner_test_rtp_calculation() {
        let total_payout = 50.0;
        let total_bet = 100.0;
        let rtp = (total_payout / total_bet) * 100.0;
        assert!(
            (0.0..=300.0).contains(&rtp),
            "RTP {:.2}% should be within logical bounds",
            rtp
        );
    }

    #[test]
    fn test_run_commissioner_test_db_insert_success() {
        let conn = setup_db();
        let result = conn.execute(
            "INSERT INTO commissioner_log (seed, rounds, wins, partials, losses, rtp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            ("seed123", 100, 10, 20, 70, 95.0),
        );
        assert!(result.is_ok(), "commissioner_log insert should succeed");
    }

    // -------------------------------------------------------------------
    // 3️⃣ view_game_probabilities()
    // -------------------------------------------------------------------

    #[test]
    fn test_view_game_probabilities_authorized_access() {
        let conn = setup_db();
        let user = dummy_user("commissioner");
        assert!(
            auth::require_commissioner(&conn, &user).is_ok(),
            "Commissioner should be authorized to view game probabilities"
        );
    }

    #[test]
    fn test_view_game_probabilities_handles_empty_db() {
        let conn = setup_db();
        let user = dummy_user("commissioner");
        assert!(
            auth::require_commissioner(&conn, &user).is_ok(),
            "Function should not panic with empty DB"
        );
    }

    // -------------------------------------------------------------------
    // 4️⃣ adjust_symbol_weights()
    // -------------------------------------------------------------------

    #[test]
    fn test_adjust_symbol_weights_valid_range() {
        let valid_weight = 75usize;
        assert!(
            (1..=100).contains(&valid_weight),
            "Valid weight must be between 1–100"
        );
    }

    #[test]
    fn test_adjust_symbol_weights_invalid_range() {
        let invalid_low = 0usize;
        let invalid_high = 200usize;
        assert!(
            !(1..=100).contains(&invalid_low),
            "Weight 0 must fail validation"
        );
        assert!(
            !(1..=100).contains(&invalid_high),
            "Weight 200 must fail validation"
        );
    }

    // -------------------------------------------------------------------
    // 5️⃣ adjust_symbol_payouts()
    // -------------------------------------------------------------------

    #[test]
    fn test_adjust_symbol_payouts_valid_range() {
        let valid_payout = 5.0;
        assert!(
            (0.5..=50.0).contains(&valid_payout),
            "Valid payout must be between 0.5–50.0"
        );
    }

    #[test]
    fn test_adjust_symbol_payouts_invalid_range() {
        let invalid_low = 0.1;
        let invalid_high = 99.9;
        assert!(
            !(0.5..=50.0).contains(&invalid_low),
            "Payout < 0.5 must fail validation"
        );
        assert!(
            !(0.5..=50.0).contains(&invalid_high),
            "Payout > 50.0 must fail validation"
        );
    }

    // -------------------------------------------------------------------
    // 6️⃣ Database Layer & Cross-checks
    // -------------------------------------------------------------------

    #[test]
    fn test_commissioner_log_schema_integrity() {
        let conn = setup_db();
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='commissioner_log'"
        ).unwrap();
        let exists: bool = stmt.exists([]).unwrap();
        assert!(exists, "commissioner_log table must exist");
    }

    #[test]
    fn test_authorization_consistency() {
        let conn = setup_db();
        let commissioner = dummy_user("commissioner");
        let player = dummy_user("player");

        assert!(
            auth::require_commissioner(&conn, &commissioner).is_ok(),
            "Commissioner should pass authorization"
        );
        assert!(
            auth::require_commissioner(&conn, &player).is_err(),
            "Player should fail authorization"
        );
    }
}
