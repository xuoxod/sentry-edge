//! Sentry High-Impact ASCII Plain Text Report Formatter

use super::ReportFormatter;
use crate::error::Result;
use crate::models::ReportDocument;

pub struct TextReportFormatter;

impl ReportFormatter for TextReportFormatter {
    fn format(document: &ReportDocument) -> Result<String> {
        let mut out = String::new();
        let border = "═".repeat(78);
        let thin_border = "─".repeat(78);

        out.push_str(&format!("╔{}╗\n", border));
        out.push_str(&format!("║ {:^76} ║\n", "🛡️  SENTRY-EDGE // EXECUTIVE SECURITY & TELEMETRY DOSSIER"));
        out.push_str(&format!("║ {:^76} ║\n", format!("Report ID: {} | Generated: {}", document.metadata.report_id, document.metadata.generated_at_utc)));
        out.push_str(&format!("╠{}╣\n", border));

        // 1. Overview Section
        out.push_str(&format!("║ Node Identity   : {:<58} ║\n", document.metadata.node_id));
        out.push_str(&format!("║ Platform Runtime: {:<58} ║\n", document.metadata.platform));
        out.push_str(&format!("║ Session Period  : {:<58} ║\n", format!("{} to {}", document.metadata.session_start_utc, document.metadata.session_end_utc)));
        out.push_str(&format!("║ Watch Duration  : {:<58} ║\n", document.metadata.elapsed_human));
        out.push_str(&format!("║ Overall Status  : {:<58} ║\n", format!("[{}]", document.summary.assessment.as_str())));
        out.push_str(&format!("║ Hash Chain Audit: {:<58} ║\n", if document.metadata.is_chain_valid { "✔ 100% UNBROKEN & TAMPER-FREE" } else { "❌ TAMPER DETECTED / DISCONTINUOUS" }));
        out.push_str(&format!("╠{}╣\n", border));

        // 2. Executive Narrative Story
        out.push_str(&format!("║ {:<76} ║\n", "📖 EXECUTIVE SUMMARY & SITUATION NARRATIVE"));
        out.push_str(&format!("╠{}╣\n", thin_border));

        // Wrap narrative story cleanly at 74 chars
        let words = document.summary.narrative_story.split_whitespace();
        let mut current_line = String::new();
        for word in words {
            if current_line.len() + word.len() + 1 > 74 {
                out.push_str(&format!("║  {:<74} ║\n", current_line));
                current_line.clear();
            }
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        if !current_line.is_empty() {
            out.push_str(&format!("║  {:<74} ║\n", current_line));
        }

        out.push_str(&format!("╠{}╣\n", border));

        // 3. Quantitative Acoustic Metrics
        out.push_str(&format!("║ {:<76} ║\n", "📊 ACOUSTIC ENVIRONMENT SPECTRUM"));
        out.push_str(&format!("╠{}╣\n", thin_border));
        out.push_str(&format!("║  • Average Ambient Baseline : {:<46} ║\n", format!("{:.1} dB SPL", document.acoustics.avg_baseline_db)));
        out.push_str(&format!("║  • Minimum Ambient Level    : {:<46} ║\n", format!("{:.1} dB SPL", document.acoustics.min_ambient_db)));
        out.push_str(&format!("║  • Peak Transient Level     : {:<46} ║\n", format!("{:.1} dB SPL (+{:.1} dB Δ)", document.acoustics.max_peak_db, document.acoustics.max_delta_db)));
        out.push_str(&format!("║  • Acoustic Spike Breaches  : {:<46} ║\n", document.acoustics.spike_count));
        out.push_str(&format!("║  • Sound Distribution       : {:<46} ║\n", format!("Quiet: {:.0}% | Moderate: {:.0}% | Loud: {:.0}%", document.acoustics.quiet_period_pct, document.acoustics.moderate_period_pct, document.acoustics.loud_period_pct)));
        out.push_str(&format!("╠{}╣\n", border));

        // 4. Key Takeaways
        out.push_str(&format!("║ {:<76} ║\n", "🔑 KEY OPERATOR TAKEAWAYS"));
        out.push_str(&format!("╠{}╣\n", thin_border));
        for takeaway in &document.summary.key_takeaways {
            out.push_str(&format!("║  ✔ {:<72} ║\n", takeaway));
        }
        out.push_str(&format!("╠{}╣\n", border));

        // 5. Recent Forensic Incident Timeline (up to 15 items in text report)
        out.push_str(&format!("║ {:<76} ║\n", "⏱️  CHRONOLOGICAL FORENSIC AUDIT TIMELINE"));
        out.push_str(&format!("╠{}╣\n", thin_border));
        out.push_str(&format!("║ {:<4} | {:<12} | {:<7} | {:<12} | {:<30} ║\n", "Seq", "Timestamp", "Level", "Subsystem", "Summary / Acoustic Δ"));
        out.push_str(&format!("╠{}╣\n", thin_border));

        let display_events = if document.timeline.len() > 20 {
            &document.timeline[document.timeline.len() - 20..]
        } else {
            &document.timeline[..]
        };

        for ev in display_events {
            let ts = if ev.timestamp_utc.len() >= 19 {
                &ev.timestamp_utc[11..19]
            } else {
                &ev.timestamp_utc
            };
            let sub_short = if ev.subsystem.len() > 12 { &ev.subsystem[..12] } else { &ev.subsystem };
            let sum_short = if ev.summary.len() > 30 { &ev.summary[..30] } else { &ev.summary };

            out.push_str(&format!(
                "║ #{:<3} | {:<12} | {:<7} | {:<12} | {:<30} ║\n",
                ev.sequence, ts, ev.severity, sub_short, sum_short
            ));
        }

        out.push_str(&format!("╠{}╣\n", border));

        // 6. Cryptographic Hash Chain Seal
        out.push_str(&format!("║ {:<76} ║\n", "🔐 TAMPER-EVIDENT CRYPTOGRAPHIC HASH CHAIN PROOF"));
        out.push_str(&format!("╠{}╣\n", thin_border));
        out.push_str(&format!("║ Genesis Hash : {:<60} ║\n", if document.metadata.genesis_hash.len() > 60 { &document.metadata.genesis_hash[..60] } else { &document.metadata.genesis_hash }));
        out.push_str(&format!("║ Terminal Hash: {:<60} ║\n", if document.metadata.terminal_hash.len() > 60 { &document.metadata.terminal_hash[..60] } else { &document.metadata.terminal_hash }));
        out.push_str(&format!("║ Records Count: {:<60} ║\n", document.metadata.total_records));
        out.push_str(&format!("║ Verification : {:<60} ║\n", document.metadata.verification_notes));
        out.push_str(&format!("╚{}╝\n", border));

        Ok(out)
    }
}
