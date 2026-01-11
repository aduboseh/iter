//! ITER-PAR-01 Governance Demo
//!
//! Demonstrates:
//! 1. Learning frozen by scarcity streak
//! 2. Policy halt on low reasoning quality
//! 3. DecisionPacket verified by checksum - byte-identical on repeated runs
//!
//! Run: cargo run --example governance_demo

use iter_mcp_server::audit::DecisionPacket;
use iter_mcp_server::contracts::{
    EnergyEnvelope, LearningEnvelope, LearningStatus, PolicyEnvelope, ReasoningEnvelope,
    SystemState,
};
use iter_mcp_server::economics::EconomicsConfig;
use iter_mcp_server::policy::{PolicyConfig, PolicyEvaluator};

fn main() {
    println!("=== ITER-PAR-01 Governance Demo ===\n");

    demo_learning_frozen_by_scarcity();
    demo_policy_halt_on_low_reasoning();
    demo_deterministic_checksums();
    
    println!("\n=== Demo Complete ===");
}

/// Demo 1: Learning frozen by scarcity streak
fn demo_learning_frozen_by_scarcity() {
    println!("--- Demo 1: Learning Frozen by Scarcity ---\n");
    
    let policy_config = PolicyConfig {
        max_scarcity_streak: 5,
        ..Default::default()
    };
    let evaluator = PolicyEvaluator::new(policy_config.clone());
    
    // Normal state
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();
    
    // Test with increasing scarcity streaks
    for streak in [0, 3, 5, 10] {
        let learning = LearningEnvelope::new(
            "capsule_001".to_string(),
            1,
            "a".repeat(64),
            0.5,
            0.5,
            0.98,
            LearningStatus::RejectedScarcity,
            streak,
        )
        .unwrap();
        
        let result = evaluator.evaluate(&reasoning, &learning, &energy);
        println!(
            "  Scarcity streak {}: {:?}{}",
            streak,
            result.decision,
            if !result.reason_codes.is_empty() {
                format!(" - {}", result.reason_codes.join(", "))
            } else {
                String::new()
            }
        );
    }
    println!();
}

/// Demo 2: Policy halt on low reasoning quality
fn demo_policy_halt_on_low_reasoning() {
    println!("--- Demo 2: Policy Halt on Low Reasoning Quality ---\n");
    
    let policy_config = PolicyConfig {
        min_reasoning_quality: 0.6,
        ..Default::default()
    };
    let evaluator = PolicyEvaluator::new(policy_config.clone());
    
    let learning = LearningEnvelope::new(
        "capsule_001".to_string(),
        1,
        "a".repeat(64),
        0.5,
        0.5,
        0.98,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();
    
    // Test with varying reasoning quality
    for quality in [0.9, 0.7, 0.5, 0.3] {
        let reasoning = ReasoningEnvelope::new(quality, 0.5, 0.1, 0.8).unwrap();
        let result = evaluator.evaluate(&reasoning, &learning, &energy);
        println!(
            "  Reasoning quality {:.1}: {:?}{}",
            quality,
            result.decision,
            if !result.reason_codes.is_empty() {
                format!(" - {}", result.reason_codes.join(", "))
            } else {
                String::new()
            }
        );
    }
    println!();
}

/// Demo 3: Deterministic checksum verification
fn demo_deterministic_checksums() {
    println!("--- Demo 3: Deterministic Checksum Verification ---\n");
    
    let policy_config = PolicyConfig::default();
    let economics_config = EconomicsConfig::default();
    let evaluator = PolicyEvaluator::new(policy_config.clone());
    
    let reasoning = ReasoningEnvelope::new(0.9, 0.5, 0.1, 0.8).unwrap();
    let learning = LearningEnvelope::new(
        "capsule_demo".to_string(),
        1,
        "b".repeat(64),
        0.5,
        0.5,
        0.98,
        LearningStatus::Committed,
        0,
    )
    .unwrap();
    let energy = EnergyEnvelope::new(100.0, 50.0, 0.95).unwrap();
    
    // Build packet twice
    let result = evaluator.evaluate(&reasoning, &learning, &energy);
    let policy_envelope = PolicyEnvelope::new(
        policy_config.compute_hash(),
        result.decision,
        result.reason_codes.clone(),
    )
    .unwrap();
    
    let state = SystemState::new(
        1000,
        energy.clone(),
        reasoning.clone(),
        learning.clone(),
        policy_envelope,
    );
    
    let packet1 = DecisionPacket::new(
        "iter-demo".to_string(),
        "scg-demo".to_string(),
        &state,
        None,
        economics_config.compute_hash(),
        result.evaluated_rules.iter().map(|s| s.to_string()).collect(),
    )
    .unwrap();
    
    // Build second packet with identical inputs
    let result2 = evaluator.evaluate(&reasoning, &learning, &energy);
    let policy_envelope2 = PolicyEnvelope::new(
        policy_config.compute_hash(),
        result2.decision,
        result2.reason_codes.clone(),
    )
    .unwrap();
    
    let state2 = SystemState::new(
        1000,
        energy,
        reasoning,
        learning,
        policy_envelope2,
    );
    
    let packet2 = DecisionPacket::new(
        "iter-demo".to_string(),
        "scg-demo".to_string(),
        &state2,
        None,
        economics_config.compute_hash(),
        result2.evaluated_rules.iter().map(|s| s.to_string()).collect(),
    )
    .unwrap();
    
    println!("  First packet checksum:  {}", &packet1.checksum[..32]);
    println!("  Second packet checksum: {}", &packet2.checksum[..32]);
    println!(
        "  Checksums match: {}\n",
        if packet1.checksum == packet2.checksum { "YES ✓" } else { "NO ✗" }
    );
    
    // Verify checksum
    match packet1.verify_checksum() {
        Ok(()) => println!("  Checksum verification: PASSED ✓"),
        Err(e) => println!("  Checksum verification: FAILED - {}", e),
    }
    
    // Show full packet structure
    println!("\n  DecisionPacket contents:");
    println!("    tick: {}", packet1.tick);
    println!("    decision: {:?}", packet1.decision());
    println!("    policy_hash: {}...", &packet1.policy.policy_hash[..16]);
    println!("    economics_hash: {}...", &packet1.economics_hash[..16]);
    println!("    evaluated_rules: {:?}", packet1.evaluated_rules);
    println!("    reason_codes: {:?}", packet1.reason_codes());
}
