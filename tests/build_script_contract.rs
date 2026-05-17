const BUILD_RS: &str = include_str!("../build.rs");

#[test]
fn build_script_declares_contract_critical_rerun_triggers() {
    for trigger in [
        "cargo:rerun-if-changed=build.rs",
        "cargo:rerun-if-changed=vendor/governance-bridge",
        "cargo:rerun-if-changed=vendor/governance-bridge/src/contract.rs",
        "cargo:rerun-if-changed=vendor/governance-bridge/src/trace.rs",
        "cargo:rerun-if-changed=vendor/governance-bridge/src/errors.rs",
        "cargo:rerun-if-changed=vendor/governance-bridge/src/lib.rs",
        "cargo:rerun-if-changed=vendor/governance-bridge/CANONICAL_VECTORS.json",
        "cargo:rerun-if-env-changed=ITER_SIMULATE_DRIFT",
    ] {
        assert!(
            BUILD_RS.contains(trigger),
            "BUILD_SCRIPT_RERUN_TRIGGER_MISSING: {}",
            trigger
        );
    }
}

#[test]
fn build_script_exports_contract_provenance_to_runtime() {
    for export in [
        "ITER_SCG_CONTRACT_VERSION",
        "ITER_SCG_SOURCE_COMMIT",
        "ITER_SCG_VENDOR_MASTER_HEAD",
        "ITER_BRIDGE_CONTRACT_RS_SHA256",
        "ITER_BRIDGE_TRACE_RS_SHA256",
        "ITER_BRIDGE_ERRORS_RS_SHA256",
        "ITER_BRIDGE_LIB_RS_SHA256",
        "ITER_CANONICAL_VECTORS_SHA256",
        "ITER_CANONICALIZATION_RULE",
        "ITER_TARGET_TRIPLE",
        "ITER_RUSTC_VERSION",
    ] {
        assert!(
            BUILD_RS.contains(export),
            "RUSTC_ENV_PROVENANCE_EXPORT_MISSING: {}",
            export
        );
    }
}
