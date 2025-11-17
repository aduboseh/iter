/// SCG Substrate: Lineage Management Infrastructure
/// 
/// Provides immutable, cryptographically verified lineage tracking:
/// - Snapshot creation and export
/// - Deterministic replay validation
/// - Hash-anchored audit trails
/// - Shard boundary management
/// - Replay episode protocol

pub mod snapshot;
pub mod shard;
pub mod replay_episode;

pub use snapshot::{
    LineageSnapshot,
    LineageEntry,
    GraphSnapshot,
    EnergySnapshot,
    SnapshotMetadata,
    LineageBuilder,
};

pub use shard::{
    LineageShard,
    ShardManager,
    SHARD_ROTATION_INTERVAL,
};

pub use replay_episode::{
    ReplayEpisode,
    EnvironmentRecord,
    ReplayProtocol,
    generate_test_scenario,
};
