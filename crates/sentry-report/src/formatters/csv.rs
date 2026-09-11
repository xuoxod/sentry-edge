//! Sentry RFC 4180 CSV Report Formatter with Formula Injection Protections

use super::ReportFormatter;
use crate::error::Result;
use crate::models::ReportDocument;
use crate::security::sanitize_csv_cell;

pub struct CsvReportFormatter;

impl ReportFormatter for CsvReportFormatter {
    fn format(document: &ReportDocument) -> Result<String> {
        let mut buffer = String::new();

        // 1. Comments / Metadata Header block
        buffer.push_str(&format!("# SENTRY-EDGE SECURITY AUDIT DOSSIER - {}\n", sanitize_csv_cell(&document.metadata.report_id)));
        buffer.push_str(&format!("# Node: {}, Platform: {}, Assessment: {}\n", sanitize_csv_cell(&document.metadata.node_id), sanitize_csv_cell(&document.metadata.platform), document.summary.assessment.as_str()));
        buffer.push_str(&format!("# Period: {} to {} ({})\n", &document.metadata.session_start_utc, &document.metadata.session_end_utc, &document.metadata.elapsed_human));
        buffer.push_str(&format!("# Hash Chain Integrity: {}\n", if document.metadata.is_chain_valid { "100% VALID" } else { "TAMPER DETECTED" }));
        buffer.push_str("# -----------------------------------------------------------------------\n");

        // 2. CSV Column Headers
        buffer.push_str("Sequence,Timestamp_UTC,Timestamp_Unix_NS,Subsystem,Severity,Summary,RMS_dB_SPL,Baseline_dB_SPL,Delta_dB,Duration_NS,Duration_PS,Prev_Hash,Record_Hash\n");

        // 3. Rows
        for event in &document.timeline {
            let seq_str = event.sequence.to_string();
            let ts_utc = sanitize_csv_cell(&event.timestamp_utc);
            let ts_ns = sanitize_csv_cell(&event.timestamp_unix_ns);
            let sub = sanitize_csv_cell(&event.subsystem);
            let sev = sanitize_csv_cell(&event.severity);
            let sum = sanitize_csv_cell(&event.summary);
            let rms = event.rms_db.map(|v| format!("{:.1}", v)).unwrap_or_default();
            let base = event.baseline_db.map(|v| format!("{:.1}", v)).unwrap_or_default();
            let delta = event.delta_db.map(|v| format!("{:.1}", v)).unwrap_or_default();
            let dur_ns = event.duration_ns.to_string();
            let dur_ps = sanitize_csv_cell(&event.duration_ps);
            let prev_hash = sanitize_csv_cell(&event.prev_hash);
            let record_hash = sanitize_csv_cell(&event.record_hash);

            buffer.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                seq_str, ts_utc, ts_ns, sub, sev, sum, rms, base, delta, dur_ns, dur_ps, prev_hash, record_hash
            ));
        }

        Ok(buffer)
    }
}
