//! Exact numeric encodings for proof-critical contract fields.

use serde::{de::Error as _, ser::Error as _, Deserialize, Deserializer, Serializer};

/// Encoding used for proof-critical f64 values in DecisionPacket JSON.
pub const F64_HEX_ENCODING: &str = "ieee754-f64-bits-lowerhex";

/// Convert an f64 to its exact IEEE-754 bit pattern as 16 lowercase hex chars.
pub fn f64_to_hex(value: f64) -> String {
    format!("{:016x}", value.to_bits())
}

/// Convert a 16-char lowercase hex bit pattern to f64.
pub fn f64_from_hex(encoded: &str) -> Result<f64, String> {
    if encoded.len() != 16 {
        return Err(format!(
            "expected 16 hex chars for f64 bit pattern, got {}",
            encoded.len()
        ));
    }
    if !encoded.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err("f64 bit pattern contains non-hex characters".to_string());
    }
    if encoded != encoded.to_ascii_lowercase() {
        return Err("f64 bit pattern must use lowercase hex".to_string());
    }

    let bits = u64::from_str_radix(encoded, 16)
        .map_err(|err| format!("invalid f64 bit pattern: {err}"))?;
    let value = f64::from_bits(bits);
    if !value.is_finite() {
        return Err("f64 bit pattern must decode to a finite value".to_string());
    }
    Ok(value)
}

/// Serde adapter for f64 values encoded as exact IEEE-754 hex strings.
pub mod f64_hex {
    use super::*;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum EncodedF64 {
        Hex(String),
        Number(f64),
    }

    /// Serialize an f64 as its exact IEEE-754 hex bit pattern.
    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !value.is_finite() {
            return Err(S::Error::custom("f64 value must be finite"));
        }
        serializer.serialize_str(&f64_to_hex(*value))
    }

    /// Deserialize an exact IEEE-754 hex bit pattern into a finite f64.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        match EncodedF64::deserialize(deserializer)? {
            EncodedF64::Hex(encoded) => f64_from_hex(&encoded).map_err(D::Error::custom),
            EncodedF64::Number(value) => {
                if value.is_finite() {
                    Ok(value)
                } else {
                    Err(D::Error::custom("f64 value must be finite"))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f64_hex_roundtrips_exact_bits() {
        for value in [0.0, 1.0, 0.95, 1e-10, 1e12] {
            let encoded = f64_to_hex(value);
            let decoded = f64_from_hex(&encoded).unwrap();

            assert_eq!(decoded.to_bits(), value.to_bits());
            assert_eq!(encoded.len(), 16);
            assert!(encoded.chars().all(|ch| ch.is_ascii_hexdigit()));
            assert_eq!(encoded, encoded.to_ascii_lowercase());
        }
    }

    #[test]
    fn f64_hex_rejects_non_finite_values() {
        assert!(f64_from_hex(&f64_to_hex(f64::INFINITY)).is_err());
        assert!(f64_from_hex(&f64_to_hex(f64::NAN)).is_err());
    }

    #[test]
    fn f64_hex_rejects_uppercase_input() {
        assert!(f64_from_hex("3FF0000000000000").is_err());
    }

    #[test]
    fn f64_hex_serde_reads_legacy_json_numbers() {
        #[derive(Deserialize)]
        struct Probe {
            #[serde(with = "super::f64_hex")]
            value: f64,
        }

        let probe: Probe = serde_json::from_str(r#"{"value":1.25}"#).unwrap();
        assert_eq!(probe.value.to_bits(), 1.25f64.to_bits());
    }

    #[test]
    fn f64_hex_serde_rejects_non_finite_on_write() {
        #[derive(serde::Serialize)]
        struct Probe {
            #[serde(with = "super::f64_hex")]
            value: f64,
        }

        let err = serde_json::to_string(&Probe {
            value: f64::INFINITY,
        })
        .expect_err("non-finite value must fail serialization");
        assert!(err.to_string().contains("f64 value must be finite"));
    }
}
