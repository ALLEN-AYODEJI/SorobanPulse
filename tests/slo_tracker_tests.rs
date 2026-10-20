// Integration tests for the SLI / SLO tracker (Issue #696).
//
// Exercises the tracker outside of the application context and verifies
// that the metric helpers and aggregate-report helpers behave as documented
// in `docs/sli-slo.md`.

use soroban_pulse::slo_tracker::{
    default_slo_definitions, record_sli_sample, shared_tracker, SliType, SloDefinition,
    SloReport, SloStatus, SloTracker,
};

#[test]
fn default_definitions_cover_documented_slos() {
    let defaults = default_slo_definitions();
    let names: Vec<&str> = defaults.iter().map(|d| d.name.as_str()).collect();
    // Each SLO referenced in `docs/sli-slo.md` must exist in the defaults.
    for required in [
        "http_availability",
        "http_p99_latency",
        "indexer_lag",
        "webhook_delivery_success",
        "notification_delivery_latency",
        "replica_replay_lag",
    ] {
        assert!(
            names.contains(&required),
            "missing default SLO definition: {required}"
        );
    }
}

#[test]
fn empty_tracker_reports_met_when_no_samples() {
    let tracker = SloTracker::with_defaults();
    let report = tracker.generate_report();
    for slo in &report.slos {
        if slo.sample_count == 0 {
            assert_eq!(
                slo.status,
                SloStatus::Met,
                "{} reported as {:?} with zero samples",
                slo.name,
                slo.status
            );
        }
    }
}

#[tokio::test]
async fn recording_only_bad_samples_breaches_error_rate_slo() {
    let tracker = SloTracker::with_defaults();
    for _ in 0..20 {
        record_sli_sample(&tracker, "http_availability", 0.0).await;
    }
    let report = tracker.generate_report();
    let http = report
        .slos
        .iter()
        .find(|s| s.name == "http_availability")
        .expect("http_availability report present");
    assert_eq!(http.status, SloStatus::Breached);
    assert!(http.completion_ratio < 0.5);
    assert!(http.burn_rate >= 1.0);
}

#[tokio::test]
async fn recording_only_good_samples_keeps_slo_met() {
    let tracker = SloTracker::with_defaults();
    for _ in 0..50 {
        record_sli_sample(&tracker, "http_availability", 1.0).await;
    }
    let report = tracker.generate_report();
    let http = report
        .slos
        .iter()
        .find(|s| s.name == "http_availability")
        .expect("http_availability report present");
    assert_eq!(http.status, SloStatus::Met);
    assert!(http.completion_ratio >= 0.99);
}

#[tokio::test]
async fn high_burn_rate_drives_breach_even_with_partial_samples() {
    let tracker = SloTracker::with_defaults();
    // Mixed: half bad, half good — completion ratio is 0.5; with target 99%
    // and 1% budget the burn rate is 50×, well above the 2× fast-burn threshold.
    for _ in 0..10 {
        record_sli_sample(&tracker, "http_availability", 0.0).await;
        record_sli_sample(&tracker, "http_availability", 1.0).await;
    }
    let report = tracker.generate_report();
    let http = report
        .slos
        .iter()
        .find(|s| s.name == "http_availability")
        .expect("http_availability report present");
    assert_eq!(http.status, SloStatus::Breached);
    assert!(http.burn_rate > 2.0);
}

#[test]
fn latency_slo_scores_under_target_as_good() {
    let def = SloDefinition {
        name: "test_latency".to_string(),
        description: "test".to_string(),
        component: "test".to_string(),
        sli_type: SliType::Latency,
        target: 0.25,
        window_secs: 3600,
    };
    let mut tracker = SloTracker::new();
    tracker.register(def);
    for _ in 0..9 {
        tracker.record_sample("test_latency", 0.10);
    }
    tracker.record_sample("test_latency", 0.50); // breach

    let report = tracker.generate_report();
    let r = report
        .slos
        .iter()
        .find(|s| s.name == "test_latency")
        .expect("test_latency report present");
    assert_eq!(r.sample_count, 10);
    assert_eq!(r.good_count, 9);
    assert!(
        r.completion_ratio > 0.85 && r.completion_ratio < 0.95,
        "completion ratio {} not in expected range",
        r.completion_ratio
    );
    assert!(matches!(r.status, SloStatus::AtRisk | SloStatus::Breached));
}

#[test]
fn unknown_slo_name_is_rejected_at_record_time() {
    let mut tracker = SloTracker::with_defaults();
    let accepted = tracker.record_sample("not_a_real_slo", 1.0);
    assert!(!accepted);
    assert_eq!(tracker.sample_count("not_a_real_slo"), 0);
}

#[test]
fn report_round_trips_to_value() {
    use serde_json::json;

    let tracker = SloTracker::with_defaults();
    let report: SloReport = tracker
        .generate_report()
        .slos
        .first()
        .expect("at least one SLO")
        .clone();
    let value = serde_json::to_value(&report).expect("serializable");
    assert_eq!(value["name"], json!(report.name));
    assert_eq!(value["status"], json!(report.status.as_str()));
    assert_eq!(value["sli_type"], json!(report.sli_type));
}

#[tokio::test]
async fn helper_returns_a_cloneable_shared_tracker() {
    use std::sync::Arc;

    let tracker = shared_tracker();
    record_sli_sample(&tracker, "http_availability", 1.0).await;
    let again = tracker.clone();
    assert!(Arc::ptr_eq(&tracker, &again));
    assert_eq!(tracker.read().await.sample_count("http_availability"), 1);
}
