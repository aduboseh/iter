//! Build script to validate substrate availability and vendored contract integrity.

use std::io::Read;

fn sha256_file(path: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut file = std::fs::File::open(path)
        .unwrap_or_else(|e| panic!("INTEGRITY: cannot open {}: {}", path, e));
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .unwrap_or_else(|e| panic!("INTEGRITY: cannot read {}: {}", path, e));

    let mut hasher = Sha256::new();
    hasher.update(&contents);
    hex::encode(hasher.finalize())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn main() {
    println!("cargo:rerun-if-changed=vendor/governance-bridge/src/contract.rs");
    println!("cargo:rerun-if-changed=vendor/governance-bridge/src/trace.rs");
    println!("cargo:rerun-if-changed=vendor/governance-bridge/src/errors.rs");
    println!("cargo:rerun-if-changed=vendor/governance-bridge/src/lib.rs");

    let expected: &[(&str, &str)] = &[
        (
            "vendor/governance-bridge/src/contract.rs",
            "1179dcdd5e8bc51f88324136fdfb55bfe58be00167cbfe091d0c8731e9b51ab0",
        ),
        (
            "vendor/governance-bridge/src/trace.rs",
            "8924a7020662a6c3bff7080e48864e2b409adb4217038b2a05b233f355eaa974",
        ),
        (
            "vendor/governance-bridge/src/errors.rs",
            "d1459d2ebfd73dfed7d1bc78990a250b72ec701e7260624e320d824c2397d0af",
        ),
        (
            "vendor/governance-bridge/src/lib.rs",
            "e2556d561acba83914a85b445186d6c6a97d4a75b19a95c37ea552c192f61f36",
        ),
    ];

    let mut failed = false;
    for (path, expected_hash) in expected {
        let actual = sha256_file(path);
        if actual != *expected_hash {
            println!(
                "cargo:warning=GOVERNANCE INTEGRITY VIOLATION: {} hash mismatch",
                path
            );
            println!("cargo:warning=  expected: {}", expected_hash);
            println!("cargo:warning=  actual:   {}", actual);
            failed = true;
        }
    }

    if failed {
        panic!(
            "Vendor governance-bridge integrity check failed. The vendored contract does not match the canonical SCG source. To update intentionally: recompute hashes in build.rs and update vendor/governance-bridge/PROVENANCE.md with the new commit and hash."
        );
    }

    let iter_identity = format!(
        "{}:{}",
        std::env::var("CARGO_PKG_NAME").unwrap_or_default(),
        std::env::var("CARGO_PKG_VERSION").unwrap_or_default()
    );
    println!(
        "cargo:rustc-env=ITER_BUILD_HASH={}",
        sha256_bytes(iter_identity.as_bytes())
    );

    let substrate_identity = expected
        .iter()
        .map(|(path, hash)| format!("{}:{}", path, hash))
        .collect::<Vec<_>>()
        .join("\n");
    println!(
        "cargo:rustc-env=SUBSTRATE_BUILD_HASH={}",
        sha256_bytes(substrate_identity.as_bytes())
    );

    // Only check for substrate in full mode (not when public_stub is enabled)
    #[cfg(all(feature = "full_substrate", not(feature = "public_stub")))]
    {
        let scg_path = std::path::Path::new("../SCG");
        if !scg_path.exists() {
            eprintln!();
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("  Error: Full substrate mode requires proprietary workspace.");
            eprintln!();
            eprintln!("  The substrate dependency path '../SCG' does not exist.");
            eprintln!();
            eprintln!("  For public builds, use:");
            eprintln!("    cargo build --features public_stub --no-default-features");
            eprintln!();
            eprintln!("  This mode provides:");
            eprintln!("    - Full MCP protocol implementation");
            eprintln!("    - Deterministic placeholder responses");
            eprintln!("    - Active sanitization and lineage tracing");
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!();
            std::process::exit(1);
        }
    }

    // Print build mode for verification
    #[cfg(feature = "public_stub")]
    println!("cargo:warning=Building in PUBLIC STUB mode");

    #[cfg(all(feature = "full_substrate", not(feature = "public_stub")))]
    println!("cargo:warning=Building with FULL SUBSTRATE");
}
