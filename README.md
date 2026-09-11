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
[![Zero Dynamic Glibc](https://img.shields.io/badge/binary-static--musl%20zero--glibc-brightgreen.svg?style=flat-square)](docs/ARCHITECTURE_AND_HAL.md)
[![Hardware HAL](https://img.shields.io/badge/HAL-Trait--Based%20Multi--OS-cyan.svg?style=flat-square)](crates/sentry-hardware)
[![Test Battery](https://img.shields.io/badge/TDD%20Battery-47%2F47%20Passed%20(100%25)-success.svg?style=flat-square)](tests)

---

## 📚 Dedicated Documentation & Field Manuals

* 📖 [**Comprehensive Operator Guide & Field Manual**](docs/OPERATOR_GUIDE.md): Real-world deployment walkthrough, ambient TV DSP adaptation, acoustic spike mechanics, and the *"If You Wanted to Know / See If..."* operator playbook.
* 🛠️ [**Exhaustive Daemon & CLI Reference Manual**](docs/DAEMON_AND_CLI_REFERENCE.md): Full command-by-command reference for `sentry-daemon.sh` (`start|stop|status|logs|stats|verify|report`) and all `sentry-edge` CLI subcommands with internal mechanics and real-world failure recovery.
* 🏛️ [**Architecture & HAL Specification**](docs/ARCHITECTURE_AND_HAL.md): Trait-based Hardware Abstraction Layer, multi-OS drivers (Linux, macOS, Windows, Procedural), static-musl zero-glibc compilation, and nanosecond/picosecond cryptographic hash-chain engine.


---

## 🏛️ System Vision: Why Sentry-Edge?

Commercial security cameras (Ring, Nest, Wyze, Arlo) force an unacceptable privacy compromise: they stream raw home/office video to third-party cloud data centers, enforce monthly recurring SaaS fees, and cannot execute programmable edge logic when internet connectivity drops.

**SENTRY-EDGE** delivers the sovereign, zero-cloud alternative:
* 🛡️ **Zero Open Inbound Ports**: Dials *outward-only* to your self-hosted [**Conduit Relay**](https://github.com/xuoxod/conduit) over encrypted TLS WebSockets.
* 🎙️ **Acoustic Noise Watchdog**: Continuous sliding-window RMS audio analysis detects sudden acoustic anomalies ($+20\text{ dB SPL}$ over baseline, glass breaking, sirens, intrusions).
* 📸 **V4L2 Burst Sentinel**: Automatically triggers a 5-frame 1080p JPEG burst upon acoustic breach with hardware LED isolation (camera powers down immediately after frame release).
* ⚡ **Sub-10ms LiveKit SFU Telepresence**: Stream real-time 60FPS video and two-way walkie-talkie intercom directly to any mobile phone browser worldwide.
* ⏱️ **Nanosecond / Picosecond Provenance**: Meticulous telemetry tracking (`Who`, `From`, `To`, `What`, `How`, `Minutiae`) sealed with an immutable cryptographic SHA-256 blockchain hash chain.
* 🔬 **Trait-Based HAL**: Native Linux ALSA & V4L2, macOS CoreAudio & AVFoundation, Windows WASAPI & MediaFoundation, with automatic procedural simulation fallback.
* 📦 **100% Platform-Agnostic Static Binary**: Built with `x86_64-unknown-linux-musl` and `crt-static`, eliminating all `GLIBC_X.XX not found` library mismatches.

---

## 🏗️ Ecosystem Architecture

```mermaid
graph TD
    subgraph EdgeDevice["🏠 Physical Sentinel Node (Laptop / Pi / Mini-PC)"]
        Mic["🎙️ ALSA/PipeWire Continuous RMS Stream"]
        Cam["📸 Native V4L2 Device (`/dev/video0`)"]
        SentryDaemon["🛡️ `sentry-edge run`<br/>• Acoustic Analyzer<br/>• Burst Capturer<br/>• SQLite WAL Ledger<br/>• Nanosecond Telemetry Engine"]
    end

    subgraph SovereignBackend["🌐 Your Self-Hosted Backend Infrastructure"]
        ConduitRelay["🔌 `conduit-relay` Gateway<br/>(relay.example.com:8084)"]
        LiveKitSFU["⚡ LiveKit SFU Media Server<br/>(sfu.example.com:7880)"]
    end

    subgraph OperatorViewport["📱 Remote End-User / Operator Client"]
        ClientApp["💻 `sentry-edge client`<br/>• Live Alert Ingestion<br/>• Instant Snapshot View<br/>• Push-to-Talk Intercom"]
        MobileHUD["🌐 Mobile Web HUD (Any Browser)<br/>• Sub-10ms LiveKit SFU Video"]
    end

    Mic --> SentryDaemon
    Cam --> SentryDaemon
    SentryDaemon <== "Outbound-Only TLS WebSocket (0 Open Ports)" ==> ConduitRelay
    SentryDaemon -.->|"Sub-10ms WebRTC DataChannel"| LiveKitSFU
    ConduitRelay <== "Encrypted WebSocket Stream" ==> ClientApp
    ConduitRelay <== "Encrypted HTTPS / WSS" ==> MobileHUD
    LiveKitSFU -.->|"Low-Latency Telepresence Stream"| ClientApp
    LiveKitSFU -.->|"Low-Latency Telepresence Stream"| MobileHUD
```

---

## 🚀 Quickstart & Usage

### 1. Build Zero-Dependency Static Release Binary
```bash
# Build 100% standalone static binary (zero host glibc dependencies)
cargo build --release --target x86_64-unknown-linux-musl

# Verify static linking
file target/x86_64-unknown-linux-musl/release/sentry-edge
```

---

### 2. Inspect Host Hardware & HAL Profile
```bash
./target/x86_64-unknown-linux-musl/release/sentry-edge --profile
```

---

### 3. Launch Edge Sentinel Hardware Daemon
```bash
./target/x86_64-unknown-linux-musl/release/sentry-edge run \
    --relay-url wss://relay.example.com:8084/ws/outpost \
    --token sentry-dev-99x \
    --label "crunchbang-laptop" \
    --camera /dev/video0 \
    --trigger-db 20.0
```

---

### 4. Connect Remote Operator Client
```bash
./target/x86_64-unknown-linux-musl/release/sentry-edge client \
    --relay-url wss://relay.example.com:8084/ws/operator \
    --operator "Operator-Rick" \
    --watch-node "crunchbang-laptop"
```

---

### 5. Essential Operator Commands

| Task | Command |
| :--- | :--- |
| **Real-Time Sound Meter HUD** | `sentry-edge monitor` |
| **Test Warning Siren / Chime** | `sentry-edge test-chime` |
| **Capture Instant Snapshot** | `sentry-edge snapshot --camera /dev/video0 --output /tmp/test.jpg` |
| **Inspect Nanosecond Logs** | `sentry-edge logs --tail 20` |
| **Verify SHA-256 Hash Chain** | `sentry-edge logs --verify-chain` |
| **View Telemetry Statistics** | `sentry-edge logs --stats` |
| **Export HTML Dossier** | `sentry-edge report --output /tmp/sentry_dossier.html` |

---

## 💡 The "If You Wanted to Know..." Quick Reference

* **If you want to know if the sentinel is actively listening:** Run `sentry-edge monitor`.
* **If you want to see if an acoustic spike was triggered:** Run `sentry-edge logs --tail 10` or check `sentry-daemon.stdout`.
* **If you want to verify that no logs were modified or tampered with:** Run `sentry-edge logs --verify-chain`.
* **If you want to view detected OS, CPU arch, and active HAL drivers:** Run `sentry-edge --profile`.
* **If you want to customize trigger thresholds or database paths:** Edit `~/.config/sentry/sentry.toml`.

---

## 🧩 Micro-OJP Multi-Crate Layout

```
sentry-edge/
├── Cargo.toml                  # Standalone workspace definition (Zero local path dependencies)
├── .cargo/config.toml          # Static-musl crt-static configuration
├── crates/
│   ├── sentry-core/            # Pure DSP acoustic math, PlatformInfo, PlatformPaths, SentryAlert
│   ├── sentry-hardware/        # Trait-based HAL: Linux (ALSA/V4L2), macOS, Windows, Procedural
│   ├── sentry-telemetry/       # Picosecond timers, provenance model, SHA-256 hash-chain engine
│   ├── sentry-bridge/          # Outbound TLS WebSocket connector linking Sentry to Conduit
│   ├── sentry-telepresence/    # Real-time WebRTC LiveKit SFU integration & 2-way intercom
│   ├── sentry-ledger/          # SQLite WAL incident persistence & SVG report engine
│   ├── sentry-client/          # End-user remote operator application & alert viewer
│   └── sentry-cli/             # Standalone operator CLI, audio meter & daemon binary
└── docs/
    ├── OPERATOR_GUIDE.md       # Complete real-world field manual & operator playbook
    └── ARCHITECTURE_AND_HAL.md # Architectural & Hardware Abstraction Layer specification
```

---

## 🧪 Test Battery & Verification

```bash
cargo test --workspace
# 47 / 47 Tests Passed (100% Success) across 8 Micro-OJP crates
```

---

## 📜 Governance & License

Crafted with sovereign pride by **Rick (`xuoxod`)**.  
Licensed under the [MIT License](LICENSE-MIT) or [Apache-2.0 License](LICENSE-APACHE).
