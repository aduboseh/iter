//! ITER-PAR-01: Contract Validation
//!
//! Strict validation for contract fields.
//! NaN and +/-Inf are hard errors. Fail fast on violations.

use thiserror::Error;

/// Contract validation errors.
#[derive(Debug, Clone, Error)]
pub enum ContractError {
    /// Float value is NaN
    #[error("{field}: value is NaN")]
    NaN {
        /// Field name that failed validation
        field: String,
    },

    /// Float value is infinite
    #[error("{field}: value is infinite")]
    Infinite {
        /// Field name that failed validation
        field: String,
    },

    /// Float value out of bounds
    #[error("{field}: value {value} out of bounds [{min}, {max}]")]
    OutOfBounds {
        /// Field name that failed validation
        field: String,
        /// Actual value
        value: f64,
        /// Minimum allowed value
        min: f64,
        /// Maximum allowed value
        max: f64,
    },

    /// Invalid hash format
    #[error("{field}: invalid hash format (expected 64 hex chars, got {length})")]
    InvalidHash {
        /// Field name that failed validation
        field: String,
        /// Actual length of hash string
        length: usize,
    },

    /// Invalid hash characters
    #[error("{field}: invalid hex characters in hash")]
    InvalidHexChars {
        /// Field name that failed validation
        field: String,
    },

    /// Unknown enum value (fail closed)
    #[error("{field}: unknown enum value '{value}'")]
    UnknownEnum {
        /// Field name that failed validation
        field: String,
        /// Unknown enum value received
        value: String,
    },

    /// Missing required field
    #[error("{field}: required field is missing")]
    MissingField {
        /// Field name that is missing
        field: String,
    },
}

/// Validate a float is finite and within bounds.
///
/// # Errors
/// - NaN => ContractError::NaN
/// - Infinite => ContractError::Infinite
/// - Out of bounds => ContractError::OutOfBounds
pub fn validate_bounded_float(
    value: f64,
    min: f64,
    max: f64,
    field: &str,
) -> Result<f64, ContractError> {
    if value.is_nan() {
        return Err(ContractError::NaN {
            field: field.to_string(),
        });
    }
    if value.is_infinite() {
        return Err(ContractError::Infinite {
            field: field.to_string(),
        });
    }
    // For max=f64::MAX, allow any finite positive value
    let effective_max = if max == f64::MAX { f64::MAX } else { max };
    if value < min || (max != f64::MAX && value > effective_max) {
        return Err(ContractError::OutOfBounds {
            field: field.to_string(),
            value,
            min,
            max,
        });
    }
    Ok(value)
}

/// Validate a SHA-256 hash (hex-encoded, 64 characters).
///
/// # Errors
/// - Wrong length => ContractError::InvalidHash
/// - Non-hex characters => ContractError::InvalidHexChars
pub fn validate_hash(hash: &str, field: &str) -> Result<(), ContractError> {
    if hash.len() != 64 {
        return Err(ContractError::InvalidHash {
            field: field.to_string(),
            length: hash.len(),
        });
    }
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ContractError::InvalidHexChars {
            field: field.to_string(),
        });
    }
    Ok(())
}

/// Validate optional hash (None is valid, Some must be valid hash).
pub fn validate_optional_hash(hash: &Option<String>, field: &str) -> Result<(), ContractError> {
    if let Some(h) = hash {
        validate_hash(h, field)?;
    }
    Ok(())
}

/// Parse hash bytes from hex string.
///
/// # Errors
/// Returns ContractError if hash is invalid format.
pub fn parse_hash_bytes(hash: &str, field: &str) -> Result<[u8; 32], ContractError> {
    validate_hash(hash, field)?;
    let mut bytes = [0u8; 32];
    for (i, chunk) in hash.as_bytes().chunks(2).enumerate() {
        let hex_str = std::str::from_utf8(chunk).map_err(|_| ContractError::InvalidHexChars {
            field: field.to_string(),
        })?;
        bytes[i] = u8::from_str_radix(hex_str, 16).map_err(|_| ContractError::InvalidHexChars {
            field: field.to_string(),
        })?;
    }
    Ok(bytes)
}

/// Encode hash bytes to hex string.
pub fn encode_hash_hex(bytes: &[u8; 32]) -> String {
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_float_rejects_nan() {
        let result = validate_bounded_float(f64::NAN, 0.0, 1.0, "test");
        assert!(matches!(result, Err(ContractError::NaN { .. })));
    }

    #[test]
    fn validate_float_rejects_pos_inf() {
        let result = validate_bounded_float(f64::INFINITY, 0.0, 1.0, "test");
        assert!(matches!(result, Err(ContractError::Infinite { .. })));
    }

    #[test]
    fn validate_float_rejects_neg_inf() {
        let result = validate_bounded_float(f64::NEG_INFINITY, 0.0, 1.0, "test");
        assert!(matches!(result, Err(ContractError::Infinite { .. })));
    }

    #[test]
    fn validate_float_rejects_below_min() {
        let result = validate_bounded_float(-0.1, 0.0, 1.0, "test");
        assert!(matches!(result, Err(ContractError::OutOfBounds { .. })));
    }

    #[test]
    fn validate_float_rejects_above_max() {
        let result = validate_bounded_float(1.1, 0.0, 1.0, "test");
        assert!(matches!(result, Err(ContractError::OutOfBounds { .. })));
    }

    #[test]
    fn validate_float_accepts_in_range() {
        assert!(validate_bounded_float(0.0, 0.0, 1.0, "test").is_ok());
        assert!(validate_bounded_float(0.5, 0.0, 1.0, "test").is_ok());
        assert!(validate_bounded_float(1.0, 0.0, 1.0, "test").is_ok());
    }

    #[test]
    fn validate_float_allows_max_unbounded() {
        let result = validate_bounded_float(1e100, 0.0, f64::MAX, "test");
        assert!(result.is_ok());
    }

    #[test]
    fn validate_hash_rejects_short() {
        let result = validate_hash("abc", "test");
        assert!(matches!(result, Err(ContractError::InvalidHash { .. })));
    }

    #[test]
    fn validate_hash_rejects_long() {
        let long_hash = "a".repeat(65);
        let result = validate_hash(&long_hash, "test");
        assert!(matches!(result, Err(ContractError::InvalidHash { .. })));
    }

    #[test]
    fn validate_hash_rejects_non_hex() {
        let bad_hash = "g".repeat(64);
        let result = validate_hash(&bad_hash, "test");
        assert!(matches!(result, Err(ContractError::InvalidHexChars { .. })));
    }

    #[test]
    fn validate_hash_accepts_valid() {
        let good_hash = "a1b2c3d4e5f6".to_string() + &"0".repeat(52);
        assert!(validate_hash(&good_hash, "test").is_ok());
    }

    #[test]
    fn validate_hash_accepts_uppercase() {
        let upper_hash = "A".repeat(64);
        assert!(validate_hash(&upper_hash, "test").is_ok());
    }

    #[test]
    fn parse_hash_bytes_roundtrip() {
        let original = "a1b2c3d4e5f60000000000000000000000000000000000000000000000000000";
        let bytes = parse_hash_bytes(original, "test").unwrap();
        let encoded = encode_hash_hex(&bytes);
        assert_eq!(encoded, original);
    }
}
