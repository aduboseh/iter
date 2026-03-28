use std::process::{Command, Stdio};

#[test]
fn scgbacked_mode_fails_closed_without_connector() {
    let bin_path = env!("CARGO_BIN_EXE_iter-server");
    let output = Command::new(bin_path)
        .arg("--json-only")
        .arg("--runtime-mode=scg-backed")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn scg-backed runtime mode");

    assert!(
        !output.status.success(),
        "scg-backed mode must exit non-zero until a real connector exists"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "scg-backed downgrade must not emit a demo fallback response, got: {}",
        stdout
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ScgBacked mode not available"),
        "stderr must contain the explicit scg-backed warning, got: {}",
        stderr
    );
    assert!(
        stderr.contains("mode error: scg-backed connector not implemented"),
        "stderr must contain the explicit fail-closed mode error, got: {}",
        stderr
    );
    assert!(
        !stderr.contains("ERROR_INVALID_RUNTIME_MODE"),
        "scg-backed must be a recognized runtime mode, got: {}",
        stderr
    );
}
