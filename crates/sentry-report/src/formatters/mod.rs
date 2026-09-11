//! Sentry Report Multi-Format Exporters

pub mod csv;
pub mod html;
pub mod json;
pub mod jsonl;
pub mod markdown;
pub mod text;

pub use self::csv::CsvReportFormatter;
pub use self::html::HtmlReportFormatter;
pub use self::json::JsonReportFormatter;
pub use self::jsonl::JsonlReportFormatter;
pub use self::markdown::MarkdownReportFormatter;
pub use self::text::TextReportFormatter;

use crate::error::Result;
use crate::models::ReportDocument;

pub trait ReportFormatter {
    fn format(document: &ReportDocument) -> Result<String>;
}
