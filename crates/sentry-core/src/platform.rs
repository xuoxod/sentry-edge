//! Sentry Cross-Platform Environment, OS Detection & Path Resolution
//! Enterprise-grade platform agnosticism across Linux, macOS, Windows, FreeBSD, and embedded musl targets.

use crate::error::{SentryError, SentryResult};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// System and runtime platform detection information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub family: String,
    pub is_musl: bool,
    pub is_container: bool,
}

impl PlatformInfo {
    /// Detect the current operating system, CPU architecture, and runtime environment.
    pub fn detect() -> Self {
        let os = std::env::consts::OS.to_string();
        let arch = std::env::consts::ARCH.to_string();
        let family = std::env::consts::FAMILY.to_string();

        let is_musl = cfg!(target_env = "musl")
            || std::fs::read_to_string("/proc/version")
                .map(|v| v.to_lowercase().contains("musl") || v.to_lowercase().contains("alpine"))
                .unwrap_or(false);

        let is_container = Path::new("/.dockerenv").exists()
            || Path::new("/run/systemd/container").exists()
            || std::env::var("CONTAINER").is_ok()
            || std::env::var("DOCKER").is_ok()
            || std::env::var("KUBERNETES_SERVICE_HOST").is_ok();

        Self {
            os,
            arch,
            family,
            is_musl,
            is_container,
        }
    }

    pub fn display_summary(&self) -> String {
        format!(
            "{}-{} (family: {}, musl: {}, container: {})",
            self.os, self.arch, self.family, self.is_musl, self.is_container
        )
    }
}

/// Cross-platform standard directory and path resolution provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformPaths;

impl PlatformPaths {
    /// Resolve standard configuration root directory across Linux, macOS, and Windows.
    pub fn config_dir() -> PathBuf {
        if let Ok(custom) = std::env::var("SENTRY_CONFIG_DIR") {
            return PathBuf::from(custom);
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return PathBuf::from(appdata).join("sentry");
            }
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                return PathBuf::from(userprofile).join(".config").join("sentry");
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("sentry");
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
                return PathBuf::from(xdg).join("sentry");
            }
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(".config").join("sentry");
            }
        }

        // Universal fallback: temp directory or local directory
        std::env::temp_dir().join("sentry")
    }

    /// Resolve default configuration file path (`sentry.toml`).
    pub fn default_config_path() -> PathBuf {
        Self::config_dir().join("sentry.toml")
    }

    /// Resolve standard data directory for SQLite ledger and persistent databases.
    pub fn data_dir() -> PathBuf {
        if let Ok(custom) = std::env::var("SENTRY_DATA_DIR") {
            return PathBuf::from(custom);
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
                return PathBuf::from(local_appdata).join("sentry").join("data");
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("sentry")
                    .join("data");
            }
        }

        Self::config_dir().join("data")
    }

    /// Resolve standard log directory for JSONL audit logs.
    pub fn log_dir() -> PathBuf {
        if let Ok(custom) = std::env::var("SENTRY_LOG_DIR") {
            return PathBuf::from(custom);
        }

        Self::config_dir().join("logs")
    }

    /// Resolve standard SQLite ledger DB path.
    pub fn default_ledger_path() -> PathBuf {
        Self::data_dir().join("sentry_ledger.db")
    }

    /// Resolve standard JSONL telemetry audit log path.
    pub fn default_audit_log_path() -> PathBuf {
        Self::log_dir().join("sentry_audit.jsonl")
    }

    /// Validate path against directory traversal attacks and forbidden Windows device names.
    pub fn validate_safe_path(path: &Path) -> SentryResult<PathBuf> {
        let path_str = path.to_string_lossy();

        // Check for path traversal components
        for component in path.components() {
            if component == Component::ParentDir {
                return Err(SentryError::Config(
                    "Security violation: path traversal detected - path contains forbidden parent directory reference ('..')".to_string(),
                ));
            }
        }

        // Check for null bytes
        if path_str.contains('\0') {
            return Err(SentryError::Config(
                "Security violation: Path contains forbidden null byte".to_string(),
            ));
        }

        // Check for forbidden Windows reserved device names (CON, PRN, AUX, NUL, COM1..9, LPT1..9)
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            let stem = file_name.split('.').next().unwrap_or("").to_uppercase();
            let forbidden = [
                "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6",
                "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7",
                "LPT8", "LPT9",
            ];
            if forbidden.contains(&stem.as_str()) {
                return Err(SentryError::Config(format!(
                    "Security violation: Path uses reserved OS device name '{}'",
                    stem
                )));
            }
        }

        Ok(path.to_path_buf())
    }
}
