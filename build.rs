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
    println!("cargo:rerun-if-changed=vendor/governance-bridge/CANONICAL_VECTORS.json");

    let expected: &[(&str, &str)] = &[
        (
            "vendor/governance-bridge/src/contract.rs",
            "1179dcdd5e8bc51f88324136fdfb55bfe58be00167cbfe091d0c8731e9b51ab0",
        ),
        (
            "vendor/governance-bridge/src/trace.rs",
            "f1749cb281554807e57be237fcb54c0e6a31d75fc496857973f6b365dbde8167",
        ),
        (
            "vendor/governance-bridge/src/errors.rs",
            "d1459d2ebfd73dfed7d1bc78990a250b72ec701e7260624e320d824c2397d0af",
        ),
        (
            "vendor/governance-bridge/src/lib.rs",
            "e2556d561acba83914a85b445186d6c6a97d4a75b19a95c37ea552c192f61f36",
        ),
        (
            "vendor/governance-bridge/CANONICAL_VECTORS.json",
            "1e804ac4342da71251d4a404bfcee5ef65a2f5b46d599e0fe9d73c80830c1d75",
        ),
    ];

    for (path, expected_hash) in expected {
        let actual = sha256_file(path);
        if actual != *expected_hash {
            panic!(
                "\n\nGOVERNANCE INTEGRITY VIOLATION: {} hash mismatch\n  expected: {}\n  actual:   {}\n\nBuild aborted — provenance cannot be established.\n",
                path, expected_hash, actual
            );
        }
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

    let is_stub_mode = std::env::var_os("CARGO_FEATURE_PUBLIC_STUB").is_some();
    let is_full_substrate = std::env::var_os("CARGO_FEATURE_FULL_SUBSTRATE").is_some();
    match (is_stub_mode, is_full_substrate) {
        (true, false) => println!("cargo:rustc-env=ITER_BUILD_MODE=PUBLIC_STUB"),
        (false, true) => println!("cargo:rustc-env=ITER_BUILD_MODE=FULL_SUBSTRATE"),
        (true, true) => {
            panic!("ITER_BUILD_MODE is ambiguous: both public_stub and full_substrate are enabled")
        }
        (false, false) => panic!(
            "ITER_BUILD_MODE is undefined: neither public_stub nor full_substrate is enabled"
        ),
    }
}
