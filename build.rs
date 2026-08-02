//! Build script to validate substrate availability and vendored contract integrity.

use std::io::Read;
use std::process::Command;

const ITER_SCG_CONTRACT_VERSION: &str = "scg.v1";
const ITER_SCG_SOURCE_COMMIT: &str = "0306feb600e12c627dc4b10963fc8f7781dc0e18";
const ITER_SCG_VENDOR_MASTER_HEAD: &str = "b6c9a3b641291631358fcf9f8deace74d71e7615";
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
    let is_stub_mode = std::env::var_os("CARGO_FEATURE_PUBLIC_STUB").is_some();
    let is_full_substrate = std::env::var_os("CARGO_FEATURE_FULL_SUBSTRATE").is_some();

    if is_full_substrate && !is_stub_mode {
        panic!(
            "FULL_SUBSTRATE_UNSUPPORTED_IN_PUBLIC_REPO: \
             full_substrate is reserved for the private substrate workspace. \
             This public repository must not emit ITER_BUILD_MODE=FULL_SUBSTRATE \
             or compile a stub-backed binary under full_substrate semantics."
        );
    }
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
            expected_sha256: "82800952f5e03851422fe4469fd159738b927e93b6a284c79f2703070516b3db",
            mismatch_code: "BRIDGE_INTEGRITY_MISMATCH",
        },
        GovernanceArtifact {
            path: "vendor/governance-bridge/src/trace.rs",
            env_name: "ITER_BRIDGE_TRACE_RS_SHA256",
            expected_sha256: "bf9fcb710f709fa73cfa53d51753e21ebbbf0fe68f9d0877b39ef7e1b6dd74dc",
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
            expected_sha256: "3681052fa2346599a9c8e1068a219994c7ffb6c33515e251d2dc13bebf8b0a05",
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

    match (is_stub_mode, is_full_substrate) {
        (true, false) => println!("cargo:rustc-env=ITER_BUILD_MODE=PUBLIC_STUB"),
        (false, true) => panic!(
            "FULL_SUBSTRATE_UNSUPPORTED_IN_PUBLIC_REPO: \
             full_substrate is reserved for the private substrate workspace. \
             This public repository must not emit ITER_BUILD_MODE=FULL_SUBSTRATE \
             or compile a stub-backed binary under full_substrate semantics."
        ),
        (true, true) => {
            panic!("ITER_BUILD_MODE is ambiguous: both public_stub and full_substrate are enabled")
        }
        (false, false) => panic!(
            "ITER_BUILD_MODE is undefined: neither public_stub nor full_substrate is enabled"
        ),
    }
}
