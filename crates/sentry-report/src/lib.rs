//! # Sentry Report (`sentry-report`)
//! Enterprise Multi-Format Zero-CDN Human-Consumable Report Engine & Single Source of Truth (SST) Orchestrator.
//! Supports HTML, JSON, JSONL, CSV, Text, and Markdown with narrative storytelling and cryptographic integrity proofs.

pub mod error;
pub mod formatters;
pub mod models;
pub mod orchestrator;
pub mod security;
pub mod storyteller;

pub use error::{ReportError, Result};
pub use formatters::{
    CsvReportFormatter, HtmlReportFormatter, JsonReportFormatter, JsonlReportFormatter,
    MarkdownReportFormatter, ReportFormatter, TextReportFormatter,
};
pub use models::{
    AcousticInsights, ExecutiveAssessment, ExecutiveSummary, ForensicEvent, ReportDocument,
    ReportFormat, ReportMetadata,
};
pub use orchestrator::ReportOrchestrator;
pub use storyteller::Storyteller;
