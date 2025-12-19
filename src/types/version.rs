//! Protocol Version Management
//!
//! Defines protocol versioning, compatibility rules, and deprecation tracking.
//!
//! # Versioning Scheme
//!
//! Iter follows semantic versioning for its MCP protocol:
//! - **Major**: Breaking changes (field removal, type changes, semantic shifts)
//! - **Minor**: Backward-compatible additions (new optional fields, new error codes)
//! - **Patch**: Bug fixes, documentation updates (no wire changes)
//!
//! # Compatibility Window
//!
//! Iter supports the current major version and one prior (N, N-1).
//! Clients should upgrade within one major release cycle.

use serde::{Deserialize, Serialize};

/// Current protocol version
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Protocol major version (for compatibility checks)
pub const PROTOCOL_MAJOR: u32 = 1;

/// Protocol minor version
pub const PROTOCOL_MINOR: u32 = 0;

/// Protocol patch version
pub const PROTOCOL_PATCH: u32 = 0;

/// Minimum supported major version (N-1 compatibility)
pub const MIN_SUPPORTED_MAJOR: u32 = 1;

/// Protocol version information included in responses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolVersion {
    /// Full version string (e.g., "1.0.0")
    pub version: String,
    /// Major version number
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Patch version number
    pub patch: u32,
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::current()
    }
}

impl ProtocolVersion {
    /// Returns the current protocol version
    pub fn current() -> Self {
        Self {
            version: PROTOCOL_VERSION.to_string(),
            major: PROTOCOL_MAJOR,
            minor: PROTOCOL_MINOR,
            patch: PROTOCOL_PATCH,
        }
    }

    /// Parse a version string (e.g., "1.0.0")
    pub fn parse(version: &str) -> Option<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;

        Some(Self {
            version: version.to_string(),
            major,
            minor,
            patch,
        })
    }

    /// Check if this version is compatible with the current protocol
    pub fn is_compatible(&self) -> bool {
        // Compatible if within supported major version range
        self.major >= MIN_SUPPORTED_MAJOR && self.major <= PROTOCOL_MAJOR
    }

    /// Check if this version is exactly the current version
    pub fn is_current(&self) -> bool {
        self.major == PROTOCOL_MAJOR && self.minor == PROTOCOL_MINOR && self.patch == PROTOCOL_PATCH
    }
}

/// Compatibility check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatibilityStatus {
    /// Fully compatible (same major, same or lower minor)
    Compatible,
    /// Forward compatible (same major, higher minor - client is newer)
    ForwardCompatible,
    /// Deprecated (older major, still supported)
    Deprecated { supported_until: &'static str },
    /// Incompatible (outside support window)
    Incompatible { reason: String },
}

impl ProtocolVersion {
    /// Detailed compatibility check
    pub fn check_compatibility(&self) -> CompatibilityStatus {
        if self.major > PROTOCOL_MAJOR {
            return CompatibilityStatus::Incompatible {
                reason: format!(
                    "Client version {}.x is newer than server {}.x",
                    self.major, PROTOCOL_MAJOR
                ),
            };
        }

        if self.major < MIN_SUPPORTED_MAJOR {
            return CompatibilityStatus::Incompatible {
                reason: format!(
                    "Client version {}.x is below minimum supported {}.x",
                    self.major, MIN_SUPPORTED_MAJOR
                ),
            };
        }

        if self.major < PROTOCOL_MAJOR {
            return CompatibilityStatus::Deprecated {
                supported_until: "2.0.0", // Update when deprecating v1
            };
        }

        if self.minor > PROTOCOL_MINOR {
            return CompatibilityStatus::ForwardCompatible;
        }

        CompatibilityStatus::Compatible
    }
}

/// Deprecation marker for fields or features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deprecation {
    /// Version when deprecated
    pub since: String,
    /// Version when removal is planned
    pub removal: String,
    /// Migration guidance
    pub message: String,
}

impl Deprecation {
    /// Create a new deprecation marker
    pub fn new(since: &str, removal: &str, message: &str) -> Self {
        Self {
            since: since.to_string(),
            removal: removal.to_string(),
            message: message.to_string(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_is_valid() {
        let v = ProtocolVersion::current();
        assert_eq!(v.version, PROTOCOL_VERSION);
        assert!(v.is_current());
        assert!(v.is_compatible());
    }

    #[test]
    fn version_parsing() {
        let v = ProtocolVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);

        assert!(ProtocolVersion::parse("invalid").is_none());
        assert!(ProtocolVersion::parse("1.2").is_none());
        assert!(ProtocolVersion::parse("1.2.3.4").is_none());
    }

    #[test]
    fn compatibility_same_version() {
        let v = ProtocolVersion::current();
        assert_eq!(v.check_compatibility(), CompatibilityStatus::Compatible);
    }

    #[test]
    fn compatibility_older_minor() {
        let v = ProtocolVersion {
            version: "1.0.0".to_string(),
            major: 1,
            minor: 0,
            patch: 0,
        };
        // Same major, same or older minor is compatible
        assert!(matches!(
            v.check_compatibility(),
            CompatibilityStatus::Compatible
        ));
    }

    #[test]
    fn compatibility_newer_client() {
        let v = ProtocolVersion {
            version: "2.0.0".to_string(),
            major: 2,
            minor: 0,
            patch: 0,
        };
        assert!(matches!(
            v.check_compatibility(),
            CompatibilityStatus::Incompatible { .. }
        ));
    }

    #[test]
    fn deprecation_marker() {
        let d = Deprecation::new("1.0.0", "2.0.0", "Use new_field instead");
        assert_eq!(d.since, "1.0.0");
        assert_eq!(d.removal, "2.0.0");
    }
}
