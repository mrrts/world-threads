//! Web-deployment Phase 1 item 7 — Stripe billing skeleton.
//!
//! Per the web-hosting architecture plan § X. Three concerns covered:
//!
//!   1. **Checkout Session creation** (POST /api/v1/billing/checkout-session)
//!      — creates a hosted Stripe Checkout page for a given tier; returns
//!      the URL the frontend redirects the user to.
//!
//!   2. **Customer Portal Session** (POST /api/v1/billing/portal-session)
//!      — creates a hosted Stripe-managed page for the user to change
//!      plan / update payment method / cancel.
//!
//!   3. **Webhook receiver** (POST /api/v1/billing/webhook) — verifies
//!      Stripe signature, parses the event, dispatches to per-type
//!      handlers (subscription.updated / subscription.deleted /
//!      checkout.session.completed / invoice.payment_failed), persists
//!      to the subscriptions table.
//!
//! Stage: SKELETON. Outbound calls are STUBBED to demonstrate shape —
//! `STRIPE_API_KEY` env var presence is checked; if unset, routes return
//! a clean 503 ("billing not configured") instead of crashing. Webhook
//! signature verification IS implemented (HMAC-SHA256 over the raw body
//! per Stripe spec); the per-event handlers parse the payload then
//! persist a stub subscription row.
//!
//! NOT yet earned:
//!   - Live Stripe API keys (Ryan needs to create the account; per the
//!     architecture plan § XI this is part of LLC formation work).
//!   - Real tier→price mapping (Stripe Price IDs from the dashboard).
//!   - Pro-ration logic / dunning / refund handling.
//!   - Subscription enforcement in content routes (Phase 1+ item 8
//!     usage metering ties this to per-request middleware).

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use hmac::{Hmac, Mac};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use tracing::{info, warn};

use crate::ApiError;

type HmacSha256 = Hmac<Sha256>;

// ── Public route handlers ──────────────────────────────────────────────

#[derive(Clone)]
pub struct BillingState {
    pub db: Arc<Mutex<Connection>>,
    pub stripe_api_key: Option<String>,
    pub stripe_webhook_secret: Option<String>,
    pub tier_price_map: TierPriceMap,
}

/// Maps WorldThreads tier names → Stripe Price IDs. Loaded from env
/// at server start (WT_STRIPE_PRICE_HEARTH / WT_STRIPE_PRICE_TOWN /
/// WT_STRIPE_PRICE_COMPENDIUM). Empty by default; routes check presence
/// before allowing checkout for a given tier.
#[derive(Clone, Default, Debug)]
pub struct TierPriceMap {
    pub hearth: Option<String>,
    pub town: Option<String>,
    pub compendium: Option<String>,
}

impl TierPriceMap {
    pub fn from_env() -> Self {
        TierPriceMap {
            hearth: std::env::var("WT_STRIPE_PRICE_HEARTH").ok(),
            town: std::env::var("WT_STRIPE_PRICE_TOWN").ok(),
            compendium: std::env::var("WT_STRIPE_PRICE_COMPENDIUM").ok(),
        }
    }

    pub fn lookup(&self, tier: &str) -> Option<&str> {
        match tier {
            "hearth" => self.hearth.as_deref(),
            "town" => self.town.as_deref(),
            "compendium" => self.compendium.as_deref(),
            _ => None,
        }
    }
}

// ── Checkout Session creation ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CheckoutSessionRequest {
    pub tier: String, // "hearth" | "town" | "compendium"
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Serialize)]
pub struct CheckoutSessionResponse {
    pub session_id: String,
    pub url: String,
}

/// Stub Checkout Session creation. Phase 1+ swaps in a real Stripe API
/// call once `STRIPE_API_KEY` is configured. The handler is structurally
/// complete: tier validation, price lookup, authentication assumed via
/// the session middleware (Phase 1+ wires user_id into this handler).
pub async fn create_checkout_session(
    State(state): State<BillingState>,
    Json(req): Json<CheckoutSessionRequest>,
) -> Result<Json<CheckoutSessionResponse>, ApiError> {
    let _api_key = state
        .stripe_api_key
        .as_deref()
        .ok_or_else(|| ApiError::server("billing not configured (STRIPE_API_KEY unset)"))?;

    let _price_id = state
        .tier_price_map
        .lookup(&req.tier)
        .ok_or_else(|| ApiError::bad_request(format!("unknown or unconfigured tier: {}", req.tier)))?;

    // Phase 1 skeleton: return a placeholder shape so the frontend can
    // wire to it; Phase 1+ replaces this with a real Stripe HTTP call
    // (POST https://api.stripe.com/v1/checkout/sessions with
    // mode=subscription, line_items[0][price]=<price_id>, line_items[0][quantity]=1,
    // customer or customer_email, success_url, cancel_url, billing_address_collection=auto).
    Ok(Json(CheckoutSessionResponse {
        session_id: "cs_skeleton_not_real".to_string(),
        url: format!("{}?stripe_session=skeleton", req.success_url),
    }))
}

// ── Customer Portal Session creation ──────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PortalSessionRequest {
    pub return_url: String,
}

#[derive(Debug, Serialize)]
pub struct PortalSessionResponse {
    pub url: String,
}

pub async fn create_portal_session(
    State(state): State<BillingState>,
    Json(req): Json<PortalSessionRequest>,
) -> Result<Json<PortalSessionResponse>, ApiError> {
    let _api_key = state
        .stripe_api_key
        .as_deref()
        .ok_or_else(|| ApiError::server("billing not configured (STRIPE_API_KEY unset)"))?;

    Ok(Json(PortalSessionResponse {
        url: format!("{}?stripe_portal=skeleton", req.return_url),
    }))
}

// ── Webhook receiver ──────────────────────────────────────────────────

/// Receives raw webhook payloads from Stripe. Performs HMAC-SHA256
/// signature verification per Stripe's webhook spec before parsing.
/// Idempotency: each event's `id` is stored in `stripe_events` table;
/// duplicate deliveries are detected and short-circuited.
pub async fn webhook(
    State(state): State<BillingState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, ApiError> {
    let webhook_secret = state
        .stripe_webhook_secret
        .as_deref()
        .ok_or_else(|| ApiError::server("billing not configured (STRIPE_WEBHOOK_SECRET unset)"))?;

    let sig_header = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| ApiError::bad_request("missing stripe-signature header"))?;

    verify_stripe_signature(sig_header, &body, webhook_secret)
        .map_err(|e| ApiError::unauthorized(format!("signature verification failed: {e}")))?;

    let event: StripeEvent =
        serde_json::from_slice(&body).map_err(|e| ApiError::bad_request(format!("bad json: {e}")))?;

    // Idempotency check: bail with 200 if we've seen this event id before.
    {
        let conn = state.db.lock().map_err(|e| ApiError::server(format!("db lock: {e}")))?;
        let seen: bool = conn
            .query_row(
                "SELECT 1 FROM stripe_events WHERE event_id = ?1",
                rusqlite::params![event.id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if seen {
            info!(event_id = %event.id, "stripe webhook: already processed, skipping");
            return Ok(StatusCode::OK);
        }
        conn.execute(
            "INSERT INTO stripe_events (event_id, event_type, payload_json) VALUES (?1, ?2, ?3)",
            rusqlite::params![
                event.id,
                event.event_type,
                String::from_utf8_lossy(&body).to_string()
            ],
        )
        .map_err(|e| ApiError::server(format!("ledger insert: {e}")))?;
    }

    // Dispatch by event type.
    match event.event_type.as_str() {
        "checkout.session.completed" => handle_checkout_completed(&state, &event)?,
        "customer.subscription.updated" => handle_subscription_updated(&state, &event)?,
        "customer.subscription.deleted" => handle_subscription_deleted(&state, &event)?,
        "invoice.payment_failed" => handle_invoice_payment_failed(&state, &event)?,
        other => {
            warn!(event_type = other, "stripe webhook: unhandled event type");
        }
    }

    Ok(StatusCode::OK)
}

// ── Stripe signature verification ──────────────────────────────────────

/// Verify Stripe's HMAC-SHA256 signature on a webhook payload. Stripe's
/// signature header has the shape:
///   t=<timestamp>,v1=<signature>[,v0=<legacy>]
/// We compute HMAC-SHA256("<timestamp>.<body>", webhook_secret) and
/// compare in constant time against the v1 element. Reject if timestamp
/// is more than 5 minutes old (replay protection).
fn verify_stripe_signature(
    sig_header: &str,
    body: &[u8],
    webhook_secret: &str,
) -> Result<(), &'static str> {
    let mut timestamp: Option<&str> = None;
    let mut v1_sig: Option<&str> = None;
    for part in sig_header.split(',') {
        let part = part.trim();
        if let Some(t) = part.strip_prefix("t=") {
            timestamp = Some(t);
        } else if let Some(s) = part.strip_prefix("v1=") {
            v1_sig = Some(s);
        }
    }
    let timestamp = timestamp.ok_or("no t= in signature")?;
    let v1_sig = v1_sig.ok_or("no v1= in signature")?;

    // Replay-protection window: reject signatures older than 5 minutes.
    let ts_secs: i64 = timestamp.parse().map_err(|_| "bad timestamp")?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| "system clock")?
        .as_secs() as i64;
    if (now - ts_secs).abs() > 300 {
        return Err("timestamp too old");
    }

    let signed_payload = format!("{}.", timestamp);
    let mut mac =
        HmacSha256::new_from_slice(webhook_secret.as_bytes()).map_err(|_| "hmac init")?;
    mac.update(signed_payload.as_bytes());
    mac.update(body);
    let computed = mac.finalize().into_bytes();
    let computed_hex = hex::encode(computed);

    if computed_hex.as_bytes().ct_eq(v1_sig.as_bytes()).into() {
        Ok(())
    } else {
        Err("signature mismatch")
    }
}

// ── Stripe event payload shapes (subset) ──────────────────────────────

#[derive(Debug, Deserialize)]
struct StripeEvent {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    data: StripeEventData,
}

#[derive(Debug, Deserialize)]
struct StripeEventData {
    object: serde_json::Value,
}

// ── Per-event handlers ─────────────────────────────────────────────────

fn handle_checkout_completed(state: &BillingState, event: &StripeEvent) -> Result<(), ApiError> {
    // Phase 1 skeleton: log + persist subscription row.
    // The Checkout Session payload includes `customer` (Stripe customer
    // ID), `subscription` (Stripe subscription ID), `client_reference_id`
    // (we'll set this to our user_id when creating the session in
    // create_checkout_session — Phase 1+ wiring).
    let obj = &event.data.object;
    let customer = obj
        .get("customer")
        .and_then(|v| v.as_str())
        .unwrap_or("(missing)");
    let subscription = obj
        .get("subscription")
        .and_then(|v| v.as_str())
        .unwrap_or("(missing)");
    let user_id = obj
        .get("client_reference_id")
        .and_then(|v| v.as_str())
        .unwrap_or("(missing)");
    info!(
        event_id = %event.id,
        user_id,
        stripe_customer = customer,
        stripe_subscription = subscription,
        "checkout.session.completed"
    );

    if user_id == "(missing)" {
        return Err(ApiError::bad_request("checkout session missing client_reference_id"));
    }

    let conn = state.db.lock().map_err(|e| ApiError::server(format!("db lock: {e}")))?;
    conn.execute(
        "INSERT OR REPLACE INTO subscriptions (user_id, stripe_customer_id, stripe_subscription_id, tier, status, updated_at)
         VALUES (?1, ?2, ?3, 'town', 'active', datetime('now'))",
        rusqlite::params![user_id, customer, subscription],
    )
    .map_err(|e| ApiError::server(format!("upsert subscription: {e}")))?;
    Ok(())
}

fn handle_subscription_updated(state: &BillingState, event: &StripeEvent) -> Result<(), ApiError> {
    let obj = &event.data.object;
    let subscription_id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::bad_request("subscription.updated missing id"))?;
    let status = obj
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let cancel_at_period_end = obj
        .get("cancel_at_period_end")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let current_period_end = obj
        .get("current_period_end")
        .and_then(|v| v.as_i64())
        .map(unix_to_rfc3339);

    info!(
        event_id = %event.id,
        stripe_subscription = subscription_id,
        status,
        cancel_at_period_end,
        "customer.subscription.updated"
    );

    let conn = state.db.lock().map_err(|e| ApiError::server(format!("db lock: {e}")))?;
    conn.execute(
        "UPDATE subscriptions SET status = ?2, cancel_at_period_end = ?3,
            current_period_end = ?4, updated_at = datetime('now')
         WHERE stripe_subscription_id = ?1",
        rusqlite::params![subscription_id, status, cancel_at_period_end as i64, current_period_end],
    )
    .map_err(|e| ApiError::server(format!("update subscription: {e}")))?;
    Ok(())
}

fn handle_subscription_deleted(state: &BillingState, event: &StripeEvent) -> Result<(), ApiError> {
    let obj = &event.data.object;
    let subscription_id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::bad_request("subscription.deleted missing id"))?;

    info!(
        event_id = %event.id,
        stripe_subscription = subscription_id,
        "customer.subscription.deleted"
    );

    let conn = state.db.lock().map_err(|e| ApiError::server(format!("db lock: {e}")))?;
    conn.execute(
        "UPDATE subscriptions SET status = 'canceled', updated_at = datetime('now')
         WHERE stripe_subscription_id = ?1",
        rusqlite::params![subscription_id],
    )
    .map_err(|e| ApiError::server(format!("cancel subscription: {e}")))?;
    Ok(())
}

fn handle_invoice_payment_failed(_state: &BillingState, event: &StripeEvent) -> Result<(), ApiError> {
    let obj = &event.data.object;
    let customer = obj
        .get("customer")
        .and_then(|v| v.as_str())
        .unwrap_or("(missing)");
    let subscription = obj
        .get("subscription")
        .and_then(|v| v.as_str())
        .unwrap_or("(missing)");
    warn!(
        event_id = %event.id,
        stripe_customer = customer,
        stripe_subscription = subscription,
        "invoice.payment_failed — dunning flow not implemented (Phase 1+)"
    );
    // Phase 1+: send dunning email; mark subscription past_due after
    // Stripe's retry exhaustion; surface to user in app UI.
    Ok(())
}

fn unix_to_rfc3339(unix_secs: i64) -> String {
    chrono::DateTime::from_timestamp(unix_secs, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}
