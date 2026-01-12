//! Adversarial Tests for Governance Plane
//!
//! These tests verify fail-closed behavior under adversarial conditions:
//! - Unknown enum values
//! - NaN/Inf injection attempts
//! - Reason code presence rules
//! - Permit expiry edge cases
//!
//! INVARIANT: No "accepted" decision with missing causality, economics, or reason codes.

use iter_mcp_server::contracts::{
    ContractError, EnergyEnvelope, LearningEnvelope, LearningStatus, PolicyDecision,
    PolicyEnvelope, ReasoningEnvelope,
};
use iter_mcp_server::economics::{EconomicsConfig, EconomicsController, LearningPermit};
use iter_mcp_server::policy::{PolicyConfig, PolicyEvaluator};

// ============================================================================
// Unknown Enum Values - Must Fail Closed
// ============================================================================

#[test]
fn unknown_learning_status_fails_closed() {
    let result = LearningStatus::from_str_closed("UNKNOWN_STATUS");
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        ContractError::UnknownEnum { field, value } => {
            assert_eq!(field, "learning.status");
            assert_eq!(value, "UNKNOWN_STATUS");
        }
        _ => panic!("Expected UnknownEnum error, got: {:?}", err),
    }
}

#[test]
fn unknown_policy_decision_fails_closed() {
    let result = PolicyDecision::from_str_closed("MAYBE");
    assert!(result.is_err());

    let err = result.unwrap_err();
    match err {
        ContractError::UnknownEnum { field, value } => {
            assert_eq!(field, "policy.decision");
            assert_eq!(value, "MAYBE");
        }
        _ => panic!("Expected UnknownEnum error, got: {:?}", err),
    }
}

#[test]
fn all_learning_status_variants_parse() {
    let variants = [
        ("COMMITTED", LearningStatus::Committed),
        ("NO_PROPOSAL_NO_DELTA", LearningStatus::NoProposalNoDelta),
        (
            "REJECTED_INPUT_QUALITY",
            LearningStatus::RejectedInputQuality,
        ),
        ("REJECTED_SCARCITY", LearningStatus::RejectedScarcity),
        ("REJECTED_INTEGRITY", LearningStatus::RejectedIntegrity),
    ];

    for (s, expected) in variants {
        let result = LearningStatus::from_str_closed(s).unwrap();
        assert_eq!(result, expected);
    }
}

#[test]
fn all_policy_decision_variants_parse() {
    let variants = [
        ("ALLOW", PolicyDecision::Allow),
        ("DENY", PolicyDecision::Deny),
        ("FREEZE_LEARNING", PolicyDecision::FreezeLearning),
        ("DEGRADED_MODE", PolicyDecision::DegradedMode),
        ("REQUIRE_REVIEW", PolicyDecision::RequireReview),
    ];

    for (s, expected) in variants {
        let result = PolicyDecision::from_str_closed(s).unwrap();
        assert_eq!(result, expected);
    }
}

// ============================================================================
// NaN/Inf Injection - Must Fail Hard
// ============================================================================

#[test]
fn energy_envelope_rejects_nan_nodes() {
    let result = EnergyEnvelope::new(f64::NAN, 10.0, 0.95);
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::NaN { field } => assert_eq!(field, "energy.nodes"),
        e => panic!("Expected NaN error, got: {:?}", e),
    }
}

#[test]
fn energy_envelope_rejects_nan_reservoir() {
    let result = EnergyEnvelope::new(100.0, f64::NAN, 0.95);
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::NaN { field } => assert_eq!(field, "energy.reservoir"),
        e => panic!("Expected NaN error, got: {:?}", e),
    }
}

#[test]
fn energy_envelope_rejects_nan_integrity() {
    let result = EnergyEnvelope::new(100.0, 10.0, f64::NAN);
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::NaN { field } => assert_eq!(field, "energy.integrity"),
        e => panic!("Expected NaN error, got: {:?}", e),
    }
}

#[test]
fn energy_envelope_rejects_positive_infinity() {
    let result = EnergyEnvelope::new(f64::INFINITY, 10.0, 0.95);
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::Infinite { field } => assert_eq!(field, "energy.nodes"),
        e => panic!("Expected Infinite error, got: {:?}", e),
    }
}

#[test]
fn energy_envelope_rejects_negative_infinity() {
    let result = EnergyEnvelope::new(100.0, f64::NEG_INFINITY, 0.95);
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::Infinite { field } => assert_eq!(field, "energy.reservoir"),
        e => panic!("Expected Infinite error, got: {:?}", e),
    }
}

#[test]
fn reasoning_envelope_rejects_nan_quality() {
    let result = ReasoningEnvelope::new(f64::NAN, 0.5, 0.1, 0.8);
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::NaN { field } => assert_eq!(field, "reasoning.quality"),
        e => panic!("Expected NaN error, got: {:?}", e),
    }
}

#[test]
fn reasoning_envelope_rejects_inf_value_signal() {
    let result = ReasoningEnvelope::new(0.9, f64::INFINITY, 0.1, 0.8);
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::Infinite { field } => assert_eq!(field, "reasoning.value_signal"),
        e => panic!("Expected Infinite error, got: {:?}", e),
    }
}

#[test]
fn learning_envelope_rejects_nan_update_cost() {
    let result = LearningEnvelope::new(
        "cap".to_string(),
        1,
        "a".repeat(64),
        f64::NAN,
        0.5,
        1.0,
        LearningStatus::Committed,
        0,
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::NaN { field } => assert_eq!(field, "learning.update_cost"),
        e => panic!("Expected NaN error, got: {:?}", e),
    }
}

#[test]
fn economics_config_rejects_nan() {
    let config = EconomicsConfig {
        learning_cost_per_update: f64::NAN,
        ..Default::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn economics_config_rejects_out_of_range_quality() {
    let config = EconomicsConfig {
        min_learning_quality: 1.5, // Out of [0.0, 1.0]
        ..Default::default()
    };
    assert!(config.validate().is_err());
}

// ============================================================================
// Reason Code Presence Rules
// ============================================================================

fn make_healthy_state() -> (ReasoningEnvelope, LearningEnvelope, EnergyEnvelope) {
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        1.0,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 10.0, 0.95).unwrap();
    (reasoning, learning, energy)
}

#[test]
fn deny_decision_produces_reason_code() {
    let config = PolicyConfig {
        min_energy_integrity: 0.99, // Will fail with 0.95
        ..Default::default()
    };
    let evaluator = PolicyEvaluator::new(config);

    let (reasoning, learning, energy) = make_healthy_state();
    let result = evaluator.evaluate(&reasoning, &learning, &energy);

    assert_eq!(result.decision, PolicyDecision::Deny);
    assert!(
        !result.reason_codes.is_empty(),
        "Deny must have reason codes"
    );
    assert!(result
        .reason_codes
        .contains(&"ENERGY_INTEGRITY_BELOW_THRESHOLD".to_string()));
}

#[test]
fn freeze_decision_produces_reason_code() {
    let config = PolicyConfig {
        max_scarcity_streak: 5,
        ..Default::default()
    };
    let evaluator = PolicyEvaluator::new(config);

    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "cap".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.1,
        0.2,
        LearningStatus::RejectedScarcity,
        5, // At threshold
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 10.0, 0.95).unwrap();

    let result = evaluator.evaluate(&reasoning, &learning, &energy);

    assert_eq!(result.decision, PolicyDecision::FreezeLearning);
    assert!(
        !result.reason_codes.is_empty(),
        "FreezeLearning must have reason codes"
    );
}

#[test]
fn degraded_mode_produces_reason_code() {
    let config = PolicyConfig {
        min_reasoning_quality: 0.5,
        ..Default::default()
    };
    let evaluator = PolicyEvaluator::new(config);

    let reasoning = ReasoningEnvelope::new(0.3, 0.5, 0.1, 0.8).unwrap(); // Below threshold
    let learning = LearningEnvelope::new(
        "cap".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        1.0,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 10.0, 0.95).unwrap();

    let result = evaluator.evaluate(&reasoning, &learning, &energy);

    assert_eq!(result.decision, PolicyDecision::DegradedMode);
    assert!(
        !result.reason_codes.is_empty(),
        "DegradedMode must have reason codes"
    );
}

#[test]
fn allow_decision_may_have_empty_reason_codes() {
    let config = PolicyConfig::default();
    let evaluator = PolicyEvaluator::new(config);

    let (reasoning, learning, energy) = make_healthy_state();
    let result = evaluator.evaluate(&reasoning, &learning, &energy);

    assert_eq!(result.decision, PolicyDecision::Allow);
    // ALLOW may have empty reason codes - this is valid
}

// ============================================================================
// Permit Expiry Edge Cases
// ============================================================================

#[test]
fn permit_valid_before_expiry() {
    let permit = LearningPermit::new("p1".to_string(), vec![], 100.0, 1000).unwrap();
    assert!(permit.is_valid_at(999)); // Last valid tick
}

#[test]
fn permit_invalid_at_expiry() {
    let permit = LearningPermit::new("p1".to_string(), vec![], 100.0, 1000).unwrap();
    assert!(!permit.is_valid_at(1000)); // Expiry tick is exclusive
}

#[test]
fn permit_invalid_after_expiry() {
    let permit = LearningPermit::new("p1".to_string(), vec![], 100.0, 1000).unwrap();
    assert!(!permit.is_valid_at(1001));
}

#[test]
fn permit_valid_at_tick_zero() {
    let permit = LearningPermit::new("p1".to_string(), vec![], 100.0, 1000).unwrap();
    assert!(permit.is_valid_at(0));
}

#[test]
fn permit_expiry_at_zero_is_never_valid() {
    let permit = LearningPermit::new("p1".to_string(), vec![], 100.0, 0).unwrap();
    assert!(!permit.is_valid_at(0));
}

#[test]
fn permit_revocation_invalidates_permit() {
    let mut permit = LearningPermit::new("p1".to_string(), vec![], 100.0, 1000).unwrap();
    assert!(permit.is_valid_at(500));

    permit.revoke();

    assert!(!permit.is_valid_at(500)); // Now invalid even before expiry
    assert!(permit.revoked);
}

#[test]
fn permit_revocation_changes_hash() {
    let mut permit = LearningPermit::new("p1".to_string(), vec![], 100.0, 1000).unwrap();
    let hash_before = permit.permit_hash.clone();

    permit.revoke();

    assert_ne!(permit.permit_hash, hash_before);
}

// ============================================================================
// Economics Controller Budget Enforcement
// ============================================================================

#[test]
fn controller_denies_over_window_budget() {
    let config = EconomicsConfig {
        max_learning_energy_per_window: 10.0,
        window_ticks: 100,
        ..Default::default()
    };
    let controller = EconomicsController::new(config).unwrap();

    // First check should pass
    let (allowed, _) = controller.check_learning_allowed(0, "cap", 5.0);
    assert!(allowed);

    // Check that would exceed budget should fail
    let (allowed, reason) = controller.check_learning_allowed(0, "cap", 15.0);
    assert!(!allowed);
    assert_eq!(reason, "WINDOW_BUDGET_EXCEEDED");
}

#[test]
fn controller_tracks_cumulative_spending() {
    let config = EconomicsConfig {
        max_learning_energy_per_window: 10.0,
        window_ticks: 100,
        ..Default::default()
    };
    let mut controller = EconomicsController::new(config).unwrap();

    // Record 8 units spent
    controller.record_learning_cost(8.0, 0);

    // 3 more would exceed budget
    let (allowed, reason) = controller.check_learning_allowed(1, "cap", 3.0);
    assert!(!allowed);
    assert_eq!(reason, "WINDOW_BUDGET_EXCEEDED");

    // But 2 more should still fit
    let (allowed, _) = controller.check_learning_allowed(1, "cap", 2.0);
    assert!(allowed);
}

// ============================================================================
// Hash Validation
// ============================================================================

#[test]
fn learning_envelope_rejects_short_hash() {
    let result = LearningEnvelope::new(
        "cap".to_string(),
        1,
        "abc".to_string(), // Too short
        0.5,
        0.5,
        1.0,
        LearningStatus::Committed,
        0,
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::InvalidHash { field, length } => {
            assert_eq!(field, "learning.version_hash");
            assert_eq!(length, 3);
        }
        e => panic!("Expected InvalidHash error, got: {:?}", e),
    }
}

#[test]
fn learning_envelope_rejects_non_hex_hash() {
    let result = LearningEnvelope::new(
        "cap".to_string(),
        1,
        "g".repeat(64), // 'g' is not hex
        0.5,
        0.5,
        1.0,
        LearningStatus::Committed,
        0,
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::InvalidHexChars { field } => {
            assert_eq!(field, "learning.version_hash");
        }
        e => panic!("Expected InvalidHexChars error, got: {:?}", e),
    }
}

#[test]
fn policy_envelope_rejects_invalid_policy_hash() {
    let result = PolicyEnvelope::new(
        "not_a_valid_hash".to_string(),
        PolicyDecision::Allow,
        vec![],
    );
    assert!(result.is_err());
}

// ============================================================================
// Range Validation
// ============================================================================

#[test]
fn energy_integrity_rejects_out_of_range() {
    let result = EnergyEnvelope::new(100.0, 10.0, 1.5); // > 1.0
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::OutOfBounds { field, .. } => {
            assert_eq!(field, "energy.integrity");
        }
        e => panic!("Expected OutOfBounds error, got: {:?}", e),
    }
}

#[test]
fn reasoning_quality_rejects_negative() {
    let result = ReasoningEnvelope::new(-0.1, 0.5, 0.1, 0.8);
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::OutOfBounds { field, .. } => {
            assert_eq!(field, "reasoning.quality");
        }
        e => panic!("Expected OutOfBounds error, got: {:?}", e),
    }
}

#[test]
fn learning_update_quality_rejects_over_one() {
    let result = LearningEnvelope::new(
        "cap".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        1.1, // > 1.0
        LearningStatus::Committed,
        0,
    );
    assert!(result.is_err());
    match result.unwrap_err() {
        ContractError::OutOfBounds { field, .. } => {
            assert_eq!(field, "learning.update_quality");
        }
        e => panic!("Expected OutOfBounds error, got: {:?}", e),
    }
}
