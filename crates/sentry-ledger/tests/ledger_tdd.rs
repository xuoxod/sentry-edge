//! TDD Test: Sentry SQLite WAL Ledger & HTML Report Engine

use sentry_core::{IncidentSeverity, SentryAlert, SentryIncidentType};
use sentry_ledger::{SentryLedgerDb, SentryReportEngine};
use uuid::Uuid;

#[test]
fn test_sentry_ledger_in_memory_crud() {
    let db = SentryLedgerDb::new_in_memory().unwrap();
    assert_eq!(db.count_alerts().unwrap(), 0);

    let alert = SentryAlert::new(
        Uuid::new_v4(),
        "Sentinel-Alpha",
        IncidentSeverity::High,
        SentryIncidentType::MotionDetected { delta_score: 0.85 },
        "Motion detected in camera frame",
        None,
    );

    db.insert_alert(&alert).unwrap();
    assert_eq!(db.count_alerts().unwrap(), 1);
}

#[test]
fn test_sentry_report_engine_html() {
    let html = SentryReportEngine::generate_html_dossier("Sentinel-Alpha", 5).unwrap();
    assert!(html.contains("SENTRY-EDGE"));
    assert!(html.contains("Sentinel-Alpha"));
    assert!(html.contains("<svg"));
}
