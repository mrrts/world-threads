//! Web-deployment Phase 0 — auth scaffolding (types only, no live logic).
//!
//! Per the web-hosting architecture plan at
//! `reports/2026-05-10-0100-web-hosting-architecture-plan.md`. Stage of
//! work: TOOLING ONLY. This module provides type definitions that the
//! future web deployment's Axum server + Tauri-side hybrid (in the
//! IndexedDB-content + server-metering architecture, § XXI.4) would
//! need. No runtime code paths invoke these types in the current Tauri
//! app; they exist so the codebase is forward-compatible without
//! committing to a launch path.
//!
//! Adding live auth logic, session middleware, password hashing
//! integration, email verification flows, etc., is explicitly Phase
//! 1+ work and NOT done here.
//!
//! Schema for `users` + `sessions` is defined in `db::schema::run_migrations`.

use serde::{Deserialize, Serialize};

/// A user account. Keyed by UUID (string form for SQLite + Postgres
/// portability). Email is unique. password_hash is argon2id (per the
/// plan); current implementation just stores whatever string is passed
/// in — actual hashing happens at the auth-service layer (Phase 1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub email_verified_at: Option<String>,
    pub password_hash: String,
    pub display_name: String,
    pub timezone: String,
    pub created_at: String,
    pub updated_at: String,
}

/// A live session. Token is stored as a hash so the database alone
/// can't impersonate the user; the raw token lives in the user's
/// HTTP-only Secure SameSite=Lax cookie (web) or in Tauri's secure
/// store (desktop, when paired with a remote backend). expires_at +
/// last_seen_at govern session lifetime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: String,
    pub last_seen_at: String,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

/// Credentials submitted at signup or login. Phase 1 will route this
/// through argon2id + email verification + rate-limit middleware before
/// any user row is created or session minted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

/// Result of a successful authentication attempt — what the auth
/// service hands back to the route layer. Carries the User (sans
/// password_hash) plus a freshly-minted session token. Phase 1 will
/// wrap this in proper Axum response types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSuccess {
    pub user: PublicUser,
    pub session_token: String,
    pub expires_at: String,
}

/// User shape safe to expose to clients (omits password_hash). Used
/// by both the web API responses AND any Tauri command that returns
/// the current user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub timezone: String,
    pub email_verified_at: Option<String>,
}

impl From<&User> for PublicUser {
    fn from(u: &User) -> Self {
        PublicUser {
            id: u.id.clone(),
            email: u.email.clone(),
            display_name: u.display_name.clone(),
            timezone: u.timezone.clone(),
            email_verified_at: u.email_verified_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_user_excludes_password_hash() {
        let u = User {
            id: "u1".into(),
            email: "ryan@example.com".into(),
            email_verified_at: None,
            password_hash: "$argon2id$...secret...".into(),
            display_name: "Ryan".into(),
            timezone: "America/Chicago".into(),
            created_at: "2026-05-10T01:00:00Z".into(),
            updated_at: "2026-05-10T01:00:00Z".into(),
        };
        let pub_user: PublicUser = (&u).into();
        let json = serde_json::to_string(&pub_user).unwrap();
        assert!(!json.contains("password"), "PublicUser must never serialize a password field");
        assert!(!json.contains("argon2"), "PublicUser must never serialize a hash");
        assert!(json.contains("ryan@example.com"));
        assert!(json.contains("America/Chicago"));
    }

    #[test]
    fn credentials_round_trip_serde() {
        let c = Credentials {
            email: "test@example.com".into(),
            password: "hunter2".into(),
        };
        let s = serde_json::to_string(&c).unwrap();
        let back: Credentials = serde_json::from_str(&s).unwrap();
        assert_eq!(back.email, c.email);
    }
}
