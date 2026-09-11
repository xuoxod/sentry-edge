//! Sentry JSONL (NDJSON) Streaming Report Formatter

use super::ReportFormatter;
use crate::error::Result;
use crate::models::ReportDocument;

pub struct JsonlReportFormatter;

impl ReportFormatter for JsonlReportFormatter {
    fn format(document: &ReportDocument) -> Result<String> {
        let mut buffer = String::new();

        // 1. Header Manifest Line
        let manifest = serde_json::json!({
            "type": "REPORT_MANIFEST",
            "report_id": document.metadata.report_id,
            "node_id": document.metadata.node_id,
            "platform": document.metadata.platform,
            "generated_at_utc": document.metadata.generated_at_utc,
            "assessment": document.summary.assessment,
            "total_records": document.metadata.total_records,
            "is_chain_valid": document.metadata.is_chain_valid,
            "acoustics": document.acoustics,
        });
        buffer.push_str(&serde_json::to_string(&manifest)?);
        buffer.push('\n');

        // 2. Timeline Event Lines
        for event in &document.timeline {
            let event_line = serde_json::to_string(event)?;
            buffer.push_str(&event_line);
            buffer.push('\n');
        }

        Ok(buffer)
    }
}
