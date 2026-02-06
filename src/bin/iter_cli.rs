//! iter-cli: Operator-grade replay and audit tooling for Iter governance decisions.
//!
//! All commands operate strictly on governed-mode artifacts and do not
//! introduce new governance decisions.
//!
//! Usage:
//!   iter-cli replay --decision-file <PATH> --policy-version <VER> --schema-version <VER>
//!   iter-cli audit export --decision-file <PATH> --output <PATH>
//!
//! Exit codes:
//!   0 - Success (VERIFIED / EXPORTED)
//!   1 - Input error (missing file, malformed JSON, missing required flags)
//!   2 - Replay/contract mismatch or integrity failure
//!   3 - Internal error

use std::fs;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return ExitCode::from(1);
    }

    match args[1].as_str() {
        "replay" => cmd_replay(&args[2..]),
        "audit" => {
            if args.get(2).map(|s| s.as_str()) == Some("export") {
                cmd_audit_export(&args[3..])
            } else {
                eprintln!("Unknown audit subcommand. Expected: audit export");
                print_usage();
                ExitCode::from(1)
            }
        }
        "--version" | "-V" => {
            println!("iter-cli {}", VERSION);
            ExitCode::SUCCESS
        }
        "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("Unknown command: {}", other);
            print_usage();
            ExitCode::from(1)
        }
    }
}

fn print_usage() {
    eprintln!(
        r#"iter-cli {} - Governance replay and audit tooling

USAGE:
    iter-cli <COMMAND> [OPTIONS]

COMMANDS:
    replay                     Replay a DecisionPacket and verify correctness
    audit export               Export and validate a DecisionPacket file

REPLAY OPTIONS:
    --decision-file <PATH>     Path to DecisionPacket JSON file
    --policy-version <VER>     Expected policy version (sha256:<hash>)
    --schema-version <VER>     Expected schema version (decision_packet:v1)

AUDIT EXPORT OPTIONS:
    --decision-file <PATH>     Path to DecisionPacket JSON file
    --output <PATH>            Output path for canonical JSON export

GENERAL OPTIONS:
    -V, --version              Print version
    -h, --help                 Print help

EXIT CODES:
    0    Success (VERIFIED / EXPORTED)
    1    Input error (file missing, malformed JSON, missing flags)
    2    Replay mismatch or integrity failure
    3    Internal error"#,
        VERSION
    );
}

fn get_flag(args: &[String], name: &str) -> Option<String> {
    for (i, arg) in args.iter().enumerate() {
        if arg == name {
            return args.get(i + 1).cloned();
        }
    }
    None
}

fn read_decision_packet(
    path: &str,
) -> Result<iter_mcp_server::audit::DecisionPacket, (String, ExitCode)> {
    let contents = fs::read_to_string(path).map_err(|e| {
        (
            format!("Failed to read file '{}': {}", path, e),
            ExitCode::from(1),
        )
    })?;

    serde_json::from_str(&contents).map_err(|e| {
        (
            format!("Failed to parse DecisionPacket JSON: {}", e),
            ExitCode::from(1),
        )
    })
}

fn cmd_replay(args: &[String]) -> ExitCode {
    let decision_file = match get_flag(args, "--decision-file") {
        Some(f) => f,
        None => {
            eprintln!("Error: --decision-file is required");
            return ExitCode::from(1);
        }
    };
    let policy_version = match get_flag(args, "--policy-version") {
        Some(v) => v,
        None => {
            eprintln!("Error: --policy-version is required");
            return ExitCode::from(1);
        }
    };
    let schema_version = match get_flag(args, "--schema-version") {
        Some(v) => v,
        None => {
            eprintln!("Error: --schema-version is required");
            return ExitCode::from(1);
        }
    };

    let packet = match read_decision_packet(&decision_file) {
        Ok(p) => p,
        Err((msg, code)) => {
            eprintln!("{}", msg);
            return code;
        }
    };

    match iter_mcp_server::runtime::replay_decision(&packet, &policy_version, &schema_version) {
        Ok(outcome) => {
            let output = serde_json::json!({
                "outcome": "VERIFIED",
                "decision": outcome.verdict,
                "checksum_match": true,
                "policy_version": policy_version,
                "schema_version": schema_version
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&output).unwrap_or_default()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            let output = serde_json::json!({
                "outcome": "MISMATCH",
                "reason": e.to_string(),
                "policy_version": policy_version,
                "schema_version": schema_version
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&output).unwrap_or_default()
            );
            ExitCode::from(2)
        }
    }
}

fn cmd_audit_export(args: &[String]) -> ExitCode {
    let decision_file = match get_flag(args, "--decision-file") {
        Some(f) => f,
        None => {
            eprintln!("Error: --decision-file is required");
            return ExitCode::from(1);
        }
    };
    let output_path = match get_flag(args, "--output") {
        Some(f) => f,
        None => {
            eprintln!("Error: --output is required");
            return ExitCode::from(1);
        }
    };

    let packet = match read_decision_packet(&decision_file) {
        Ok(p) => p,
        Err((msg, code)) => {
            eprintln!("{}", msg);
            return code;
        }
    };

    if let Err(e) = packet.verify_checksum() {
        let output = serde_json::json!({
            "status": "INTEGRITY_FAILURE",
            "reason": e.to_string()
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
        return ExitCode::from(2);
    }

    let canonical = packet.export();
    if let Err(e) = fs::write(&output_path, &canonical) {
        eprintln!("Failed to write output file '{}': {}", output_path, e);
        return ExitCode::from(1);
    }

    let output = serde_json::json!({
        "status": "EXPORTED",
        "decision_id": packet.checksum,
        "output_file": output_path
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).unwrap_or_default()
    );
    ExitCode::SUCCESS
}
