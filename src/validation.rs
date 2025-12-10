//! Input Validation for MCP Handlers
//!
//! Provides validation functions for MCP request parameters.
//! Validation is about well-posedness (types, ranges, payload sizes),
//! not domain-specific physics or ethics (handled by substrate).

use crate::types::McpError;

// ============================================================================
// Validation Constants
// ============================================================================

/// Maximum belief value (contractual)
pub const MAX_BELIEF: f64 = 1.0;

/// Minimum belief value (contractual)
pub const MIN_BELIEF: f64 = 0.0;

/// Maximum energy value (practical limit)
pub const MAX_ENERGY: f64 = 1e12;

/// Minimum energy value
pub const MIN_ENERGY: f64 = 0.0;

/// Maximum weight for edges
pub const MAX_WEIGHT: f64 = 1e6;

/// Maximum JSON payload size in bytes
pub const MAX_PAYLOAD_SIZE: usize = 1024 * 1024; // 1MB

// ============================================================================
// Validation Functions
// ============================================================================

/// Validate belief value is in contractual range [0.0, 1.0].
pub fn validate_belief(belief: f64) -> Result<f64, McpError> {
    if belief.is_nan() {
        return Err(McpError::BadRequest {
            message: "belief cannot be NaN".to_string(),
        });
    }
    if belief.is_infinite() {
        return Err(McpError::BadRequest {
            message: "belief cannot be infinite".to_string(),
        });
    }
    if !(MIN_BELIEF..=MAX_BELIEF).contains(&belief) {
        return Err(McpError::BadRequest {
            message: format!("belief {} out of valid range [{}, {}]", belief, MIN_BELIEF, MAX_BELIEF),
        });
    }
    Ok(belief)
}

/// Validate energy value is non-negative and within practical limits.
pub fn validate_energy(energy: f64) -> Result<f64, McpError> {
    if energy.is_nan() {
        return Err(McpError::BadRequest {
            message: "energy cannot be NaN".to_string(),
        });
    }
    if energy.is_infinite() {
        return Err(McpError::BadRequest {
            message: "energy cannot be infinite".to_string(),
        });
    }
    if energy < MIN_ENERGY {
        return Err(McpError::BadRequest {
            message: format!("energy {} cannot be negative", energy),
        });
    }
    if energy > MAX_ENERGY {
        return Err(McpError::BadRequest {
            message: format!("energy {} exceeds maximum {}", energy, MAX_ENERGY),
        });
    }
    Ok(energy)
}

/// Validate edge weight.
pub fn validate_weight(weight: f64) -> Result<f64, McpError> {
    if weight.is_nan() {
        return Err(McpError::BadRequest {
            message: "weight cannot be NaN".to_string(),
        });
    }
    if weight.is_infinite() {
        return Err(McpError::BadRequest {
            message: "weight cannot be infinite".to_string(),
        });
    }
    if weight.abs() > MAX_WEIGHT {
        return Err(McpError::BadRequest {
            message: format!("weight {} exceeds maximum magnitude {}", weight, MAX_WEIGHT),
        });
    }
    Ok(weight)
}

/// Validate node ID string can be parsed as u64.
pub fn validate_node_id(node_id: &str) -> Result<u64, McpError> {
    node_id.parse::<u64>().map_err(|e| McpError::BadRequest {
        message: format!("Invalid node ID '{}': {}", node_id, e),
    })
}

/// Validate payload size is within limits.
pub fn validate_payload_size(payload: &[u8]) -> Result<(), McpError> {
    if payload.len() > MAX_PAYLOAD_SIZE {
        return Err(McpError::BadRequest {
            message: format!(
                "Payload size {} exceeds maximum {} bytes",
                payload.len(),
                MAX_PAYLOAD_SIZE
            ),
        });
    }
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_belief_valid() {
        assert!(validate_belief(0.0).is_ok());
        assert!(validate_belief(0.5).is_ok());
        assert!(validate_belief(1.0).is_ok());
    }

    #[test]
    fn test_validate_belief_invalid() {
        assert!(validate_belief(-0.1).is_err());
        assert!(validate_belief(1.1).is_err());
        assert!(validate_belief(f64::NAN).is_err());
        assert!(validate_belief(f64::INFINITY).is_err());
    }

    #[test]
    fn test_validate_energy_valid() {
        assert!(validate_energy(0.0).is_ok());
        assert!(validate_energy(100.0).is_ok());
        assert!(validate_energy(1e10).is_ok());
    }

    #[test]
    fn test_validate_energy_invalid() {
        assert!(validate_energy(-1.0).is_err());
        assert!(validate_energy(f64::NAN).is_err());
        assert!(validate_energy(f64::INFINITY).is_err());
        assert!(validate_energy(1e15).is_err());
    }

    #[test]
    fn test_validate_weight_valid() {
        assert!(validate_weight(0.0).is_ok());
        assert!(validate_weight(0.5).is_ok());
        assert!(validate_weight(-0.5).is_ok());
    }

    #[test]
    fn test_validate_weight_invalid() {
        assert!(validate_weight(f64::NAN).is_err());
        assert!(validate_weight(f64::INFINITY).is_err());
        assert!(validate_weight(1e10).is_err());
    }

    #[test]
    fn test_validate_node_id_valid() {
        assert_eq!(validate_node_id("0").unwrap(), 0);
        assert_eq!(validate_node_id("123").unwrap(), 123);
        assert_eq!(validate_node_id("18446744073709551615").unwrap(), u64::MAX);
    }

    #[test]
    fn test_validate_node_id_invalid() {
        assert!(validate_node_id("").is_err());
        assert!(validate_node_id("-1").is_err());
        assert!(validate_node_id("abc").is_err());
        assert!(validate_node_id("1.5").is_err());
    }

    #[test]
    fn test_validate_payload_size() {
        let small = vec![0u8; 100];
        assert!(validate_payload_size(&small).is_ok());

        let large = vec![0u8; MAX_PAYLOAD_SIZE + 1];
        assert!(validate_payload_size(&large).is_err());
    }
}
