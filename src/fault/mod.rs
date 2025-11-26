pub mod governor_correction;
pub mod quarantine;
/// SCG Substrate: Fault Domain Infrastructure
///
/// Provides deterministic error handling and recovery:
/// - Rollback-to-last-stable-state
/// - Quarantine mode for catastrophic failures
/// - Immutable fault traces for audit
/// - Governor correction cycle logging
pub mod rollback;

pub use quarantine::{QuarantineController, QuarantineReason};
