/// SCG Substrate: Governor Correction Cycle Logger
/// 
/// Logs every correction attempt with full traceability:
/// - Pre-correction drift delta
/// - Attempted correction magnitude
/// - Post-correction drift delta
/// - Convergence status
/// 
/// Follows SCG Space principle: "no silent failures"

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorrectionStatus {
    Success,
    Partial,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionAttempt {
    pub attempt_id: Uuid,
    pub timestamp: String,
    pub correction_status: CorrectionStatus,
    pub pre_delta: f64,
    pub attempted_correction: f64,
    pub post_delta: f64,
    pub converged: bool,
    pub cycle_number: u64,
}

impl CorrectionAttempt {
    pub fn new(
        pre_delta: f64,
        attempted_correction: f64,
        post_delta: f64,
        cycle_number: u64,
    ) -> Self {
        let converged = post_delta.abs() < pre_delta.abs();
        
        let correction_status = if post_delta.abs() <= 1e-10 {
            CorrectionStatus::Success
        } else if converged {
            CorrectionStatus::Partial
        } else {
            CorrectionStatus::Failed
        };
        
        Self {
            attempt_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            correction_status,
            pre_delta,
            attempted_correction,
            post_delta,
            converged,
            cycle_number,
        }
    }
    
    /// Logs the correction attempt to stderr for audit trail
    pub fn log(&self) {
        eprintln!("[GOVERNOR_CORRECTION] {}", serde_json::to_string(self).unwrap_or_default());
    }
}

/// Governor correction cycle manager
pub struct GovernorCorrectionLogger {
    attempts: parking_lot::Mutex<Vec<CorrectionAttempt>>,
    cycle_count: parking_lot::Mutex<u64>,
}

impl GovernorCorrectionLogger {
    pub fn new() -> Self {
        Self {
            attempts: parking_lot::Mutex::new(Vec::new()),
            cycle_count: parking_lot::Mutex::new(0),
        }
    }
    
    /// Logs a correction attempt and returns the attempt record
    pub fn log_attempt(
        &self,
        pre_delta: f64,
        attempted_correction: f64,
        post_delta: f64,
    ) -> CorrectionAttempt {
        let mut cycle_count = self.cycle_count.lock();
        *cycle_count += 1;
        
        let attempt = CorrectionAttempt::new(
            pre_delta,
            attempted_correction,
            post_delta,
            *cycle_count,
        );
        
        attempt.log();
        
        self.attempts.lock().push(attempt.clone());
        
        attempt
    }
    
    /// Returns all correction attempts for audit
    pub fn get_attempts(&self) -> Vec<CorrectionAttempt> {
        self.attempts.lock().clone()
    }
    
    /// Returns correction success rate
    pub fn success_rate(&self) -> f64 {
        let attempts = self.attempts.lock();
        if attempts.is_empty() {
            return 1.0;
        }
        
        let successful = attempts.iter()
            .filter(|a| matches!(a.correction_status, CorrectionStatus::Success))
            .count();
        
        successful as f64 / attempts.len() as f64
    }
    
    /// Exports correction log to JSON for audit
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let attempts = self.attempts.lock();
        serde_json::to_string_pretty(&*attempts)
    }
}

impl Default for GovernorCorrectionLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_correction_attempt_success() {
        let attempt = CorrectionAttempt::new(1e-9, -9e-10, 5e-11, 1);
        
        assert!(attempt.converged);
        assert!(matches!(attempt.correction_status, CorrectionStatus::Success));
    }
    
    #[test]
    fn test_correction_attempt_partial() {
        let attempt = CorrectionAttempt::new(1e-9, -5e-10, 5e-10, 1);
        
        assert!(attempt.converged);
        assert!(matches!(attempt.correction_status, CorrectionStatus::Partial));
    }
    
    #[test]
    fn test_correction_attempt_failed() {
        let attempt = CorrectionAttempt::new(1e-9, 5e-10, 1.5e-9, 1);
        
        assert!(!attempt.converged);
        assert!(matches!(attempt.correction_status, CorrectionStatus::Failed));
    }
    
    #[test]
    fn test_correction_logger() {
        let logger = GovernorCorrectionLogger::new();
        
        logger.log_attempt(1e-9, -9e-10, 5e-11);
        logger.log_attempt(5e-11, -4e-11, 1e-11);
        
        assert_eq!(logger.get_attempts().len(), 2);
        assert_eq!(logger.success_rate(), 1.0);
    }
}
