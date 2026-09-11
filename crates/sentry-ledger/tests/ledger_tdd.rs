//! TDD Test: Sentry SQLite WAL Ledger Persistence & Query Engine

use sentry_core::{IncidentSeverity, SentryAlert, SentryIncidentType};
use sentry_ledger::SentryLedgerDb;
use uuid::Uuid;

#[test]
fn test_sentry_ledger_in_memory_crud() {
    let db = SentryLedgerDb::new_in_memory().unwrap();
    assert_eq!(db.count_alerts().unwrap(), 0);

    let alert1 = SentryAlert::new(
        Uuid::new_v4(),
        "Sentinel-Alpha",
        IncidentSeverity::High,
        SentryIncidentType::MotionDetected { delta_score: 0.85 },
        "Motion detected in camera frame",
        None,
    );

    let alert2 = SentryAlert::new(
        Uuid::new_v4(),
        "Sentinel-Alpha",
        IncidentSeverity::Critical,
        SentryIncidentType::AcousticSpike {
            peak_db: 88.5,
            baseline_db: 52.0,
            delta_db: 36.5,
        },
        "Acoustic threshold breach",
        None,
    );

    db.insert_alert(&alert1).unwrap();
    db.insert_alert(&alert2).unwrap();
    assert_eq!(db.count_alerts().unwrap(), 2);

    let alerts = db.fetch_recent_alerts(10).unwrap();
    assert_eq!(alerts.len(), 2);
    assert_eq!(alerts[0].3, "Acoustic threshold breach");
}
