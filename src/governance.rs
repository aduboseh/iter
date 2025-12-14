//! Governance metadata for Iter Server.
//!
//! This module intentionally exposes only a small, stable status surface.
//! Detailed governance doctrine and internal operating procedures are maintained privately.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Governance version (public surface only; detailed doctrine is private).
pub const GOVERNANCE_VERSION: &str = "1.0";

/// Governance health status response (sanitized).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceHealthStatus {
    pub governance_version: String,
    pub drift_within_bounds: bool,
    pub current_drift: f64,
    pub esv_enabled: bool,
    pub last_audit: Option<String>,
    pub server_version: String,
}

impl Default for GovernanceHealthStatus {
    fn default() -> Self {
        Self {
            governance_version: format!("v{}", GOVERNANCE_VERSION),
            drift_within_bounds: true,
            current_drift: 0.0,
            esv_enabled: true,
            last_audit: None,
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Minimal governance validator (behavioral signals only).
pub struct GovernanceValidator {
    current_drift: f64,
    last_audit: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for GovernanceValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl GovernanceValidator {
    pub fn new() -> Self {
        Self {
            current_drift: 0.0,
            last_audit: None,
        }
    }

    pub fn set_drift(&mut self, drift: f64) {
        self.current_drift = drift;
    }

    pub fn record_audit(&mut self) {
        self.last_audit = Some(chrono::Utc::now());
    }

    pub fn drift_within_bounds(&self, epsilon: f64) -> bool {
        self.current_drift.abs() <= epsilon
    }

    pub fn health_status(&self, epsilon: f64) -> GovernanceHealthStatus {
        GovernanceHealthStatus {
            governance_version: format!("v{}", GOVERNANCE_VERSION),
            drift_within_bounds: self.drift_within_bounds(epsilon),
            current_drift: self.current_drift,
            esv_enabled: true,
            last_audit: self.last_audit.map(|dt| dt.to_rfc3339()),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_constants() {
        assert!(!GOVERNANCE_VERSION.is_empty());
    }

    #[test]
    fn test_drift_bounds() {
        let mut validator = GovernanceValidator::new();
        validator.set_drift(0.0);
        assert!(validator.drift_within_bounds(1e-10));

        validator.set_drift(1e-9);
        assert!(!validator.drift_within_bounds(1e-10));
    }
}
