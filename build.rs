//! Build script to validate substrate availability in full mode.

fn main() {
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
