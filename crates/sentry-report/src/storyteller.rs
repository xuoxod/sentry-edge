//! Sentry Report Storyteller Engine
//! Converts structured nanosecond telemetry audit records into uncluttered, human-consumable executive narratives.

use crate::models::{
    AcousticInsights, ExecutiveAssessment, ExecutiveSummary, ForensicEvent, ReportDocument,
    ReportMetadata,
};
use sentry_telemetry::TelemetryRecord;
use std::collections::HashMap;
use uuid::Uuid;

pub struct Storyteller;

impl Storyteller {
    pub fn build_document(
        node_id: &str,
        platform_info: &str,
        records: &[TelemetryRecord],
    ) -> ReportDocument {
        if records.is_empty() {
            return Self::build_empty_document(node_id, platform_info);
        }

        let total_records = records.len() as u64;
        let first_record = &records[0];
        let last_record = &records[records.len() - 1];

        let sequence_start = first_record.sequence;
        let sequence_end = last_record.sequence;
        let genesis_hash = first_record.prev_record_hash.clone();
        let terminal_hash = last_record.record_hash.clone();

        // 1. Verify hash chain integrity across the entire record set
        let mut is_chain_valid = true;
        let mut expected_prev = genesis_hash.clone();
        for rec in records {
            if rec.prev_record_hash != expected_prev {
                is_chain_valid = false;
                break;
            }
            if !rec.verify_integrity() {
                is_chain_valid = false;
                break;
            }
            expected_prev = rec.record_hash.clone();
        }

        // 2. Compute breakdowns
        let mut subsystem_breakdown = HashMap::new();
        let mut severity_breakdown = HashMap::new();
        let mut spike_count = 0u64;
        let mut camera_bursts = 0u64;
        let mut bridge_dispatches = 0u64;

        let mut baseline_sum = 0.0f32;
        let mut baseline_count = 0u64;
        let mut min_ambient_db = 120.0f32;
        let mut max_peak_db = 0.0f32;
        let mut max_delta_db = 0.0f32;

        let mut bucket_quiet = 0u64; // < 55 dB
        let mut bucket_mod = 0u64;   // 55 - 75 dB
        let mut bucket_loud = 0u64;  // > 75 dB

        let mut forensic_timeline = Vec::with_capacity(records.len());

        for rec in records {
            *subsystem_breakdown
                .entry(rec.subsystem.as_str().to_string())
                .or_insert(0) += 1;
            *severity_breakdown
                .entry(rec.severity.as_str().to_string())
                .or_insert(0) += 1;

            if rec.subsystem.as_str().contains("CAMERA") || rec.what.summary.to_lowercase().contains("burst") {
                camera_bursts += 1;
            }
            if rec.subsystem.as_str().contains("BRIDGE") || rec.what.summary.to_lowercase().contains("dispatch") {
                bridge_dispatches += 1;
            }
            if rec.severity.as_str() == "ALERT" || rec.severity.as_str() == "SECURITY" {
                spike_count += 1;
            }

            if let Some(base) = rec.what.baseline_db {
                baseline_sum += base;
                baseline_count += 1;
                if base < min_ambient_db {
                    min_ambient_db = base;
                }
            }

            if let Some(rms) = rec.what.rms_db {
                if rms > max_peak_db {
                    max_peak_db = rms;
                }
                if rms < 55.0 {
                    bucket_quiet += 1;
                } else if rms <= 75.0 {
                    bucket_mod += 1;
                } else {
                    bucket_loud += 1;
                }
            }

            if let Some(delta) = rec.what.delta_db {
                if delta > max_delta_db {
                    max_delta_db = delta;
                }
            }

            forensic_timeline.push(ForensicEvent {
                sequence: rec.sequence,
                timestamp_utc: rec.timestamp_utc.clone(),
                timestamp_unix_ns: rec.timestamp_unix_ns.to_string(),
                subsystem: rec.subsystem.as_str().to_string(),
                severity: rec.severity.as_str().to_string(),
                summary: rec.what.summary.clone(),
                rms_db: rec.what.rms_db,
                baseline_db: rec.what.baseline_db,
                delta_db: rec.what.delta_db,
                duration_ns: rec.what.duration_ns,
                duration_ps: rec.what.duration_ps.to_string(),
                record_hash: rec.record_hash.clone(),
                prev_hash: rec.prev_record_hash.clone(),
            });
        }

        let avg_baseline_db = if baseline_count > 0 {
            baseline_sum / baseline_count as f32
        } else {
            38.5
        };

        if min_ambient_db > 100.0 {
            min_ambient_db = avg_baseline_db;
        }

        let total_samples = (bucket_quiet + bucket_mod + bucket_loud).max(1) as f32;
        let quiet_period_pct = (bucket_quiet as f32 / total_samples) * 100.0;
        let moderate_period_pct = (bucket_mod as f32 / total_samples) * 100.0;
        let loud_period_pct = (bucket_loud as f32 / total_samples) * 100.0;

        let acoustics = AcousticInsights {
            avg_baseline_db,
            min_ambient_db,
            max_peak_db,
            max_delta_db,
            spike_count,
            quiet_period_pct,
            moderate_period_pct,
            loud_period_pct,
        };

        // 3. Determine Assessment
        let assessment = if !is_chain_valid {
            ExecutiveAssessment::Critical
        } else if spike_count > 0 || max_peak_db >= 80.0 {
            ExecutiveAssessment::Alert
        } else if max_peak_db >= 65.0 {
            ExecutiveAssessment::Elevated
        } else {
            ExecutiveAssessment::Nominal
        };

        // 4. Calculate elapsed time
        let start_time_str = &first_record.timestamp_utc;
        let end_time_str = &last_record.timestamp_utc;
        let elapsed_human = Self::calculate_elapsed(first_record.timestamp_unix_ns, last_record.timestamp_unix_ns);

        // 5. Generate human storytelling narrative
        let mut story = format!(
            "During an autonomous sentinel watch session spanning {} across {} cryptographically chained telemetry records, ",
            elapsed_human, total_records
        );

        if !is_chain_valid {
            story.push_str("⚠️ CRITICAL ALERT: Tampering or discontinuity was detected in the SHA-256 hash chain! ");
        } else {
            story.push_str("the cryptographic SHA-256 hash chain remained 100% unbroken and tamper-free. ");
        }

        story.push_str(&format!(
            "Ambient sound levels averaged {:.1} dB SPL (ranging from {:.1} dB to a peak transient of {:.1} dB SPL). ",
            avg_baseline_db, min_ambient_db, max_peak_db
        ));

        if spike_count > 0 {
            story.push_str(&format!(
                "The sentinel successfully intercepted {} acoustic threshold breach events with a maximum transient spike of +{:.1} dB over baseline. Automated surveillance responded with {} optical camera burst captures. ",
                spike_count, max_delta_db, camera_bursts
            ));
        } else {
            story.push_str("No acoustic boundary breaches were observed; the physical environment remained within nominal parameters throughout the surveillance period. ");
        }

        let mut key_takeaways = Vec::new();
        key_takeaways.push(format!("Monitored node '{}' on runtime [{}].", node_id, platform_info));
        key_takeaways.push(format!("Recorded {} verified events spanning sequences #{} to #{}.", total_records, sequence_start, sequence_end));
        key_takeaways.push(format!("Average acoustic baseline: {:.1} dB SPL | Peak sound pressure: {:.1} dB SPL (+{:.1} dB Δ).", avg_baseline_db, max_peak_db, max_delta_db));
        if spike_count > 0 {
            key_takeaways.push(format!("Threat Interceptions: {} acoustic spike alerts triggered {} camera frame bursts.", spike_count, camera_bursts));
        } else {
            key_takeaways.push("Threat Interceptions: Zero acoustic boundary violations detected.".to_string());
        }
        key_takeaways.push(if is_chain_valid {
            "Cryptographic Audit: 100% Tamper-Evident SHA-256 rolling hash chain verified.".to_string()
        } else {
            "Cryptographic Audit: FAILED - Hash mismatch detected in ledger.".to_string()
        });

        let summary = ExecutiveSummary {
            assessment,
            narrative_story: story,
            key_takeaways,
            total_incidents: spike_count,
            total_camera_bursts: camera_bursts,
            total_bridge_dispatches: bridge_dispatches,
        };

        let metadata = ReportMetadata {
            report_id: format!("REP-{}", &Uuid::new_v4().to_string()[..8].to_uppercase()),
            generated_at_utc: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            node_id: node_id.to_string(),
            platform: platform_info.to_string(),
            session_start_utc: start_time_str.clone(),
            session_end_utc: end_time_str.clone(),
            elapsed_human,
            total_records,
            sequence_start,
            sequence_end,
            genesis_hash,
            terminal_hash,
            is_chain_valid,
            verification_notes: if is_chain_valid {
                "Continuous SHA-256 hash-chain verified from genesis to terminal block.".to_string()
            } else {
                "Integrity breach: prev_record_hash sequence mismatch detected.".to_string()
            },
        };

        ReportDocument {
            metadata,
            summary,
            acoustics,
            subsystem_breakdown,
            severity_breakdown,
            timeline: forensic_timeline,
        }
    }

    fn calculate_elapsed(start_ns: u128, end_ns: u128) -> String {
        if end_ns <= start_ns {
            return "0s".to_string();
        }
        let total_secs = (end_ns - start_ns) / 1_000_000_000;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    fn build_empty_document(node_id: &str, platform_info: &str) -> ReportDocument {
        ReportDocument {
            metadata: ReportMetadata {
                report_id: format!("REP-{}", &Uuid::new_v4().to_string()[..8].to_uppercase()),
                generated_at_utc: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                node_id: node_id.to_string(),
                platform: platform_info.to_string(),
                session_start_utc: "N/A".to_string(),
                session_end_utc: "N/A".to_string(),
                elapsed_human: "0s".to_string(),
                total_records: 0,
                sequence_start: 0,
                sequence_end: 0,
                genesis_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                terminal_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                is_chain_valid: true,
                verification_notes: "Empty dataset, zero records.".to_string(),
            },
            summary: ExecutiveSummary {
                assessment: ExecutiveAssessment::Nominal,
                narrative_story: "No telemetry records were found in the specified audit source.".to_string(),
                key_takeaways: vec!["Dataset is empty.".to_string()],
                total_incidents: 0,
                total_camera_bursts: 0,
                total_bridge_dispatches: 0,
            },
            acoustics: AcousticInsights {
                avg_baseline_db: 0.0,
                min_ambient_db: 0.0,
                max_peak_db: 0.0,
                max_delta_db: 0.0,
                spike_count: 0,
                quiet_period_pct: 0.0,
                moderate_period_pct: 0.0,
                loud_period_pct: 0.0,
            },
            subsystem_breakdown: HashMap::new(),
            severity_breakdown: HashMap::new(),
            timeline: Vec::new(),
        }
    }
}
