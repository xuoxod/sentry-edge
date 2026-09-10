//! Two-Way Sovereign Audio Intercom Bridge

use sentry_core::SentryResult;

#[derive(Debug, Clone)]
pub struct SentryIntercomBridge {
    pub is_active: bool,
    pub active_operator: Option<String>,
}

impl SentryIntercomBridge {
    pub fn new() -> Self {
        Self {
            is_active: false,
            active_operator: None,
        }
    }

    /// Open incoming audio walkie-talkie stream from remote operator.
    pub fn open_operator_intercom(&mut self, operator: &str) -> SentryResult<()> {
        self.is_active = true;
        self.active_operator = Some(operator.to_string());
        Ok(())
    }

    /// Close audio stream.
    pub fn close_intercom(&mut self) -> SentryResult<()> {
        self.is_active = false;
        self.active_operator = None;
        Ok(())
    }
}

impl Default for SentryIntercomBridge {
    fn default() -> Self {
        Self::new()
    }
}
