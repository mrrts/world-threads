//! Web-deployment Phase 0 — Storage trait scaffolding.
//!
//! Per the web-hosting architecture plan at
//! `reports/2026-05-10-0100-web-hosting-architecture-plan.md` § III.2.
//! Stage of work: TOOLING ONLY. This module defines the abstraction
//! shape that future Phase 1 work will use to thread storage operations
//! through both the existing Tauri SQLite backend and a future
//! Postgres-backed Axum server (for user-metadata) or stateless-proxy
//! mode (where content lives client-side per § XXI.4).
//!
//! Phase 0 commitment: SIGNATURES + a small auth-related implementation
//! pair (SqliteAuthStorage / future MemoryAuthStorage for tests) that
//! prove the abstraction shape works. No existing Tauri command paths
//! are refactored to use this trait yet — that's Phase 1+ and explicitly
//! NOT done in this commit.
//!
//! The trait splits intentionally narrow: AuthStorage covers the
//! users + sessions tables only. Domain-storage traits (CharacterStorage,
//! WorldStorage, MessageStorage, etc.) come later when their existing
//! query functions in commands/* and db/queries/* get extracted into
//! core/.

use crate::auth::{Session, User};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

/// Storage operations against the `users` + `sessions` tables. Both the
/// Tauri SQLite backend and the future Postgres server impl will satisfy
/// this trait. Errors carry a human-readable message; Phase 1 will refine
/// to typed errors with retry semantics.
///
/// Phase 0 trait does NOT impose Send+Sync. The SQLite impl borrows a
/// `&Connection` which is `!Sync` (internal RefCell). Phase 1 will
/// revisit concurrency via a connection pool (deadpool-sqlite for the
/// Tauri-mode worker pool, or sqlx::PgPool for Postgres-backed servers)
/// at which point the trait can add Send+Sync as appropriate.
pub trait AuthStorage {
    fn create_user(&self, input: NewUser) -> Result<User, String>;
    fn find_user_by_email(&self, email: &str) -> Result<Option<User>, String>;
    fn find_user_by_id(&self, id: &str) -> Result<Option<User>, String>;
    fn create_session(&self, input: NewSession) -> Result<Session, String>;
    fn find_session_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>, String>;
    fn delete_session(&self, session_id: &str) -> Result<(), String>;
    fn purge_expired_sessions(&self) -> Result<usize, String>;
}

/// Input shape for create_user. Caller is responsible for hashing the
/// password BEFORE handing the hash to this trait — the storage layer
/// never sees plaintext passwords.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub timezone: String,
}

/// Input shape for create_session. Caller is responsible for hashing
/// the session token (e.g., SHA-256) before storage; the raw token
/// lives only in the client cookie / secure store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSession {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
}

/// SQLite implementation of AuthStorage. Wraps a borrow of a Connection
/// for the duration of each call. The existing Tauri app uses a
/// shared Arc<Mutex<Connection>>; integrating this trait into the runtime
/// is Phase 1 work.
pub struct SqliteAuthStorage<'conn> {
    pub conn: &'conn Connection,
}

impl<'conn> SqliteAuthStorage<'conn> {
    pub fn new(conn: &'conn Connection) -> Self {
        Self { conn }
    }
}

impl<'conn> AuthStorage for SqliteAuthStorage<'conn> {
    fn create_user(&self, input: NewUser) -> Result<User, String> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO users (id, email, password_hash, display_name, timezone, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
                params![
                    input.id,
                    input.email,
                    input.password_hash,
                    input.display_name,
                    input.timezone,
                    now,
                ],
            )
            .map_err(|e| format!("create_user: {e}"))?;
        self.find_user_by_id(&input.id)?
            .ok_or_else(|| "create_user: user not found after insert".to_string())
    }

    fn find_user_by_email(&self, email: &str) -> Result<Option<User>, String> {
        self.conn
            .query_row(
                "SELECT id, email, email_verified_at, password_hash, display_name, timezone, created_at, updated_at
                 FROM users WHERE email = ?1",
                params![email],
                row_to_user,
            )
            .map(Some)
            .or_else(|e| {
                if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                    Ok(None)
                } else {
                    Err(format!("find_user_by_email: {e}"))
                }
            })
    }

    fn find_user_by_id(&self, id: &str) -> Result<Option<User>, String> {
        self.conn
            .query_row(
                "SELECT id, email, email_verified_at, password_hash, display_name, timezone, created_at, updated_at
                 FROM users WHERE id = ?1",
                params![id],
                row_to_user,
            )
            .map(Some)
            .or_else(|e| {
                if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                    Ok(None)
                } else {
                    Err(format!("find_user_by_id: {e}"))
                }
            })
    }

    fn create_session(&self, input: NewSession) -> Result<Session, String> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO sessions (id, user_id, token_hash, expires_at, last_seen_at, user_agent, ip_address, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?5)",
                params![
                    input.id,
                    input.user_id,
                    input.token_hash,
                    input.expires_at,
                    now,
                    input.user_agent,
                    input.ip_address,
                ],
            )
            .map_err(|e| format!("create_session: {e}"))?;
        self.find_session_by_token_hash(&input.token_hash)?
            .ok_or_else(|| "create_session: session not found after insert".to_string())
    }

    fn find_session_by_token_hash(&self, token_hash: &str) -> Result<Option<Session>, String> {
        self.conn
            .query_row(
                "SELECT id, user_id, token_hash, expires_at, last_seen_at, user_agent, ip_address, created_at
                 FROM sessions WHERE token_hash = ?1",
                params![token_hash],
                row_to_session,
            )
            .map(Some)
            .or_else(|e| {
                if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                    Ok(None)
                } else {
                    Err(format!("find_session_by_token_hash: {e}"))
                }
            })
    }

    fn delete_session(&self, session_id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM sessions WHERE id = ?1", params![session_id])
            .map_err(|e| format!("delete_session: {e}"))?;
        Ok(())
    }

    fn purge_expired_sessions(&self) -> Result<usize, String> {
        self.conn
            .execute(
                "DELETE FROM sessions WHERE expires_at < datetime('now')",
                params![],
            )
            .map(|n| n)
            .map_err(|e| format!("purge_expired_sessions: {e}"))
    }
}

fn row_to_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get(0)?,
        email: row.get(1)?,
        email_verified_at: row.get(2)?,
        password_hash: row.get(3)?,
        display_name: row.get(4)?,
        timezone: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        user_id: row.get(1)?,
        token_hash: row.get(2)?,
        expires_at: row.get(3)?,
        last_seen_at: row.get(4)?,
        user_agent: row.get(5)?,
        ip_address: row.get(6)?,
        created_at: row.get(7)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::run_migrations;

    fn fresh_db() -> Connection {
        // Load sqlite-vec auto-extension once per process (idempotent under
        // FFI no-op semantics for repeated registration). The full schema
        // includes a vec0 virtual table that fails to create without it.
        use std::sync::Once;
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
    fn create_and_find_user_round_trip() {
        let conn = fresh_db();
        let storage = SqliteAuthStorage::new(&conn);

        let new_user = NewUser {
            id: "u-test-1".into(),
            email: "test@example.com".into(),
            password_hash: "$argon2id$v=19$m=...$hash".into(),
            display_name: "Tester".into(),
            timezone: "America/Chicago".into(),
        };
        let created = storage.create_user(new_user.clone()).unwrap();
        assert_eq!(created.email, "test@example.com");
        assert_eq!(created.display_name, "Tester");

        let by_email = storage.find_user_by_email("test@example.com").unwrap();
        assert!(by_email.is_some());
        assert_eq!(by_email.unwrap().id, "u-test-1");

        let by_id = storage.find_user_by_id("u-test-1").unwrap();
        assert!(by_id.is_some());

        let missing = storage.find_user_by_email("missing@example.com").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn duplicate_email_rejected() {
        let conn = fresh_db();
        let storage = SqliteAuthStorage::new(&conn);
        let new_user = NewUser {
            id: "u-1".into(),
            email: "dup@example.com".into(),
            password_hash: "x".into(),
            display_name: "A".into(),
            timezone: "UTC".into(),
        };
        storage.create_user(new_user.clone()).unwrap();
        let dup = NewUser {
            id: "u-2".into(),
            ..new_user
        };
        let err = storage.create_user(dup).unwrap_err();
        assert!(err.contains("UNIQUE") || err.contains("constraint"), "expected UNIQUE error: {err}");
    }

    #[test]
    fn session_lifecycle() {
        let conn = fresh_db();
        let storage = SqliteAuthStorage::new(&conn);

        storage
            .create_user(NewUser {
                id: "u-sess".into(),
                email: "sess@example.com".into(),
                password_hash: "x".into(),
                display_name: "S".into(),
                timezone: "UTC".into(),
            })
            .unwrap();

        let session = storage
            .create_session(NewSession {
                id: "sess-1".into(),
                user_id: "u-sess".into(),
                token_hash: "hash-of-token-1".into(),
                expires_at: "2099-01-01T00:00:00Z".into(),
                user_agent: Some("test-agent".into()),
                ip_address: None,
            })
            .unwrap();
        assert_eq!(session.user_id, "u-sess");

        let found = storage
            .find_session_by_token_hash("hash-of-token-1")
            .unwrap();
        assert!(found.is_some());

        storage.delete_session(&session.id).unwrap();
        let gone = storage
            .find_session_by_token_hash("hash-of-token-1")
            .unwrap();
        assert!(gone.is_none());
    }

    #[test]
    fn purge_expired_sessions_works() {
        let conn = fresh_db();
        let storage = SqliteAuthStorage::new(&conn);

        storage
            .create_user(NewUser {
                id: "u-p".into(),
                email: "p@example.com".into(),
                password_hash: "x".into(),
                display_name: "P".into(),
                timezone: "UTC".into(),
            })
            .unwrap();

        // One expired, one fresh.
        storage
            .create_session(NewSession {
                id: "sess-old".into(),
                user_id: "u-p".into(),
                token_hash: "h-old".into(),
                expires_at: "2000-01-01T00:00:00Z".into(),
                user_agent: None,
                ip_address: None,
            })
            .unwrap();
        storage
            .create_session(NewSession {
                id: "sess-new".into(),
                user_id: "u-p".into(),
                token_hash: "h-new".into(),
                expires_at: "2099-01-01T00:00:00Z".into(),
                user_agent: None,
                ip_address: None,
            })
            .unwrap();

        let purged = storage.purge_expired_sessions().unwrap();
        assert_eq!(purged, 1);
        assert!(storage
            .find_session_by_token_hash("h-old")
            .unwrap()
            .is_none());
        assert!(storage
            .find_session_by_token_hash("h-new")
            .unwrap()
            .is_some());
    }
}
