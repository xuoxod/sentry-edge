//! Sentry Zero-CDN Multi-Format Dossier Generator

use sentry_core::SentryResult;

pub struct SentryReportEngine;

impl SentryReportEngine {
    /// Generate dark-mode HTML dossier with embedded SVG charts.
    pub fn generate_html_dossier(node_label: &str, alert_count: usize) -> SentryResult<String> {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n");
        html.push_str(&format!("<title>SENTRY-EDGE // Security Dossier - {}</title>\n", node_label));
        html.push_str("<style>\n");
        html.push_str("body { background-color: #0d1117; color: #c9d1d9; font-family: monospace; padding: 24px; }\n");
        html.push_str(".card { background-color: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 20px; margin-bottom: 20px; }\n");
        html.push_str("h1, h2 { color: #58a6ff; margin-top: 0; }\n");
        html.push_str(".stat-pill { display: inline-block; background: #238636; color: #fff; padding: 4px 12px; border-radius: 12px; font-weight: bold; font-size: 14px; }\n");
        html.push_str("</style>\n</head>\n<body>\n<div class=\"card\">\n");
        html.push_str("  <h1>🛡️ SENTRY-EDGE Security & Telepresence Dossier</h1>\n");
        html.push_str(&format!("  <p><strong>Node Identity:</strong> {}</p>\n", node_label));
        html.push_str(&format!("  <p><strong>Total Security Events Recorded:</strong> <span class=\"stat-pill\">{}</span></p>\n", alert_count));
        html.push_str("  <h2>📊 Sovereign Acoustic Incident Spectrum</h2>\n");
        html.push_str("  <svg width=\"200\" height=\"200\" viewBox=\"0 0 42 42\">\n");
        html.push_str("    <circle cx=\"21\" cy=\"21\" r=\"15.915\" fill=\"transparent\" stroke=\"#21262d\" stroke-width=\"4\"></circle>\n");
        html.push_str("    <circle cx=\"21\" cy=\"21\" r=\"15.915\" fill=\"transparent\" stroke=\"#238636\" stroke-width=\"4\" stroke-dasharray=\"75 25\" stroke-dashoffset=\"25\"></circle>\n");
        html.push_str("    <circle cx=\"21\" cy=\"21\" r=\"15.915\" fill=\"transparent\" stroke=\"#da3633\" stroke-width=\"4\" stroke-dasharray=\"25 75\" stroke-dashoffset=\"0\"></circle>\n");
        html.push_str("  </svg>\n</div>\n</body>\n</html>");
        Ok(html)
    }
}
