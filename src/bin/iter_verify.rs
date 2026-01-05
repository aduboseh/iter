//! iter-verify: External artifact verification CLI for Iter governance decisions.
//!
//! Phase 3 deliverable per ITER_VERIFY_SPEC.md
//!
//! Usage:
//!   iter-verify verify <PATH>              Verify artifact(s)
//!   iter-verify batch <MANIFEST>           Verify from manifest
//!   iter-verify info <PATH>                Display artifact metadata
//!
//! Exit codes:
//!   0 - All artifacts verified successfully
//!   1 - One or more artifacts failed verification
//!   2 - Tool error (invalid input, parse failure, etc.)

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return ExitCode::from(2);
    }

    match args[1].as_str() {
        "verify" => {
            if args.len() < 3 {
                eprintln!("Error: verify requires a path argument");
                print_usage();
                return ExitCode::from(2);
            }
            let format = get_format_arg(&args);
            verify_path(&args[2], format)
        }
        "batch" => {
            if args.len() < 3 {
                eprintln!("Error: batch requires a manifest argument");
                print_usage();
                return ExitCode::from(2);
            }
            let format = get_format_arg(&args);
            batch_verify(&args[2], format)
        }
        "info" => {
            if args.len() < 3 {
                eprintln!("Error: info requires a path argument");
                print_usage();
                return ExitCode::from(2);
            }
            show_info(&args[2])
        }
        "--version" | "-V" => {
            println!("iter-verify {}", VERSION);
            ExitCode::SUCCESS
        }
        "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            ExitCode::from(2)
        }
    }
}

fn print_usage() {
    eprintln!(
        r#"iter-verify {} - Governance artifact verification tool

USAGE:
    iter-verify <COMMAND> [OPTIONS]

COMMANDS:
    verify <PATH>       Verify artifact file or directory
    batch <MANIFEST>    Verify artifacts from JSON manifest
    info <PATH>         Display artifact metadata

OPTIONS:
    --format <FORMAT>   Output format: text, json [default: text]
    --strict            Fail on any warning
    --quiet             Suppress non-error output
    -V, --version       Print version
    -h, --help          Print help

EXIT CODES:
    0    All artifacts verified successfully
    1    One or more artifacts failed verification
    2    Tool error (invalid input, parse failure, etc.)
"#,
        VERSION
    );
}

fn get_format_arg(args: &[String]) -> OutputFormat {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--format" && i + 1 < args.len() {
            return match args[i + 1].as_str() {
                "json" => OutputFormat::Json,
                _ => OutputFormat::Text,
            };
        }
    }
    OutputFormat::Text
}

#[derive(Debug, Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ArtifactData {
    schema_version: u8,
    proposal_id: String,
    proposal_hash: String,
    verdict: String,
    coherence: f64,
    drift: f64,
    cih: String,
    artifact_hash: String,
    replay_ref: String,
    timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct VerificationResult {
    artifact: String,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    coherence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    drift: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actual: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct VerificationReport {
    verified: bool,
    total: usize,
    passed: usize,
    failed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_drift: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    coherence_min: Option<f64>,
    results: Vec<VerificationResult>,
}

#[derive(Debug, Clone, Deserialize)]
struct ManifestEntry {
    path: String,
    #[serde(default)]
    expected_cih: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Manifest {
    artifacts: Vec<ManifestEntry>,
}

fn verify_path(path_str: &str, format: OutputFormat) -> ExitCode {
    let path = Path::new(path_str);

    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path_str);
        return ExitCode::from(2);
    }

    let artifacts = if path.is_dir() {
        collect_artifacts_from_dir(path)
    } else {
        vec![path.to_path_buf()]
    };

    if artifacts.is_empty() {
        eprintln!("No artifacts found at {}", path_str);
        return ExitCode::from(2);
    }

    let report = verify_artifacts(&artifacts, &[]);
    output_report(&report, format);

    if report.verified {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn batch_verify(manifest_path: &str, format: OutputFormat) -> ExitCode {
    let manifest_str = match fs::read_to_string(manifest_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading manifest: {}", e);
            return ExitCode::from(2);
        }
    };

    let manifest: Manifest = match serde_json::from_str(&manifest_str) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error parsing manifest: {}", e);
            return ExitCode::from(2);
        }
    };

    let base_dir = Path::new(manifest_path).parent().unwrap_or(Path::new("."));
    let artifacts: Vec<PathBuf> = manifest
        .artifacts
        .iter()
        .map(|e| base_dir.join(&e.path))
        .collect();

    let expected_cihs: Vec<Option<String>> = manifest
        .artifacts
        .iter()
        .map(|e| e.expected_cih.clone())
        .collect();

    let report = verify_artifacts(&artifacts, &expected_cihs);
    output_report(&report, format);

    if report.verified {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn show_info(path_str: &str) -> ExitCode {
    let path = Path::new(path_str);

    if !path.exists() {
        eprintln!("Error: Path does not exist: {}", path_str);
        return ExitCode::from(2);
    }

    match load_artifact(path) {
        Ok(artifact) => {
            println!("Schema version: {}", artifact.schema_version);
            println!("Proposal ID: {}", artifact.proposal_id);
            println!("Proposal hash: {}", artifact.proposal_hash);
            println!("Verdict: {}", artifact.verdict);
            println!("Coherence: {:.4}", artifact.coherence);
            println!("Drift: {:.2e}", artifact.drift);
            println!("CIH: {}", artifact.cih);
            println!("Artifact hash: {}", artifact.artifact_hash);
            if let Some(ts) = &artifact.timestamp {
                println!("Timestamp: {}", ts);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error loading artifact: {}", e);
            ExitCode::from(2)
        }
    }
}

fn collect_artifacts_from_dir(dir: &Path) -> Vec<PathBuf> {
    let mut artifacts = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str());
                if matches!(ext, Some("json") | Some("bin")) {
                    artifacts.push(path);
                }
            }
        }
    }

    artifacts.sort();
    artifacts
}

fn verify_artifacts(artifacts: &[PathBuf], expected_cihs: &[Option<String>]) -> VerificationReport {
    let mut results = Vec::new();
    let mut passed = 0;
    let mut max_drift: Option<f64> = None;
    let mut min_coherence: Option<f64> = None;

    for (i, path) in artifacts.iter().enumerate() {
        let expected_cih = expected_cihs.get(i).and_then(|c| c.clone());
        let result = verify_single_artifact(path, expected_cih.as_deref());

        if result.status == "PASS" {
            passed += 1;
            if let Some(drift) = result.drift {
                max_drift = Some(max_drift.map_or(drift, |d| d.max(drift)));
            }
            if let Some(coherence) = result.coherence {
                min_coherence = Some(min_coherence.map_or(coherence, |c| c.min(coherence)));
            }
        }

        results.push(result);
    }

    VerificationReport {
        verified: passed == artifacts.len(),
        total: artifacts.len(),
        passed,
        failed: artifacts.len() - passed,
        max_drift,
        coherence_min: min_coherence,
        results,
    }
}

fn verify_single_artifact(path: &Path, expected_cih: Option<&str>) -> VerificationResult {
    let artifact_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let artifact = match load_artifact(path) {
        Ok(a) => a,
        Err(e) => {
            return VerificationResult {
                artifact: artifact_name,
                status: "FAIL",
                fingerprint: None,
                coherence: None,
                drift: None,
                reason: Some(format!("load_error: {}", e)),
                expected: None,
                actual: None,
            };
        }
    };

    // Verify CIH if expected is provided
    if let Some(expected) = expected_cih {
        if artifact.cih != expected {
            return VerificationResult {
                artifact: artifact_name,
                status: "FAIL",
                fingerprint: Some(artifact.cih.clone()),
                coherence: Some(artifact.coherence),
                drift: Some(artifact.drift),
                reason: Some("cih_mismatch".to_string()),
                expected: Some(expected.to_string()),
                actual: Some(artifact.cih),
            };
        }
    }

    // Verify artifact hash integrity
    let computed_hash = compute_artifact_hash(&artifact);
    if computed_hash != artifact.artifact_hash {
        return VerificationResult {
            artifact: artifact_name,
            status: "FAIL",
            fingerprint: Some(artifact.cih.clone()),
            coherence: Some(artifact.coherence),
            drift: Some(artifact.drift),
            reason: Some("artifact_hash_mismatch".to_string()),
            expected: Some(artifact.artifact_hash),
            actual: Some(computed_hash),
        };
    }

    // Check drift bounds (≤1×10⁻¹⁰)
    if artifact.drift > 1e-10 {
        return VerificationResult {
            artifact: artifact_name,
            status: "FAIL",
            fingerprint: Some(artifact.cih.clone()),
            coherence: Some(artifact.coherence),
            drift: Some(artifact.drift),
            reason: Some("drift_exceeded".to_string()),
            expected: Some("≤1e-10".to_string()),
            actual: Some(format!("{:.2e}", artifact.drift)),
        };
    }

    VerificationResult {
        artifact: artifact_name,
        status: "PASS",
        fingerprint: Some(artifact.cih),
        coherence: Some(artifact.coherence),
        drift: Some(artifact.drift),
        reason: None,
        expected: None,
        actual: None,
    }
}

fn load_artifact(path: &Path) -> Result<ArtifactData, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    serde_json::from_str(&content).map_err(|e| format!("Failed to parse artifact: {}", e))
}

fn compute_artifact_hash(artifact: &ArtifactData) -> String {
    let payload = format!(
        "{}:{}:{}:{:.10}:{:.10}",
        artifact.proposal_id,
        artifact.proposal_hash,
        artifact.verdict,
        artifact.coherence,
        artifact.drift,
    );

    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn output_report(report: &VerificationReport, format: OutputFormat) {
    match format {
        OutputFormat::Text => output_text_report(report),
        OutputFormat::Json => output_json_report(report),
    }
}

fn output_text_report(report: &VerificationReport) {
    println!("Verifying {} artifacts...", report.total);

    for result in &report.results {
        let status_marker = if result.status == "PASS" {
            "PASS"
        } else {
            "FAIL"
        };

        if result.status == "PASS" {
            println!(
                "  [{}] {} (coherence: {:.2}, drift: {:.1e})",
                status_marker,
                result.artifact,
                result.coherence.unwrap_or(0.0),
                result.drift.unwrap_or(0.0),
            );
        } else {
            println!(
                "  [{}] {} ({})",
                status_marker,
                result.artifact,
                result.reason.as_deref().unwrap_or("unknown"),
            );
        }
    }

    println!();
    println!(
        "Result: {}/{} verified, {} failed",
        report.passed, report.total, report.failed
    );
}

fn output_json_report(report: &VerificationReport) {
    let json = serde_json::to_string_pretty(report).unwrap_or_else(|_| "{}".to_string());
    println!("{}", json);
}
