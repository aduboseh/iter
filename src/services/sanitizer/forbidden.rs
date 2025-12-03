// SCG Governance: Deterministic | ESV-Compliant | Drift ≤1e-10
// Lineage: MCP_BOUNDARY_V2.0
// Generated under SCG_Governance_v1.0
//
// ╔══════════════════════════════════════════════════════════════════════════╗
// ║  IMMUTABLE REGISTRY — DO NOT MODIFY WITHOUT FOUNDER-LEVEL OVERRIDE       ║
// ║  Version: 2.0.0 | Sealed: 2025-12-03 | Authority: SCG Governor           ║
// ║  Any modification requires CODEOWNERS approval and audit trail entry.    ║
// ╚══════════════════════════════════════════════════════════════════════════╝
//
//! Forbidden Pattern Registry for MCP Boundary Sanitization
//!
//! This module defines the canonical list of patterns that must NEVER appear
//! in MCP responses. It represents SCG substrate internals that would enable:
//! - Adversarial model reconstruction
//! - Ethical constraint bypass
//! - Lineage forgery
//!
//! Zero-Touch Zones (from Hardening Directive v2.0):
//! - ❌ No substrate introspection endpoints
//! - ❌ No DAG topology logging
//! - ❌ No ESV/energy matrix exposure
//! - ❌ No debug interfaces that leak substrate internals

/// Forbidden field patterns that must NEVER appear in MCP responses
///
/// These represent SCG substrate internals that would enable adversarial
/// model reconstruction, ethical constraint bypass, or lineage forgery.
pub const FORBIDDEN_PATTERNS: &[&str] = &[
    // DAG topology internals
    "dag_topology",
    "node_ids",
    "edge_weights",
    "adjacency_matrix",
    "adjacency_list",
    "adjacency",
    "topology",
    "dag_structure",
    "node_connections",
    "edge_list",
    "internal_edges",
    "node_internal_state",
    "belief_vector",
    "energy_allocation",
    "propagation_path",
    
    // ESV (Ethical State Vector) internals
    "esv_raw",
    "esv_matrix",
    "esv_checksum_internal",
    "ethical_gradient",
    "harm_potential_raw",
    "truth_confidence_internal",
    "moral_vector",
    "raw_tau",
    "raw_harm",
    "raw_chi",
    "ethical_potential_raw",
    
    // Energy system internals
    "energy_matrix",
    "node_energies",
    "energy_distribution",
    "energy_redistribution_log",
    "governor_correction_delta",
    "thermodynamic_state",
    "entropy_internal",
    "energy_delta",
    "node_energy_allocation",
    "hamiltonian",
    "internal_energy",
    
    // Lineage ledger internals
    "lineage_hash_chain",
    "lineage_chain",
    "full_lineage",
    "lineage_entries",
    "ledger_raw",
    "cascade_hash_internal",
    "parent_hash",
    "shard_id",
    "hash_chain",
    "state_snapshots",
    
    // Elastic Governor internals
    "governor_quorum_state",
    "consensus_votes",
    "drift_correction_vector",
    "node_energy_deltas",
    "quorum_members",
    
    // Meta-cognitive layer internals
    "reflective_state",
    "coherence_raw",
    "meta_cognitive_variance",
    "self_referential_state",
    
    // Implementation details / Debug
    "internal_state",
    "debug_info",
    "substrate_state",
    "raw_state",
    "stack_trace",
    "backtrace",
    "panic_message",
    "internal_error",
];

/// Sensitive field patterns that should only appear in sanitized form
///
/// These are acceptable if properly aggregated/summarized but forbidden in raw form.
pub const SENSITIVE_PATTERNS: &[&str] = &[
    "energy",     // OK: "energy_summary", forbidden: "energy_matrix"
    "coherence",  // OK: "coherence_index", forbidden: "coherence_raw"
    "drift",      // OK: "drift_status", forbidden: "drift_correction_vector"
    "checksum",   // OK: "validation_status", forbidden: "esv_checksum_internal"
    "hash",       // OK: "integrity_verified", forbidden: "lineage_hash_chain"
];

/// Check if a field name matches any forbidden pattern
pub fn is_forbidden(field_name: &str) -> bool {
    let normalized = normalize_for_matching(field_name);
    FORBIDDEN_PATTERNS
        .iter()
        .any(|pattern| normalized.contains(&normalize_for_matching(pattern)))
}

/// Check if a text contains any forbidden patterns
pub fn contains_forbidden(text: &str) -> Vec<String> {
    let normalized = normalize_for_matching(text);
    FORBIDDEN_PATTERNS
        .iter()
        .filter(|pattern| normalized.contains(&normalize_for_matching(pattern)))
        .map(|s| s.to_string())
        .collect()
}

/// Unicode normalization for pattern matching
///
/// Prevents bypass via zero-width characters, lookalike Unicode, etc.
pub fn normalize_for_matching(text: &str) -> String {
    // Remove zero-width characters and normalize
    text.chars()
        .filter(|c| !matches!(
            *c,
            '\u{200B}' | // Zero-width space
            '\u{200C}' | // Zero-width non-joiner
            '\u{200D}' | // Zero-width joiner
            '\u{FEFF}' | // BOM / Zero-width no-break space
            '\u{00AD}'   // Soft hyphen
        ))
        .map(|c| normalize_char(c))
        .collect::<String>()
        .to_lowercase()
}

/// Normalize lookalike Unicode characters to ASCII equivalents
fn normalize_char(c: char) -> char {
    match c {
        // Cyrillic lookalikes
        'а' => 'a', // Cyrillic а
        'е' => 'e', // Cyrillic е
        'о' => 'o', // Cyrillic о
        'р' => 'p', // Cyrillic р
        'с' => 'c', // Cyrillic с
        'у' => 'y', // Cyrillic у
        'х' => 'x', // Cyrillic х
        // Greek lookalikes
        'α' => 'a', // Greek alpha
        'ο' => 'o', // Greek omicron
        // Other lookalikes
        'ı' => 'i', // Turkish dotless i
        'ℓ' => 'l', // Script small l
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_forbidden_basic() {
        assert!(is_forbidden("dag_topology"));
        assert!(is_forbidden("esv_raw"));
        assert!(is_forbidden("energy_matrix"));
        assert!(is_forbidden("lineage_hash_chain"));
        assert!(!is_forbidden("belief"));
        assert!(!is_forbidden("status"));
    }

    #[test]
    fn test_unicode_normalization_zero_width() {
        let obfuscated = "dag\u{200B}_topology"; // Zero-width space
        let normalized = normalize_for_matching(obfuscated);
        assert_eq!(normalized, "dag_topology");
        assert!(is_forbidden(obfuscated));
    }

    #[test]
    fn test_unicode_normalization_cyrillic() {
        let obfuscated = "dаg_topology"; // Cyrillic 'а'
        let normalized = normalize_for_matching(obfuscated);
        assert_eq!(normalized, "dag_topology");
        assert!(is_forbidden(obfuscated));
    }

    #[test]
    fn test_unicode_normalization_mixed() {
        let obfuscated = "e\u{200C}sv\u{200D}_rаw"; // Mixed zero-width + Cyrillic
        let normalized = normalize_for_matching(obfuscated);
        assert_eq!(normalized, "esv_raw");
        assert!(is_forbidden(obfuscated));
    }

    #[test]
    fn test_contains_forbidden() {
        let text = r#"{"dag_topology": [], "esv_raw": [1,2,3], "status": "ok"}"#;
        let violations = contains_forbidden(text);
        assert!(violations.contains(&"dag_topology".to_string()));
        assert!(violations.contains(&"esv_raw".to_string()));
        assert!(!violations.contains(&"status".to_string()));
    }

    #[test]
    fn test_forbidden_patterns_no_duplicates() {
        use std::collections::HashSet;
        let unique: HashSet<_> = FORBIDDEN_PATTERNS.iter().collect();
        assert_eq!(
            unique.len(),
            FORBIDDEN_PATTERNS.len(),
            "Forbidden patterns contain duplicates"
        );
    }

    #[test]
    fn test_case_insensitive_matching() {
        assert!(is_forbidden("DAG_TOPOLOGY"));
        assert!(is_forbidden("ESV_Raw"));
        assert!(is_forbidden("Energy_MATRIX"));
    }
}
