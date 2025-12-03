pub mod fault;
pub mod governance;
pub mod lineage;
pub mod mcp_handler;
pub mod scg_core;
pub mod services;
pub mod telemetry;
/// SCG MCP Server - Library Interface
///
/// Exposes public modules for testing and external integration.
pub mod types;

// Connectome v2.0.0-alpha (ISOLATED from substrate)
// Imported from SCG monorepo as first-class library module.
// Zero coupling enforced by CI in SCG repo (.github/workflows/connectome_audit.yml)
pub use scg_connectome as connectome;

// Re-export commonly used types
pub use scg_core::ScgRuntime;
pub use types::*;
