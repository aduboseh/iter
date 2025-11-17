/// SCG MCP Server - Library Interface
/// 
/// Exposes public modules for testing and external integration.

pub mod types;
pub mod scg_core;
pub mod mcp_handler;
pub mod fault;
pub mod telemetry;
pub mod lineage;

// Connectome v2.0.0-alpha (ISOLATED from substrate)
// Zero coupling enforced by CI (.github/workflows/connectome_audit.yml)
pub mod connectome;
// Re-export commonly used types
pub use types::*;
pub use scg_core::ScgRuntime;
