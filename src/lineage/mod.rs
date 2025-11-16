/// SCG Substrate: Lineage Management Infrastructure
/// 
/// Provides immutable, cryptographically verified lineage tracking:
/// - Snapshot creation and export
/// - Deterministic replay validation
/// - Hash-anchored audit trails

pub mod snapshot;

pub use snapshot::{
    LineageSnapshot,
    LineageEntry,
    GraphSnapshot,
    EnergySnapshot,
    SnapshotMetadata,
    LineageBuilder,
};
