# 🏛️ SENTRY-EDGE: Architecture, Hardware Abstraction Layer (HAL) & Telemetry Specification

> **Document ID:** `SENTRY-DOC-ARCH-01`  
> **Target Audience:** Core Contributors, System Architects, Security Auditors  
> **Status:** Production Reference Specification  

---

## 🧭 Multi-Crate Micro-OJP Taxonomy

`sentry-edge` is architected strictly under the **One Job Principle (OJP)** across 8 standalone crates:

```
sentry-edge/
├── Cargo.toml                  # Workspace manifest with target profiles
├── crates/
│   ├── sentry-core/            # DSP math, acoustic RMS, PlatformInfo, PlatformPaths, SentryAlert
│   ├── sentry-hardware/        # Trait-based HAL: Linux (ALSA/V4L2), macOS, Windows, Procedural
│   ├── sentry-telemetry/       # Picosecond timers, provenance model, SHA-256 hash-chain engine
│   ├── sentry-bridge/          # Outbound-only TLS WebSocket tunnel client for Conduit
│   ├── sentry-telepresence/    # LiveKit SFU JWT generation & WebRTC 2-way intercom bridge
│   ├── sentry-ledger/          # Embedded SQLite WAL incident database & HTML/SVG dossier renderer
│   ├── sentry-client/          # Remote operator HUD and live alert ingestion client
│   └── sentry-cli/             # Binary entrypoint: daemon, client, monitor, profile, logs
└── docs/
    ├── OPERATOR_GUIDE.md       # Real-world field manual & operator playbook
    └── ARCHITECTURE_AND_HAL.md # This architectural & HAL design specification
```

---

## 🔬 Hardware Abstraction Layer (HAL) Architecture

The HAL decouples physical device access into 3 core traits with `Send + Sync` invariants:

```mermaid
classDiagram
    class AudioInputDevice {
        <<trait>>
        +name() &str
        +is_available() bool
        +sample_ambient_db() SentryResult~f32~
        +capture_samples(count) SentryResult~Vec~i16~~
        +driver_info() DriverInfo
    }

    class AudioOutputDevice {
        <<trait>>
        +name() &str
        +is_available() bool
        +play_warning_chime(freq, ms) SentryResult
        +driver_info() DriverInfo
    }

    class CameraDevice {
        <<trait>>
        +path() &str
        +is_available() bool
        +capture_frame() SentryResult~Vec~u8~~
        +capture_burst(count) SentryResult~Vec~Vec~u8~~~
        +driver_info() DriverInfo
    }

    class HardwareFactory {
        +create_audio_input(name) Box~AudioInputDevice~
        +create_audio_output(name) Box~AudioOutputDevice~
        +create_camera(path) Box~CameraDevice~
        +detect_profile(audio, cam) HardwareProfile
    }

    AudioInputDevice <|-- LinuxAlsaAudioInput
    AudioInputDevice <|-- MacOsCoreAudioInput
    AudioInputDevice <|-- WindowsWasapiAudioInput
    AudioInputDevice <|-- ProceduralAudioInput

    CameraDevice <|-- LinuxV4l2Camera
    CameraDevice <|-- MacOsAvFoundationCamera
    CameraDevice <|-- WindowsMediaFoundationCamera
    CameraDevice <|-- ProceduralCamera

    HardwareFactory ..> AudioInputDevice
    HardwareFactory ..> CameraDevice
```

### Driver Dispatch Strategy
1. **Compile-Time Dispatch**: Conditional compilation (`#[cfg(target_os = "...")]`) selects the host-native HAL drivers.
2. **Runtime Auto-Detection**: If physical device nodes (e.g. `/dev/video0`, ALSA soundcard) are absent, busy, or unreadable due to container boundaries or permissions, the driver **gracefully falls back to procedural simulation** rather than panicking.
3. **Telemetry Tagging**: Every record contains the concrete `driver_type` (`Alsa`, `CoreAudio`, `Wasapi`, `ProceduralSynthetic`) for complete audit provenance.

---

## ⏱️ Nanosecond / Picosecond Telemetry Engine (`sentry-telemetry`)

The telemetry engine delivers forensic-grade visibility into all edge operations:

```json
{
  "sequence": 42,
  "timestamp_utc": "2026-09-11T01:10:52.533455681Z",
  "timestamp_unix_ns": 1789089052533455681,
  "node_id": "crunchbang-laptop",
  "subsystem": "CameraV4l2",
  "severity": "Alert",
  "who": {
    "identity": "sentry-daemon",
    "token_prefix": "sentry-dev-99x",
    "session_id": "850ad835-1ca3-4ccc-ab9a-a3ff386f1e18",
    "peer_id": null
  },
  "from": {
    "source_device": "default",
    "thread_id": "main-daemon",
    "physical_addr": null,
    "endpoint": "local-hardware"
  },
  "to": {
    "destination_hardware": "/dev/video0",
    "remote_relay": "wss://relay.example.com:8084/ws/outpost",
    "database_wal": "/home/rick/.config/sentry/data/sentry_ledger.db",
    "client_ui": null
  },
  "what": {
    "summary": "Acoustic spike breach detected: 91.4 dB SPL (+26.3 dB over baseline)",
    "rms_db": 91.4,
    "baseline_db": 65.1,
    "delta_db": 26.3,
    "frames_captured": 5,
    "shutter_latency_ns": 1420000,
    "shutter_latency_ps": 1420000000,
    "payload_bytes": 35840,
    "payload_sha256": "e4f89a2bc912384a...",
    "duration_ns": 1425000,
    "duration_ps": 1425000000
  },
  "how": {
    "protocol": "HAL_DSP -> V4L2_MMAP -> SQLITE_WAL -> WSS_TLS",
    "transport": "Local Hardware Loop",
    "cipher": "ChaCha20-Poly1305 / HMAC-SHA256",
    "compression": "zstd"
  },
  "minutiae": {
    "cpu_rss_mb": 18.2,
    "dsp_ema_alpha": 0.15,
    "wal_page_count": 4,
    "sqlite_commit_ns": 25000,
    "network_rtt_ms": 1.0
  },
  "prev_record_hash": "a43b45829f049aba6ad7cb6adce8b4de96c1cf6d370e7d5983104b8dfb036e54",
  "record_hash": "df9f58d37bc5a434da9c27f51b178b16b754cfb022866d20b16ee6ebddcfa2a5"
}
```

### Cryptographic Hash Chain Mechanics
$$\text{RecordHash}_N = \text{SHA256}(\text{Sequence}_N \parallel \text{TimestampNs}_N \parallel \text{Summary}_N \parallel \text{PrevRecordHash}_{N-1})$$

* **Genesis Block**: Sequence 1 uses `prev_record_hash = "0000000000000000000000000000000000000000000000000000000000000000"`.
* **Tamper Evident**: Modifying any field (e.g. dB level, timestamp) or deleting a line invalidates all subsequent hash links across the entire chain.
* **Verification Algorithm**: `TelemetryRecord::verify_chain(&records)` validates $100\%$ of hash links in $O(N)$ time.

---

## 🔒 Static Musl Zero-GLIBC Cross-Compilation

To guarantee zero dependency on host glibc versions across heterogeneous Linux distros:

### `.cargo/config.toml`
```toml
[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=+crt-static"]

[target.aarch64-unknown-linux-musl]
rustflags = ["-C", "target-feature=+crt-static"]
```

### Build Invocation
```bash
cargo build --release --target x86_64-unknown-linux-musl
```

### Binary Verification
```bash
$ file target/x86_64-unknown-linux-musl/release/sentry-edge
ELF 64-bit LSB pie executable, x86-64, statically linked, not stripped

$ ldd target/x86_64-unknown-linux-musl/release/sentry-edge
statically linked
```
