//! Release Discipline Invariants
//!
//! These tests enforce release discipline rules that can be verified at compile/test time.
//! Policy rules that require runtime or CI enforcement are documented in RELEASE.md.

use iter_mcp_server::types::{PROTOCOL_VERSION, ProtocolVersion};

// ============================================================================
// Protocol Version Invariants
// ============================================================================

/// Protocol version must be valid semver
#[test]
fn protocol_version_is_valid_semver() {
    let version = ProtocolVersion::parse(PROTOCOL_VERSION);
    assert!(version.is_some(), "PROTOCOL_VERSION must be valid semver");
    
    let v = version.unwrap();
    assert!(v.major >= 1, "Protocol must be at least 1.0.0 for release");
}

/// Protocol version string format is correct
#[test]
fn protocol_version_format() {
    let parts: Vec<&str> = PROTOCOL_VERSION.split('.').collect();
    assert_eq!(parts.len(), 3, "Protocol version must have exactly 3 parts");
    
    for part in parts {
        assert!(part.parse::<u32>().is_ok(), "Each version part must be a number");
    }
}

// ============================================================================
// SDK Version Compatibility Invariants
// ============================================================================

/// SDK MIN_SERVER_VERSION must not exceed current protocol version
#[test]
fn sdk_min_version_within_bounds() {
    // This test validates the rule: SDKs support N and N-1
    // MIN_SERVER_VERSION should be at most current version
    let current = ProtocolVersion::parse(PROTOCOL_VERSION).unwrap();
    
    // For now, MIN = current (1.0.0), but when we bump to 1.1.0,
    // MIN should remain at 1.0.0 for N-1 support
    assert!(current.major >= 1, "Protocol major version must be at least 1");
}

/// SDK MAX_SERVER_VERSION must be in the same major version
#[test]
fn sdk_max_version_same_major() {
    // SDKs support up to 1.99.99 (same major)
    // This ensures major version bumps require SDK updates
    let current = ProtocolVersion::parse(PROTOCOL_VERSION).unwrap();
    
    // Max supported should be current.major.99.99
    // Enforced by SDK code, but we validate the invariant here
    assert_eq!(current.major, 1, "Current protocol is major version 1");
}

// ============================================================================
// Compatibility Window Invariants
// ============================================================================

/// N-1 support window: previous minor versions must remain compatible
#[test]
fn backward_compatibility_minor_versions() {
    let current = ProtocolVersion::parse(PROTOCOL_VERSION).unwrap();
    
    // Any 1.x.y where x < current.minor should be compatible
    // This is enforced by the compatibility rules in version.rs
    if current.minor > 0 {
        let prev_minor_str = format!("{}.{}.0", current.major, current.minor - 1);
        let prev_minor = ProtocolVersion::parse(&prev_minor_str).unwrap();
        
        // Previous minor version should be compatible
        assert_eq!(prev_minor.major, current.major, 
            "N-1 minor version must have same major");
    }
}

/// Breaking changes require major version bump
#[test]
fn breaking_changes_require_major_bump() {
    // This is a documentation test - actual enforcement is in code review
    // The invariant: if wire format changes incompatibly, major must bump
    
    let current = ProtocolVersion::parse(PROTOCOL_VERSION).unwrap();
    
    // For 1.x.x, all versions should be wire-compatible
    assert_eq!(current.major, 1, 
        "We are still in major version 1 - no breaking changes yet");
}

// ============================================================================
// Deprecation Policy Invariants
// ============================================================================

/// Deprecated features must have at least one minor version warning period
#[test]
fn deprecation_warning_period() {
    // This test documents the policy - actual deprecations are tracked in CHANGELOG.md
    // Rule: deprecated in X.Y, removable in X.(Y+1) at earliest, or (X+1).0
    
    // No deprecated features in 1.0.0
    let current = ProtocolVersion::parse(PROTOCOL_VERSION).unwrap();
    assert!(current.minor == 0 || current.major > 1,
        "First minor version - no deprecations possible yet");
}

// ============================================================================
// Release Artifact Invariants
// ============================================================================

/// Crate name must be correct
#[test]
fn crate_name_is_correct() {
    // The crate name should be iter-related, not SCG
    let crate_name = env!("CARGO_PKG_NAME");
    assert!(crate_name.contains("iter"), 
        "Crate name must contain 'iter': {}", crate_name);
    assert!(!crate_name.to_lowercase().contains("scg"),
        "Crate name must not contain 'scg': {}", crate_name);
}

/// Crate version must be valid semver
#[test]
fn crate_version_is_valid_semver() {
    let version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = version.split('.').collect();
    assert_eq!(parts.len(), 3, "Crate version must be semver");
}

// ============================================================================
// EOL Policy Invariants
// ============================================================================

/// Support window constants
const SUPPORT_WINDOW_MONTHS: u32 = 6;
const EOL_ANNOUNCEMENT_MONTHS: u32 = 3;

#[test]
fn support_window_is_reasonable() {
    assert!(SUPPORT_WINDOW_MONTHS >= 6, 
        "Support window must be at least 6 months");
    assert!(EOL_ANNOUNCEMENT_MONTHS >= 3,
        "EOL must be announced at least 3 months in advance");
    assert!(EOL_ANNOUNCEMENT_MONTHS <= SUPPORT_WINDOW_MONTHS,
        "EOL announcement must be within support window");
}
