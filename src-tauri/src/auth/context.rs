//! Web-deployment Phase 2 (first thread-through) — user_id context helper.
//!
//! Per the web-hosting architecture plan § V (auth) + readiness summary
//! § IV item 9. The user_id-everywhere invariant is now landed at column
//! level (commits 3bc4ea1 + 65c8b0d + 661300a — 23/23+ per-user tables
//! scoped). Phase 2 work threads user_id through every query path so
//! INSERT and SELECT operations include the column.
//!
//! Stage of work: FIRST THREAD-THROUGH. This module provides the
//! `current_user_id()` helper that Tauri commands call to learn whose
//! data they're operating on. In Tauri-mode the answer is the SENTINEL
//! user — Tauri has a single implicit operator (the desktop user) and
//! the sentinel row exists in users table (created by the migration's
//! batch-1 backfill if any pre-existing rows were found; created
//! on-demand by this helper otherwise).
//!
//! In web-mode (when api-server replaces Tauri IPC for content routes)
//! the helper isn't used — Axum routes extract user_id from the session
//! cookie via auth_middleware and pass it explicitly to core functions.
//! The two paths converge at the core function signatures (which take
//! user_id as a parameter from either source).
//!
//! Phase 2+ wires this into every commands/*.rs entry that touches a
//! per-user table; this commit demonstrates the pattern with
//! create_world_cmd as a worked example.

use rusqlite::Connection;

/// The sentinel user_id assigned to all pre-existing Tauri rows by the
/// migration backfill. Same constant as the migration's SENTINEL_USER_ID
/// (db::schema::run_migrations). Tauri-mode `current_user_id()` always
/// returns this value.
pub const TAURI_SENTINEL_USER_ID: &str = "00000000-0000-0000-0000-000000000001";

/// Resolve the current user_id for a Tauri-mode operation. Always
/// returns TAURI_SENTINEL_USER_ID. The helper exists for parity with
/// the future Axum-mode equivalent (which reads user_id from a session
/// cookie); both call sites get a `&str` user_id from a single helper
/// signature regardless of runtime mode.
///
/// Phase 2+ may extend this to support multi-tenant Tauri installs
/// (e.g., family-share-desktop where multiple humans log in separately
/// on the same Tauri app) — for now the assumption of a single implicit
/// operator matches existing Tauri behavior.
///
/// Side effect: if the sentinel user row doesn't yet exist in the users
/// table (e.g., fresh install that has never run the backfill UPDATE
/// because no pre-existing rows were there to trigger it), this helper
/// INSERTs it on first call so the FK constraint future-added by
/// Phase 2+ tightening will not break. Idempotent via INSERT OR IGNORE.
pub fn current_user_id(conn: &Connection) -> Result<&'static str, rusqlite::Error> {
    ensure_sentinel_exists(conn)?;
    Ok(TAURI_SENTINEL_USER_ID)
}

/// Ensure the Tauri sentinel user row exists. Idempotent.
fn ensure_sentinel_exists(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR IGNORE INTO users (id, email, password_hash, display_name, timezone)
         VALUES (?1, 'local-tauri@worldthreads.localdomain', '$disabled$cannot-login',
                 'Local Tauri (sentinel)', 'UTC')",
        rusqlite::params![TAURI_SENTINEL_USER_ID],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;
    use std::sync::Once;

    fn fresh_db() -> Connection {
        static INIT: Once = Once::new();
        INIT.call_once(|| unsafe {
            rusqlite::ffi::sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        });
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn current_user_id_returns_sentinel() {
        let conn = fresh_db();
        let uid = current_user_id(&conn).unwrap();
        assert_eq!(uid, TAURI_SENTINEL_USER_ID);
    }

    #[test]
    fn current_user_id_creates_sentinel_idempotently() {
        let conn = fresh_db();
        // First call creates (if not already created by migration)
        current_user_id(&conn).unwrap();
        // Subsequent calls succeed without duplicates
        current_user_id(&conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM users WHERE id = ?1",
                rusqlite::params![TAURI_SENTINEL_USER_ID],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1, "exactly one sentinel row should exist");
    }
}
