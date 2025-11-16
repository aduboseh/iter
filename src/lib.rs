/// SCG MCP Server - Library Interface
/// 
/// Exposes public modules for testing and external integration.

pub mod types;
pub mod scg_core;
pub mod mcp_handler;
pub mod fault;
pub mod telemetry;
pub mod lineage;

// Re-export commonly used types
pub use types::*;
pub use scg_core::ScgRuntime;
