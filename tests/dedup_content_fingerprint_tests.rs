//! Integration coverage for `dedup::is_content_duplicate` (Issue #702).
//!
//! `compute_fingerprint` already has thorough unit tests in `src/dedup.rs`,
//! and the wiring into the indexer (`enable_content_dedup` /
//! `dedup_window_secs`) is exercised implicitly by config tests, but
//! `is_content_duplicate` — the actual DB-backed half of the dedup guard
//! described in docs/event-deduplication.md — had no test coverage of its
//! own. This exercises it directly against a real database: a fresh
//! fingerprint match, a non-matching fingerprint, and the lookback-window
//! boundary.

use sqlx::PgPool;

use soroban_pulse::dedup::{compute_fingerprint, is_content_duplicate};

const CONTRACT_ID: &str = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM";

async fn insert_event_with_fingerprint(
    pool: &PgPool,
    tx_hash: &str,
    fingerprint: &str,
    created_at: chrono::DateTime<chrono::Utc>,
) {
    sqlx::query(
        "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data, fingerprint, created_at)
         VALUES ($1, 'contract', $2, 1, NOW(), '{}'::jsonb, $3, $4)",
    )
    .bind(CONTRACT_ID)
    .bind(tx_hash)
    .bind(fingerprint)
    .bind(created_at)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn matching_fingerprint_within_window_is_duplicate(pool: PgPool) {
    let fingerprint = compute_fingerprint("tx1", CONTRACT_ID, "contract", &serde_json::json!({"v": 1}));
    insert_event_with_fingerprint(&pool, "tx1", &fingerprint, chrono::Utc::now()).await;

    let is_dup = is_content_duplicate(&pool, &fingerprint, 3600).await.unwrap();
    assert!(is_dup);
}

#[sqlx::test(migrations = "./migrations")]
async fn different_fingerprint_is_not_duplicate(pool: PgPool) {
    let stored = compute_fingerprint("tx1", CONTRACT_ID, "contract", &serde_json::json!({"v": 1}));
    insert_event_with_fingerprint(&pool, "tx1", &stored, chrono::Utc::now()).await;

    let other = compute_fingerprint("tx2", CONTRACT_ID, "contract", &serde_json::json!({"v": 2}));
    let is_dup = is_content_duplicate(&pool, &other, 3600).await.unwrap();
    assert!(!is_dup);
}

#[sqlx::test(migrations = "./migrations")]
async fn matching_fingerprint_outside_window_is_not_duplicate(pool: PgPool) {
    let fingerprint = compute_fingerprint("tx1", CONTRACT_ID, "contract", &serde_json::json!({"v": 1}));
    // Stored two hours ago, well outside a 1-hour (3600s) lookback window.
    let old = chrono::Utc::now() - chrono::Duration::hours(2);
    insert_event_with_fingerprint(&pool, "tx1", &fingerprint, old).await;

    let is_dup = is_content_duplicate(&pool, &fingerprint, 3600).await.unwrap();
    assert!(!is_dup);
}

#[sqlx::test(migrations = "./migrations")]
async fn matching_fingerprint_just_inside_window_is_duplicate(pool: PgPool) {
    let fingerprint = compute_fingerprint("tx1", CONTRACT_ID, "contract", &serde_json::json!({"v": 1}));
    // Stored 30 minutes ago, inside a 1-hour (3600s) lookback window.
    let recent = chrono::Utc::now() - chrono::Duration::minutes(30);
    insert_event_with_fingerprint(&pool, "tx1", &fingerprint, recent).await;

    let is_dup = is_content_duplicate(&pool, &fingerprint, 3600).await.unwrap();
    assert!(is_dup);
}

#[sqlx::test(migrations = "./migrations")]
async fn no_events_means_not_duplicate(pool: PgPool) {
    let fingerprint = compute_fingerprint("tx1", CONTRACT_ID, "contract", &serde_json::json!({"v": 1}));
    let is_dup = is_content_duplicate(&pool, &fingerprint, 3600).await.unwrap();
    assert!(!is_dup);
}
