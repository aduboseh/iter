//! Build script to validate substrate availability and vendored contract integrity.

use std::io::Read;
use std::process::Command;

const ITER_SCG_CONTRACT_VERSION: &str = "scg.v1";
const ITER_SCG_SOURCE_COMMIT: &str = "da14c8390ba8ceeb0ab15d85c598d2042a2029cf";
const ITER_SCG_VENDOR_MASTER_HEAD: &str = "3e0675073a50ce20bdad7c342f7a5caaa3801504";
const ITER_CANONICALIZATION_RULE: &str = "sorted-key-json+utf8-nfc+sha256";

struct GovernanceArtifact {
    path: &'static str,
    env_name: &'static str,
    expected_sha256: &'static str,
    mismatch_code: &'static str,
}

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

fn rustc_version() -> String {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let output = Command::new(rustc)
        .arg("--version")
        .output()
        .expect("RUSTC_VERSION_EXPORT_FAILED: rustc --version must run");
    if !output.status.success() {
        panic!("RUSTC_VERSION_EXPORT_FAILED: rustc --version failed");
    }

    String::from_utf8(output.stdout)
        .expect("RUSTC_VERSION_EXPORT_FAILED: rustc --version must emit UTF-8")
        .trim()
        .to_string()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=vendor/governance-bridge");
    println!("cargo:rerun-if-changed=vendor/governance-bridge/src/contract.rs");
    println!("cargo:rerun-if-changed=vendor/governance-bridge/src/trace.rs");
    println!("cargo:rerun-if-changed=vendor/governance-bridge/src/errors.rs");
    println!("cargo:rerun-if-changed=vendor/governance-bridge/src/lib.rs");
    println!("cargo:rerun-if-changed=vendor/governance-bridge/CANONICAL_VECTORS.json");
    println!("cargo:rerun-if-env-changed=ITER_SIMULATE_DRIFT");

    println!(
        "cargo:rustc-env=ITER_SCG_CONTRACT_VERSION={}",
        ITER_SCG_CONTRACT_VERSION
    );
    println!(
        "cargo:rustc-env=ITER_SCG_SOURCE_COMMIT={}",
        ITER_SCG_SOURCE_COMMIT
    );
    println!(
        "cargo:rustc-env=ITER_SCG_VENDOR_MASTER_HEAD={}",
        ITER_SCG_VENDOR_MASTER_HEAD
    );
    println!(
        "cargo:rustc-env=ITER_CANONICALIZATION_RULE={}",
        ITER_CANONICALIZATION_RULE
    );
    println!(
        "cargo:rustc-env=ITER_TARGET_TRIPLE={}",
        std::env::var("TARGET").expect("TARGET_TRIPLE_EXPORT_FAILED: TARGET must be set")
    );
    println!("cargo:rustc-env=ITER_RUSTC_VERSION={}", rustc_version());

    let expected: &[GovernanceArtifact] = &[
        GovernanceArtifact {
            path: "vendor/governance-bridge/src/contract.rs",
            env_name: "ITER_BRIDGE_CONTRACT_RS_SHA256",
            expected_sha256: "1179dcdd5e8bc51f88324136fdfb55bfe58be00167cbfe091d0c8731e9b51ab0",
            mismatch_code: "BRIDGE_INTEGRITY_MISMATCH",
        },
        GovernanceArtifact {
            path: "vendor/governance-bridge/src/trace.rs",
            env_name: "ITER_BRIDGE_TRACE_RS_SHA256",
            expected_sha256: "620892e1986dc22a2a5c17f60ec650e6da70dbe90b847a2862e13c1bf14bce20",
            mismatch_code: "BRIDGE_INTEGRITY_MISMATCH",
        },
        GovernanceArtifact {
            path: "vendor/governance-bridge/src/errors.rs",
            env_name: "ITER_BRIDGE_ERRORS_RS_SHA256",
            expected_sha256: "d1459d2ebfd73dfed7d1bc78990a250b72ec701e7260624e320d824c2397d0af",
            mismatch_code: "BRIDGE_INTEGRITY_MISMATCH",
        },
        GovernanceArtifact {
            path: "vendor/governance-bridge/src/lib.rs",
            env_name: "ITER_BRIDGE_LIB_RS_SHA256",
            expected_sha256: "e2556d561acba83914a85b445186d6c6a97d4a75b19a95c37ea552c192f61f36",
            mismatch_code: "BRIDGE_INTEGRITY_MISMATCH",
        },
        GovernanceArtifact {
            path: "vendor/governance-bridge/CANONICAL_VECTORS.json",
            env_name: "ITER_CANONICAL_VECTORS_SHA256",
            expected_sha256: "1e804ac4342da71251d4a404bfcee5ef65a2f5b46d599e0fe9d73c80830c1d75",
            mismatch_code: "CANONICAL_VECTOR_HASH_MISMATCH",
        },
    ];

    if ITER_SCG_CONTRACT_VERSION != "scg.v1" {
        panic!("CONTRACT_VERSION_MISMATCH: expected scg.v1");
    }

    let simulate_drift = matches!(std::env::var("ITER_SIMULATE_DRIFT").as_deref(), Ok("1"));

    for artifact in expected {
        println!(
            "cargo:rustc-env={}={}",
            artifact.env_name, artifact.expected_sha256
        );

        let actual = if simulate_drift && artifact.env_name == "ITER_BRIDGE_CONTRACT_RS_SHA256" {
            "000000000000000000000000000000000000000000000000000000000000dead".to_string()
        } else {
            sha256_file(artifact.path)
        };

        if actual != artifact.expected_sha256 {
            let code = if simulate_drift {
                "BRIDGE_INTEGRITY_MISMATCH_SIMULATED"
            } else {
                artifact.mismatch_code
            };
            panic!(
                "\n\n{}\nartifact : {}\nexpected : {}\nactual   : {}\n\nBuild aborted — provenance cannot be established.\n",
                code, artifact.path, artifact.expected_sha256, actual
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
        .map(|artifact| format!("{}:{}", artifact.path, artifact.expected_sha256))
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
