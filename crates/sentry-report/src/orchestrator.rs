//! Sentry Report Single Source of Truth (SST) Orchestrator Engine

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use rusqlite::Connection;

use crate::error::Result;
use crate::formatters::{
    CsvReportFormatter, HtmlReportFormatter, JsonReportFormatter, JsonlReportFormatter,
    MarkdownReportFormatter, ReportFormatter, TextReportFormatter,
};
use crate::models::{ReportDocument, ReportFormat};
use crate::storyteller::Storyteller;
use sentry_telemetry::TelemetryRecord;

pub struct ReportOrchestrator;

impl ReportOrchestrator {
    /// Ingest in-memory slice of telemetry records and construct the SST ReportDocument.
    pub fn from_records(
        node_id: &str,
        platform_info: &str,
        records: &[TelemetryRecord],
    ) -> ReportDocument {
        Storyteller::build_document(node_id, platform_info, records)
    }

    /// Ingest records from a JSONL audit log file (e.g. sentry_audit.jsonl) and construct the SST ReportDocument.
    pub fn from_jsonl_file(path: &Path, node_id: &str, platform_info: &str) -> Result<ReportDocument> {
        if !path.exists() {
            return Ok(Storyteller::build_document(node_id, platform_info, &[]));
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(record) = serde_json::from_str::<TelemetryRecord>(trimmed) {
                records.push(record);
            }
        }

        Ok(Storyteller::build_document(node_id, platform_info, &records))
    }

    /// Ingest records from a SQLite audit ledger database (sentry_ledger.db).
    pub fn from_ledger_db(db_path: &Path, node_id: &str, platform_info: &str) -> Result<ReportDocument> {
        if !db_path.exists() {
            return Ok(Storyteller::build_document(node_id, platform_info, &[]));
        }

        let conn = Connection::open(db_path)?;
        let mut stmt = conn.prepare(
            "SELECT record_json FROM telemetry_audit_ledger ORDER BY sequence ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            let json_str: String = row.get(0)?;
            Ok(json_str)
        })?;

        let mut records = Vec::new();
        for json_res in rows {
            if let Ok(json_str) = json_res {
                if let Ok(record) = serde_json::from_str::<TelemetryRecord>(&json_str) {
                    records.push(record);
                }
            }
        }

        Ok(Storyteller::build_document(node_id, platform_info, &records))
    }

    /// Render SST ReportDocument to target format string.
    pub fn render(document: &ReportDocument, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Html => HtmlReportFormatter::format(document),
            ReportFormat::Json => JsonReportFormatter::format(document),
            ReportFormat::Jsonl => JsonlReportFormatter::format(document),
            ReportFormat::Csv => CsvReportFormatter::format(document),
            ReportFormat::Text => TextReportFormatter::format(document),
            ReportFormat::Markdown => MarkdownReportFormatter::format(document),
        }
    }

    /// Render and save SST ReportDocument directly to a file destination.
    pub fn render_to_file(document: &ReportDocument, format: ReportFormat, destination: &Path) -> Result<()> {
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let rendered = Self::render(document, format)?;
        let mut file = File::create(destination)?;
        file.write_all(rendered.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Infer target format from file extension.
    pub fn infer_format(path: &Path) -> ReportFormat {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext.to_lowercase().as_str() {
                "html" | "htm" => ReportFormat::Html,
                "json" => ReportFormat::Json,
                "jsonl" | "ndjson" => ReportFormat::Jsonl,
                "csv" => ReportFormat::Csv,
                "txt" | "log" => ReportFormat::Text,
                "md" | "markdown" => ReportFormat::Markdown,
                _ => ReportFormat::Html,
            }
        } else {
            ReportFormat::Html
        }
    }
}
