// Governance: MCP boundary sanitization
// Lineage: MCP_BOUNDARY_V2.0
//
// ╔══════════════════════════════════════════════════════════════════════════╗
// ║  IMMUTABLE REGISTRY — DO NOT MODIFY WITHOUT OWNER APPROVAL               ║
// ║  Version: 2.0.0 | Sealed: 2025-12-03                                     ║
// ║  Any modification requires CODEOWNERS approval and an audit trail entry. ║
// ╚══════════════════════════════════════════════════════════════════════════╝
//
//! Forbidden Pattern Registry for MCP boundary sanitization.
//!
//! This registry is security-sensitive. Visitor-facing documentation intentionally
//! avoids enumerating internal details.

/// Forbidden field patterns that must NEVER appear in MCP responses.
pub const FORBIDDEN_PATTERNS: &[&str] = &[
    // Internal patterns
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
    
    // Internal patterns
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
    
    // Internal patterns
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
    
    // Internal patterns
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
    
    // Internal patterns
    "governor_quorum_state",
    "consensus_votes",
    "drift_correction_vector",
    "node_energy_deltas",
    "quorum_members",
    
    // Internal patterns
    "reflective_state",
    "coherence_raw",
    "meta_cognitive_variance",
    "self_referential_state",
    
    // Internal patterns
    "internal_state",
    "debug_info",
    "substrate_state",
    "raw_state",
    "stack_trace",
    "backtrace",
    "panic_message",
    "internal_error",
];

/// Sensitive field patterns that should only appear in sanitized form.
pub const SENSITIVE_PATTERNS: &[&str] = &[
    "energy",
    "coherence",
    "drift",
    "checksum",
    "hash",
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

/// Normalize text for matching.
pub fn normalize_for_matching(text: &str) -> String {
    // Normalize for matching
    text.chars()
        .filter(|c| !matches!(
            *c,
            '\u{200B}' |
            '\u{200C}' |
            '\u{200D}' |
            '\u{FEFF}' |
            '\u{00AD}'
        ))
        .map(normalize_char)
        .collect::<String>()
        .to_lowercase()
}

/// Normalize Unicode characters for matching.
fn normalize_char(c: char) -> char {
    match c {
        'а' => 'a',
        'е' => 'e',
        'о' => 'o',
        'р' => 'p',
        'с' => 'c',
        'у' => 'y',
        'х' => 'x',
        'α' => 'a',
        'ο' => 'o',
        'ı' => 'i',
        'ℓ' => 'l',
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
        let obfuscated = "dag\u{200B}_topology";
        let normalized = normalize_for_matching(obfuscated);
        assert_eq!(normalized, "dag_topology");
        assert!(is_forbidden(obfuscated));
    }

    #[test]
    fn test_unicode_normalization_cyrillic() {
        let obfuscated = "dаg_topology";
        let normalized = normalize_for_matching(obfuscated);
        assert_eq!(normalized, "dag_topology");
        assert!(is_forbidden(obfuscated));
    }

    #[test]
    fn test_unicode_normalization_mixed() {
        let obfuscated = "e\u{200C}sv\u{200D}_rаw";
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
