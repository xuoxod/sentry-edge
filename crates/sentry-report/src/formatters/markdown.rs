//! Sentry GitHub-Flavored Markdown Report Formatter

use super::ReportFormatter;
use crate::error::Result;
use crate::models::ReportDocument;

pub struct MarkdownReportFormatter;

impl ReportFormatter for MarkdownReportFormatter {
    fn format(document: &ReportDocument) -> Result<String> {
        let mut out = String::new();

        out.push_str(&format!("# 🛡️ SENTRY-EDGE Security & Telemetry Dossier\n\n"));
        out.push_str(&format!("**Report ID:** `{}` | **Generated:** `{}`\n\n", document.metadata.report_id, document.metadata.generated_at_utc));

        // 1. Executive Briefing Box
        out.push_str("## 📋 Executive Overview\n\n");
        out.push_str("| Attribute | Value |\n");
        out.push_str("| :--- | :--- |\n");
        out.push_str(&format!("| **Monitored Node** | `{}` |\n", document.metadata.node_id));
        out.push_str(&format!("| **Platform Runtime** | `{}` |\n", document.metadata.platform));
        out.push_str(&format!("| **Observation Period** | `{}` to `{}` ({}) |\n", document.metadata.session_start_utc, document.metadata.session_end_utc, document.metadata.elapsed_human));
        out.push_str(&format!("| **Status Assessment** | **{}** |\n", document.summary.assessment.as_str()));
        out.push_str(&format!("| **Cryptographic Hash Chain** | {} |\n", if document.metadata.is_chain_valid { "🟢 `100% UNBROKEN & TAMPER-FREE`" } else { "🔴 `TAMPER DETECTED / BROKEN`" }));
        out.push_str(&format!("| **Total Verified Events** | `{}` (Sequences #{} to #{}) |\n\n", document.metadata.total_records, document.metadata.sequence_start, document.metadata.sequence_end));

        // 2. Executive Narrative Story
        out.push_str("## 📖 Situation Narrative\n\n");
        out.push_str(&format!("> [!NOTE]\n> {}\n\n", document.summary.narrative_story));

        // 3. Acoustic Insights
        out.push_str("## 📊 Acoustic Environment Analysis\n\n");
        out.push_str("| Metric | Measured Value |\n");
        out.push_str("| :--- | :--- |\n");
        out.push_str(&format!("| **Average Baseline Noise** | `{:.1} dB SPL` |\n", document.acoustics.avg_baseline_db));
        out.push_str(&format!("| **Minimum Ambient Level** | `{:.1} dB SPL` |\n", document.acoustics.min_ambient_db));
        out.push_str(&format!("| **Maximum Peak Transient** | `{:.1} dB SPL (+{:.1} dB Δ)` |\n", document.acoustics.max_peak_db, document.acoustics.max_delta_db));
        out.push_str(&format!("| **Sound Distribution** | Quiet (<55dB): `{:.0}%` | Moderate (55-75dB): `{:.0}%` | Loud (>75dB): `{:.0}%` |\n\n", document.acoustics.quiet_period_pct, document.acoustics.moderate_period_pct, document.acoustics.loud_period_pct));

        // 4. Key Takeaways
        out.push_str("## 🔑 Key Takeaways & Operator Actions\n\n");
        for item in &document.summary.key_takeaways {
            out.push_str(&format!("* {}\n", item));
        }
        out.push('\n');

        // 5. Forensic Timeline Table
        out.push_str("## ⏱️ Chronological Audit Timeline\n\n");
        out.push_str("| Seq | Timestamp (UTC) | Severity | Subsystem | Summary | Δ dB | Latency | SHA-256 Hash |\n");
        out.push_str("| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n");

        for ev in &document.timeline {
            let hash_short = if ev.record_hash.len() > 12 { &ev.record_hash[..12] } else { &ev.record_hash };
            let delta_str = ev.delta_db.map(|d| format!("+{:.1} dB", d)).unwrap_or_else(|| "-".to_string());
            out.push_str(&format!(
                "| #{} | `{}` | `{}` | `{}` | {} | `{}` | `{}ns` | `{}` |\n",
                ev.sequence, ev.timestamp_utc, ev.severity, ev.subsystem, ev.summary, delta_str, ev.duration_ns, hash_short
            ));
        }
        out.push('\n');

        // 6. Cryptographic Proof
        out.push_str("## 🔐 Cryptographic Chain Audit Proof\n\n");
        out.push_str("```text\n");
        out.push_str(&format!("Genesis Block Hash : {}\n", document.metadata.genesis_hash));
        out.push_str(&format!("Terminal Block Hash: {}\n", document.metadata.terminal_hash));
        out.push_str(&format!("Integrity Status   : {}\n", document.metadata.verification_notes));
        out.push_str("```\n");

        Ok(out)
    }
}
