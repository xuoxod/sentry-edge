use rusqlite::Connection;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::error::Result;
use crate::models::{
    FromProvenance, HowProvenance, MinutiaeMetadata, Severity, SubsystemTag, TelemetryRecord,
    ToProvenance, WhatTelemetry, WhoProvenance,
};
use crate::sink::{ConsoleSink, FileSink, LedgerSink};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TelemetryStats {
    pub total_records: u64,
    pub subsystem_counts: HashMap<String, u64>,
    pub severity_counts: HashMap<String, u64>,
    pub avg_baseline_db: f32,
    pub max_rms_db: f32,
    pub total_triggers: u64,
    pub avg_duration_ns: u64,
}

pub struct TelemetryEngine {
    node_id: String,
    sequence: Mutex<u64>,
    last_hash: Mutex<String>,
    file_sink: Option<FileSink>,
    ledger_sink: Option<LedgerSink>,
    echo_console: bool,
}

impl TelemetryEngine {
    pub fn new(node_id: &str, echo_console: bool) -> Self {
        Self {
            node_id: node_id.to_string(),
            sequence: Mutex::new(0),
            last_hash: Mutex::new("GENESIS_BLOCK_0000000000000000000000000000000000000000000000000000000000000000".to_string()),
            file_sink: None,
            ledger_sink: None,
            echo_console,
        }
    }

    pub fn with_file_sink(mut self, path: &Path) -> Result<Self> {
        if let Some(last_rec) = FileSink::get_last_record(path) {
            *self.sequence.lock().unwrap() = last_rec.sequence;
            *self.last_hash.lock().unwrap() = last_rec.record_hash;
        }
        self.file_sink = Some(FileSink::new(path)?);
        Ok(self)
    }

    pub fn with_ledger_sink(mut self, conn: Arc<Mutex<Connection>>) -> Result<Self> {
        {
            let conn_guard = conn.lock().unwrap();
            let stmt = conn_guard.prepare("SELECT sequence, record_hash FROM telemetry_audit_ledger ORDER BY sequence DESC LIMIT 1").ok();
            if let Some(mut s) = stmt {
                if let Ok(row) = s.query_row([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))) {
                    let mut seq_guard = self.sequence.lock().unwrap();
                    if (row.0 as u64) > *seq_guard {
                        *seq_guard = row.0 as u64;
                        *self.last_hash.lock().unwrap() = row.1;
                    }
                }
            }
        }
        self.ledger_sink = Some(LedgerSink::new(conn)?);
        Ok(self)
    }

    pub fn log(
        &self,
        subsystem: SubsystemTag,
        severity: Severity,
        who: WhoProvenance,
        from: FromProvenance,
        to: ToProvenance,
        what: WhatTelemetry,
        how: HowProvenance,
        minutiae: MinutiaeMetadata,
    ) -> Result<TelemetryRecord> {
        let mut seq_guard = self.sequence.lock().unwrap();
        *seq_guard += 1;
        let seq = *seq_guard;

        let mut hash_guard = self.last_hash.lock().unwrap();
        let prev_hash = hash_guard.clone();

        let record = TelemetryRecord::new(
            seq,
            &self.node_id,
            subsystem,
            severity,
            who,
            from,
            to,
            what,
            how,
            minutiae,
            &prev_hash,
        );

        *hash_guard = record.record_hash.clone();

        if self.echo_console {
            ConsoleSink::emit(&record);
        }

        if let Some(fs) = &self.file_sink {
            fs.write_record(&record)?;
        }

        if let Some(ls) = &self.ledger_sink {
            ls.insert_record(&record)?;
        }

        Ok(record)
    }

    pub fn current_sequence(&self) -> u64 {
        *self.sequence.lock().unwrap()
    }

    pub fn last_record_hash(&self) -> String {
        self.last_hash.lock().unwrap().clone()
    }
}
