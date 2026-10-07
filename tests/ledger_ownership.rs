use iter_mcp_server::audit::PersistentAuditLedger;
use std::process::{Command, Stdio};

#[test]
fn governed_servers_reject_busy_ledger_before_serving() {
    let path = std::env::temp_dir().join(format!(
        "iter-ledger-startup-{}-{}.jsonl",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
    ));
    let owner = PersistentAuditLedger::open(&path).unwrap();
    let server = |mode: &str| {
        Command::new(env!("CARGO_BIN_EXE_iter-server"))
            .arg("--json-only")
            .arg(format!("--runtime-mode={mode}"))
            .env("ITER_AUDIT_LEDGER_PATH", &path)
            .env("ITER_REQUIRE_AUDIT_LEDGER", "1")
            .env("SCG_ENDPOINT", "http://127.0.0.1:1")
            .env(
                "SCG_GOVERNANCE_HASH_PATH",
                concat!(env!("CARGO_MANIFEST_DIR"), "/governance/governance.hash"),
            )
            .stdin(Stdio::null())
            .output()
            .unwrap()
    };
    for mode in ["governed-local", "scg-backed"] {
        let output = server(mode);
        assert!(
            !output.status.success(),
            "{mode} must reject competing startup"
        );
        assert!(String::from_utf8_lossy(&output.stderr).contains("writer lock"));
        assert!(
            output.stdout.is_empty(),
            "busy startup must not publish output"
        );
    }
    assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);
    drop(owner);
    for mode in ["governed-local", "scg-backed"] {
        let output = server(mode);
        assert!(
            output.status.success(),
            "{mode} must start after lock release: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::remove_file(path).unwrap();
}
