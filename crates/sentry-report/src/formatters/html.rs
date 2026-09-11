//! Sentry Zero-CDN Multi-Device Standalone HTML5 Report Formatter
//! Executive storytelling dashboard with embedded SVG analytics, responsive CSS, and tamper-evident audit badges.

use super::ReportFormatter;
use crate::error::Result;
use crate::models::ReportDocument;
use crate::security::escape_html;

pub struct HtmlReportFormatter;

impl ReportFormatter for HtmlReportFormatter {
    fn format(document: &ReportDocument) -> Result<String> {
        let mut html = String::with_capacity(32768);

        let report_id = escape_html(&document.metadata.report_id);
        let node_id = escape_html(&document.metadata.node_id);
        let platform = escape_html(&document.metadata.platform);
        let gen_time = escape_html(&document.metadata.generated_at_utc);
        let session_start = escape_html(&document.metadata.session_start_utc);
        let session_end = escape_html(&document.metadata.session_end_utc);
        let elapsed = escape_html(&document.metadata.elapsed_human);
        let assessment_str = document.summary.assessment.as_str();
        let assessment_color = document.summary.assessment.color_code();

        let chain_status_badge = if document.metadata.is_chain_valid {
            "<span class=\"badge badge-success\">✔ SHA-256 HASH CHAIN VERIFIED (100% UNBROKEN)</span>"
        } else {
            "<span class=\"badge badge-danger\">❌ TAMPER DETECTED / DISCONTINUOUS</span>"
        };

        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        html.push_str(&format!("<title>SENTRY-EDGE // Dossier - {}</title>\n", report_id));
        html.push_str("<style>\n");
        html.push_str(r#"
            :root {
                --bg-primary: #0a0c10;
                --bg-card: #12161f;
                --bg-card-hover: #171c28;
                --border-color: #21262d;
                --border-focus: #388bfd;
                --text-primary: #e6edf3;
                --text-secondary: #8b949e;
                --text-muted: #6e7681;
                --accent-blue: #58a6ff;
                --accent-green: #238636;
                --accent-amber: #d29922;
                --accent-red: #f85149;
                --accent-purple: #bc8cff;
                --font-mono: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, Courier, monospace;
                --font-sans: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
            }
            * { box-sizing: border-box; margin: 0; padding: 0; }
            body { background: var(--bg-primary); color: var(--text-primary); font-family: var(--font-sans); line-height: 1.5; padding: 32px 24px; }
            .container { max-width: 1200px; margin: 0 auto; }
            .header-banner { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 12px; padding: 28px; margin-bottom: 24px; display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 16px; border-left: 6px solid var(--accent-blue); }
            .header-title h1 { font-size: 26px; font-weight: 700; color: #fff; letter-spacing: -0.5px; display: flex; align-items: center; gap: 10px; }
            .header-title p { color: var(--text-secondary); font-size: 14px; margin-top: 4px; }
            .badge { display: inline-flex; align-items: center; padding: 6px 14px; border-radius: 20px; font-size: 13px; font-weight: 600; font-family: var(--font-mono); letter-spacing: 0.5px; }
            .badge-success { background: rgba(35, 134, 54, 0.2); color: #3fb950; border: 1px solid rgba(35, 134, 54, 0.4); }
            .badge-warning { background: rgba(210, 153, 34, 0.2); color: #e3b341; border: 1px solid rgba(210, 153, 34, 0.4); }
            .badge-danger { background: rgba(248, 81, 73, 0.2); color: #f85149; border: 1px solid rgba(248, 81, 73, 0.4); }
            .badge-info { background: rgba(88, 166, 255, 0.2); color: #58a6ff; border: 1px solid rgba(88, 166, 255, 0.4); }
            .badge-purple { background: rgba(188, 140, 255, 0.2); color: #bc8cff; border: 1px solid rgba(188, 140, 255, 0.4); }
            
            .grid-overview { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 16px; margin-bottom: 24px; }
            .card-stat { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 10px; padding: 20px; }
            .card-stat .label { font-size: 12px; font-weight: 600; text-transform: uppercase; color: var(--text-muted); letter-spacing: 0.5px; }
            .card-stat .value { font-size: 24px; font-weight: 700; color: #fff; margin-top: 6px; font-family: var(--font-mono); }
            .card-stat .subtext { font-size: 12px; color: var(--text-secondary); margin-top: 4px; }

            .card-section { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 12px; padding: 24px; margin-bottom: 24px; }
            .card-section h2 { font-size: 18px; font-weight: 600; color: #fff; margin-bottom: 16px; display: flex; align-items: center; gap: 8px; border-bottom: 1px solid var(--border-color); padding-bottom: 10px; }
            
            .story-box { background: rgba(88, 166, 255, 0.05); border-left: 4px solid var(--accent-blue); padding: 18px; border-radius: 0 8px 8px 0; margin-bottom: 18px; font-size: 15px; color: #d0d7de; }
            .takeaways-list { list-style: none; display: flex; flex-direction: column; gap: 10px; }
            .takeaways-list li { display: flex; align-items: flex-start; gap: 10px; font-size: 14px; color: var(--text-primary); }
            .takeaways-list li::before { content: "✔"; color: var(--accent-green); font-weight: bold; }

            .chart-row { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 20px; align-items: center; }
            .chart-panel { background: rgba(0,0,0,0.25); border: 1px solid var(--border-color); border-radius: 8px; padding: 16px; text-align: center; }

            .table-container { overflow-x: auto; margin-top: 12px; }
            table { width: 100%; border-collapse: collapse; font-size: 13px; text-align: left; }
            th { background: rgba(255,255,255,0.03); color: var(--text-muted); font-weight: 600; text-transform: uppercase; font-size: 11px; padding: 12px; border-bottom: 1px solid var(--border-color); }
            td { padding: 12px; border-bottom: 1px solid var(--border-color); color: var(--text-primary); }
            tr:hover td { background: var(--bg-card-hover); }
            .font-mono { font-family: var(--font-mono); }
            .hash-pill { font-family: var(--font-mono); font-size: 11px; background: rgba(255,255,255,0.05); padding: 2px 6px; border-radius: 4px; color: var(--accent-blue); }

            .footer-seal { background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 12px; padding: 20px; text-align: center; font-size: 13px; color: var(--text-secondary); }
        "#);
        html.push_str("</style>\n</head>\n<body>\n<div class=\"container\">\n");

        // 1. Header Banner
        html.push_str("<div class=\"header-banner\">\n");
        html.push_str("  <div class=\"header-title\">\n");
        html.push_str("    <h1>🛡️ SENTRY-EDGE // EXECUTIVE DOSSIER</h1>\n");
        html.push_str(&format!("    <p>Node: <strong>{}</strong> | Platform: <code>{}</code> | Generated: <code>{}</code></p>\n", node_id, platform, gen_time));
        html.push_str("  </div>\n");
        html.push_str("  <div>\n");
        html.push_str(&format!("    <span class=\"badge\" style=\"background: rgba({}, 0.2); color: {}; border: 1px solid {};\">ASSESSMENT: {}</span>\n", assessment_color, assessment_color, assessment_color, assessment_str));
        html.push_str(&format!("    {}\n", chain_status_badge));
        html.push_str("  </div>\n");
        html.push_str("</div>\n");

        // 2. Stat Metric Cards
        html.push_str("<div class=\"grid-overview\">\n");
        html.push_str(&format!(
            "  <div class=\"card-stat\"><div class=\"label\">Watch Duration</div><div class=\"value\">{}</div><div class=\"subtext\">{} to {}</div></div>\n",
            elapsed, session_start, session_end
        ));
        html.push_str(&format!(
            "  <div class=\"card-stat\"><div class=\"label\">Verified Records</div><div class=\"value\">{}</div><div class=\"subtext\">Sequences #{} to #{}</div></div>\n",
            document.metadata.total_records, document.metadata.sequence_start, document.metadata.sequence_end
        ));
        html.push_str(&format!(
            "  <div class=\"card-stat\"><div class=\"label\">Avg Acoustic Baseline</div><div class=\"value\">{:.1} <span style=\"font-size: 14px;\">dB SPL</span></div><div class=\"subtext\">Ambient Floor: {:.1} dB SPL</div></div>\n",
            document.acoustics.avg_baseline_db, document.acoustics.min_ambient_db
        ));
        html.push_str(&format!(
            "  <div class=\"card-stat\"><div class=\"label\">Peak Sound Pressure</div><div class=\"value\" style=\"color: var(--accent-red);\">{:.1} <span style=\"font-size: 14px;\">dB SPL</span></div><div class=\"subtext\">Transient Δ: +{:.1} dB</div></div>\n",
            document.acoustics.max_peak_db, document.acoustics.max_delta_db
        ));
        html.push_str("</div>\n");

        // 3. Executive Storytelling Narrative
        html.push_str("<div class=\"card-section\">\n");
        html.push_str("  <h2>📖 Executive Summary & Situation Narrative</h2>\n");
        html.push_str(&format!("  <div class=\"story-box\">{}</div>\n", escape_html(&document.summary.narrative_story)));
        html.push_str("  <ul class=\"takeaways-list\">\n");
        for takeaway in &document.summary.key_takeaways {
            html.push_str(&format!("    <li>{}</li>\n", escape_html(takeaway)));
        }
        html.push_str("  </ul>\n");
        html.push_str("</div>\n");

        // 4. Acoustic Spectrum SVG Chart
        html.push_str("<div class=\"card-section\">\n");
        html.push_str("  <h2>📊 Sovereign Acoustic Environment Analysis</h2>\n");
        html.push_str("  <div class=\"chart-row\">\n");
        
        // Gauge / Distribution Chart
        let quiet_width = document.acoustics.quiet_period_pct;
        let mod_width = document.acoustics.moderate_period_pct;
        let loud_width = document.acoustics.loud_period_pct;

        html.push_str("    <div class=\"chart-panel\">\n");
        html.push_str("      <h3 style=\"font-size: 14px; margin-bottom: 12px; color: var(--text-secondary);\">Acoustic Exposure Distribution</h3>\n");
        html.push_str("      <div style=\"display: flex; height: 24px; border-radius: 6px; overflow: hidden; background: #21262d; margin-bottom: 12px;\">\n");
        html.push_str(&format!("        <div style=\"width: {:.1}%; background: var(--accent-green);\" title=\"Quiet (<55dB): {:.1}%\"></div>\n", quiet_width, quiet_width));
        html.push_str(&format!("        <div style=\"width: {:.1}%; background: var(--accent-amber);\" title=\"Moderate (55-75dB): {:.1}%\"></div>\n", mod_width, mod_width));
        html.push_str(&format!("        <div style=\"width: {:.1}%; background: var(--accent-red);\" title=\"Loud (>75dB): {:.1}%\"></div>\n", loud_width, loud_width));
        html.push_str("      </div>\n");
        html.push_str("      <div style=\"display: flex; justify-content: space-around; font-size: 12px; color: var(--text-secondary);\">\n");
        html.push_str(&format!("        <span>🟢 Quiet (<55dB): <strong>{:.0}%</strong></span>\n", quiet_width));
        html.push_str(&format!("        <span>🟡 Moderate (55-75dB): <strong>{:.0}%</strong></span>\n", mod_width));
        html.push_str(&format!("        <span>🔴 Loud (>75dB): <strong>{:.0}%</strong></span>\n", loud_width));
        html.push_str("      </div>\n");
        html.push_str("    </div>\n");

        // Subsystem Breakdown
        html.push_str("    <div class=\"chart-panel\">\n");
        html.push_str("      <h3 style=\"font-size: 14px; margin-bottom: 12px; color: var(--text-secondary);\">Subsystem Activity Breakdown</h3>\n");
        html.push_str("      <div style=\"display: flex; flex-wrap: wrap; gap: 8px; justify-content: center;\">\n");
        for (sub, count) in &document.subsystem_breakdown {
            html.push_str(&format!("        <span class=\"badge badge-info\">{}: <strong>{}</strong></span>\n", escape_html(sub), count));
        }
        html.push_str("      </div>\n");
        html.push_str("    </div>\n");

        html.push_str("  </div>\n");
        html.push_str("</div>\n");

        // 5. Forensic Incident Timeline
        html.push_str("<div class=\"card-section\">\n");
        html.push_str(&format!("  <h2>⏱️ Forensic Audit Trail ({} Verified Events)</h2>\n", document.timeline.len()));
        html.push_str("  <div class=\"table-container\">\n");
        html.push_str("    <table>\n");
        html.push_str("      <thead>\n");
        html.push_str("        <tr><th>#</th><th>Timestamp (UTC)</th><th>Severity</th><th>Subsystem</th><th>Event Summary</th><th>RMS / Base / Δ</th><th>Latency</th><th>SHA-256 Proof</th></tr>\n");
        html.push_str("      </thead>\n");
        html.push_str("      <tbody>\n");

        for ev in &document.timeline {
            let sev_badge = match ev.severity.as_str() {
                "ALERT" => "<span class=\"badge badge-danger\">ALERT</span>",
                "SECURITY" => "<span class=\"badge badge-purple\">SECURITY</span>",
                "WARN" => "<span class=\"badge badge-warning\">WARN</span>",
                "INFO" => "<span class=\"badge badge-success\">INFO</span>",
                _ => "<span class=\"badge badge-info\">TRACE</span>",
            };

            let acoustic_str = if let (Some(rms), Some(base)) = (ev.rms_db, ev.baseline_db) {
                let delta = ev.delta_db.unwrap_or(0.0);
                format!("<span class=\"font-mono\">{:.1}dB / {:.1}dB (+{:.1}dB)</span>", rms, base, delta)
            } else {
                "<span style=\"color: var(--text-muted);\">-</span>".to_string()
            };

            let hash_short = if ev.record_hash.len() > 16 {
                format!("{}...", &ev.record_hash[..16])
            } else {
                ev.record_hash.clone()
            };

            html.push_str(&format!(
                "        <tr><td class=\"font-mono\">#{}</td><td class=\"font-mono\">{}</td><td>{}</td><td><strong>{}</strong></td><td>{}</td><td>{}</td><td class=\"font-mono\" style=\"color: var(--text-muted);\">{}ns</td><td><span class=\"hash-pill\" title=\"SHA-256: {}\">{}</span></td></tr>\n",
                ev.sequence,
                escape_html(&ev.timestamp_utc),
                sev_badge,
                escape_html(&ev.subsystem),
                escape_html(&ev.summary),
                acoustic_str,
                ev.duration_ns,
                escape_html(&ev.record_hash),
                hash_short
            ));
        }

        html.push_str("      </tbody>\n");
        html.push_str("    </table>\n");
        html.push_str("  </div>\n");
        html.push_str("</div>\n");

        // 6. Cryptographic Seal Footer
        html.push_str("<div class=\"footer-seal\">\n");
        html.push_str(&format!("  <p>🔐 <strong>Cryptographic Audit Seal:</strong> {}</p>\n", escape_html(&document.metadata.verification_notes)));
        html.push_str(&format!("  <p style=\"font-size: 11px; margin-top: 4px;\">Genesis: <code class=\"font-mono\">{}</code> | Terminal: <code class=\"font-mono\">{}</code></p>\n", escape_html(&document.metadata.genesis_hash), escape_html(&document.metadata.terminal_hash)));
        html.push_str("</div>\n");

        html.push_str("</div>\n</body>\n</html>\n");

        Ok(html)
    }
}
