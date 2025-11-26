/// SCG Substrate: Telemetry and Observability Infrastructure
///
/// Provides real-time monitoring of:
/// - Energy conservation invariants
/// - ESV validation ratios
/// - Coherence and entropy indices
/// - Lineage event streams
pub mod schema;

pub use schema::TelemetryEmitter;
