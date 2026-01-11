//! ITER-PAR-01: Policy Primitives Module
//!
//! Deterministic policy evaluation for SCG governance.
//! Rules evaluated in declaration order; first Terminal wins.
//!
//! # Invariants
//!
//! - INV-ITER-02: Deterministic order of evaluation
//! - INV-ITER-03: Explicit reason codes for every decision
//! - Fail closed when ambiguous

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::contracts::{
    ContractError, EnergyEnvelope, LearningEnvelope, LearningStatus, PolicyDecision,
    PolicyEnvelope, ReasoningEnvelope,
};

/// Policy rule identifier.
pub type RuleId = &'static str;

/// Rule evaluation outcome.
#[derive(Debug, Clone, PartialEq)]
pub enum RuleOutcome {
    /// Continue to next rule
    Continue,
    /// Terminal decision with reason code
    Terminal(PolicyDecision, &'static str),
}

/// Policy rule trait - evaluates a single condition.
pub trait PolicyRule: Send + Sync {
    /// Rule identifier for audit
    fn id(&self) -> RuleId;

    /// Evaluate rule against state. Returns Continue or Terminal.
    fn evaluate(
        &self,
        reasoning: &ReasoningEnvelope,
        learning: &LearningEnvelope,
        energy: &EnergyEnvelope,
    ) -> RuleOutcome;
}

// ============================================================================
// Gate Implementations
// ============================================================================

/// Reasoning quality gate - denies if quality below threshold.
pub struct ReasoningQualityGate {
    /// Minimum quality threshold [0.0, 1.0]
    pub threshold: f64,
    /// Decision when quality is below threshold
    pub on_fail: PolicyDecision,
}

impl PolicyRule for ReasoningQualityGate {
    fn id(&self) -> RuleId {
        "REASONING_QUALITY_GATE"
    }

    fn evaluate(
        &self,
        reasoning: &ReasoningEnvelope,
        _learning: &LearningEnvelope,
        _energy: &EnergyEnvelope,
    ) -> RuleOutcome {
        if reasoning.quality < self.threshold {
            RuleOutcome::Terminal(self.on_fail, "REASONING_QUALITY_BELOW_THRESHOLD")
        } else {
            RuleOutcome::Continue
        }
    }
}

/// Learning permission gate - freezes on integrity violation or scarcity streak.
pub struct LearningPermissionGate {
    /// Maximum scarcity streak before freeze
    pub max_scarcity_streak: u64,
    /// Freeze duration in ticks (for audit only; enforcement is external)
    pub freeze_ticks: u64,
}

impl PolicyRule for LearningPermissionGate {
    fn id(&self) -> RuleId {
        "LEARNING_PERMISSION_GATE"
    }

    fn evaluate(
        &self,
        _reasoning: &ReasoningEnvelope,
        learning: &LearningEnvelope,
        _energy: &EnergyEnvelope,
    ) -> RuleOutcome {
        // Integrity violation => immediate freeze
        if learning.status == LearningStatus::RejectedIntegrity {
            return RuleOutcome::Terminal(PolicyDecision::FreezeLearning, "INTEGRITY_VIOLATION");
        }

        // Scarcity streak exceeded => freeze
        if learning.scarcity_streak >= self.max_scarcity_streak {
            return RuleOutcome::Terminal(PolicyDecision::FreezeLearning, "SCARCITY_STREAK_EXCEEDED");
        }

        RuleOutcome::Continue
    }
}

/// Energy integrity gate - denies if integrity below threshold.
pub struct EnergyIntegrityGate {
    /// Minimum integrity ratio [0.0, 1.0]
    pub min_integrity: f64,
}

impl PolicyRule for EnergyIntegrityGate {
    fn id(&self) -> RuleId {
        "ENERGY_INTEGRITY_GATE"
    }

    fn evaluate(
        &self,
        _reasoning: &ReasoningEnvelope,
        _learning: &LearningEnvelope,
        energy: &EnergyEnvelope,
    ) -> RuleOutcome {
        if energy.integrity < self.min_integrity {
            RuleOutcome::Terminal(PolicyDecision::Deny, "ENERGY_INTEGRITY_BELOW_THRESHOLD")
        } else {
            RuleOutcome::Continue
        }
    }
}

/// Input quality gate for learning - rejects learning if input quality below threshold.
pub struct InputQualityGate {
    /// Minimum input quality for learning
    pub min_input_quality: f64,
}

impl PolicyRule for InputQualityGate {
    fn id(&self) -> RuleId {
        "INPUT_QUALITY_GATE"
    }

    fn evaluate(
        &self,
        reasoning: &ReasoningEnvelope,
        learning: &LearningEnvelope,
        _energy: &EnergyEnvelope,
    ) -> RuleOutcome {
        // Only applies if learning was attempted and rejected for input quality
        if learning.status == LearningStatus::RejectedInputQuality
            && reasoning.quality < self.min_input_quality
        {
            RuleOutcome::Terminal(PolicyDecision::DegradedMode, "INPUT_QUALITY_INSUFFICIENT")
        } else {
            RuleOutcome::Continue
        }
    }
}

/// Learning quality gate - requires minimum payment quality for commits.
pub struct LearningQualityGate {
    /// Minimum learning quality (payment ratio)
    pub min_learning_quality: f64,
}

impl PolicyRule for LearningQualityGate {
    fn id(&self) -> RuleId {
        "LEARNING_QUALITY_GATE"
    }

    fn evaluate(
        &self,
        _reasoning: &ReasoningEnvelope,
        learning: &LearningEnvelope,
        _energy: &EnergyEnvelope,
    ) -> RuleOutcome {
        // Only check if an update was proposed (cost > 0)
        if learning.update_cost > 0.0 && learning.update_quality < self.min_learning_quality {
            RuleOutcome::Terminal(PolicyDecision::Deny, "LEARNING_QUALITY_BELOW_THRESHOLD")
        } else {
            RuleOutcome::Continue
        }
    }
}

// ============================================================================
// Policy Evaluator
// ============================================================================

/// Policy configuration for the evaluator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Minimum reasoning quality threshold
    pub min_reasoning_quality: f64,
    /// Minimum energy integrity
    pub min_energy_integrity: f64,
    /// Maximum scarcity streak before learning freeze
    pub max_scarcity_streak: u64,
    /// Freeze duration in ticks
    pub freeze_ticks: u64,
    /// Minimum input quality for learning
    pub min_input_quality_for_learning: f64,
    /// Minimum learning quality (payment ratio)
    pub min_learning_quality: f64,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            min_reasoning_quality: 0.5,
            min_energy_integrity: 0.8,
            max_scarcity_streak: 10,
            freeze_ticks: 100,
            min_input_quality_for_learning: 0.5,
            min_learning_quality: 0.95,
        }
    }
}

impl PolicyConfig {
    /// Compute SHA-256 hash of config for audit trail.
    pub fn compute_hash(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Policy evaluation result.
#[derive(Debug, Clone)]
pub struct PolicyResult {
    /// Final decision
    pub decision: PolicyDecision,
    /// All reason codes (in evaluation order)
    pub reason_codes: Vec<String>,
    /// Rules that were evaluated
    pub evaluated_rules: Vec<RuleId>,
    /// Terminal rule (if any)
    pub terminal_rule: Option<RuleId>,
}

/// Deterministic policy evaluator.
///
/// Rules are evaluated in order. First Terminal outcome wins.
/// If all rules return Continue, default is Allow.
pub struct PolicyEvaluator {
    config: PolicyConfig,
}

impl PolicyEvaluator {
    /// Create evaluator with config.
    pub fn new(config: PolicyConfig) -> Self {
        Self { config }
    }

    /// Evaluate policy against state.
    ///
    /// # Rule Order (deterministic)
    /// 1. Energy integrity gate
    /// 2. Reasoning quality gate
    /// 3. Input quality gate
    /// 4. Learning permission gate
    /// 5. Learning quality gate
    ///
    /// First Terminal wins; otherwise Allow.
    pub fn evaluate(
        &self,
        reasoning: &ReasoningEnvelope,
        learning: &LearningEnvelope,
        energy: &EnergyEnvelope,
    ) -> PolicyResult {
        let mut reason_codes = Vec::new();
        let mut evaluated_rules = Vec::new();

        // Build rules from config
        let rules: Vec<Box<dyn PolicyRule>> = vec![
            Box::new(EnergyIntegrityGate {
                min_integrity: self.config.min_energy_integrity,
            }),
            Box::new(ReasoningQualityGate {
                threshold: self.config.min_reasoning_quality,
                on_fail: PolicyDecision::DegradedMode,
            }),
            Box::new(InputQualityGate {
                min_input_quality: self.config.min_input_quality_for_learning,
            }),
            Box::new(LearningPermissionGate {
                max_scarcity_streak: self.config.max_scarcity_streak,
                freeze_ticks: self.config.freeze_ticks,
            }),
            Box::new(LearningQualityGate {
                min_learning_quality: self.config.min_learning_quality,
            }),
        ];

        // Evaluate rules in order
        for rule in &rules {
            evaluated_rules.push(rule.id());
            let outcome = rule.evaluate(reasoning, learning, energy);

            match outcome {
                RuleOutcome::Continue => continue,
                RuleOutcome::Terminal(decision, reason) => {
                    reason_codes.push(reason.to_string());
                    return PolicyResult {
                        decision,
                        reason_codes,
                        evaluated_rules,
                        terminal_rule: Some(rule.id()),
                    };
                }
            }
        }

        // All rules passed => Allow
        PolicyResult {
            decision: PolicyDecision::Allow,
            reason_codes,
            evaluated_rules,
            terminal_rule: None,
        }
    }

    /// Build policy envelope from evaluation result.
    pub fn build_envelope(&self, result: &PolicyResult) -> Result<PolicyEnvelope, ContractError> {
        PolicyEnvelope::new(
            self.config.compute_hash(),
            result.decision,
            result.reason_codes.clone(),
        )
    }

    /// Get config reference.
    pub fn config(&self) -> &PolicyConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::LearningStatus;

    fn make_reasoning(quality: f64) -> ReasoningEnvelope {
        ReasoningEnvelope::new(quality, 0.5, 0.1, 0.5).unwrap()
    }

    fn make_learning(status: LearningStatus, scarcity_streak: u64) -> LearningEnvelope {
        LearningEnvelope::new(
            "cap1".to_string(),
            1,
            "a".repeat(64),
            0.5,
            0.5,
            1.0,
            status,
            scarcity_streak,
        )
        .unwrap()
    }

    fn make_energy(integrity: f64) -> EnergyEnvelope {
        EnergyEnvelope::new(100.0, 10.0, integrity).unwrap()
    }

    #[test]
    fn evaluator_allows_when_all_pass() {
        let config = PolicyConfig::default();
        let evaluator = PolicyEvaluator::new(config);

        let reasoning = make_reasoning(0.9);
        let learning = make_learning(LearningStatus::Committed, 0);
        let energy = make_energy(0.95);

        let result = evaluator.evaluate(&reasoning, &learning, &energy);
        assert_eq!(result.decision, PolicyDecision::Allow);
        assert!(result.terminal_rule.is_none());
    }

    #[test]
    fn evaluator_denies_low_energy_integrity() {
        let config = PolicyConfig {
            min_energy_integrity: 0.8,
            ..Default::default()
        };
        let evaluator = PolicyEvaluator::new(config);

        let reasoning = make_reasoning(0.9);
        let learning = make_learning(LearningStatus::Committed, 0);
        let energy = make_energy(0.5); // Below threshold

        let result = evaluator.evaluate(&reasoning, &learning, &energy);
        assert_eq!(result.decision, PolicyDecision::Deny);
        assert_eq!(result.terminal_rule, Some("ENERGY_INTEGRITY_GATE"));
    }

    #[test]
    fn evaluator_degrades_on_low_reasoning_quality() {
        let config = PolicyConfig {
            min_reasoning_quality: 0.5,
            ..Default::default()
        };
        let evaluator = PolicyEvaluator::new(config);

        let reasoning = make_reasoning(0.3); // Below threshold
        let learning = make_learning(LearningStatus::Committed, 0);
        let energy = make_energy(0.95);

        let result = evaluator.evaluate(&reasoning, &learning, &energy);
        assert_eq!(result.decision, PolicyDecision::DegradedMode);
        assert_eq!(result.terminal_rule, Some("REASONING_QUALITY_GATE"));
    }

    #[test]
    fn evaluator_freezes_on_integrity_violation() {
        let config = PolicyConfig::default();
        let evaluator = PolicyEvaluator::new(config);

        let reasoning = make_reasoning(0.9);
        let learning = make_learning(LearningStatus::RejectedIntegrity, 0);
        let energy = make_energy(0.95);

        let result = evaluator.evaluate(&reasoning, &learning, &energy);
        assert_eq!(result.decision, PolicyDecision::FreezeLearning);
        assert!(result.reason_codes.contains(&"INTEGRITY_VIOLATION".to_string()));
    }

    #[test]
    fn evaluator_freezes_on_scarcity_streak() {
        let config = PolicyConfig {
            max_scarcity_streak: 5,
            ..Default::default()
        };
        let evaluator = PolicyEvaluator::new(config);

        let reasoning = make_reasoning(0.9);
        let learning = make_learning(LearningStatus::RejectedScarcity, 5);
        let energy = make_energy(0.95);

        let result = evaluator.evaluate(&reasoning, &learning, &energy);
        assert_eq!(result.decision, PolicyDecision::FreezeLearning);
        assert!(result
            .reason_codes
            .contains(&"SCARCITY_STREAK_EXCEEDED".to_string()));
    }

    #[test]
    fn policy_config_hash_is_deterministic() {
        let config1 = PolicyConfig::default();
        let config2 = PolicyConfig::default();

        assert_eq!(config1.compute_hash(), config2.compute_hash());
    }

    #[test]
    fn policy_config_hash_changes_with_values() {
        let config1 = PolicyConfig::default();
        let config2 = PolicyConfig {
            min_reasoning_quality: 0.7,
            ..Default::default()
        };

        assert_ne!(config1.compute_hash(), config2.compute_hash());
    }
}
