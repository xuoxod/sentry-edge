# 🛡️ SENTRY-EDGE (`sentry-edge-rs`)
### *Autonomous Sovereign Telepresence Sentinel & Edge Acoustic Watchdog*

```
  ____  _____ _   _ _____ ______   __  _____ ____   ____ _____ 
 / ___|| ____| \ | |_   _|  _ \ \ / / | ____|  _ \ / ___| ____|
 \___ \|  _| |  \| | | | | |_) \ V /  |  _| | | | | |  _|  _|  
  ___) | |___| |\  | | | |  _ < | |   | |___| |_| | |_| | |___ 
 |____/|_____|_| \_| |_| |_| \_\|_|   |_____|____/ \____|_____|
```

[![Rust Version](https://img.shields.io/badge/rust-2021%20edition-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![Zero Inbound Ports](https://img.shields.io/badge/firewall-0%20inbound%20ports-brightgreen.svg?style=flat-square)](https://github.com/xuoxod/conduit)
[![Hardware Isolation](https://img.shields.io/badge/V4L2-RAII%20Camera%20Lock-cyan.svg?style=flat-square)](crates/sentry-hardware)
[![Test Battery](https://img.shields.io/badge/TDD%20Battery-9%2F9%20Passed-success.svg?style=flat-square)](tests)

---

## 🏛️ System Vision: Why Sentry-Edge?

Commercial security cameras (Ring, Nest, Wyze, Arlo) force an unacceptable privacy compromise: they stream raw home/office video to third-party cloud data centers, enforce monthly recurring SaaS fees, and cannot execute programmable edge logic when internet connectivity drops.

**SENTRY-EDGE** delivers the sovereign, zero-cloud alternative:
* 🛡️ **Zero Open Inbound Ports**: Dials *outward-only* to your self-hosted [**Conduit Relay**](https://github.com/xuoxod/conduit) over encrypted TLS WebSockets.
* 🎙️ **Acoustic Noise Watchdog**: Continuous sliding-window RMS audio analysis detects sudden acoustic anomalies ($+20\text{ dB SPL}$ over baseline, glass breaking, sirens, intrusions).
* 📸 **V4L2 Burst Sentinel**: Automatically triggers a 5-frame 1080p JPEG burst upon acoustic breach with hardware LED isolation (camera powers down immediately after frame release).
* ⚡ **Sub-10ms LiveKit SFU Telepresence**: Stream real-time 60FPS video and two-way walkie-talkie intercom directly to any mobile phone browser worldwide.
* 📜 **Immutable SQLite WAL Ledger**: All incidents are cryptographically signed with SHA-256 and persisted in an embedded SQLite WAL database with dark-mode SVG dossier exports.

---

## 🏗️ Ecosystem Architecture

```mermaid
graph TD
    subgraph EdgeDevice["🏠 Physical Sentinel Node (Laptop / Pi / Mini-PC)"]
        Mic["🎙️ ALSA/PipeWire Continuous RMS Stream"]
        Cam["📸 Native V4L2 Device (`/dev/video0`)"]
        SentryDaemon["🛡️ `sentry-edge` Daemon<br/>• Acoustic Analyzer<br/>• Burst Capturer<br/>• SQLite WAL Ledger"]
    end

    subgraph SovereignBackend["🌐 Your Self-Hosted Backend Infrastructure"]
        ConduitRelay["🔌 `conduit-relay` Gateway<br/>(relay.example.com:8084)"]
        LiveKitSFU["⚡ LiveKit SFU Media Server<br/>(sfu.example.com:7880)"]
    end

    subgraph OperatorViewport["📱 Remote Operator (Any Browser / Mobile Phone)"]
        MobileHUD["🌐 Glassmorphic Mobile Web HUD<br/>• Live Sub-10ms WebRTC Video<br/>• 2-Way Push-to-Talk Walkie-Talkie<br/>• Instant Acoustic Anomaly Alerts"]
    end

    Mic --> SentryDaemon
    Cam --> SentryDaemon
    SentryDaemon <== "Outbound-Only TLS WebSocket (0 Open Ports)" ==> ConduitRelay
    SentryDaemon -.->|"Sub-10ms WebRTC DataChannel"| LiveKitSFU
    ConduitRelay <== "Encrypted HTTPS / WSS" ==> MobileHUD
    LiveKitSFU -.->|"Low-Latency Telepresence Stream"| MobileHUD
```

---

## 🚀 Quickstart & Usage

### 1. Build from Source
```bash
git clone https://github.com/xuoxod/sentry-edge.git
cd sentry-edge
cargo build --release
```

### 2. Arm the Sentinel Daemon
```bash
# Launch background sentinel connected to your self-hosted Conduit Relay
./target/release/sentry-edge run \
    --relay-url wss://relay.example.com:8084/ws/outpost \
    --token your-secret-token \
    --label "Server-Room-Sentinel (hyperion-prime)" \
    --camera /dev/video0 \
    --trigger-db 20.0
```

#### What to Expect:
```text
==========================================================================
🛡️  SENTRY-EDGE // AUTONOMOUS SOVEREIGN TELEPRESENCE SENTINEL
==========================================================================
  ✔ Target Relay URL      : wss://relay.example.com:8084/ws/outpost
  ✔ Node Identity Label   : Server-Room-Sentinel (hyperion-prime)
  ✔ Camera Device         : /dev/video0
  ✔ Acoustic Trigger Delta: +20 dB SPL
==========================================================================
▶ Sentinel armed. Monitoring acoustic baseline & camera...
  [dB Meter] Current: 38.2 dB | Baseline: 35.0 dB
  [dB Meter] Current: 39.1 dB | Baseline: 35.4 dB
🚨 [ACOUSTIC SPIKE DETECTED] Sudden sound burst!
  📸 Capturing 3-frame V4L2 snapshot burst...
  ✔ Dispatched SentryWirePacket #1 over Conduit WSS tunnel!
```

### 3. Essential Operator Commands
```bash
# Live real-time audio decibel level meter in terminal
sentry-edge monitor

# Test 880Hz attention warning siren / chime
sentry-edge test-chime

# Capture instant high-resolution V4L2 camera snapshot
sentry-edge snapshot --camera /dev/video0 --output /tmp/snapshot.jpg

# Generate dark-mode HTML security dossier with embedded SVG charts
sentry-edge report --output /tmp/security_dossier.html
```

---

## 🧩 Micro-OJP Multi-Crate Workspace Layout

```
sentry-edge/
├── Cargo.toml                  # Standalone workspace definition (Zero local path dependencies)
├── crates/
│   ├── sentry-core/            # Pure acoustic RMS math, dB SPL calibration & data models
│   ├── sentry-hardware/        # Native Linux V4L2 camera capture & ALSA/PipeWire audio
│   ├── sentry-telepresence/    # Real-time WebRTC LiveKit SFU integration & 2-way intercom
│   ├── sentry-bridge/          # Outbound TLS WebSocket connector linking Sentry to Conduit
│   ├── sentry-ledger/          # SQLite WAL incident persistence & SVG report engine
│   └── sentry-cli/             # Standalone operator CLI, audio meter & daemon binary
```

---

## 🧪 Test Suite & Verification

```bash
cargo test --workspace
```

---

## 📜 Governance & License

Crafted with sovereign pride by **Rick (`xuoxod`)**.  
Licensed under the [MIT License](LICENSE-MIT) or [Apache-2.0 License](LICENSE-APACHE).
