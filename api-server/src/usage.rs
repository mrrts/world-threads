//! Web-deployment Phase 1 item 8 — usage metering skeleton.
//!
//! Per the web-hosting architecture plan § VIII. Each LLM-bearing
//! request records a usage_events row; per-tier limits are checked
//! before LLM calls; 429 with structured reset times returned when
//! exhausted. Frontend renders friendly "you're out of turns this
//! session" UI with reset times in user timezone.
//!
//! Stage: SKELETON. The data shape + helpers are real and tested;
//! enforcement wiring into content-route middleware is Phase 2 work
//! because api-server doesn't HAVE content routes yet — those live
//! in the Tauri app today and migrate over per item 9 (user_id scoping).
//!
//! What's here:
//!   - TierLimit struct + TIER_LIMITS table
//!   - Session-rollover logic (4-hour inactivity threshold)
//!   - Week-start computation in user's timezone via chrono-tz
//!   - record_event() writes a row + assigns/reuses session_id
//!   - compute_usage() returns current session + weekly counts + reset times
//!   - enforce_tier() returns Err(UsageExhausted) when over limit
//!   - GET /api/v1/usage/current route (skeleton)
//!
//! NOT here:
//!   - Content-route middleware (Phase 2 — when content routes migrate
//!     to api-server, each one wraps in enforce_tier before LLM call)
//!   - Authenticated user extraction from session cookie in the route
//!     (auth_middleware Phase 0 stub doesn't thread user_id yet —
//!     route below uses a placeholder query-param for skeleton testing)
//!   - Per-kind separate accounting (current model counts all 'chat'
//!     events together; § VIII may differentiate dreams/narrations
//!     later)
//!   - Frontend UI integration

use std::sync::{Arc, Mutex};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::ApiError;

// ── Tier configuration ──────────────────────────────────────────────────

/// Per-tier limits. Numbers match the recommended tiers in architecture
/// plan § VII.2; trial mirrors the 60-turn-over-3-days shape via a low
/// weekly cap + session cap. Phase 1+ may adjust based on real cost-per-
/// active-user data after first 30 days.
#[derive(Debug, Clone, Copy)]
pub struct TierLimit {
    pub session_turns: u32,
    pub weekly_turns: u32,
}

pub fn tier_limit(tier: &str) -> Option<TierLimit> {
    match tier {
        "trial" => Some(TierLimit { session_turns: 30, weekly_turns: 60 }),
        "hearth" => Some(TierLimit { session_turns: 30, weekly_turns: 250 }),
        "town" => Some(TierLimit { session_turns: 60, weekly_turns: 700 }),
        "compendium" => Some(TierLimit { session_turns: 100, weekly_turns: 1500 }),
        _ => None,
    }
}

const SESSION_ROLLOVER_HOURS: i64 = 4;

// ── Usage event types ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub user_id: String,
    pub kind: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageSnapshot {
    pub tier: String,
    pub session_turns_used: u32,
    pub session_turns_limit: u32,
    pub session_resets_at: String, // RFC-3339 in user's timezone
    pub weekly_turns_used: u32,
    pub weekly_turns_limit: u32,
    pub weekly_resets_at: String, // RFC-3339 in user's timezone
    pub timezone: String,
}

#[derive(Debug, thiserror::Error)]
pub enum UsageError {
    #[error("usage exhausted: {0}")]
    Exhausted(String),
    #[error("db: {0}")]
    Db(String),
    #[error("user not found")]
    UserNotFound,
    #[error("unknown tier: {0}")]
    UnknownTier(String),
    #[error("bad timezone: {0}")]
    BadTimezone(String),
}

// ── Time helpers ────────────────────────────────────────────────────────

/// Compute the start of the ISO week (Monday 00:00:00) in the user's
/// timezone, returned as an RFC-3339 string. Used as the key for
/// weekly aggregates.
fn iso_week_start_in_tz(now: DateTime<Utc>, tz_name: &str) -> Result<(String, String), UsageError> {
    let tz: Tz = tz_name
        .parse()
        .map_err(|_| UsageError::BadTimezone(tz_name.to_string()))?;
    let local = now.with_timezone(&tz);
    // ISO week starts Monday. compute days since Monday in user's local
    // calendar, subtract, zero out time of day.
    let weekday = local.weekday();
    let days_from_monday = weekday.num_days_from_monday() as i64;
    let week_start_date: NaiveDate = local.date_naive() - Duration::days(days_from_monday);
    let week_start_local = tz
        .from_local_datetime(&week_start_date.and_hms_opt(0, 0, 0).unwrap())
        .single()
        .or_else(|| {
            tz.from_local_datetime(&week_start_date.and_hms_opt(1, 0, 0).unwrap())
                .single()
        })
        .ok_or_else(|| UsageError::BadTimezone(format!("ambiguous local time for {tz_name}")))?;
    let week_end_local = week_start_local + Duration::days(7);
    Ok((week_start_local.to_rfc3339(), week_end_local.to_rfc3339()))
}

/// Compute session_resets_at — when the current session would
/// "naturally" roll over to a new session_id, assuming the user
/// stops interacting now. RFC-3339 in user's timezone.
fn session_resets_at_in_tz(last_event_at: DateTime<Utc>, tz_name: &str) -> Result<String, UsageError> {
    let tz: Tz = tz_name
        .parse()
        .map_err(|_| UsageError::BadTimezone(tz_name.to_string()))?;
    let resets_at = last_event_at + Duration::hours(SESSION_ROLLOVER_HOURS);
    Ok(resets_at.with_timezone(&tz).to_rfc3339())
}

// ── Record + compute helpers ───────────────────────────────────────────

/// Record a usage event for a user. Computes session_id (reusing the
/// most recent if within 4 hours of the user's last event; else
/// generating a new one) and week_start_iso (computed in user's tz).
/// Returns the assigned session_id.
pub fn record_event(
    conn: &Connection,
    event: &UsageEvent,
    user_tz: &str,
) -> Result<String, UsageError> {
    let now = Utc::now();
    let (week_start_iso, _) = iso_week_start_in_tz(now, user_tz)?;

    // Look up the user's most recent usage_event to decide session_id.
    let session_id: String = conn
        .query_row(
            "SELECT session_id, created_at FROM usage_events
             WHERE user_id = ?1
             ORDER BY id DESC LIMIT 1",
            rusqlite::params![event.user_id],
            |row| {
                let sid: String = row.get(0)?;
                let last_at: String = row.get(1)?;
                Ok((sid, last_at))
            },
        )
        .ok()
        .and_then(|(sid, last_at)| {
            // Parse last_at (SQLite datetime('now') format) as UTC.
            let last_at_dt = DateTime::parse_from_rfc3339(&last_at)
                .ok()
                .map(|d| d.with_timezone(&Utc))
                .or_else(|| {
                    chrono::NaiveDateTime::parse_from_str(&last_at, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|nd| DateTime::<Utc>::from_naive_utc_and_offset(nd, Utc))
                })?;
            if (now - last_at_dt).num_hours() < SESSION_ROLLOVER_HOURS {
                Some(sid)
            } else {
                None
            }
        })
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    conn.execute(
        "INSERT INTO usage_events (user_id, kind, input_tokens, output_tokens, session_id, week_start_iso)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![
            event.user_id,
            event.kind,
            event.input_tokens,
            event.output_tokens,
            session_id,
            week_start_iso,
        ],
    )
    .map_err(|e| UsageError::Db(e.to_string()))?;

    Ok(session_id)
}

/// Compute the current usage snapshot for a user. Returns session +
/// weekly turn counts + reset times. "Turn" = one usage_event row;
/// Phase 1+ may differentiate kinds.
pub fn compute_usage(
    conn: &Connection,
    user_id: &str,
    tier: &str,
    user_tz: &str,
) -> Result<UsageSnapshot, UsageError> {
    let limit = tier_limit(tier).ok_or_else(|| UsageError::UnknownTier(tier.to_string()))?;
    let now = Utc::now();
    let (week_start_iso, week_end_iso) = iso_week_start_in_tz(now, user_tz)?;

    let weekly_used: u32 = conn
        .query_row(
            "SELECT COUNT(*) FROM usage_events WHERE user_id = ?1 AND week_start_iso = ?2",
            rusqlite::params![user_id, week_start_iso],
            |row| row.get::<_, i64>(0).map(|n| n as u32),
        )
        .unwrap_or(0);

    // Most recent event determines session. If no events yet OR most
    // recent is >4h old, session_used = 0 + session_resets_at = now
    // (because the next event would start a new session).
    let recent: Option<(String, String)> = conn
        .query_row(
            "SELECT session_id, created_at FROM usage_events
             WHERE user_id = ?1 ORDER BY id DESC LIMIT 1",
            rusqlite::params![user_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    let (session_used, session_resets_at) = if let Some((sid, last_at_str)) = recent {
        let last_at_dt = DateTime::parse_from_rfc3339(&last_at_str)
            .ok()
            .map(|d| d.with_timezone(&Utc))
            .or_else(|| {
                chrono::NaiveDateTime::parse_from_str(&last_at_str, "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .map(|nd| DateTime::<Utc>::from_naive_utc_and_offset(nd, Utc))
            });
        if let Some(last_at) = last_at_dt {
            if (now - last_at).num_hours() < SESSION_ROLLOVER_HOURS {
                let used: u32 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM usage_events WHERE user_id = ?1 AND session_id = ?2",
                        rusqlite::params![user_id, sid],
                        |row| row.get::<_, i64>(0).map(|n| n as u32),
                    )
                    .unwrap_or(0);
                let resets = session_resets_at_in_tz(last_at, user_tz)?;
                (used, resets)
            } else {
                let resets = session_resets_at_in_tz(now, user_tz)?;
                (0, resets)
            }
        } else {
            warn!(last_at = %last_at_str, "couldn't parse last usage_events.created_at");
            (0, session_resets_at_in_tz(now, user_tz)?)
        }
    } else {
        (0, session_resets_at_in_tz(now, user_tz)?)
    };

    Ok(UsageSnapshot {
        tier: tier.to_string(),
        session_turns_used: session_used,
        session_turns_limit: limit.session_turns,
        session_resets_at,
        weekly_turns_used: weekly_used,
        weekly_turns_limit: limit.weekly_turns,
        weekly_resets_at: week_end_iso,
        timezone: user_tz.to_string(),
    })
}

/// Enforcement gate. Returns Ok if the user has remaining capacity in
/// both session AND weekly windows; Err(Exhausted) with a structured
/// snapshot otherwise. Future content-route middleware calls this
/// BEFORE the LLM call; if Err, returns 429 with snapshot as JSON.
#[allow(dead_code)] // Phase 2 wires this from content-route middleware
pub fn enforce_tier(
    conn: &Connection,
    user_id: &str,
    tier: &str,
    user_tz: &str,
) -> Result<UsageSnapshot, UsageError> {
    let snapshot = compute_usage(conn, user_id, tier, user_tz)?;
    if snapshot.session_turns_used >= snapshot.session_turns_limit {
        return Err(UsageError::Exhausted(format!(
            "session limit reached ({}/{}); resets at {}",
            snapshot.session_turns_used, snapshot.session_turns_limit, snapshot.session_resets_at
        )));
    }
    if snapshot.weekly_turns_used >= snapshot.weekly_turns_limit {
        return Err(UsageError::Exhausted(format!(
            "weekly limit reached ({}/{}); resets at {}",
            snapshot.weekly_turns_used, snapshot.weekly_turns_limit, snapshot.weekly_resets_at
        )));
    }
    Ok(snapshot)
}

// ── HTTP route ──────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct UsageState {
    pub db: Arc<Mutex<Connection>>,
}

#[derive(Debug, Deserialize)]
pub struct UsageQuery {
    // Phase 1+ removes — user_id will come from authenticated session.
    pub user_id: String,
    #[serde(default = "default_tier")]
    pub tier: String,
}

fn default_tier() -> String {
    "trial".to_string()
}

/// GET /api/v1/usage/current — returns the current usage snapshot for
/// the queried user. Phase 1+ removes the query-param and reads
/// user_id from the authenticated session cookie via the session
/// middleware skeleton already in main.rs.
pub async fn get_usage_current(
    State(state): State<UsageState>,
    Query(q): Query<UsageQuery>,
) -> Result<Json<UsageSnapshot>, ApiError> {
    let conn = state.db.lock().map_err(|e| ApiError::server(format!("db lock: {e}")))?;
    let user_tz: String = conn
        .query_row(
            "SELECT timezone FROM users WHERE id = ?1",
            rusqlite::params![q.user_id],
            |row| row.get(0),
        )
        .map_err(|_| ApiError::bad_request("user not found"))?;
    let snapshot = compute_usage(&conn, &q.user_id, &q.tier, &user_tz)
        .map_err(|e| ApiError::server(e.to_string()))?;
    Ok(Json(snapshot))
}

impl From<UsageError> for axum::response::Response {
    fn from(err: UsageError) -> Self {
        let status = match &err {
            UsageError::Exhausted(_) => StatusCode::TOO_MANY_REQUESTS,
            UsageError::UserNotFound | UsageError::UnknownTier(_) | UsageError::BadTimezone(_) => {
                StatusCode::BAD_REQUEST
            }
            UsageError::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, err.to_string()).into_response()
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_limit_known_tiers() {
        assert_eq!(tier_limit("trial").unwrap().weekly_turns, 60);
        assert_eq!(tier_limit("hearth").unwrap().weekly_turns, 250);
        assert_eq!(tier_limit("town").unwrap().weekly_turns, 700);
        assert_eq!(tier_limit("compendium").unwrap().weekly_turns, 1500);
        assert!(tier_limit("unknown").is_none());
    }

    #[test]
    fn iso_week_start_chicago_known_date() {
        // Wednesday 2026-05-13 in America/Chicago — week starts
        // Monday 2026-05-11 00:00 CST.
        let now = Utc.with_ymd_and_hms(2026, 5, 13, 18, 0, 0).unwrap();
        let (start, end) = iso_week_start_in_tz(now, "America/Chicago").unwrap();
        assert!(start.starts_with("2026-05-11T00:00:00"), "start: {start}");
        assert!(end.starts_with("2026-05-18T00:00:00"), "end: {end}");
    }

    #[test]
    fn session_resets_at_4hr() {
        let last = Utc.with_ymd_and_hms(2026, 5, 10, 20, 0, 0).unwrap();
        let resets = session_resets_at_in_tz(last, "America/Chicago").unwrap();
        // 4h after 20:00 UTC = 00:00 UTC the next day, which in CST is 19:00 same-day.
        assert!(resets.contains("2026-05-10T19:00:00"), "resets: {resets}");
    }

    #[test]
    fn bad_tz_rejected() {
        let now = Utc::now();
        assert!(iso_week_start_in_tz(now, "Mars/Olympus_Mons").is_err());
    }
}
