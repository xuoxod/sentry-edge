//! Sentry SQLite WAL Persistence Ledger

use rusqlite::{params, Connection};
use sentry_core::{SentryAlert, SentryError, SentryIncidentType, SentryResult};

pub struct SentryLedgerDb {
    conn: Connection,
}

impl SentryLedgerDb {
    pub fn new_in_memory() -> SentryResult<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| SentryError::Ledger(format!("SQLite open error: {}", e)))?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    pub fn new(path: &str) -> SentryResult<Self> {
        let conn = Connection::open(path)
            .map_err(|e| SentryError::Ledger(format!("SQLite open error: {}", e)))?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")
            .map_err(|e| SentryError::Ledger(format!("PRAGMA error: {}", e)))?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> SentryResult<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sentry_alerts (
                alert_id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                node_id TEXT NOT NULL,
                node_label TEXT NOT NULL,
                severity TEXT NOT NULL,
                incident_type TEXT NOT NULL,
                description TEXT NOT NULL,
                signature_sha256 TEXT NOT NULL
            );"
        ).map_err(|e| SentryError::Ledger(format!("Init schema error: {}", e)))?;
        Ok(())
    }

    pub fn insert_alert(&self, alert: &SentryAlert) -> SentryResult<()> {
        let severity_str = format!("{:?}", alert.severity);
        let type_str = match &alert.incident {
            SentryIncidentType::AcousticSpike { .. } => "ACOUSTIC_SPIKE",
            SentryIncidentType::MotionDetected { .. } => "MOTION_DETECTED",
            SentryIncidentType::TamperDetected { .. } => "TAMPER_DETECTED",
            SentryIncidentType::ManualPanic { .. } => "MANUAL_PANIC",
            SentryIncidentType::PeriodicHeartbeat => "HEARTBEAT",
        };

        self.conn.execute(
            "INSERT INTO sentry_alerts (
                alert_id, timestamp, node_id, node_label, severity, incident_type, description, signature_sha256
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                alert.alert_id.to_string(),
                alert.timestamp.to_rfc3339(),
                alert.node_id.to_string(),
                alert.node_label,
                severity_str,
                type_str,
                alert.description,
                alert.event_signature_sha256,
            ],
        ).map_err(|e| SentryError::Ledger(format!("Insert alert error: {}", e)))?;

        Ok(())
    }

    pub fn count_alerts(&self) -> SentryResult<usize> {
        let count: i64 = self.conn
            .query_row("SELECT COUNT(*) FROM sentry_alerts", [], |r| r.get(0))
            .map_err(|e| SentryError::Ledger(format!("Count error: {}", e)))?;
        Ok(count as usize)
    }

    pub fn fetch_recent_alerts(&self, limit: usize) -> SentryResult<Vec<(String, String, String, String)>> {
        let mut stmt = self.conn
            .prepare("SELECT alert_id, severity, incident_type, description FROM sentry_alerts ORDER BY timestamp DESC LIMIT ?1")
            .map_err(|e| SentryError::Ledger(format!("Prepare error: {}", e)))?;

        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|e| SentryError::Ledger(format!("Query error: {}", e)))?;

        let mut results = Vec::new();
        for r in rows {
            if let Ok(item) = r {
                results.push(item);
            }
        }
        Ok(results)
    }
}
