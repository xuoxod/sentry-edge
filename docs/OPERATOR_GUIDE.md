# 🛡️ SENTRY-EDGE: Comprehensive Operator Guide & Real-World Field Manual

> **Document ID:** `SENTRY-DOC-OP-01`  
> **Target Audience:** Edge Operators, Homelab Administrators, Security Engineers, Sovereign Conduit Users  
> **Scope:** Real-World Hardware Deployment, Background Soak Testing, Acoustic DSP Adaptation, Telemetry Auditing & Troubleshooting  

---

## 🧭 Executive Summary

**SENTRY-EDGE** (`sentry-edge-rs`) is an autonomous sovereign edge sentinel and acoustic decibel watchdog designed to operate continuously on edge devices (laptops, mini-PCs, Raspberry Pis, rack servers) without relying on third-party cloud surveillance services or opening inbound firewall ports.

This guide provides end-to-end operational instructions for deploying, running, monitoring, and verifying `sentry-edge` in real-world environments.

---

## ⚡ Real-World Deployment & What to Expect

### 1. Acoustic Baseline & Adaptive DSP Adaptation

When `sentry-edge` boots, it immediately connects to the host audio input device (e.g. ALSA microphone on Linux, CoreAudio on macOS, WASAPI on Windows) and samples room acoustic energy.

```text
==========================================================================
🛡️  SENTRY-EDGE // AUTONOMOUS SOVEREIGN TELEPRESENCE SENTINEL
  ✔ Platform Runtime      : linux-x86_64 (family: unix, musl: true, container: false)
==========================================================================
  ✔ [MODE]                : EDGE SENTINEL HARDWARE DAEMON
  ✔ Target Relay URL      : wss://relay.example.com:8084/ws/outpost
  ✔ Node Identity Label   : crunchbang-laptop
  ✔ Camera Device         : /dev/video0
  ✔ Acoustic Trigger Delta: +20.0 dB SPL
==========================================================================
▶ Sentinel armed. Monitoring acoustic baseline & camera...
  ✔ Telemetry Log Path    : /home/rick/.config/sentry/logs/sentry_audit.jsonl
  ✔ Active HAL Profile    : Audio [Alsa], Camera [V4l2]
  ✔ Continuous Loop       : Active (Press Ctrl+C to stop)
==========================================================================
```

#### How the Audio DSP Analyzer Works:
* **Sound Pressure Level Reference**: Audio samples are converted to calibrated Root-Mean-Square (RMS) decibels sound pressure level ($\text{dB SPL}$).
  * **Quiet Room / Ambient Whisper**: $\approx 30 - 45\text{ dB SPL}$
  * **Normal TV / Background Conversation**: $\approx 60 - 72\text{ dB SPL}$
  * **Sudden Noise (Door Slam, Clapping, Shouting, Dropped Keys)**: $\approx 85 - 105+\text{ dB SPL}$
* **Exponential Moving Average (EMA)**: The baseline noise level dynamically adapts to constant environmental background sounds using an EMA smoothing factor ($\alpha = 0.15$). If a television or fan is running in the room, the sentinel will smoothly establish the TV volume as the new baseline over several seconds rather than triggering false alarms.
* **Trigger Threshold**: When a sudden acoustic jump occurs ($\Delta \ge +20\text{ dB SPL}$ above the adapted baseline and absolute volume $> 45\text{ dB SPL}$), the sentinel instantly flags an acoustic breach.

---

### 2. What Happens When Noise Occurs in the Room

```text
--------------------------------------------------------------------------
  [dB Monitor #10]: Current  68.2 dB | Baseline:  69.0 dB  █████████████
  [dB Monitor #20]: Current  68.4 dB | Baseline:  65.1 dB  █████████████
--------------------------------------------------------------------------
🚨 [ACOUSTIC SPIKE DETECTED] Peak: 91.4 dB SPL (Δ +26.3 dB)
  📸 Capturing 5-frame optical burst from /dev/video0...
  🔐 Computing SHA-256 event signature: e4f89a2b...
  💾 Persisting alert to SQLite WAL Ledger (/home/rick/.config/sentry/sentry_ledger.db)
  📡 Dispatching cryptographically signed SentryAlert over Conduit WSS Bridge
  ⛓️ Appending nanosecond telemetry record to cryptographic SHA-256 hash chain
--------------------------------------------------------------------------
```

Upon a trigger:
1. **Camera Burst**: The camera capturer executes a rapid multi-frame optical burst (default: 5 frames) with RAII device isolation.
2. **Cryptographic Alert Creation**: Creates a `SentryAlert` containing the node UUID, timestamp, incident type, peak/baseline dB, and base64 snapshot payload.
3. **SHA-256 Signature**: Generates an immutable SHA-256 payload signature preventing in-flight tampering.
4. **Ledger Commit**: Inserts the incident into the local embedded SQLite WAL database.
5. **Nanosecond Telemetry Logging**: Records picosecond-accurate elapsed time (`shutter_latency_ps`, `duration_ps`), system metrics (`cpu_rss_mb`, `wal_page_count`), and chains the event into the tamper-evident SHA-256 hash log.

---

## 💡 The "If You Wanted to Know / See If..." Operator Playbook

Use this quick-reference playbook whenever you need to inspect or verify specific sentinel behaviors:

### 1. *If you want to know if the sentinel is actively listening to the room:*
Run the real-time audio decibel meter HUD:
```bash
sentry-edge monitor
```
**Expected Output:**
```text
▶ Streaming live audio dB meter (Press Ctrl+C to stop)...
  [AUDIO]  68.1 dB SPL  █████████████
  [AUDIO]  70.8 dB SPL  ██████████████
  [AUDIO]  67.3 dB SPL  █████████████
```

---

### 2. *If you want to see if an acoustic spike was triggered:*
Check the live daemon stdout or query recent logs:
```bash
# On local workstation or over SSH:
sentry-daemon.sh logs 10
# Or inspect daemon stdout log:
tail -f ~/.config/sentry/logs/sentry_daemon.stdout
```

---

### 3. *If you want to verify that the audit log has NOT been tampered with:*
Run cryptographic blockchain verification:
```bash
sentry-edge logs --verify-chain
# Or via daemon management script:
sentry-daemon.sh verify
```
**Expected Output:**
```text
🔐 Verifying Cryptographic SHA-256 Hash Chain (142 records)... ✔ 100% UNBROKEN & TAMPER-FREE
```
> If any record in the JSONL stream is deleted, modified, or reordered by an attacker, verification will immediately fail and report the exact corrupted sequence number.

---

### 4. *If you want to inspect host platform & HAL driver mappings:*
Inspect the active Hardware Abstraction Layer profile:
```bash
sentry-edge --profile
```
**Expected Output:**
```text
==========================================================================
🔬  SENTRY-EDGE // HARDWARE ABSTRACTION LAYER (HAL) PROFILE
==========================================================================
  ✔ Host OS               : linux
  ✔ CPU Architecture      : x86_64
  ✔ Static Musl Binary    : YES (Static Linked)
  ✔ Container Environment : NO (Bare Metal Hardware)
  ✔ Config Path           : /home/rick/.config/sentry/sentry.toml
  ✔ Ledger Database       : /home/rick/.config/sentry/data/sentry_ledger.db
  ✔ Audit Log Path        : /home/rick/.config/sentry/logs/sentry_audit.jsonl
--------------------------------------------------------------------------
  🔊 Audio Input Driver   : Alsa -> Native Linux ALSA Sound Card
  📢 Audio Output Driver  : Alsa -> Linux ALSA Audio Output Transducer
  📷 Camera Sensor Driver : V4l2 -> Native Linux Video4Linux2 Camera
==========================================================================
```

---

### 5. *If you want to test the camera sensor manually:*
Capture an instant JPEG frame to verify lens focus and illumination:
```bash
sentry-edge snapshot --camera /dev/video0 --output /tmp/snapshot_test.jpg
```

---

### 6. *If you want to test the 880Hz attention siren / warning chime:*
```bash
sentry-edge test-chime
```

---

### 7. *If you want to generate an interactive HTML security dossier:*
Compile all stored SQLite incidents into a zero-CDN standalone HTML compliance report:
```bash
sentry-edge report --output /tmp/sentry_dossier.html
```

---

### 8. *If you want to export raw nanosecond audit logs for external SIEM:*
```bash
sentry-edge logs --json --tail 500 --export /tmp/sentry_audit_export.json
```

---

## 🛠️ Management Daemon: `sentry-daemon.sh`

For long-running 2–3 day endurance soak tests, `sentry-daemon.sh` provides standard process lifecycle management:

| Command | Action |
| :--- | :--- |
| `sentry-daemon.sh start` | Launches sentinel daemon in the background with output redirected to `~/.config/sentry/logs/sentry_daemon.stdout`. |
| `sentry-daemon.sh stop` | Gracefully terminates background daemon via `SIGINT` (allows flushing WAL & hash chain). |
| `sentry-daemon.sh status` | Checks PID, CPU%, Memory%, RSS memory, and elapsed uptime. |
| `sentry-daemon.sh logs [N]` | Tails the last $N$ formatted nanosecond telemetry audit records. |
| `sentry-daemon.sh stats` | Summarizes event breakdown (`AudioDsp`, `CameraV4l2`), ambient noise floor average, commit latency. |
| `sentry-daemon.sh verify` | Cryptographically verifies the unbroken SHA-256 hash chain. |
| `sentry-daemon.sh report` | Compiles an updated HTML incident dossier into `~/.config/sentry/reports/`. |

---

## 🧠 The GLIBC & Portability Learning-Lesson

### The Problem Encountered
When transferring compiled Rust binaries from a modern development machine (e.g. Ubuntu 24.04 with `glibc 2.39`) to an older or stripped edge system (e.g. Debian 12 / Crunchbang++ with `glibc 2.36`), running the binary fails immediately at dynamic loader invocation:

```text
$ ./bin/sentry-edge
./bin/sentry-edge: /lib/x86_64-linux-gnu/libc.so.6: version `GLIBC_2.39' not found (required by ./bin/sentry-edge)
```

### Why Dynamic Linking Fails for Sovereign Edge Software
1. **Dynamic C Library Version Lock**: A binary linked dynamically against glibc 2.39 requires symbols (`__isoc23_strtol`, `posix_spawn`, etc.) that do not exist on older Linux distributions.
2. **Non-Agnostic File & Hardware Assumptions**: Direct system calls to Linux-only `/dev/video0` or ALSA kernel structures crash on macOS, Windows, or headless container environments.

### The Permanent Sovereign Architectural Solution
1. **Static Musl Compilation (`x86_64-unknown-linux-musl`)**:
   - `sentry-edge` is compiled with static musl libc (`crt-static`).
   - The resulting executable is a completely standalone ELF static binary with **zero dynamic library dependencies** (`ldd sentry-edge` reports `statically linked`).
   - It runs seamlessly on any Linux kernel $\ge 3.2$ regardless of distro (Debian, Ubuntu, Arch, Alpine, CentOS, Rocky, Void).
2. **Trait-Based Hardware Abstraction Layer (HAL)**:
   - Physical hardware interfaces are isolated behind abstract Rust traits (`AudioInputDevice`, `AudioOutputDevice`, `CameraDevice`).
   - If physical hardware is busy or missing (e.g. CI environments, cloud VMs, sandboxed macOS), the system seamlessly delegates to `ProceduralAudioInput` and `ProceduralCamera` without panicking.
3. **Cross-Platform Path Resolution (`PlatformPaths`)**:
   - Automatically adapts directory structures to the host OS (`XDG_CONFIG_HOME` on Linux/BSD, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows).

---

## ⚙️ Configuration Reference (`~/.config/sentry/sentry.toml`)

```toml
[node]
label = "crunchbang-laptop"
environment = "production"

[network]
relay_url = "wss://relay.example.com:8084/ws/outpost"
auth_token = "sentry-dev-99x"
connect_timeout_secs = 10
heartbeat_interval_secs = 30

[hardware]
# Camera device node (or virtual path)
camera_device = "/dev/video0"
# Sound input card (default, hw:0,0, or pulse)
audio_device = "default"
# Initial ambient acoustic baseline (dB SPL)
acoustic_baseline_db = 38.0
# Sound jump above baseline to trigger security breach (+dB SPL)
acoustic_trigger_delta_db = 20.0
# Multi-frame optical snapshot count upon trigger
snapshot_burst_count = 5

[telepresence]
sfu_url = "https://sfu.example.com:7880"
default_room = "crunchbang-telepresence-room"

[storage]
ledger_db_path = "/home/rick/.config/sentry/data/sentry_ledger.db"
export_dir = "/home/rick/.config/sentry/reports"
```
