//! Security test suite entry point.
//!
//! Run all security tests:
//!   cargo test --test security
//!
//! Run a specific sub-module:
//!   cargo test --test security owasp_tests
//!   cargo test --test security auth_bypass_tests
//!   cargo test --test security crypto_tests
//!   cargo test --test security regression_tests
//!   cargo test --test security rate_limit_tests
//!   cargo test --test security security_headers_tests
//!   cargo test --test security reencryption_tests
//!   cargo test --test security callback_allowlist_tests

mod security;
```

File: tests/security/mod.rs
Current content:
```rust
//! Security test suite.
//!
//! This module aggregates all security-focused integration tests.

pub mod owasp_tests;
pub mod auth_bypass_tests;
pub mod crypto_tests;
pub mod regression_tests;
```

New content:
```rust
//! Security test suite.
//!
//! This module aggregates all security-focused integration tests.

pub mod owasp_tests;
pub mod auth_bypass_tests;
pub mod crypto_tests;
pub mod regression_tests;
pub mod rate_limit_tests;
pub mod security_headers_tests;
pub mod reencryption_tests;
pub mod callback_allowlist_tests;
```

File: tests/security/rate_limit_tests.rs
Current content:
```rust
//! Rate limiting integration tests.
//!
//! Covers issues #1268 (subscription + SSE rate limits).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use soroban_pulse::metrics::Metrics;

/// Build a minimal test router. The exact constructor is provided by the
/// application crate; this helper keeps the tests focused on behaviour.
async fn test_app() -> axum::Router {
    soroban_pulse::test_support::build_test_router().await
}

fn subscription_request(ip: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/v1/subscriptions")
        .header("x-forwarded-for", ip)
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"contract_id":"CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF","event_type":"transfer"}"#,
        ))
        .unwrap()
}

fn sse_request(ip: &str) -> Request<Body> {
    Request::builder()
        .method("GET")
        .uri("/v1/events/stream")
        .header("x-forwarded-for", ip)
        .body(Body::empty())
        .unwrap()
}

#[tokio::test]
async fn sixth_subscription_from_same_ip_returns_429() {
    let app = test_app().await;
    let ip = "203.0.113.10";

    // First five requests should be accepted (or at least not rate limited).
    for i in 0..5 {
        let resp = app
            .clone()
            .oneshot(subscription_request(ip))
            .await
            .expect("request failed");
        assert_ne!(
            resp.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "request {i} unexpectedly rate limited"
        );
    }

    let before = Metrics::global()
        .subscription_rate_limited_total
        .get();

    let resp = app
        .oneshot(subscription_request(ip))
        .await
        .expect("request failed");

    assert_eq!(
        resp.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "6th subscription should be rate limited"
    );
    assert!(
        resp.headers().contains_key("retry-after"),
        "429 response must include Retry-After header"
    );

    let after = Metrics::global()
        .subscription_rate_limited_total
        .get();
    assert_eq!(
        after,
        before + 1,
        "soroban_pulse_subscription_rate_limited_total should increment"
    );
}

#[tokio::test]
async fn sixth_concurrent_sse_connection_from_same_ip_returns_429() {
    let app = test_app().await;
    let ip = "203.0.113.20";

    // Open five concurrent SSE connections. We hold the responses (which are
    // streaming bodies) to keep the connections "open" from the limiter's
    // perspective.
    let mut held = Vec::new();
    for i in 0..5 {
        let resp = app
            .clone()
            .oneshot(sse_request(ip))
            .await
            .expect("request failed");
        assert_ne!(
            resp.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "SSE connection {i} unexpectedly rate limited"
        );
        held.push(resp);
    }

    let before = Metrics::global().sse_rate_limited_total.get();

    let resp = app
        .oneshot(sse_request(ip))
        .await
        .expect("request failed");

    assert_eq!(
        resp.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "6th concurrent SSE connection should be rate limited"
    );
    assert!(
        resp.headers().contains_key("retry-after"),
        "429 response must include Retry-After header"
    );

    let after = Metrics::global().sse_rate_limited_total.get();
    assert_eq!(
        after,
        before + 1,
        "soroban_pulse_sse_rate_limited_total should increment"
    );

    // Drop held connections to release the per-IP slots.
    drop(held);
}

#[tokio::test]
async fn subscription_rate_limit_is_per_ip() {
    let app = test_app().await;

    for _ in 0..5 {
        let _ = app
            .clone()
            .oneshot(subscription_request("198.51.100.1"))
            .await
            .expect("request failed");
    }

    // A different IP should not be affected.
    let resp = app
        .oneshot(subscription_request("198.51.100.2"))
        .await
        .expect("request failed");
    assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}
```

File: tests/security/security_headers_tests.rs
Current content:
```rust
//! Security header integration tests.
//!
//! Covers issue #1269 (CSP, HSTS, nosniff on Swagger UI and API responses).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

async fn test_app() -> axum::Router {
    soroban_pulse::test_support::build_test_router().await
}

fn assert_security_headers(headers: &axum::http::HeaderMap, context: &str) {
    let csp = headers
        .get("content-security-policy")
        .unwrap_or_else(|| panic!("{context}: missing Content-Security-Policy header"))
        .to_str()
        .expect("CSP header not valid ASCII");
    assert!(
        csp.contains("default-src"),
        "{context}: CSP should define default-src"
    );
    assert!(
        csp.contains("script-src"),
        "{context}: CSP should define script-src"
    );

    let hsts = headers
        .get("strict-transport-security")
        .unwrap_or_else(|| panic!("{context}: missing Strict-Transport-Security header"))
        .to_str()
        .expect("HSTS header not valid ASCII");
    assert!(
        hsts.contains("max-age=31536000"),
        "{context}: HSTS max-age should be 31536000, got {hsts}"
    );
    assert!(
        hsts.contains("includeSubDomains"),
        "{context}: HSTS should include includeSubDomains"
    );

    let nosniff = headers
        .get("x-content-type-options")
        .unwrap_or_else(|| panic!("{context}: missing X-Content-Type-Options header"))
        .to_str()
        .expect("nosniff header not valid ASCII");
    assert_eq!(
        nosniff, "nosniff",
        "{context}: X-Content-Type-Options should be nosniff"
    );
}

#[tokio::test]
async fn docs_endpoint_sets_security_headers() {
    let app = test_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::OK);
    assert_security_headers(resp.headers(), "/docs");
}

#[tokio::test]
async fn api_responses_set_security_headers() {
    let app = test_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    assert_security_headers(resp.headers(), "/v1/health");
}

#[tokio::test]
async fn docs_csp_restricts_scripts_to_swagger_cdn_or_self() {
    let app = test_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("request failed");

    let csp = resp
        .headers()
        .get("content-security-policy")
        .expect("missing CSP")
        .to_str()
        .unwrap();

    // Scripts must be limited to self and/or the Swagger CDN.
    assert!(
        csp.contains("'self'") || csp.contains("cdn.jsdelivr.net"),
        "CSP script-src should restrict to 'self' or Swagger CDN, got: {csp}"
    );
    assert!(
        !csp.contains("script-src *") && !csp.contains("script-src 'unsafe-eval'"),
        "CSP must not allow wildcard or unsafe-eval scripts, got: {csp}"
    );
}
```

File: tests/security/reencryption_tests.rs
Current content:
```rust
//! Encryption key rotation integration tests.
//!
//! Covers issue #1270 (admin re-encryption endpoint, transactional batches,
//! resumability, and old-key invalidation).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use soroban_pulse::crypto::{decrypt_credential, encrypt_credential};
use soroban_pulse::test_support::{build_test_router, seed_credentials};

const ADMIN_KEY: &str = "test-admin-key";

fn rotate_request(admin_key: Option<&str>, new_key: &str) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/v1/admin/encryption/rotate")
        .header("content-type", "application/json");
    if let Some(k) = admin_key {
        builder = builder.header("x-admin-api-key", k);
    }
    builder
        .body(Body::from(format!(r#"{{"new_key":"{new_key}"}}"#)))
        .unwrap()
}

#[tokio::test]
async fn rotate_requires_admin_api_key() {
    let app = build_test_router().await;

    // Missing key.
    let resp = app
        .clone()
        .oneshot(rotate_request(None, "new-key-32-bytes-long-aaaaaaaaaaaa"))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Wrong key.
    let resp = app
        .oneshot(rotate_request(Some("wrong-key"), "new-key-32-bytes-long-aaaaaaaaaaaa"))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn rotate_reencrypts_credentials_and_invalidates_old_key() {
    let app = build_test_router().await;

    let old_key = "old-key-32-bytes-long-aaaaaaaaaaaaaa";
    let new_key = "new-key-32-bytes-long-bbbbbbbbbbbbbb";

    // Seed credentials encrypted with the old key.
    let ids = seed_credentials(&app, old_key, 250).await;
    assert_eq!(ids.len(), 250);

    let resp = app
        .clone()
        .oneshot(rotate_request(Some(ADMIN_KEY), new_key))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::OK);

    // All credentials should now decrypt with the new key.
    for id in &ids {
        let ciphertext = soroban_pulse::test_support::fetch_ciphertext(&app, *id).await;
        assert!(
            decrypt_credential(&ciphertext, new_key).is_ok(),
            "credential {id} should decrypt with new key"
        );
        assert!(
            decrypt_credential(&ciphertext, old_key).is_err(),
            "credential {id} should NOT decrypt with old key after rotation"
        );
    }
}

#[tokio::test]
async fn rotate_reports_progress_gauge() {
    let app = build_test_router().await;
    let old_key = "old-key-32-bytes-long-aaaaaaaaaaaaaa";
    let new_key = "new-key-32-bytes-long-bbbbbbbbbbbbbb";

    seed_credentials(&app, old_key, 120).await;

    let resp = app
        .oneshot(rotate_request(Some(ADMIN_KEY), new_key))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::OK);

    let gauge = soroban_pulse::metrics::Metrics::global()
        .reencryption_progress
        .get();
    // Gauge is expressed as a fraction 0.0..=1.0; after a complete run it
    // should be 1.0.
    assert!(
        (gauge - 1.0).abs() < f64::EPSILON,
        "reencryption progress gauge should be 1.0 after completion, got {gauge}"
    );
}

#[tokio::test]
async fn rotate_resumes_after_interruption() {
    let app = build_test_router().await;
    let old_key = "old-key-32-bytes-long-aaaaaaaaaaaaaa";
    let new_key = "new-key-32-bytes-long-bbbbbbbbbbbbbb";

    // Seed enough rows to span multiple batches (batch size = 100).
    let ids = seed_credentials(&app, old_key, 250).await;

    // Simulate an interruption after the first batch by configuring the
    // test support hook to abort the job after N rows.
    soroban_pulse::test_support::set_reencryption_abort_after(100);

    let resp = app
        .clone()
        .oneshot(rotate_request(Some(ADMIN_KEY), new_key))
        .await
        .expect("request failed");
    // Interrupted run should surface a non-success status (server error or
    // accepted) but must not leave the system in a broken state.
    assert!(
        resp.status() == StatusCode::INTERNAL_SERVER_ERROR
            || resp.status() == StatusCode::ACCEPTED,
        "interrupted rotation should return 5xx or 202, got {}",
        resp.status()
    );

    // Clear the abort hook and resume.
    soroban_pulse::test_support::set_reencryption_abort_after(usize::MAX);

    let resp = app
        .clone()
        .oneshot(rotate_request(Some(ADMIN_KEY), new_key))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::OK);

    // Every credential must now decrypt with the new key and fail with old.
    for id in &ids {
        let ciphertext = soroban_pulse::test_support::fetch_ciphertext(&app, *id).await;
        assert!(
            decrypt_credential(&ciphertext, new_key).is_ok(),
            "credential {id} should decrypt with new key after resume"
        );
        assert!(
            decrypt_credential(&ciphertext, old_key).is_err(),
            "credential {id} should NOT decrypt with old key after resume"
        );
    }
}

#[tokio::test]
async fn encrypt_decrypt_roundtrip_sanity() {
    // Sanity check that the crypto helpers used by the tests behave as
    // expected, independent of the rotation endpoint.
    let key = "sanity-key-32-bytes-long-ccccccccccc";
    let plaintext = b"super-secret-token";
    let ct = encrypt_credential(plaintext, key).expect("encrypt");
    let pt = decrypt_credential(&ct, key).expect("decrypt");
    assert_eq!(pt, plaintext);
    assert!(decrypt_credential(&ct, "different-key-32-bytes-long-ddddddddd").is_err());
}
```

File: tests/security/callback_allowlist_tests.rs
Current content:
```rust
//! Callback URL allowlist integration tests.
//!
//! Covers issue #1271 (validate OAuth callback URLs against
//! ALLOWED_CALLBACK_DOMAINS for GitHub, Slack, and Telegram integrations).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use soroban_pulse::test_support::build_test_router_with_callback_allowlist;

const ALLOWED: &[&str] = &["app.example.com", "hooks.example.org"];

fn integration_request(provider: &str, callback_url: &str) -> Request<Body> {
    let uri = format!("/v1/integrations/{provider}");
    Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"callback_url":"{callback_url}"}}"#
        )))
        .unwrap()
}

async fn app() -> axum::Router {
    build_test_router_with_callback_allowlist(ALLOWED).await
}

#[tokio::test]
async fn github_rejects_unlisted_callback_domain() {
    let app = app().await;
    let resp = app
        .oneshot(integration_request(
            "github",
            "https://evil.attacker.com/oauth/callback",
        ))
        .await
        .expect("request failed");

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body);
    assert!(
        text.to_lowercase().contains("callback") || text.to_lowercase().contains("domain"),
        "error message should mention callback/domain, got: {text}"
    );
}

#[tokio::test]
async fn slack_rejects_unlisted_callback_domain() {
    let app = app().await;
    let resp = app
        .oneshot(integration_request(
            "slack",
            "https://evil.attacker.com/oauth/callback",
        ))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn telegram_rejects_unlisted_callback_domain() {
    let app = app().await;
    let resp = app
        .oneshot(integration_request(
            "telegram",
            "https://evil.attacker.com/oauth/callback",
        ))
        .await
        .expect("request failed");
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn allowed_domains_pass_validation() {
    let app = app().await;

    for provider in ["github", "slack", "telegram"] {
        for url in [
            "https://app.example.com/oauth/callback",
            "https://hooks.example.org/cb",
            "https://sub.app.example.com/oauth/callback",
        ] {
            let resp = app
                .clone()
                .oneshot(integration_request(provider, url))
                .await
                .expect("request failed");
            assert_ne!(
                resp.status(),
                StatusCode::BAD_REQUEST,
                "{provider} should accept allowed callback {url}"
            );
        }
    }
}

#[tokio::test]
async fn rejects_lookalike_domains() {
    let app = app().await;
    // Domains that merely *contain* an allowed domain as a substring must be
    // rejected (e.g. app.example.com.evil.com).
    for url in [
        "https://app.example.com.evil.com/cb",
        "https://evil-app.example.com/cb",
        "https://example.com/cb",
    ] {
        let resp = app
            .clone()
            .oneshot(integration_request("github", url))
            .await
            .expect("request failed");
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "lookalike domain {url} should be rejected"
        );
    }
}

#[tokio::test]
async fn rejects_non_http_schemes() {
    let app = app().await;
    for url in [
        "javascript:alert(1)",
        "file:///etc/passwd",
        "ftp://app.example.com/cb",
    ] {
        let resp = app
            .clone()
            .oneshot(integration_request("github", url))
            .await
            .expect("request failed");
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "non-http(s) scheme {url} should be rejected"
        );
    }
}