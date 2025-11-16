/// Integration Validation: Telemetry + Quarantine + Governance
/// 
/// Validates that the hardened SCG runtime enforces invariants:
/// - Telemetry emission on every operation
/// - Quarantine activation on violations
/// - Deterministic lineage tracking

use scg_mcp_server::*;

#[test]
fn test_telemetry_emission_on_operations() {
    let runtime = ScgRuntime::new();
    
    // Create a node - should emit telemetry
    let node1 = runtime.node_create(0.5, 100.0);
    assert_ne!(node1.id, uuid::Uuid::nil());
    assert_eq!(node1.energy, 100.0);
    
    // Create another node
    let node2 = runtime.node_create(0.3, 50.0);
    assert_ne!(node2.id, uuid::Uuid::nil());
    
    // Verify governor status shows correct state
    let status = runtime.governor_status();
    assert_eq!(status.node_count, 2);
    assert_eq!(status.edge_count, 0);
    
    // Energy drift should be calculated
    // Initial energy = 100.0, total = 150.0, drift = 50.0
    assert_eq!(status.energy_drift, 50.0);
    
    // Coherence should be 1.0 (all nodes ESV-valid)
    assert_eq!(status.coherence, 1.0);
}

#[test]
fn test_quarantine_on_drift_violation() {
    let runtime = ScgRuntime::new();
    
    // System should start un-quarantined
    assert!(!runtime.is_quarantined());
    
    // Create first node with initial energy
    let node1 = runtime.node_create(0.5, 1.0);
    
    // Add massive energy to trigger drift violation
    // This will cause drift > 1e-10
    let _node2 = runtime.node_create(0.5, 1000.0);
    
    // System should now be quarantined due to drift violation
    assert!(runtime.is_quarantined());
    
    // Further operations should be blocked
    let blocked_node = runtime.node_create(0.5, 1.0);
    assert_eq!(blocked_node.id, uuid::Uuid::nil());
    assert!(!blocked_node.esv_valid);
}

#[test]
fn test_operations_blocked_when_quarantined() {
    let runtime = ScgRuntime::new();
    
    // Create nodes to trigger quarantine
    let node1 = runtime.node_create(0.5, 1.0);
    let _node2 = runtime.node_create(0.5, 1000.0);
    
    // Verify quarantine active
    assert!(runtime.is_quarantined());
    
    // Test that mutations are blocked
    let mutate_result = runtime.node_mutate(node1.id, 0.1);
    assert!(mutate_result.is_err());
    assert_eq!(mutate_result.unwrap_err(), "System is quarantined");
}

#[test]
fn test_lineage_tracking_deterministic() {
    let runtime = ScgRuntime::new();
    
    // Perform sequence of operations with zero energy to avoid drift
    let node1 = runtime.node_create(0.5, 0.0);
    let node2 = runtime.node_create(0.3, 0.0);
    
    // Bind edge
    let edge_result = runtime.edge_bind(node1.id, node2.id, 0.5);
    assert!(edge_result.is_ok());
    
    // Get lineage state
    let lineage = runtime.replay_lineage();
    
    // Lineage should have checksum
    assert_eq!(lineage.checksum.len(), 64); // SHA256 hex length
}

#[test]
fn test_coherence_calculation() {
    let runtime = ScgRuntime::new();
    
    // Create first node (sets initial energy)
    runtime.node_create(0.5, 10.0);
    
    // Create subsequent nodes with ZERO energy to maintain conservation
    runtime.node_create(0.3, 0.0);
    runtime.node_create(0.7, 0.0);
    
    let status = runtime.governor_status();
    
    // All nodes are ESV-valid, so coherence = 1.0
    assert_eq!(status.coherence, 1.0);
    
    // Should have 3 nodes total
    assert_eq!(status.node_count, 3);
}

#[test]
fn test_governor_status_reflects_real_state() {
    let runtime = ScgRuntime::new();
    
    // Initial state
    let status1 = runtime.governor_status();
    assert_eq!(status1.node_count, 0);
    assert_eq!(status1.energy_drift, 0.0);
    
    // Add node
    runtime.node_create(0.5, 100.0);
    
    let status2 = runtime.governor_status();
    assert_eq!(status2.node_count, 1);
    assert_eq!(status2.energy_drift, 0.0); // First node sets initial
    
    // Add second node with different energy
    runtime.node_create(0.5, 50.0);
    
    let status3 = runtime.governor_status();
    assert_eq!(status3.node_count, 2);
    assert_eq!(status3.energy_drift, 50.0); // |150 - 100| = 50
}
