pub mod replay_episode;
pub mod shard;
/// SCG Substrate: Lineage Management Infrastructure
///
/// Provides immutable, cryptographically verified lineage tracking:
/// - Snapshot creation and export
/// - Deterministic replay validation
/// - Hash-anchored audit trails
/// - Shard boundary management
/// - Replay episode protocol
pub mod snapshot;

pub use snapshot::LineageEntry;
