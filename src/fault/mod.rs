/// SCG Substrate: Fault Domain Infrastructure
/// 
/// Provides deterministic error handling and recovery:
/// - Rollback-to-last-stable-state
/// - Quarantine mode for catastrophic failures
/// - Immutable fault traces for audit
/// - Governor correction cycle logging

pub mod rollback;
pub mod quarantine;
pub mod governor_correction;

pub use rollback::{
    Checkpoint, 
    CheckpointNodeState, 
    CheckpointEdgeState,
    RollbackResult,
    create_checkpoint,
    rollback_to_checkpoint,
    export_checkpoint_json,
    import_checkpoint_json,
};

pub use quarantine::{
    QuarantineReason,
    QuarantineState,
    QuarantineController,
    error_codes,
};

pub use governor_correction::{
    CorrectionStatus,
    CorrectionAttempt,
    GovernorCorrectionLogger,
};
