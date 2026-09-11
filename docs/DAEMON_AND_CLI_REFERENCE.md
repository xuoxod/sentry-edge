# 🛠️ SENTRY-EDGE: Exhaustive Daemon & CLI Reference Manual

> **Document ID:** `SENTRY-DOC-CLI-01`  
> **Target Audience:** Edge Node Operators, Homelab Administrators, SREs  
> **Scope:** Full, exhaustive reference for `sentry-daemon.sh` and `sentry-edge` CLI commands, internal mechanics, and real-world example outputs  

---

## 🧭 Overview

`sentry-edge` provides two complementary operational interfaces:
1. **`sentry-daemon.sh`**: The host-level daemon supervisor for background execution, log rotation, soak monitoring, and automated verification.
2. **`sentry-edge`**: The sovereign Rust binary supporting direct CLI invocations for edge monitoring, operator client feeds, telepresence sessions, and hardware profiling.

---

## 📜 Table of Contents
1. [Management Daemon Reference (`sentry-daemon.sh`)](#1-management-daemon-reference-sentry-daemonsh)
   * [1.1 `start`](#11-sentry-daemonsh-start)
   * [1.2 `stop`](#12-sentry-daemonsh-stop)
   * [1.3 `status`](#13-sentry-daemonsh-status)
   * [1.4 `logs`](#14-sentry-daemonsh-logs-n)
   * [1.5 `stats`](#15-sentry-daemonsh-stats)
   * [1.6 `verify`](#16-sentry-daemonsh-verify)
   * [1.7 `report`](#17-sentry-daemonsh-report)
2. [Standalone Binary CLI Reference (`sentry-edge`)](#2-standalone-binary-cli-reference-sentry-edge)
   * [2.1 `--profile` / `profile`](#21---profile--profile)
   * [2.2 `run`](#22-run)
   * [2.3 `client`](#23-client)
   * [2.4 `telepresence`](#24-telepresence)
   * [2.5 `monitor`](#25-monitor)
   * [2.6 `logs`](#26-logs)
   * [2.7 `snapshot`](#27-snapshot)
   * [2.8 `test-chime`](#28-test-chime)
   * [2.9 `report`](#29-report)
   * [2.10 `--generate-config`](#210---generate-config)
3. [Real-World Failure & Recovery Scenarios](#3-real-world-failure--recovery-scenarios)

---

## 1. Management Daemon Reference (`sentry-daemon.sh`)

The daemon management script is installed at `~/.local/bin/sentry-daemon.sh` on the edge node.

### 1.1 `sentry-daemon.sh start`
* **Purpose**: Spawns `sentry-edge run` as a detached background daemon.
* **Under the Hood**:
  1. Checks `~/.config/sentry/sentry_daemon.pid` to prevent duplicate daemon processes.
  2. Spawns `nohup ~/.local/bin/sentry-edge run` in the background.
  3. Redirects stdout/stderr to `~/.config/sentry/logs/sentry_daemon.stdout`.
  4. Stores the child PID into `~/.config/sentry/sentry_daemon.pid`.
* **Example Usage**:
  ```bash
  $ sentry-daemon.sh start
  ```
* **Real-World Output**:
  ```text
  🚀 Launching Sentry-Edge Hardware Sentinel Daemon in background...
  ✔ Sentry daemon started (PID: 999107)
  📄 Stdout Log: /home/rick/.config/sentry/logs/sentry_daemon.stdout
  ```

---

### 1.2 `sentry-daemon.sh stop`
* **Purpose**: Gracefully halts the active background sentinel daemon.
* **Under the Hood**:
  1. Reads PID from `~/.config/sentry/sentry_daemon.pid`.
  2. Sends `SIGINT` (Ctrl+C equivalent), allowing the Tokio async runtime to flush active SQLite WAL transactions, write the final shutdown telemetry record, and close audio/video streams cleanly.
  3. Waits up to 5 seconds for clean termination; if still active, sends `SIGTERM`.
  4. Deletes the PID file.
* **Example Usage**:
  ```bash
  $ sentry-daemon.sh stop
  ```
* **Real-World Output**:
  ```text
  🛑 Stopping Sentry daemon (PID: 999107)...
  ✔ Sentry daemon stopped successfully.
  ```

---

### 1.3 `sentry-daemon.sh status`
* **Purpose**: Reports the real-time operational status, PID, CPU utilization, memory footprint, and elapsed soak uptime.
* **Under the Hood**:
  1. Inspects `~/.config/sentry/sentry_daemon.pid`.
  2. Queries `ps -p <PID> -o pid,user,%cpu,%mem,rss,etime,command` for physical Linux kernel process metrics.
* **Example Usage**:
  ```bash
  $ sentry-daemon.sh status
  ```
* **Real-World Output**:
  ```text
  🟢 Sentry Daemon is RUNNING (PID: 999107)
      PID USER     %CPU %MEM   RSS     ELAPSED COMMAND
   999107 rick      0.0  0.1  5820       08:42 /home/rick/.local/bin/sentry-edge run
  ```

---

### 1.4 `sentry-daemon.sh logs [N]`
* **Purpose**: Formats and streams the last $N$ nanosecond-precision audit events directly from the cryptographic JSONL log.
* **Parameter**: `[N]` — Number of recent log lines to display (default: 20).
* **Under the Hood**:
  1. Invokes `sentry-edge logs --tail N`.
  2. Reads from `~/.config/sentry/logs/sentry_audit.jsonl`.
  3. Formats timestamps with nanosecond precision (`HH:MM:SS.nnnnnnnnn`), color-codes severities (`INFO` vs `ALERT`), and extracts RMS sound levels and picosecond durations.
* **Example Usage**:
  ```bash
  $ sentry-daemon.sh logs 5
  ```
* **Real-World Output**:
  ```text
  ==========================================================================
  🛡️  SENTRY-EDGE // AUTONOMOUS SOVEREIGN TELEPRESENCE SENTINEL
    ✔ Platform Runtime      : linux-x86_64 (family: unix, musl: true, container: false)
  ==========================================================================
    ✔ [MODE]                : HYPER-METICULOUS NANOSECOND TELEMETRY AUDIT
    ✔ Log Source            : SQLite WAL Ledger & JSONL Stream
  ==========================================================================
  01:14:39.578707871 [INFO ] [AUDIO_DSP] Ambient DSP baseline tracking nominal (38.2dB) [RMS: 38.2dB, Base: 38.2dB, Δ: +0.0dB] (took 18517ns / 18765000ps) #8
  01:14:39.579013017 [INFO ] [AUDIO_DSP] Ambient DSP baseline tracking nominal (38.2dB) [RMS: 38.2dB, Base: 38.2dB, Δ: +0.0dB] (took 18794ns / 19084000ps) #9
  01:14:39.579397882 [INFO ] [AUDIO_DSP] Ambient DSP baseline tracking nominal (38.2dB) [RMS: 38.2dB, Base: 38.2dB, Δ: +0.0dB] (took 19621ns / 19887000ps) #10
  01:15:02.129481903 [ALERT] [CAMERA_V4L2] Acoustic spike breach (+23.4dB) at Server Rack 01 [RMS: 61.6dB, Base: 38.2dB, Δ: +23.4dB] (took 18095ns / 18248000ps) #11
  01:15:35.891204855 [INFO ] [AUDIO_DSP] Ambient DSP baseline tracking nominal (38.2dB) [RMS: 38.2dB, Base: 38.2dB, Δ: +0.0dB] (took 17641ns / 17786000ps) #12
  ```

---

### 1.5 `sentry-daemon.sh stats`
* **Purpose**: Compiles high-level soak analytics, ambient noise floor trends, and subsystem event distributions.
* **Under the Hood**:
  1. Invokes `sentry-edge logs --stats`.
  2. Aggregates records by `SubsystemTag` (`AudioDsp`, `CameraV4l2`, `SystemHeartbeat`).
  3. Computes average SQLite commit latency in nanoseconds and picoseconds.
* **Example Usage**:
  ```bash
  $ sentry-daemon.sh stats
  ```
* **Real-World Output**:
  ```text
  ==========================================================================
  📊 Telemetry Subsystem & Endurance Analytics (242 Total Events):
    ✔ [AUDIO_DSP] Events    : 236
    ✔ [CAMERA_V4L2] Bursts  : 6
    ✔ Ambient Noise Floor   : Avg 38.2 dB SPL (Peak: 61.6 dB SPL)
    ✔ Avg Commit Latency    : 28,000 ns (28.0 µs / 28,000,000 ps)
    ✔ Hash Chain Genesis    : 0000000000000000... (Verified)
  ==========================================================================
  ```

---

### 1.6 `sentry-daemon.sh verify`
* **Purpose**: Validates the cryptographic tamper-evident SHA-256 hash chain of the entire audit stream.
* **Under the Hood**:
  1. Reads all records from `~/.config/sentry/logs/sentry_audit.jsonl`.
  2. Iteratively recomputes $\text{SHA256}(\text{Seq} \parallel \text{TimestampNs} \parallel \text{Summary} \parallel \text{PrevHash})$ for every block.
  3. Returns green confirmation if 100% unbroken; otherwise flags exact corrupted record index.
* **Example Usage**:
  ```bash
  $ sentry-daemon.sh verify
  ```
* **Real-World Output**:
  ```text
  ==========================================================================
  🔐 Verifying Cryptographic SHA-256 Hash Chain (242 records)... ✔ 100% UNBROKEN & TAMPER-FREE
  ```

---

### 1.7 `sentry-daemon.sh report`
* **Purpose**: Compiles stored SQLite security incidents into a zero-CDN standalone dark-mode HTML dossier.
* **Under the Hood**:
  1. Queries SQLite ledger at `~/.config/sentry/data/sentry_ledger.db`.
  2. Renders incident timeline, acoustic decibel bar charts, and SHA-256 signatures into `~/.config/sentry/reports/sentry_dossier.html`.
* **Example Usage**:
  ```bash
  $ sentry-daemon.sh report
  ```
* **Real-World Output**:
  ```text
  📊 Compiling security dossier...
  ✔ Report saved to /home/rick/.config/sentry/reports/sentry_dossier.html
  ```

---

## 2. Standalone Binary CLI Reference (`sentry-edge`)

### 2.1 `--profile` / `profile`
* **Purpose**: Inspect host runtime environment, CPU architecture, static musl linkage, and active HAL driver mappings.
* **Usage**:
  ```bash
  sentry-edge --profile
  # or
  sentry-edge profile
  ```
* **Output**:
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

### 2.2 `run`
* **Purpose**: Launch the active hardware watchdog sentinel in foreground interactive mode.
* **Options**:
  * `--relay-url <URL>`: Override self-hosted Conduit Relay gateway (default: from config).
  * `--token <SECRET>`: Master auth token for pairing.
  * `--label <NAME>`: Node identity label (e.g. `crunchbang-laptop`).
  * `--camera <DEVICE>`: Camera device node (e.g. `/dev/video0`).
  * `--trigger-db <FLOAT>`: Acoustic jump threshold above baseline in dB SPL (default: `20.0`).
* **Usage**:
  ```bash
  sentry-edge run --label "crunchbang-laptop" --trigger-db 20.0
  ```

---

### 2.3 `client`
* **Purpose**: Connect as a remote operator on desktop or phone to ingest live sentinel alerts and verify camera feeds.
* **Options**:
  * `--relay-url <URL>`: WebSocket endpoint of Conduit Relay.
  * `--token <SECRET>`: Authentication token.
  * `--operator <NAME>`: Operator identifier.
  * `--watch-node <LABEL>`: Target sentinel node label to monitor.
* **Usage**:
  ```bash
  sentry-edge client --operator "Operator-Rick" --watch-node "crunchbang-laptop"
  ```
* **Output**:
  ```text
  ==========================================================================
  🛡️  SENTRY-EDGE // AUTONOMOUS SOVEREIGN TELEPRESENCE SENTINEL
  ==========================================================================
    ✔ [MODE]                : END-USER REMOTE OPERATOR CLIENT
    ✔ Target Relay URL      : wss://relay.example.com:8084/ws/operator
    ✔ Operator Identity     : Operator-Rick
    ✔ Watching Node         : crunchbang-laptop
  ==========================================================================
  ▶ Connected to Conduit Relay. Awaiting live edge sentinel alerts...

  [CRITICAL] [22:27:06] 🚨 Node: crunchbang-laptop >> Acoustic Spike (+26.3 dB over baseline | Peak: 91.4 dB)
     Description: Acoustic threshold breach (91.4 dB peak)
     Integrity SHA-256: 903ba9da620e5a89...

  ✔ Alert verified & cryptographically authenticated.
  ```

---

### 2.4 `telepresence`
* **Purpose**: Establish sub-10ms LiveKit SFU WebRTC room session with full-duplex walkie-talkie audio and 60FPS video.
* **Options**:
  * `--sfu-url <URL>`: LiveKit SFU endpoint (e.g. `https://sfu.example.com:7880`).
  * `--target-node <LABEL>`: Sentinel node to dial.
  * `--operator <NAME>`: Operator name.
* **Usage**:
  ```bash
  sentry-edge telepresence --target-node crunchbang-laptop --operator rick
  ```
* **Output**:
  ```text
  ✔ Connecting WebRTC LiveKit SFU Telepresence...
  ✔ LiveKit Room Created  : sentry-telepresence-crunchbang-laptop
  ✔ Auth Token Generated  : eyJhbGciOiJIUzI1NiIsInR5cCI6...
  ▶ Full-Duplex WebRTC Walkie-Talkie & 60FPS Video Active!
  ```

---

### 2.5 `monitor`
* **Purpose**: Stream a live terminal decibel meter with graphical VU bars.
* **Usage**:
  ```bash
  sentry-edge monitor
  ```
* **Output**:
  ```text
  ▶ Streaming live audio dB meter (Press Ctrl+C to stop)...
    [AUDIO]  68.1 dB SPL  █████████████
    [AUDIO]  70.8 dB SPL  ██████████████
    [AUDIO]  67.3 dB SPL  █████████████
  ```

---

### 2.6 `logs`
* **Purpose**: Query, tail, filter, and verify nanosecond telemetry records.
* **Options**:
  * `-t, --tail <N>`: Display last $N$ records (default: 20).
  * `--json`: Output raw JSONL lines for script piping.
  * `--verify-chain`: Verify cryptographic SHA-256 hash links.
  * `--stats`: Display summary statistics.
  * `-e, --export <PATH>`: Write formatted JSON audit trail to destination file.
* **Usage**:
  ```bash
  # Tail last 50 records in JSON format
  sentry-edge logs --tail 50 --json

  # Export complete audit trail
  sentry-edge logs --export /tmp/audit_trail.json
  ```

---

### 2.7 `snapshot`
* **Purpose**: Trigger an instant test camera frame capture and save to disk.
* **Options**:
  * `-c, --camera <DEVICE>`: Video device path (default: `/dev/video0`).
  * `-o, --output <PATH>`: Destination JPEG image path.
* **Usage**:
  ```bash
  sentry-edge snapshot --camera /dev/video0 --output /tmp/test_frame.jpg
  ```

---

### 2.8 `test-chime`
* **Purpose**: Play an 880Hz attention tone / warning siren over the host speaker.
* **Usage**:
  ```bash
  sentry-edge test-chime
  ```

---

### 2.9 `report`
* **Purpose**: Compile an HTML security dossier with embedded SVG charts.
* **Options**:
  * `-o, --output <PATH>`: Output HTML destination file.
* **Usage**:
  ```bash
  sentry-edge report --output /tmp/dossier.html
  ```

---

### 2.10 `--generate-config`
* **Purpose**: Create a clean starter configuration template in the OS standard configuration directory.
* **Usage**:
  ```bash
  sentry-edge --generate-config
  ```
* **Output**:
  ```text
  ==========================================================================
  ⚙️   SENTRY-EDGE // CONFIGURATION GENERATOR
  ==========================================================================
    ✔ Target File Path      : /home/rick/.config/sentry/sentry.toml
    ✔ Platform Profile      : linux-x86_64 (family: unix, musl: true, container: false)
    ✔ Template Status       : Successfully Generated Starter Configuration
  ==========================================================================
  ```

---

## 3. Real-World Failure & Recovery Scenarios

| Scenario | Symptom | Root Cause | Automatic / Manual Recovery |
| :--- | :--- | :--- | :--- |
| **Abrupt Power Loss / Hard Reset** | Machine reboots during 72-hour soak | Host power cut | SQLite WAL journal automatically recovers uncommitted pages on next boot. Run `sentry-daemon.sh start` and `sentry-daemon.sh verify`. |
| **Microphone Disconnected / Busy** | Warning in stdout log | Physical mic unplugged or used by another app | HAL driver automatically falls back to `ProceduralAudioInput` baseline without crashing. |
| **Camera Unplugged / Missing Node** | `/dev/video0` not found | USB camera disconnected | HAL driver automatically generates valid synthetic JPEG security frames with watermarked node timestamp. |
| **Log Injection Attempt (CRLF)** | Malicious string with `\r\n` | Adversary injecting fake log headers | `LogSanitizer` automatically neutralizes control characters (`\r` $\to$ `\u{240D}`, `\n` $\to$ `\u{240A}`). |
| **Log File Tampering** | Attacker deleted line in JSONL | Unauthorized modification | `sentry-daemon.sh verify` fails instantly and flags the corrupted sequence index. |
