//! ITER-PAR-01: Economic Control Plane
//!
//! Iter controls learning economics without code changes in SCG.
//! All tunable economics (costs, thresholds, permits) set by Iter surface.
//!
//! # Modes
//!
//! - Mode 1 (Parameter Authority): Config-based thresholds
//! - Mode 2 (Permit Authority): Per-window permits with budget caps
//!
//! # Invariants
//!
//! - INV-ITER-04: All economics from config/permits, never hardcoded

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::contracts::validation::{validate_bounded_float, validate_hash, ContractError};

// ============================================================================
// Mode 1: Parameter Authority (simpler)
// ============================================================================

/// Economics configuration - controls learning costs and thresholds.
///
/// Delivered through scg-gateway mapping and/or MCP control endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EconomicsConfig {
    /// Energy cost per learning update
    pub learning_cost_per_update: f64,
    /// Minimum payment quality to commit update [0.0, 1.0]
    pub min_learning_quality: f64,
    /// Minimum cortex input quality for learning [0.0, 1.0]
    pub min_input_quality_for_learning: f64,
    /// Maximum total learning energy per window
    pub max_learning_energy_per_window: f64,
    /// Window size in ticks
    pub window_ticks: u64,
}

impl Default for EconomicsConfig {
    fn default() -> Self {
        Self {
            learning_cost_per_update: 0.5,
            min_learning_quality: 0.95,
            min_input_quality_for_learning: 0.5,
            max_learning_energy_per_window: 100.0,
            window_ticks: 1000,
        }
    }
}

impl EconomicsConfig {
    /// Validate configuration values.
    pub fn validate(&self) -> Result<(), ContractError> {
        validate_bounded_float(
            self.learning_cost_per_update,
            0.0,
            f64::MAX,
            "economics.learning_cost_per_update",
        )?;
        validate_bounded_float(
            self.min_learning_quality,
            0.0,
            1.0,
            "economics.min_learning_quality",
        )?;
        validate_bounded_float(
            self.min_input_quality_for_learning,
            0.0,
            1.0,
            "economics.min_input_quality_for_learning",
        )?;
        validate_bounded_float(
            self.max_learning_energy_per_window,
            0.0,
            f64::MAX,
            "economics.max_learning_energy_per_window",
        )?;
        Ok(())
    }

    /// Compute SHA-256 hash of config.
    pub fn compute_hash(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

// ============================================================================
// Mode 2: Permit Authority (stronger)
// ============================================================================

/// Learning permit - per-window authorization with budget caps.
///
/// SCG includes permit_hash in LearningAudit for replay completeness.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearningPermit {
    /// Permit identifier
    pub permit_id: String,
    /// Allowed capsule identifiers (empty = all allowed)
    pub allowed_capsules: Vec<String>,
    /// Maximum learning energy for this window
    pub max_learning_energy: f64,
    /// Tick when permit expires (exclusive)
    pub expiry_tick: u64,
    /// SHA-256 hash of permit for audit
    pub permit_hash: String,
    /// Whether permit has been revoked
    pub revoked: bool,
}

impl LearningPermit {
    /// Create a new permit (hash computed automatically).
    pub fn new(
        permit_id: String,
        allowed_capsules: Vec<String>,
        max_learning_energy: f64,
        expiry_tick: u64,
    ) -> Result<Self, ContractError> {
        validate_bounded_float(max_learning_energy, 0.0, f64::MAX, "permit.max_learning_energy")?;

        let mut permit = Self {
            permit_id,
            allowed_capsules,
            max_learning_energy,
            expiry_tick,
            permit_hash: String::new(),
            revoked: false,
        };
        permit.permit_hash = permit.compute_hash();
        Ok(permit)
    }

    /// Compute SHA-256 hash of permit (excluding permit_hash field).
    fn compute_hash(&self) -> String {
        // Serialize without the hash field to compute hash
        let payload = serde_json::json!({
            "permit_id": self.permit_id,
            "allowed_capsules": self.allowed_capsules,
            "max_learning_energy": self.max_learning_energy,
            "expiry_tick": self.expiry_tick,
            "revoked": self.revoked,
        });
        let json = serde_json::to_string(&payload).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Check if permit is valid at given tick.
    pub fn is_valid_at(&self, tick: u64) -> bool {
        !self.revoked && tick < self.expiry_tick
    }

    /// Check if capsule is allowed by permit.
    pub fn allows_capsule(&self, capsule_id: &str) -> bool {
        if self.allowed_capsules.is_empty() {
            true // Empty list = all allowed
        } else {
            self.allowed_capsules.iter().any(|c| c == capsule_id)
        }
    }

    /// Revoke permit.
    pub fn revoke(&mut self) {
        self.revoked = true;
        self.permit_hash = self.compute_hash();
    }
}

// ============================================================================
// Economics Controller
// ============================================================================

/// Economics controller - manages config and permits.
pub struct EconomicsController {
    config: EconomicsConfig,
    active_permit: Option<LearningPermit>,
    /// Energy spent in current window
    window_energy_spent: f64,
    /// Window start tick
    window_start_tick: u64,
}

impl EconomicsController {
    /// Create controller with config.
    pub fn new(config: EconomicsConfig) -> Result<Self, ContractError> {
        config.validate()?;
        Ok(Self {
            config,
            active_permit: None,
            window_energy_spent: 0.0,
            window_start_tick: 0,
        })
    }

    /// Get config reference.
    pub fn config(&self) -> &EconomicsConfig {
        &self.config
    }

    /// Update config.
    pub fn set_config(&mut self, config: EconomicsConfig) -> Result<(), ContractError> {
        config.validate()?;
        self.config = config;
        Ok(())
    }

    /// Issue a new permit.
    pub fn issue_permit(&mut self, permit: LearningPermit) -> Result<(), ContractError> {
        validate_hash(&permit.permit_hash, "permit.permit_hash")?;
        self.active_permit = Some(permit);
        Ok(())
    }

    /// Revoke active permit.
    pub fn revoke_permit(&mut self) {
        if let Some(ref mut permit) = self.active_permit {
            permit.revoke();
        }
    }

    /// Check if learning is allowed at tick for capsule.
    ///
    /// Returns (allowed, reason_code).
    pub fn check_learning_allowed(
        &self,
        tick: u64,
        capsule_id: &str,
        proposed_cost: f64,
    ) -> (bool, &'static str) {
        // Check window budget
        let window_tick = tick - self.window_start_tick;
        if window_tick >= self.config.window_ticks {
            // Window expired - would reset in real impl
            // For now, allow if within per-window budget
        }

        if self.window_energy_spent + proposed_cost > self.config.max_learning_energy_per_window {
            return (false, "WINDOW_BUDGET_EXCEEDED");
        }

        // Check permit if active
        if let Some(ref permit) = self.active_permit {
            if !permit.is_valid_at(tick) {
                return (false, "PERMIT_EXPIRED");
            }
            if !permit.allows_capsule(capsule_id) {
                return (false, "CAPSULE_NOT_PERMITTED");
            }
            if self.window_energy_spent + proposed_cost > permit.max_learning_energy {
                return (false, "PERMIT_BUDGET_EXCEEDED");
            }
        }

        (true, "ALLOWED")
    }

    /// Record energy spent on learning.
    pub fn record_learning_cost(&mut self, cost: f64, tick: u64) {
        // Reset window if needed
        if tick - self.window_start_tick >= self.config.window_ticks {
            self.window_start_tick = tick;
            self.window_energy_spent = 0.0;
        }
        self.window_energy_spent += cost;
    }

    /// Get active permit hash (if any).
    pub fn active_permit_hash(&self) -> Option<&str> {
        self.active_permit.as_ref().map(|p| p.permit_hash.as_str())
    }

    /// Check if learning is globally disabled.
    pub fn is_learning_disabled(&self) -> bool {
        // Learning disabled if max energy is 0
        self.config.max_learning_energy_per_window <= 0.0
    }
}

impl Default for EconomicsController {
    fn default() -> Self {
        Self::new(EconomicsConfig::default()).expect("default config is valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_validates_valid() {
        let config = EconomicsConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn config_rejects_nan() {
        let config = EconomicsConfig {
            learning_cost_per_update: f64::NAN,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_rejects_out_of_range() {
        let config = EconomicsConfig {
            min_learning_quality: 1.5,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn config_hash_is_deterministic() {
        let c1 = EconomicsConfig::default();
        let c2 = EconomicsConfig::default();
        assert_eq!(c1.compute_hash(), c2.compute_hash());
    }

    #[test]
    fn permit_creation() {
        let permit = LearningPermit::new(
            "permit-1".to_string(),
            vec!["cap1".to_string()],
            100.0,
            1000,
        )
        .unwrap();

        assert!(!permit.permit_hash.is_empty());
        assert!(permit.is_valid_at(500));
        assert!(!permit.is_valid_at(1000));
        assert!(permit.allows_capsule("cap1"));
        assert!(!permit.allows_capsule("cap2"));
    }

    #[test]
    fn permit_empty_allows_all() {
        let permit = LearningPermit::new("permit-1".to_string(), vec![], 100.0, 1000).unwrap();
        assert!(permit.allows_capsule("any_capsule"));
    }

    #[test]
    fn permit_revocation() {
        let mut permit =
            LearningPermit::new("permit-1".to_string(), vec![], 100.0, 1000).unwrap();
        let hash_before = permit.permit_hash.clone();

        permit.revoke();

        assert!(permit.revoked);
        assert_ne!(permit.permit_hash, hash_before);
        assert!(!permit.is_valid_at(500));
    }

    #[test]
    fn controller_window_budget() {
        let config = EconomicsConfig {
            max_learning_energy_per_window: 10.0,
            window_ticks: 100,
            ..Default::default()
        };
        let mut controller = EconomicsController::new(config).unwrap();

        // Should allow within budget
        let (allowed, _) = controller.check_learning_allowed(0, "cap1", 5.0);
        assert!(allowed);

        controller.record_learning_cost(5.0, 0);

        // Should allow more within budget
        let (allowed, _) = controller.check_learning_allowed(1, "cap1", 4.0);
        assert!(allowed);

        // Should deny over budget
        let (allowed, reason) = controller.check_learning_allowed(2, "cap1", 6.0);
        assert!(!allowed);
        assert_eq!(reason, "WINDOW_BUDGET_EXCEEDED");
    }

    #[test]
    fn controller_learning_disabled() {
        let config = EconomicsConfig {
            max_learning_energy_per_window: 0.0,
            ..Default::default()
        };
        let controller = EconomicsController::new(config).unwrap();
        assert!(controller.is_learning_disabled());
    }
}
