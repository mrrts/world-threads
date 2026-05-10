//! WorldThreads HTTP API server — Phase 1 item 6 skeleton.
//!
//! Per the web-hosting architecture plan
//! (reports/2026-05-10-0100-web-hosting-architecture-plan.md) and the
//! readiness summary at reports/2026-05-10-1720-web-deployment-phase-0-...
//!
//! Stage: SKELETON. This commit ships a buildable Axum server with:
//!   - /health endpoint (smoke test)
//!   - /api/v1/auth/signup, /api/v1/auth/login, /api/v1/auth/logout routes
//!   - argon2id password hashing for credentials
//!   - SHA-256 token-hash session storage (raw token in HTTP-only Secure
//!     SameSite=Lax cookie; only hash hits the database)
//!   - Session-extracting middleware (auth_middleware)
//!   - Shared AppState carrying the auth storage
//!   - SQLite-backed storage via SqliteAuthStorage from app_lib
//!
//! NOT yet wired:
//!   - Per-user content storage routes (worlds/characters/messages/etc) —
//!     Phase 2 work; this server only handles auth + sessions so far
//!   - Stripe webhooks + subscription state (Phase 1 item 7)
//!   - Usage metering (Phase 1 item 8)
//!   - Email verification flow
//!   - Forgot-password flow
//!   - CSRF protection (cookies are SameSite=Lax which mitigates most CSRF;
//!     a CSRF token + double-submit pattern is Phase 1+ hardening)
//!
//! Honest scope statement: this is the SKELETON. End-to-end signup/login
//! flow against this server has not been tested with the WebLoginSketch
//! component yet. Tomorrow's work: (a) verify cargo build clean from
//! the api-server crate, (b) run the server locally + curl the routes,
//! (c) wire the frontend's content_kdf_salt response field (which the
//! login flow expects per WebLoginSketch line 39).

use std::sync::{Arc, Mutex};

use anyhow::Result;
use app_lib::{
    auth::{AuthSuccess, PublicUser, User},
    storage::{AuthStorage, NewSession, NewUser, SqliteAuthStorage},
};
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
    Argon2,
};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tower_cookies::{Cookie, Cookies, CookieManagerLayer};
use tracing::info;

mod billing;
mod usage;

// ── Shared app state ────────────────────────────────────────────────────

/// Carried across all route handlers via axum's State extractor. The
/// Connection lives behind Arc<Mutex<>> because rusqlite Connection is
/// !Sync. Phase 1+ may upgrade to a connection pool (deadpool-sqlite
/// or sqlx::SqlitePool) when concurrent route handling becomes the
/// bottleneck; for the auth-only skeleton a single connection is fine.
#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
}

// ── Cookie + token helpers ──────────────────────────────────────────────

const SESSION_COOKIE: &str = "wt_session";
const SESSION_TTL_DAYS: i64 = 30;

/// Hash a raw session token via SHA-256 to obtain the value stored in
/// the database. The raw token lives only in the user's cookie; even
/// a database breach can't grant impersonation without the raw token.
fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

/// Generate a cryptographically-random session token. 32 bytes = 256
/// bits of entropy, hex-encoded.
fn generate_session_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

// ── Payload shapes ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SignupRequest {
    email: String,
    password: String,
    #[serde(default)]
    display_name: String,
    #[serde(default = "default_tz")]
    timezone: String,
}

fn default_tz() -> String {
    "UTC".to_string()
}

#[derive(Debug, Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

/// Response shape returned to the frontend on successful signup or
/// login. Mirrors app_lib::auth::AuthSuccess plus a `content_kdf_salt`
/// field — the per-user salt the browser passes to Argon2id when
/// deriving its content_key. The salt is not secret; it's just a
/// per-user namespace value. Frontend reads this to call deriveContentKey
/// (see WebLoginSketch).
#[derive(Debug, Serialize)]
struct AuthResponse {
    user: PublicUser,
    session_token: String,
    expires_at: String,
    content_kdf_salt: String,
}

impl AuthResponse {
    fn from_auth_success(success: AuthSuccess, content_kdf_salt: String) -> Self {
        AuthResponse {
            user: success.user,
            session_token: success.session_token,
            expires_at: success.expires_at,
            content_kdf_salt,
        }
    }
}

// ── Route handlers ──────────────────────────────────────────────────────

async fn health() -> &'static str {
    "ok"
}

async fn signup(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(req): Json<SignupRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    if req.email.trim().is_empty() {
        return Err(ApiError::bad_request("email required"));
    }
    if req.password.len() < 12 {
        return Err(ApiError::bad_request("password must be at least 12 characters"));
    }

    let conn = state.db.lock().map_err(|e| ApiError::server(format!("db lock: {e}")))?;
    let storage = SqliteAuthStorage::new(&conn);

    if storage.find_user_by_email(&req.email).map_err(ApiError::server)?.is_some() {
        return Err(ApiError::bad_request("email already registered"));
    }

    // Argon2id hash the password using the argon2 crate's PasswordHasher
    // trait — produces a PHC-format string that embeds the salt + params
    // + hash. This is DISTINCT from the content_kdf_salt used for the
    // browser-side content_key derivation (see Credentials hashing vs
    // content encryption in the architecture plan § V vs § XXI.2).
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| ApiError::server(format!("hash: {e}")))?
        .to_string();

    // Generate a content-KDF salt for this user. The frontend uses this
    // to derive its content_key. Stored alongside the user record; not
    // secret, but unique per user.
    let content_kdf_salt = hex::encode({
        use rand::RngCore;
        let mut bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    });

    let user_id = uuid::Uuid::new_v4().to_string();
    let new_user = NewUser {
        id: user_id.clone(),
        email: req.email.trim().to_lowercase(),
        password_hash,
        display_name: req.display_name,
        timezone: req.timezone,
    };
    let user = storage.create_user(new_user).map_err(ApiError::server)?;

    // Store content_kdf_salt in settings (Phase 1+ may move to a
    // dedicated column; for now reuses the existing key/value setting
    // pattern via a direct INSERT).
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        rusqlite::params![format!("content_kdf_salt.{}", user_id), content_kdf_salt],
    )
    .map_err(|e| ApiError::server(format!("save kdf salt: {e}")))?;

    let success = mint_session(&storage, &user)?;
    set_session_cookie(&cookies, &success.session_token, &success.expires_at);
    drop(conn);

    Ok(Json(AuthResponse::from_auth_success(success, content_kdf_salt)))
}

async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let conn = state.db.lock().map_err(|e| ApiError::server(format!("db lock: {e}")))?;
    let storage = SqliteAuthStorage::new(&conn);

    let user = storage
        .find_user_by_email(&req.email.trim().to_lowercase())
        .map_err(ApiError::server)?
        .ok_or_else(|| ApiError::unauthorized("invalid email or password"))?;

    let parsed = PasswordHash::new(&user.password_hash)
        .map_err(|e| ApiError::server(format!("password hash parse: {e}")))?;
    if Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed)
        .is_err()
    {
        return Err(ApiError::unauthorized("invalid email or password"));
    }

    let content_kdf_salt: String = conn
        .query_row(
            "SELECT value FROM settings WHERE key = ?1",
            rusqlite::params![format!("content_kdf_salt.{}", user.id)],
            |row| row.get(0),
        )
        .map_err(|e| ApiError::server(format!("read kdf salt: {e}")))?;

    let success = mint_session(&storage, &user)?;
    set_session_cookie(&cookies, &success.session_token, &success.expires_at);
    drop(conn);

    Ok(Json(AuthResponse::from_auth_success(success, content_kdf_salt)))
}

async fn logout(State(state): State<AppState>, cookies: Cookies) -> Result<StatusCode, ApiError> {
    if let Some(cookie) = cookies.get(SESSION_COOKIE) {
        let token_hash = hash_token(cookie.value());
        let conn = state.db.lock().map_err(|e| ApiError::server(format!("db lock: {e}")))?;
        let storage = SqliteAuthStorage::new(&conn);
        if let Some(session) = storage
            .find_session_by_token_hash(&token_hash)
            .map_err(ApiError::server)?
        {
            storage.delete_session(&session.id).map_err(ApiError::server)?;
        }
    }
    cookies.remove(Cookie::new(SESSION_COOKIE, ""));
    Ok(StatusCode::NO_CONTENT)
}

// ── Session-minting + cookie helpers ────────────────────────────────────

fn mint_session(storage: &SqliteAuthStorage, user: &User) -> Result<AuthSuccess, ApiError> {
    let raw_token = generate_session_token();
    let token_hash = hash_token(&raw_token);
    let expires_at = (Utc::now() + Duration::days(SESSION_TTL_DAYS)).to_rfc3339();
    storage
        .create_session(NewSession {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user.id.clone(),
            token_hash,
            expires_at: expires_at.clone(),
            user_agent: None,
            ip_address: None,
        })
        .map_err(ApiError::server)?;
    Ok(AuthSuccess {
        user: PublicUser::from(user),
        session_token: raw_token,
        expires_at,
    })
}

fn set_session_cookie(cookies: &Cookies, token: &str, _expires_at_iso: &str) {
    let mut cookie = Cookie::new(SESSION_COOKIE, token.to_string());
    cookie.set_http_only(true);
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_secure(true);
    cookie.set_path("/");
    // Note: setting max-age via cookie.set_max_age() would mirror the
    // server-side expires_at. tower-cookies::Cookie max-age API
    // accepts a tower_cookies::cookie::time::Duration — Phase 1+ can
    // add the conversion. Session-only cookie is acceptable for skeleton.
    cookies.add(cookie);
}

// ── Auth middleware (skeleton) ──────────────────────────────────────────

/// Extract the current user from a session cookie. Returns None if no
/// cookie or no matching session. Future content-route handlers use
/// this to enforce auth + thread user_id through queries.
#[allow(dead_code)]
async fn extract_user(state: &AppState, headers: &HeaderMap) -> Option<User> {
    // Manual cookie parsing for the helper variant; the route handlers
    // use tower_cookies::Cookies extractor instead. Both reach the same
    // SqliteAuthStorage path.
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    let token = cookie_header
        .split(';')
        .map(|s| s.trim())
        .find_map(|s| s.strip_prefix(&format!("{SESSION_COOKIE}=")))?;
    let token_hash = hash_token(token);
    let conn = state.db.lock().ok()?;
    let storage = SqliteAuthStorage::new(&conn);
    let session = storage.find_session_by_token_hash(&token_hash).ok().flatten()?;
    storage.find_user_by_id(&session.user_id).ok().flatten()
}

// ── Error type ──────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("server error: {0}")]
    Server(String),
}

impl ApiError {
    pub fn bad_request(m: impl Into<String>) -> Self {
        ApiError::BadRequest(m.into())
    }
    pub fn unauthorized(m: impl Into<String>) -> Self {
        ApiError::Unauthorized(m.into())
    }
    pub fn server(m: impl Into<String>) -> Self {
        ApiError::Server(m.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, body) = match &self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.clone()),
            ApiError::Server(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };
        (status, body).into_response()
    }
}

// ── Server bootstrap ────────────────────────────────────────────────────

fn open_db(path: &str) -> Result<Connection> {
    // Mirror the Tauri-side approach: load sqlite-vec auto-extension
    // for vec0 virtual tables that may exist in the migration. The
    // api-server's DB path will likely be DIFFERENT from the Tauri
    // app's local DB in production (server-side DB lives in cloud
    // Postgres per architecture plan; SQLite here is for local dev
    // + skeleton testing only).
    //
    // For Phase 1 skeleton: open a sibling SQLite file alongside the
    // Tauri DB. Phase 1+ swaps for Postgres via sqlx.
    use rusqlite::ffi::sqlite3_auto_extension;
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite_vec::sqlite3_vec_init as *const (),
        )));
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    app_lib::db::schema::run_migrations(&conn)?;
    Ok(conn)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .init();

    let db_path = std::env::var("WT_API_DB_PATH").unwrap_or_else(|_| "wt-api-dev.db".into());
    info!("opening database at {db_path}");
    let conn = open_db(&db_path)?;
    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
    };

    let billing_state = billing::BillingState {
        db: state.db.clone(),
        stripe_api_key: std::env::var("STRIPE_API_KEY").ok(),
        stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET").ok(),
        tier_price_map: billing::TierPriceMap::from_env(),
    };

    let state_for_usage = state.db.clone();

    let auth_routes = Router::new()
        .route("/api/v1/auth/signup", post(signup))
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .with_state(state);

    let billing_routes = Router::new()
        .route("/api/v1/billing/checkout-session", post(billing::create_checkout_session))
        .route("/api/v1/billing/portal-session", post(billing::create_portal_session))
        .route("/api/v1/billing/webhook", post(billing::webhook))
        .with_state(billing_state);

    let usage_state = usage::UsageState { db: state_for_usage };
    let usage_routes = Router::new()
        .route("/api/v1/usage/current", get(usage::get_usage_current))
        .with_state(usage_state);

    let app = Router::new()
        .route("/health", get(health))
        .merge(auth_routes)
        .merge(billing_routes)
        .merge(usage_routes)
        .layer(CookieManagerLayer::new());

    let bind_addr = std::env::var("WT_API_BIND").unwrap_or_else(|_| "127.0.0.1:8787".into());
    info!("api-server listening on {bind_addr}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
