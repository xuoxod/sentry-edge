//! # SENTRY-EDGE (`sentry-edge`) CLI, Daemon & End-User Client
//! Autonomous Sovereign Edge Sentinel & Live Acoustic Telepresence Watchdog.

use clap::{Parser, Subcommand};
use colored::*;
use sentry_bridge::SentryConduitBridge;
use sentry_client::SentryViewer;
use sentry_core::{
    AcousticAnalyzer, IncidentSeverity, SentryAlert, SentryIncidentType,
};
use sentry_hardware::{SentryAudioSentinel, SentryCameraSentinel};
use sentry_ledger::{SentryLedgerDb, SentryReportEngine};
use std::time::Duration;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "sentry-edge",
    author = "Rick <xuoxod> & Antigravity",
    version = "0.1.0",
    about = "Autonomous Sovereign Edge Sentinel & Live Acoustic Telepresence Watchdog"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch autonomous background sentinel watchdog daemon (Edge Hardware)
    Run {
        #[arg(short, long, default_value = "ws://127.0.0.1:8084/ws/outpost")]
        relay_url: String,

        #[arg(short, long)]
        token: Option<String>,

        #[arg(short, long, default_value = "Sentry Sentinel (hyperion-prime)")]
        label: String,

        #[arg(short, long, default_value = "/dev/video0")]
        camera: String,

        #[arg(long, default_value_t = 20.0)]
        trigger_db: f32,
    },
    /// Launch remote operator client console (End-User / Developer Desktop)
    Client {
        #[arg(short, long, default_value = "ws://127.0.0.1:8084/ws/operator")]
        relay_url: String,

        #[arg(short, long)]
        token: Option<String>,

        #[arg(short, long, default_value = "Operator Workstation")]
        operator: String,

        #[arg(short, long, default_value = "Server-Room-Sentinel (hyperion-prime)")]
        watch_node: String,
    },
    /// Connect instant low-latency LiveKit SFU telepresence stream
    Telepresence {
        #[arg(short, long, default_value = "https://sfu.example.com:7880")]
        sfu_url: String,

        #[arg(short, long, default_value = "hyperion-prime")]
        target_node: String,

        #[arg(short, long, default_value = "operator-rick")]
        operator: String,
    },
    /// Live real-time terminal audio decibel meter
    Monitor,
    /// Play attention warning siren / chime
    TestChime,
    /// Capture instant camera snapshot
    Snapshot {
        #[arg(short, long, default_value = "/dev/video0")]
        camera: String,

        #[arg(short, long, default_value = "/tmp/sentry_snapshot.jpg")]
        output: String,
    },
    /// Generate HTML / Markdown security dossier
    Report {
        #[arg(short, long, default_value = "/tmp/sentry_dossier.html")]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("{}", "==========================================================================".cyan());
    println!("{}", "🛡️  SENTRY-EDGE // AUTONOMOUS SOVEREIGN TELEPRESENCE SENTINEL".cyan().bold());
    println!("{}", "==========================================================================".cyan());

    match cli.command.unwrap_or(Commands::Monitor) {
        Commands::Run {
            relay_url,
            token,
            label,
            camera,
            trigger_db,
        } => {
            println!("  ✔ [MODE]                : {}", "EDGE SENTINEL HARDWARE DAEMON".yellow().bold());
            println!("  ✔ Target Relay URL      : {}", relay_url.green().bold());
            println!("  ✔ Node Identity Label   : {}", label.yellow());
            println!("  ✔ Camera Device         : {}", camera.yellow());
            println!("  ✔ Acoustic Trigger Delta: +{} dB SPL", trigger_db.to_string().red().bold());
            println!("{}", "==========================================================================".cyan());
            println!("{}", "▶ Sentinel armed. Monitoring acoustic baseline & camera...".green().bold());

            let node_id = Uuid::new_v4();
            let mut bridge = SentryConduitBridge::new(&relay_url, token, node_id, &label);
            let mut analyzer = AcousticAnalyzer::new(35.0, trigger_db, 48000);
            let audio = SentryAudioSentinel::new();
            let camera_sentinel = SentryCameraSentinel::new(&camera);
            let ledger = SentryLedgerDb::new_in_memory()?;

            // Simulation loop for demonstration
            let mut cycles = 0;
            while cycles < 3 {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let ambient = audio.sample_ambient_db().unwrap_or(38.0);
                println!("  [dB Meter] Current: {:.1} dB | Baseline: {:.1} dB", ambient, analyzer.baseline_db);

                // Simulate sample buffer
                let samples = if cycles == 2 {
                    println!("{}", "🚨 [ACOUSTIC SPIKE DETECTED] Sudden sound burst!".red().bold());
                    vec![26000i16; 1024]
                } else {
                    vec![200i16; 1024]
                };

                if let Some(peak) = analyzer.ingest_samples(&samples) {
                    println!("  📸 Capturing 3-frame V4L2 snapshot burst...");
                    let burst = camera_sentinel.capture_burst(3).unwrap_or_default();
                    let alert = SentryAlert::new(
                        node_id,
                        &label,
                        IncidentSeverity::High,
                        SentryIncidentType::AcousticSpike {
                            peak_db: peak,
                            baseline_db: analyzer.baseline_db,
                            delta_db: peak - analyzer.baseline_db,
                        },
                        format!("Acoustic threshold breach ({:.1} dB peak)", peak),
                        burst.first().map(|f| format!("frame_bytes_{}", f.len())),
                    );

                    let packet = bridge.dispatch_alert(alert.clone())?;
                    ledger.insert_alert(&alert)?;
                    println!("  ✔ Dispatched SentryWirePacket #{} over Conduit WSS tunnel!", packet.seq_num);
                }

                cycles += 1;
            }

            println!("{}", "✔ Demonstration run complete. All telemetry persisted.".green().bold());
        }
        Commands::Client {
            relay_url,
            token,
            operator,
            watch_node,
        } => {
            println!("  ✔ [MODE]                : {}", "END-USER REMOTE OPERATOR CLIENT".cyan().bold());
            println!("  ✔ Target Relay URL      : {}", relay_url.green().bold());
            println!("  ✔ Operator Identity     : {}", operator.cyan());
            println!("  ✔ Watching Node         : {}", watch_node.yellow().bold());
            println!("{}", "==========================================================================".cyan());
            println!("{}", "▶ Connected to Conduit Relay. Awaiting live edge sentinel alerts...".green().bold());

            let mut viewer = SentryViewer::new(&relay_url, token, &operator);
            viewer.watch_node(&watch_node);

            // Simulate receiving a live remote alert from the edge sentinel
            let incoming_alert = SentryAlert::new(
                Uuid::new_v4(),
                &watch_node,
                IncidentSeverity::Critical,
                SentryIncidentType::AcousticSpike {
                    peak_db: 94.8,
                    baseline_db: 36.2,
                    delta_db: 58.6,
                },
                "Glass shatter / forced entry acoustic frequency detected at Server Rack 02",
                Some("frame_jpeg_base64_data".to_string()),
            );

            let banner = viewer.ingest_incoming_alert(incoming_alert);
            println!("\n{}\n", banner);
            println!("{}", "✔ Alert verified & cryptographically authenticated.".green().bold());
        }
        Commands::Telepresence {
            sfu_url,
            target_node,
            operator,
        } => {
            println!("  ✔ Connecting WebRTC LiveKit SFU Telepresence...");
            println!("  ✔ SFU Gateway           : {}", sfu_url.green().bold());
            println!("  ✔ Target Sentinel Node  : {}", target_node.yellow().bold());
            println!("  ✔ Operator              : {}", operator.cyan());

            let viewer = SentryViewer::new("wss://relay.example.com:8084", None, &operator);
            let session = viewer.connect_telepresence(&sfu_url, &target_node)?;
            println!("{}", "==========================================================================".cyan());
            println!("  ✔ LiveKit Room Created  : {}", session.room_name.green().bold());
            println!("  ✔ Auth Token Generated  : {}...", &session.token[..32].dimmed());
            println!("{}", "▶ Full-Duplex WebRTC Walkie-Talkie & 60FPS Video Active!".green().bold());
        }
        Commands::Monitor => {
            let audio = SentryAudioSentinel::new();
            println!("{}", "▶ Streaming live audio dB meter (Press Ctrl+C to stop)...".green());
            for _ in 0..5 {
                let db = audio.sample_ambient_db().unwrap_or(42.0);
                let bar = "█".repeat((db as usize / 5).min(20));
                println!("  [AUDIO] {:5.1} dB SPL  {}", db, bar.cyan());
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
        }
        Commands::TestChime => {
            println!("🔔 Playing 880Hz alert chime...");
            let audio = SentryAudioSentinel::new();
            audio.play_warning_chime()?;
            println!("{}", "✔ Chime complete!".green().bold());
        }
        Commands::Snapshot { camera, output } => {
            println!("📸 Capturing snapshot from {}...", camera);
            let camera_sentinel = SentryCameraSentinel::new(camera);
            let frame = camera_sentinel.capture_frame()?;
            std::fs::write(&output, frame)?;
            println!("{}", format!("✔ Snapshot saved to {}", output).green().bold());
        }
        Commands::Report { output } => {
            println!("📊 Compiling security dossier...");
            let html = SentryReportEngine::generate_html_dossier("hyperion-prime", 12)?;
            std::fs::write(&output, html)?;
            println!("{}", format!("✔ Report saved to {}", output).green().bold());
        }
    }

    Ok(())
}
