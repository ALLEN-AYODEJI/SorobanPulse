//! Additional integration coverage for GET /v1/events/temporal (Issue #701).
//!
//! The endpoint itself, its relative-time parser, and its aggregation path
//! were already implemented and unit-tested under issue #581
//! (see `src/handlers.rs::temporal_tests`). This file adds coverage for
//! filter combinations and pagination that weren't previously exercised by
//! an HTTP-level test: contract_id / event_type filters actually narrowing
//! results, absolute timestamp ranges returning real rows, pagination, and
//! multi-bucket aggregation counts.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

use soroban_pulse::config::{HealthState, IndexerState};
use soroban_pulse::metrics::init_metrics;
use soroban_pulse::routes::create_router;

const CONTRACT_A: &str = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM";
const CONTRACT_B: &str = "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";

fn make_router(pool: PgPool) -> axum::Router {
    let health_state = Arc::new(HealthState::new(60));
    health_state.update_last_poll();
    let indexer_state = Arc::new(IndexerState::new());
    let prometheus_handle = init_metrics();
    let config = soroban_pulse::config::Config::default();
    create_router(
        pool,
        Vec::new(),
        &[],
        60,
        health_state,
        indexer_state,
        prometheus_handle,
        15000,
        config,
    )
}

async fn get_json(app: axum::Router, uri: &str) -> (StatusCode, serde_json::Value) {
    let resp = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, v)
}

#[sqlx::test(migrations = "./migrations")]
async fn temporal_contract_id_filter_narrows_results(pool: PgPool) {
    sqlx::query(
        "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data)
         VALUES ($1, 'contract', 'txcontracta0000000000000000000000000000000000000000', 100, NOW(), '{}'::jsonb)",
    )
    .bind(CONTRACT_A)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data)
         VALUES ($1, 'contract', 'txcontractb0000000000000000000000000000000000000000', 101, NOW(), '{}'::jsonb)",
    )
    .bind(CONTRACT_B)
    .execute(&pool)
    .await
    .unwrap();

    let app = make_router(pool);
    let (status, v) = get_json(
        app,
        &format!("/v1/events/temporal?since=24h&contract_id={CONTRACT_A}"),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["total"], 1);
    let events = v["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["contract_id"], CONTRACT_A);
}

#[sqlx::test(migrations = "./migrations")]
async fn temporal_event_type_filter_narrows_results(pool: PgPool) {
    sqlx::query(
        "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data)
         VALUES ($1, 'contract', 'txtypecontract00000000000000000000000000000000000000', 200, NOW(), '{}'::jsonb)",
    )
    .bind(CONTRACT_A)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data)
         VALUES ($1, 'diagnostic', 'txtypediagnostic0000000000000000000000000000000000000', 201, NOW(), '{}'::jsonb)",
    )
    .bind(CONTRACT_A)
    .execute(&pool)
    .await
    .unwrap();

    let app = make_router(pool);
    let (status, v) = get_json(app, "/v1/events/temporal?since=24h&event_type=diagnostic").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["total"], 1);
    assert_eq!(v["events"][0]["event_type"], "diagnostic");
}

#[sqlx::test(migrations = "./migrations")]
async fn temporal_absolute_timestamp_range_returns_matching_event(pool: PgPool) {
    sqlx::query(
        "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data)
         VALUES ($1, 'contract', 'txabsrange0000000000000000000000000000000000000000000', 300, '2026-01-01T12:00:00Z', '{}'::jsonb)",
    )
    .bind(CONTRACT_A)
    .execute(&pool)
    .await
    .unwrap();

    let app = make_router(pool);
    let (status, v) = get_json(
        app,
        "/v1/events/temporal?from_timestamp=2026-01-01T00:00:00Z&to_timestamp=2026-01-02T00:00:00Z",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["total"], 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn temporal_absolute_timestamp_range_excludes_event_outside_window(pool: PgPool) {
    sqlx::query(
        "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data)
         VALUES ($1, 'contract', 'txabsoutside00000000000000000000000000000000000000000', 301, '2026-03-01T12:00:00Z', '{}'::jsonb)",
    )
    .bind(CONTRACT_A)
    .execute(&pool)
    .await
    .unwrap();

    let app = make_router(pool);
    let (status, v) = get_json(
        app,
        "/v1/events/temporal?from_timestamp=2026-01-01T00:00:00Z&to_timestamp=2026-01-02T00:00:00Z",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["total"], 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn temporal_pagination_second_page_returns_remaining_event(pool: PgPool) {
    sqlx::query(
        "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data)
         VALUES ($1, 'contract', 'txpage1000000000000000000000000000000000000000000000', 400, NOW(), '{}'::jsonb)",
    )
    .bind(CONTRACT_A)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data)
         VALUES ($1, 'contract', 'txpage2000000000000000000000000000000000000000000000', 401, NOW(), '{}'::jsonb)",
    )
    .bind(CONTRACT_A)
    .execute(&pool)
    .await
    .unwrap();

    let app = make_router(pool);
    let (status, v) = get_json(app, "/v1/events/temporal?since=24h&limit=1&page=2").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["events"].as_array().unwrap().len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn temporal_aggregate_counts_match_inserted_events(pool: PgPool) {
    // Two events in the current hour bucket.
    for i in 0..2 {
        sqlx::query(
            "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data)
             VALUES ($1, 'contract', $2, $3, NOW(), '{}'::jsonb)",
        )
        .bind(CONTRACT_A)
        .bind(format!(
            "txagg{i}0000000000000000000000000000000000000000000000"
        ))
        .bind(500i64 + i as i64)
        .execute(&pool)
        .await
        .unwrap();
    }

    let app = make_router(pool);
    let (status, v) = get_json(
        app,
        "/v1/events/temporal?since=1h&aggregate=true&window=1h",
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let buckets = v["buckets"].as_array().unwrap();
    let total_bucketed: i64 = buckets
        .iter()
        .map(|b| b["event_count"].as_i64().unwrap_or(0))
        .sum();
    assert_eq!(total_bucketed, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn temporal_before_excludes_recent_event(pool: PgPool) {
    sqlx::query(
        "INSERT INTO events (contract_id, event_type, tx_hash, ledger, timestamp, event_data)
         VALUES ($1, 'contract', 'txbeforerecent000000000000000000000000000000000000000', 600, NOW(), '{}'::jsonb)",
    )
    .bind(CONTRACT_A)
    .execute(&pool)
    .await
    .unwrap();

    let app = make_router(pool);
    // Window is [now-24h, now-1h]; the event inserted at NOW() falls outside it.
    let (status, v) = get_json(app, "/v1/events/temporal?since=24h&before=1h").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["total"], 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn temporal_invalid_contract_id_returns_400(pool: PgPool) {
    let app = make_router(pool);
    let (status, _) = get_json(app, "/v1/events/temporal?since=24h&contract_id=not-a-valid-id").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}
