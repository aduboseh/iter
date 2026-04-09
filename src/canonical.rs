//! Canonical hash computation — single source of truth for build and runtime.
//!
//! Both `build.rs` (file-integrity hashes) and the runtime proposal attestation
//! path in `stub.rs` route through these functions. This guarantees identical
//! algorithm and encoding regardless of call site.
//!
//! # Rules
//!
//! - All hashes are SHA-256 of raw bytes, returned as lowercase hex.
//! - No normalization is applied. Callers are responsible for producing
//!   well-formed input before hashing (e.g. NFC normalization, JCS
//!   canonicalization).
//! - `hash_bytes` operates on a raw byte slice — use this when the data
//!   originated from a binary encoding (base64 decode, file read, etc.).
//! - `hash_str` is a convenience wrapper for UTF-8 string data only. Do NOT
//!   use it on bytes that were decoded from base64 or other binary encodings.

use sha2::{Digest, Sha256};

/// Compute SHA-256 of a raw byte slice. Returns lowercase hex string.
///
/// This is the canonical primitive. All hash computations in this codebase
/// must ultimately call this function to guarantee algorithm and encoding
/// consistency.
///
/// # Example
/// ```
/// let hash = iter_mcp_server::canonical::hash_bytes(b"hello");
/// assert_eq!(hash.len(), 64); // 32 bytes × 2 hex chars
/// ```
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Compute SHA-256 of a UTF-8 string's byte representation.
///
/// Equivalent to `hash_bytes(s.as_bytes())`. Use this for string data.
/// Do **not** use this to hash bytes that were decoded from base64 or
/// another binary encoding — call `hash_bytes` on the raw slice instead,
/// otherwise lossy UTF-8 conversion may silently corrupt the input.
///
/// # Example
/// ```
/// let hash = iter_mcp_server::canonical::hash_str("hello");
/// assert_eq!(hash, iter_mcp_server::canonical::hash_bytes(b"hello"));
/// ```
pub fn hash_str(s: &str) -> String {
    hash_bytes(s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_bytes_and_hash_str_agree_on_utf8() {
        let s = "proposal:123:snapshot:abc";
        assert_eq!(hash_str(s), hash_bytes(s.as_bytes()));
    }

    #[test]
    fn hash_is_stable_across_calls() {
        let data = b"deterministic input";
        assert_eq!(hash_bytes(data), hash_bytes(data));
    }

    #[test]
    fn hash_output_is_lowercase_hex_64_chars() {
        let h = hash_bytes(b"test");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit() && !c.is_uppercase()));
    }

    #[test]
    fn empty_input_produces_known_sha256() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let h = hash_bytes(b"");
        assert_eq!(
            h,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn hash_bytes_does_not_apply_utf8_lossy_conversion() {
        // Raw bytes that are not valid UTF-8.
        // hash_bytes must hash them as-is; hash_str would be wrong to call here.
        let raw: &[u8] = &[0xFF, 0xFE, 0x00, 0x01];
        let h1 = hash_bytes(raw);
        let h2 = hash_bytes(raw);
        assert_eq!(h1, h2);
        // Verify it does NOT equal the hash of the lossy string version
        // (which would replace 0xFF with U+FFFD replacement character bytes).
        let lossy = String::from_utf8_lossy(raw);
        let h_lossy = hash_bytes(lossy.as_bytes());
        assert_ne!(h1, h_lossy, "hash_bytes must not apply lossy UTF-8 conversion");
    }
}
