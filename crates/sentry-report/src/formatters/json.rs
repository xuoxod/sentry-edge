//! Sentry Pretty JSON Report Formatter

use super::ReportFormatter;
use crate::error::Result;
use crate::models::ReportDocument;

pub struct JsonReportFormatter;

impl ReportFormatter for JsonReportFormatter {
    fn format(document: &ReportDocument) -> Result<String> {
        let json_str = serde_json::to_string_pretty(document)?;
        Ok(json_str)
    }
}
