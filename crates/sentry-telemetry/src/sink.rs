use colored::Colorize;
use rusqlite::{params, Connection};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use crate::error::Result;
use crate::models::{Severity, TelemetryRecord};

pub struct ConsoleSink;

impl ConsoleSink {
    pub fn emit(record: &TelemetryRecord) {
        let sev_str = match record.severity {
            Severity::Trace => "[TRACE]".bright_black(),
            Severity::Debug => "[DEBUG]".bright_cyan(),
            Severity::Info => "[INFO ]".bright_green(),
            Severity::Warn => "[WARN ]".bright_yellow().bold(),
            Severity::Alert => "[ALERT]".bright_red().bold(),
            Severity::Security => "[SEC  ]".on_red().white().bold(),
        };

        let sub_str = format!("[{}]", record.subsystem.as_str()).bright_blue();
        let ts_str = &record.timestamp_utc[11..29]; // HH:MM:SS.NNNNNNNNN

        let duration_info = if record.what.duration_ps > 0 {
            format!("(took {}ns / {}ps)", record.what.duration_ns, record.what.duration_ps).bright_black()
        } else {
            "".normal()
        };

        let db_info = if let (Some(rms), Some(base)) = (record.what.rms_db, record.what.baseline_db) {
            format!("[RMS: {:.1}dB, Base: {:.1}dB, Δ: +{:.1}dB]", rms, base, record.what.delta_db.unwrap_or(0.0)).bright_magenta()
        } else {
            "".normal()
        };

        println!(
            "{} {} {} {} {} {} {}",
            ts_str.bright_black(),
            sev_str,
            sub_str,
            record.what.summary.bright_white(),
            db_info,
            duration_info,
            format!("#{}", record.sequence).bright_black()
        );
    }
}

pub struct FileSink {
    path: PathBuf,
    file: Mutex<File>,
}

impl FileSink {
    pub fn new(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            path: path.to_path_buf(),
            file: Mutex::new(file),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write_record(&self, record: &TelemetryRecord) -> Result<()> {
        let json_line = serde_json::to_string(record)?;
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", json_line)?;
        file.flush()?;
        Ok(())
    }
}

pub struct LedgerSink {
    conn: Arc<Mutex<Connection>>,
}

impl LedgerSink {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Result<Self> {
        {
            let conn_guard = conn.lock().unwrap();
            conn_guard.execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS telemetry_audit_ledger (
                    sequence INTEGER PRIMARY KEY,
                    timestamp_utc TEXT NOT NULL,
                    timestamp_unix_ns TEXT NOT NULL,
                    node_id TEXT NOT NULL,
                    subsystem TEXT NOT NULL,
                    severity TEXT NOT NULL,
                    summary TEXT NOT NULL,
                    rms_db REAL,
                    baseline_db REAL,
                    delta_db REAL,
                    duration_ns INTEGER NOT NULL,
                    duration_ps TEXT NOT NULL,
                    record_json TEXT NOT NULL,
                    prev_record_hash TEXT NOT NULL,
                    record_hash TEXT NOT NULL
                );
                CREATE INDEX IF NOT EXISTS idx_telemetry_subsystem ON telemetry_audit_ledger(subsystem);
                CREATE INDEX IF NOT EXISTS idx_telemetry_timestamp ON telemetry_audit_ledger(timestamp_unix_ns);
                "#,
            )?;
        }
        Ok(Self { conn })
    }

    pub fn insert_record(&self, record: &TelemetryRecord) -> Result<()> {
        let json_str = serde_json::to_string(record)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO telemetry_audit_ledger (
                sequence, timestamp_utc, timestamp_unix_ns, node_id, subsystem, severity,
                summary, rms_db, baseline_db, delta_db, duration_ns, duration_ps,
                record_json, prev_record_hash, record_hash
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            "#,
            params![
                record.sequence as i64,
                record.timestamp_utc,
                record.timestamp_unix_ns.to_string(),
                record.node_id,
                record.subsystem.as_str(),
                record.severity.as_str(),
                record.what.summary,
                record.what.rms_db.map(|v| v as f64),
                record.what.baseline_db.map(|v| v as f64),
                record.what.delta_db.map(|v| v as f64),
                record.what.duration_ns as i64,
                record.what.duration_ps.to_string(),
                json_str,
                record.prev_record_hash,
                record.record_hash,
            ],
        )?;
        Ok(())
    }
}
