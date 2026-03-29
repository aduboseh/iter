use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("SCG endpoint unreachable: {0}")]
    Unreachable(String),

    #[error("Governance hash missing or malformed: {0}")]
    GovernanceHashInvalid(String),

    #[error("Contract version mismatch: expected {expected}, got {got}")]
    ContractVersionMismatch { expected: String, got: String },

    #[error("SCG response schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("Execution trace is nondeterministic: {0}")]
    TraceDeterminismViolation(String),

    #[error("Replay ID verification failed: {0}")]
    ReplayIntegrityViolation(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}
