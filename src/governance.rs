// SCG Governance: Deterministic | ESV-Compliant | Drift ≤1e-10
// Lineage: 9D7623E581D982D8F9BC816738EF0880E9631E6FD5789C36AF80698DF2BAA527
// Generated under SCG_Governance_v1.0

//! SCG Governance Module for MCP Server
//!
//! Provides governance validation and health endpoints for the MCP server.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Governance version
pub const GOVERNANCE_VERSION: &str = "1.0";

/// Expected SHA256 checksum of SCG_Governance_v1.0.md
pub const GOVERNANCE_SHA256: &str = "9D7623E581D982D8F9BC816738EF0880E9631E6FD5789C36AF80698DF2BAA527";

/// Maximum allowed deterministic drift (ε)
pub const DRIFT_EPSILON: f64 = 1e-10;

/// Governance health status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceHealthStatus {
    pub governance_version: String,
    pub checksum_valid: bool,
    pub expected_checksum: String,
    pub actual_checksum: Option<String>,
    pub repo_sync: bool,
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
            checksum_valid: false,
            expected_checksum: GOVERNANCE_SHA256.to_string(),
            actual_checksum: None,
            repo_sync: false,
            drift_within_bounds: true,
            current_drift: 0.0,
            esv_enabled: true,
            last_audit: None,
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Governance validator for MCP server
pub struct GovernanceValidator {
    governance_path: String,
    current_drift: f64,
    last_audit: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for GovernanceValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl GovernanceValidator {
    /// Create a new governance validator
    pub fn new() -> Self {
        Self {
            governance_path: "governance/SCG_Governance_v1.0.md".to_string(),
            current_drift: 0.0,
            last_audit: None,
        }
    }

    /// Create validator with custom path
    pub fn with_path(path: impl Into<String>) -> Self {
        Self {
            governance_path: path.into(),
            current_drift: 0.0,
            last_audit: None,
        }
    }

    /// Update drift measurement
    pub fn set_drift(&mut self, drift: f64) {
        self.current_drift = drift;
    }

    /// Record audit timestamp
    pub fn record_audit(&mut self) {
        self.last_audit = Some(chrono::Utc::now());
    }

    /// Validate governance checksum
    pub fn validate_checksum(&self) -> (bool, Option<String>) {
        let path = Path::new(&self.governance_path);
        
        match std::fs::read(path) {
            Ok(content) => {
                let mut hasher = Sha256::new();
                hasher.update(&content);
                let hash = hex::encode(hasher.finalize()).to_uppercase();
                let valid = hash == GOVERNANCE_SHA256;
                (valid, Some(hash))
            }
            Err(_) => (false, None),
        }
    }

    /// Check if drift is within bounds
    pub fn drift_within_bounds(&self) -> bool {
        self.current_drift.abs() <= DRIFT_EPSILON
    }

    /// Get full health status
    pub fn health_status(&self) -> GovernanceHealthStatus {
        let (checksum_valid, actual_checksum) = self.validate_checksum();
        
        GovernanceHealthStatus {
            governance_version: format!("v{}", GOVERNANCE_VERSION),
            checksum_valid,
            expected_checksum: GOVERNANCE_SHA256.to_string(),
            actual_checksum,
            repo_sync: checksum_valid, // Simplified: if checksum matches, repos are in sync
            drift_within_bounds: self.drift_within_bounds(),
            current_drift: self.current_drift,
            esv_enabled: true,
            last_audit: self.last_audit.map(|dt| dt.to_rfc3339()),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Compute SHA256 hash of data
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize()).to_uppercase()
}

/// Embedded governance constants for binary metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedGovernance {
    pub version: &'static str,
    pub sha256: &'static str,
    pub drift_epsilon: f64,
}

pub const EMBEDDED: EmbeddedGovernance = EmbeddedGovernance {
    version: GOVERNANCE_VERSION,
    sha256: GOVERNANCE_SHA256,
    drift_epsilon: DRIFT_EPSILON,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_constants() {
        assert_eq!(GOVERNANCE_VERSION, "1.0");
        assert_eq!(GOVERNANCE_SHA256.len(), 64);
        assert!(DRIFT_EPSILON < 1e-9);
    }

    #[test]
    fn test_drift_bounds() {
        let mut validator = GovernanceValidator::new();
        
        validator.set_drift(0.0);
        assert!(validator.drift_within_bounds());
        
        validator.set_drift(1e-11);
        assert!(validator.drift_within_bounds());
        
        validator.set_drift(1e-9);
        assert!(!validator.drift_within_bounds());
    }

    #[test]
    fn test_sha256() {
        let hash = compute_sha256(b"test");
        assert_eq!(hash.len(), 64);
    }
}
