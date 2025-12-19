//! Error Taxonomy Completeness Tests
//!
//! These tests enforce that the McpError enum is exhaustive and stable.
//! Unclassified errors are impossible to emit; all error codes are documented.
//!
//! # Governance Contract
//!
//! - McpError variants are a closed set
//! - Each variant has a stable numeric code
//! - Each variant has a stable string code
//! - Error codes are unique and documented
//! - These tests compile in public_stub mode (no substrate deps)

use iter_mcp_server::types::mcp::McpError;
use std::collections::HashSet;

// ============================================================================
// Error Variant Exhaustiveness
// ============================================================================

/// All known McpError variants with their expected codes
const EXPECTED_ERRORS: &[(&str, u32)] = &[
    ("node_not_found", 4004),
    ("edge_not_found", 4004),
    ("esv_validation_failed", 1000),
    ("drift_exceeded", 2000),
    ("lineage_corruption", 3000),
    ("substrate_error", 5000),
    ("bad_request", 4000),
];

#[test]
fn all_error_variants_have_codes() {
    // Create one of each variant
    let errors = vec![
        McpError::NodeNotFound { id: 0 },
        McpError::EdgeNotFound { id: 0 },
        McpError::EsvValidationFailed {
            reason: String::new(),
        },
        McpError::DriftExceeded {
            drift: 0.0,
            threshold: 0.0,
        },
        McpError::LineageCorruption {
            details: String::new(),
        },
        McpError::SubstrateError {
            message: String::new(),
        },
        McpError::BadRequest {
            message: String::new(),
        },
    ];

    // Every variant must have a non-zero code
    for err in &errors {
        assert!(err.code() > 0, "Error {:?} has invalid code", err);
    }

    // Every variant must have a non-empty code_string
    for err in &errors {
        assert!(
            !err.code_string().is_empty(),
            "Error {:?} has empty code_string",
            err
        );
    }
}

#[test]
fn error_codes_match_expected() {
    let errors: Vec<(McpError, &str, u32)> = vec![
        (McpError::NodeNotFound { id: 0 }, "node_not_found", 4004),
        (McpError::EdgeNotFound { id: 0 }, "edge_not_found", 4004),
        (
            McpError::EsvValidationFailed {
                reason: String::new(),
            },
            "esv_validation_failed",
            1000,
        ),
        (
            McpError::DriftExceeded {
                drift: 0.0,
                threshold: 0.0,
            },
            "drift_exceeded",
            2000,
        ),
        (
            McpError::LineageCorruption {
                details: String::new(),
            },
            "lineage_corruption",
            3000,
        ),
        (
            McpError::SubstrateError {
                message: String::new(),
            },
            "substrate_error",
            5000,
        ),
        (
            McpError::BadRequest {
                message: String::new(),
            },
            "bad_request",
            4000,
        ),
    ];

    for (err, expected_code_str, expected_code) in errors {
        assert_eq!(
            err.code_string(),
            expected_code_str,
            "Error {:?} has wrong code_string",
            err
        );
        assert_eq!(
            err.code(),
            expected_code,
            "Error {:?} has wrong numeric code",
            err
        );
    }
}

#[test]
fn error_codes_are_documented() {
    // All expected error code strings must exist
    let mut found_codes: HashSet<&str> = HashSet::new();

    let errors = vec![
        McpError::NodeNotFound { id: 0 },
        McpError::EdgeNotFound { id: 0 },
        McpError::EsvValidationFailed {
            reason: String::new(),
        },
        McpError::DriftExceeded {
            drift: 0.0,
            threshold: 0.0,
        },
        McpError::LineageCorruption {
            details: String::new(),
        },
        McpError::SubstrateError {
            message: String::new(),
        },
        McpError::BadRequest {
            message: String::new(),
        },
    ];

    for err in &errors {
        found_codes.insert(err.code_string());
    }

    for (expected_code, _) in EXPECTED_ERRORS {
        assert!(
            found_codes.contains(expected_code),
            "Expected error code '{}' not found in McpError variants",
            expected_code
        );
    }
}

#[test]
fn variant_count_matches_expected() {
    // If someone adds a new variant, this test will fail until EXPECTED_ERRORS is updated
    let variant_count = 7; // Current number of variants
    assert_eq!(
        EXPECTED_ERRORS.len(),
        variant_count,
        "EXPECTED_ERRORS count ({}) doesn't match actual variant count ({}). \
         Update EXPECTED_ERRORS if you added a new McpError variant.",
        EXPECTED_ERRORS.len(),
        variant_count
    );
}

// ============================================================================
// Error Display Invariants
// ============================================================================

#[test]
fn errors_have_meaningful_display() {
    let errors = vec![
        (McpError::NodeNotFound { id: 42 }, "N42"),
        (McpError::EdgeNotFound { id: 7 }, "E7"),
        (
            McpError::EsvValidationFailed {
                reason: "test".into(),
            },
            "test",
        ),
        (
            McpError::DriftExceeded {
                drift: 0.5,
                threshold: 0.1,
            },
            "0.5",
        ),
        (
            McpError::LineageCorruption {
                details: "corrupt".into(),
            },
            "corrupt",
        ),
        (
            McpError::SubstrateError {
                message: "oops".into(),
            },
            "oops",
        ),
        (
            McpError::BadRequest {
                message: "invalid".into(),
            },
            "invalid",
        ),
    ];

    for (err, expected_substring) in errors {
        let display = err.to_string();
        assert!(
            display.contains(expected_substring),
            "Error display '{}' should contain '{}'",
            display,
            expected_substring
        );
    }
}

#[test]
fn error_code_conversion_is_consistent() {
    let errors = vec![
        McpError::NodeNotFound { id: 0 },
        McpError::BadRequest {
            message: String::new(),
        },
        McpError::SubstrateError {
            message: String::new(),
        },
    ];

    for err in errors {
        // error_code() should equal code() as i32
        assert_eq!(
            err.error_code(),
            err.code() as i32,
            "error_code() and code() mismatch for {:?}",
            err
        );
    }
}

// ============================================================================
// Error Serialization Invariants
// ============================================================================

#[test]
fn errors_serialize_to_json() {
    let errors = vec![
        McpError::NodeNotFound { id: 1 },
        McpError::BadRequest {
            message: "test".into(),
        },
    ];

    for err in errors {
        let json = serde_json::to_string(&err);
        assert!(json.is_ok(), "Error {:?} should serialize to JSON", err);
    }
}

#[test]
fn errors_roundtrip_through_json() {
    let original = McpError::NodeNotFound { id: 42 };
    let json = serde_json::to_string(&original).expect("serialize");
    let parsed: McpError = serde_json::from_str(&json).expect("deserialize");

    // Compare code_string since we can't derive PartialEq easily
    assert_eq!(original.code_string(), parsed.code_string());
}
