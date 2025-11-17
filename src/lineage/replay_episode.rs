/// SCG Substrate: Replay Episode Protocol
/// 
/// Ensures deterministic 250-cycle reasoning episodes can be:
/// - Identified uniquely (by seed, scenario, or explicit log reference)
/// - Re-run deterministically across all three environments
/// - Validated to use identical tool/node sequences
/// 
/// Replay variance tolerance: epsilon <= 1e-10

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEpisode {
    pub episode_id: String,
    pub seed: u64,
    pub scenario: String,
    pub cycle_count: usize,
    pub environments: Vec<EnvironmentRecord>,
    pub variance: f64,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentRecord {
    pub name: String,
    pub config: String,
    pub os: String,
    pub hash_ref: String,
    pub image_id: Option<String>,
    pub cluster: Option<String>,
}

impl ReplayEpisode {
    pub fn new(episode_id: impl Into<String>, seed: u64, scenario: impl Into<String>) -> Self {
        Self {
            episode_id: episode_id.into(),
            seed,
            scenario: scenario.into(),
            cycle_count: 250,
            environments: Vec::new(),
            variance: 0.0,
            passed: false,
        }
    }
    
    /// Adds an environment execution record
    pub fn add_environment(
        &mut self,
        name: impl Into<String>,
        config: impl Into<String>,
        os: impl Into<String>,
        hash_ref: impl Into<String>,
    ) {
        self.environments.push(EnvironmentRecord {
            name: name.into(),
            config: config.into(),
            os: os.into(),
            hash_ref: hash_ref.into(),
            image_id: None,
            cluster: None,
        });
    }
    
    /// Validates that all environments produced identical hashes
    pub fn validate(&mut self) -> Result<(), String> {
        if self.environments.is_empty() {
            return Err("No environment records to validate".into());
        }
        
        if self.environments.len() < 3 {
            return Err("Replay protocol requires 3 environments (local, docker, kubernetes)".into());
        }
        
        let reference_hash = &self.environments[0].hash_ref;
        
        for (i, env) in self.environments.iter().enumerate().skip(1) {
            if env.hash_ref != *reference_hash {
                self.variance = 1.0; // Non-zero variance indicates hash mismatch
                self.passed = false;
                return Err(format!(
                    "Hash mismatch: env[0] = {}, env[{}] = {}",
                    reference_hash,
                    i,
                    env.hash_ref
                ));
            }
        }
        
        self.variance = 0.0;
        self.passed = true;
        
        Ok(())
    }
    
    /// Exports episode to JSON for audit
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
    
    /// Imports episode from JSON
    pub fn import_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Manages replay episode execution and validation
pub struct ReplayProtocol {
    episodes: parking_lot::Mutex<Vec<ReplayEpisode>>,
}

impl ReplayProtocol {
    pub fn new() -> Self {
        Self {
            episodes: parking_lot::Mutex::new(Vec::new()),
        }
    }
    
    /// Creates a new replay episode
    pub fn create_episode(
        &self,
        episode_id: impl Into<String>,
        seed: u64,
        scenario: impl Into<String>,
    ) -> ReplayEpisode {
        let episode = ReplayEpisode::new(episode_id, seed, scenario);
        self.episodes.lock().push(episode.clone());
        episode
    }
    
    /// Updates an episode with new environment record
    pub fn update_episode(
        &self,
        episode_id: &str,
        name: impl Into<String>,
        config: impl Into<String>,
        os: impl Into<String>,
        hash_ref: impl Into<String>,
    ) -> Result<(), String> {
        let mut episodes = self.episodes.lock();
        
        let episode = episodes.iter_mut()
            .find(|e| e.episode_id == episode_id)
            .ok_or("Episode not found")?;
        
        episode.add_environment(name, config, os, hash_ref);
        
        Ok(())
    }
    
    /// Validates an episode (requires 3 environments with matching hashes)
    pub fn validate_episode(&self, episode_id: &str) -> Result<bool, String> {
        let mut episodes = self.episodes.lock();
        
        let episode = episodes.iter_mut()
            .find(|e| e.episode_id == episode_id)
            .ok_or("Episode not found")?;
        
        episode.validate()?;
        
        Ok(episode.passed)
    }
    
    /// Returns all episodes
    pub fn get_episodes(&self) -> Vec<ReplayEpisode> {
        self.episodes.lock().clone()
    }
    
    /// Exports all episodes to JSON for audit
    pub fn export_audit_report(&self) -> Result<String, serde_json::Error> {
        let episodes = self.episodes.lock();
        serde_json::to_string_pretty(&*episodes)
    }
}

impl Default for ReplayProtocol {
    fn default() -> Self {
        Self::new()
    }
}

/// Generates a standard 250-cycle test scenario
pub fn generate_test_scenario(seed: u64) -> Vec<String> {
    let mut operations = Vec::new();
    
    // Use seed for deterministic generation
    let mut rng_state = seed;
    
    for i in 0..250 {
        let op_type = (rng_state % 5) as u8;
        
        let operation = match op_type {
            0 => format!("node.create(belief={}, energy={})", 
                (rng_state % 100) as f64 / 100.0, 
                (rng_state % 10) as f64
            ),
            1 => format!("node.mutate(id=node_{}, delta={})", 
                (rng_state % 10), 
                ((rng_state % 20) as f64 - 10.0) / 100.0
            ),
            2 => format!("edge.bind(src=node_{}, dst=node_{}, weight={})", 
                (rng_state % 10), 
                ((rng_state / 10) % 10), 
                (rng_state % 100) as f64 / 100.0
            ),
            3 => format!("edge.propagate(edge_id=edge_{})", rng_state % 5),
            4 => format!("governor.status()"),
            _ => unreachable!(),
        };
        
        operations.push(operation);
        
        // Simple deterministic RNG update
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
    }
    
    operations
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_episode_creation() {
        let episode = ReplayEpisode::new("REPLAY_001", 42, "tool_chain_inference_5_steps");
        
        assert_eq!(episode.episode_id, "REPLAY_001");
        assert_eq!(episode.seed, 42);
        assert_eq!(episode.cycle_count, 250);
        assert!(!episode.passed);
    }
    
    #[test]
    fn test_episode_validation_success() {
        let mut episode = ReplayEpisode::new("REPLAY_001", 42, "test");
        
        let hash = "abc123def456";
        
        episode.add_environment("local", "config.json", "Linux", hash);
        episode.add_environment("docker", "image:latest", "Linux", hash);
        episode.add_environment("kubernetes", "cluster", "Linux", hash);
        
        assert!(episode.validate().is_ok());
        assert!(episode.passed);
        assert_eq!(episode.variance, 0.0);
    }
    
    #[test]
    fn test_episode_validation_failure() {
        let mut episode = ReplayEpisode::new("REPLAY_001", 42, "test");
        
        episode.add_environment("local", "config.json", "Linux", "hash1");
        episode.add_environment("docker", "image:latest", "Linux", "hash2");
        episode.add_environment("kubernetes", "cluster", "Linux", "hash3");
        
        assert!(episode.validate().is_err());
        assert!(!episode.passed);
        assert_eq!(episode.variance, 1.0);
    }
    
    #[test]
    fn test_scenario_generation_deterministic() {
        let scenario1 = generate_test_scenario(42);
        let scenario2 = generate_test_scenario(42);
        
        assert_eq!(scenario1, scenario2);
        assert_eq!(scenario1.len(), 250);
    }
    
    #[test]
    fn test_replay_protocol() {
        let protocol = ReplayProtocol::new();
        
        let episode = protocol.create_episode("REPLAY_001", 42, "test");
        
        let hash = "abc123";
        
        protocol.update_episode("REPLAY_001", "local", "config.json", "Linux", hash).unwrap();
        protocol.update_episode("REPLAY_001", "docker", "image", "Linux", hash).unwrap();
        protocol.update_episode("REPLAY_001", "k8s", "cluster", "Linux", hash).unwrap();
        
        assert!(protocol.validate_episode("REPLAY_001").unwrap());
        
        let episodes = protocol.get_episodes();
        assert_eq!(episodes.len(), 1);
        assert!(episodes[0].passed);
    }
}
