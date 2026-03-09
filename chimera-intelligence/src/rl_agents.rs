//! Reinforcement Learning Agents Module
//!
//! Implements RL algorithms for adaptive mining optimization.
//! Supports PPO, DQN, A2C, DDPG, FQL, and hybrid variants from the Algorithm Compendium.

use chimera_core::primitives::{OpCost, NodeId, Hash};
use chimera_core::transforms::Transform;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use rand::Rng;
use ndarray::{Array1, Array2};

#[derive(Error, Debug)]
pub enum RLError {
    #[error("Training failed: {0}")]
    TrainingFailed(String),
    #[error("Inference error: {0}")]
    InferenceError(String),
    #[error("Invalid state space: {0}")]
    InvalidStateSpace(String),
    #[error("Invalid action space: {0}")]
    InvalidActionSpace(String),
    #[error("Model not trained: {0}")]
    ModelNotTrained(String),
    #[error("Reward computation failed: {0}")]
    RewardComputationFailed(String),
}

/// Configuration for RL agent training and inference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLConfig {
    pub algorithm: RLAlgorithm,
    pub learning_rate: f64,
    pub discount_factor: f64,      // Gamma (γ)
    pub exploration_rate: f64,      // Epsilon (ε) for ε-greedy
    pub batch_size: usize,
    pub replay_buffer_size: usize,
    pub target_update_freq: u32,
    pub max_episodes: u32,
    pub enable_transfer_learning: bool,
}

impl Default for RLConfig {
    fn default() -> Self {
        Self {
            algorithm: RLAlgorithm::PPO,
            learning_rate: 0.0003,
            discount_factor: 0.99,
            exploration_rate: 0.1,
            batch_size: 64,
            replay_buffer_size: 10000,
            target_update_freq: 100,
            max_episodes: 1000,
            enable_transfer_learning: false,
        }
    }
}

/// Available RL algorithms from the Compendium.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RLAlgorithm {
    PPO,       // Proximal Policy Optimization
    DQN,       // Deep Q-Network
    A2C,       // Advantage Actor-Critic
    DDPG,      // Deep Deterministic Policy Gradient
    FQL,       // Fuzzy Q-Learning
    GAPPO,     // Genetic Algorithm + PPO
    TLM,       // Transfer Learning Model
}

/// Represents a state observation from the environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLState {
    pub hashrate: f64,           // Current hashrate (H/s)
    pub power_draw: f64,         // Current power consumption (W)
    pub thermal_state: f64,      // Thermal state (0.0 - 1.0)
    pub memory_usage: f64,       // Memory usage percentage (0.0 - 1.0)
    pub opcost: OpCost,          // Operational cost metrics
    pub node_id: NodeId,         // Source node identifier
    pub timestamp: u64,          // Unix timestamp
}

impl RLState {
    pub fn to_array(&self) -> Array1<f64> {
        Array1::from_vec(vec![
            self.hashrate,
            self.power_draw,
            self.thermal_state,
            self.memory_usage,
            self.opcost.joules,
            self.opcost.seconds,
            self.opcost.dollars,
        ])
    }

    pub fn from_array(array: &Array1<f64>, node_id: NodeId) -> Result<Self, RLError> {
        if array.len() < 7 {
            return Err(RLError::InvalidStateSpace(
                "State array must have at least 7 elements".to_string()
            ));
        }

        Ok(Self {
            hashrate: array[0],
            power_draw: array[1],
            thermal_state: array[2],
            memory_usage: array[3],
            opcost: OpCost {
                joules: array[4],
                seconds: array[5],
                dollars: array[6],
            },
            node_id,
            timestamp: 0,
        })
    }
}

/// Represents an action taken by the RL agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLAction {
    pub action_type: ActionType,
    pub parameters: Vec<f64>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionType {
    AdjustThreadCount,
    AdjustMemoryLimit,
    AdjustPowerLimit,
    SwitchAlgorithm,
    MigrateTask,
    PauseMining,
    ResumeMining,
}

impl RLAction {
    pub fn to_array(&self, action_space_size: usize) -> Array1<f64> {
        let mut array = Array1::zeros(action_space_size);
        let action_idx = self.action_type as usize % action_space_size;
        array[action_idx] = 1.0;
        array
    }
}

/// Experience tuple for replay buffer.
#[derive(Debug, Clone)]
pub struct Experience {
    pub state: RLState,
    pub action: RLAction,
    pub reward: f64,
    pub next_state: RLState,
    pub done: bool,
}

/// Replay buffer for off-policy RL algorithms (DQN, DDPG).
pub struct ReplayBuffer {
    experiences: Vec<Experience>,
    max_size: usize,
}

impl ReplayBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            experiences: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn push(&mut self, experience: Experience) {
        if self.experiences.len() >= self.max_size {
            self.experiences.remove(0);
        }
        self.experiences.push(experience);
    }

    pub fn sample(&self, batch_size: usize) -> Vec<Experience> {
        let mut rng = rand::thread_rng();
        let mut samples = Vec::with_capacity(batch_size);
        
        for _ in 0..batch_size.min(self.experiences.len()) {
            let idx = rng.gen_range(0..self.experiences.len());
            samples.push(self.experiences[idx].clone());
        }
        
        samples
    }

    pub fn len(&self) -> usize {
        self.experiences.len()
    }

    pub fn is_empty(&self) -> bool {
        self.experiences.is_empty()
    }
}

/// Central RL agent for mining optimization.
/// Referenced by `chimera-intelligence` for adaptive strategy selection.
pub struct RLAgent {
    config: RLConfig,
    policy_network: Option<Array2<f64>>,      // Simplified neural network representation
    value_network: Option<Array2<f64>>,
    target_network: Option<Array2<f64>>,
    replay_buffer: ReplayBuffer,
    episode_rewards: Vec<f64>,
    training_steps: u32,
    state_space_size: usize,
    action_space_size: usize,
}

impl RLAgent {
    pub fn new(
        config: RLConfig,
        state_space_size: usize,
        action_space_size: usize,
    ) -> Result<Self, RLError> {
        if state_space_size == 0 {
            return Err(RLError::InvalidStateSpace(
                "State space size must be > 0".to_string()
            ));
        }
        if action_space_size == 0 {
            return Err(RLError::InvalidActionSpace(
                "Action space size must be > 0".to_string()
            ));
        }

        let replay_buffer = ReplayBuffer::new(config.replay_buffer_size);

        Ok(Self {
            config,
            policy_network: None,
            value_network: None,
            target_network: None,
            replay_buffer,
            episode_rewards: Vec::new(),
            training_steps: 0,
            state_space_size,
            action_space_size,
        })
    }

    /// Initialize neural network weights.
    fn initialize_network(&mut self) {
        let mut rng = rand::thread_rng();
        
        // Policy network: state_space -> action_space
        self.policy_network = Some(Array2::from_shape_fn(
            (self.state_space_size, self.action_space_size),
            |_| rng.gen_range(-0.1..0.1),
        ));

        // Value network: state_space -> 1 (value estimation)
        self.value_network = Some(Array2::from_shape_fn(
            (self.state_space_size, 1),
            |_| rng.gen_range(-0.1..0.1),
        ));

        // Target network (copy of policy for stable training)
        self.target_network = self.policy_network.clone();
    }

    /// Select action based on current policy (with exploration).
    pub async fn select_action(&self, state: &RLState) -> Result<RLAction, RLError> {
        if self.policy_network.is_none() {
            return Err(RLError::ModelNotTrained(
                "Policy network not initialized".to_string()
            ));
        }

        let mut rng = rand::thread_rng();
        
        // Epsilon-greedy exploration
        if rng.gen_range(0.0..1.0) < self.config.exploration_rate {
            // Random action for exploration
            let action_idx = rng.gen_range(0..self.action_space_size);
            return Ok(RLAction {
                action_type: match action_idx {
                    0 => ActionType::AdjustThreadCount,
                    1 => ActionType::AdjustMemoryLimit,
                    2 => ActionType::AdjustPowerLimit,
                    3 => ActionType::SwitchAlgorithm,
                    4 => ActionType::MigrateTask,
                    5 => ActionType::PauseMining,
                    _ => ActionType::ResumeMining,
                },
                parameters: vec![rng.gen_range(0.0..1.0)],
                confidence: 0.0,
            });
        }

        // Exploitation: use policy network
        let policy = self.policy_network.as_ref().unwrap();
        let state_vec = state.to_array();
        
        // Simple linear policy: action_probs = state * policy
        let mut action_probs = Array1::zeros(self.action_space_size);
        for a in 0..self.action_space_size {
            for s in 0..self.state_space_size.min(state_vec.len()) {
                action_probs[a] += state_vec[s] * policy[[s, a]];
            }
        }

        // Select best action
        let best_action_idx = action_probs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);

        let confidence = action_probs[best_action_idx].max(0.0).min(1.0);

        Ok(RLAction {
            action_type: match best_action_idx {
                0 => ActionType::AdjustThreadCount,
                1 => ActionType::AdjustMemoryLimit,
                2 => ActionType::AdjustPowerLimit,
                3 => ActionType::SwitchAlgorithm,
                4 => ActionType::MigrateTask,
                5 => ActionType::PauseMining,
                _ => ActionType::ResumeMining,
            },
            parameters: vec![confidence],
            confidence,
        })
    }

    /// Train the agent using collected experiences.
    pub async fn train(&mut self, experiences: Vec<Experience>) -> Result<TrainingMetrics, RLError> {
        if self.policy_network.is_none() {
            self.initialize_network();
        }

        let mut total_loss = 0.0;
        let mut policy_updates = 0;

        match self.config.algorithm {
            RLAlgorithm::PPO => self.train_ppo(&experiences, &mut total_loss, &mut policy_updates)?,
            RLAlgorithm::DQN => self.train_dqn(&experiences, &mut total_loss, &mut policy_updates)?,
            RLAlgorithm::A2C => self.train_a2c(&experiences, &mut total_loss, &mut policy_updates)?,
            RLAlgorithm::DDPG => self.train_ddpg(&experiences, &mut total_loss, &mut policy_updates)?,
            RLAlgorithm::FQL => self.train_fql(&experiences, &mut total_loss, &mut policy_updates)?,
            RLAlgorithm::GAPPO => self.train_gappo(&experiences, &mut total_loss, &mut policy_updates)?,
            RLAlgorithm::TLM => self.train_tlm(&experiences, &mut total_loss, &mut policy_updates)?,
        }

        self.training_steps += 1;

        // Update target network periodically
        if self.training_steps % self.config.target_update_freq == 0 {
            self.target_network = self.policy_network.clone();
        }

        Ok(TrainingMetrics {
            average_loss: total_loss / policy_updates.max(1) as f64,
            policy_updates,
            training_steps: self.training_steps,
            episode_count: self.episode_rewards.len() as u32,
        })
    }

    /// Proximal Policy Optimization (PPO) training.
    fn train_ppo(
        &mut self,
        experiences: &[Experience],
        total_loss: &mut f64,
        policy_updates: &mut u32,
    ) -> Result<(), RLError> {
        // PPO clip objective: L^CLIP = E[min(rA, clip(r, 1±ε)A)]
        let epsilon = 0.2; // Clip parameter
        
        for exp in experiences {
            let reward = exp.reward;
            let state_vec = exp.state.to_array();
            
            // Compute advantage estimate (simplified)
            let advantage = reward - self.estimate_value(&state_vec)?;
            
            // Compute probability ratio
            let ratio = self.compute_probability_ratio(&state_vec, &exp.action)?;
            
            // PPO clipped objective
            let clipped_ratio = ratio.clamp(1.0 - epsilon, 1.0 + epsilon);
            let objective = advantage * ratio.min(clipped_ratio);
            
            // Update policy network
            self.update_policy(&state_vec, &exp.action, objective)?;
            
            *total_loss += objective.abs();
            *policy_updates += 1;
        }

        Ok(())
    }

    /// Deep Q-Network (DQN) training.
    fn train_dqn(
        &mut self,
        experiences: &[Experience],
        total_loss: &mut f64,
        policy_updates: &mut u32,
    ) -> Result<(), RLError> {
        // DQN: Q_θ ≈ Q*
        
        for exp in experiences {
            let state_vec = exp.state.to_array();
            let next_state_vec = exp.next_state.to_array();
            
            // Compute target Q-value
            let current_q = self.estimate_q(&state_vec, &exp.action)?;
            let next_q = if !exp.done {
                self.estimate_max_q(&next_state_vec)?
            } else {
                0.0
            };
            
            let target_q = exp.reward + self.config.discount_factor * next_q;
            let td_error = target_q - current_q;
            
            // Update Q-network
            self.update_q_network(&state_vec, &exp.action, td_error)?;
            
            *total_loss += td_error.powi(2);
            *policy_updates += 1;
        }

        Ok(())
    }

    /// Advantage Actor-Critic (A2C) training.
    fn train_a2c(
        &mut self,
        experiences: &[Experience],
        total_loss: &mut f64,
        policy_updates: &mut u32,
    ) -> Result<(), RLError> {
        // A2C: ∇J = E[∇log π A]
        
        for exp in experiences {
            let state_vec = exp.state.to_array();
            let reward = exp.reward;
            
            // Compute advantage
            let value = self.estimate_value(&state_vec)?;
            let advantage = reward - value;
            
            // Update policy using advantage
            self.update_policy_advantage(&state_vec, &exp.action, advantage)?;
            
            // Update value function
            self.update_value_function(&state_vec, reward)?;
            
            *total_loss += advantage.abs();
            *policy_updates += 1;
        }

        Ok(())
    }

    /// Deep Deterministic Policy Gradient (DDPG) training.
    fn train_ddpg(
        &mut self,
        experiences: &[Experience],
        total_loss: &mut f64,
        policy_updates: &mut u32,
    ) -> Result<(), RLError> {
        // DDPG: Actor-critic with continuous action space
        
        for exp in experiences {
            let state_vec = exp.state.to_array();
            let next_state_vec = exp.next_state.to_array();
            
            // Compute target
            let next_action = self.select_action(&exp.next_state).await?;
            let next_q = self.estimate_q(&next_state_vec, &next_action)?;
            let target_q = exp.reward + self.config.discount_factor * next_q;
            
            // Update critic
            let current_q = self.estimate_q(&state_vec, &exp.action)?;
            let critic_loss = (target_q - current_q).powi(2);
            
            // Update actor (policy)
            self.update_policy(&state_vec, &exp.action, target_q)?;
            
            *total_loss += critic_loss;
            *policy_updates += 1;
        }

        Ok(())
    }

    /// Fuzzy Q-Learning (FQL) training.
    fn train_fql(
        &mut self,
        experiences: &[Experience],
        total_loss: &mut f64,
        policy_updates: &mut u32,
    ) -> Result<(), RLError> {
        // FQL: π = Flow(z) with fuzzy rules
        
        for exp in experiences {
            let state_vec = exp.state.to_array();
            
            // Fuzzy state discretization
            let fuzzy_state = self.fuzzify_state(&state_vec)?;
            
            // Compute Q-value with fuzzy rules
            let q_value = self.estimate_fuzzy_q(&fuzzy_state, &exp.action)?;
            
            // Update fuzzy Q-table
            let target_q = exp.reward + self.config.discount_factor * q_value;
            let td_error = target_q - q_value;
            
            self.update_fuzzy_q(&fuzzy_state, &exp.action, td_error)?;
            
            *total_loss += td_error.abs();
            *policy_updates += 1;
        }

        Ok(())
    }

    /// Genetic Algorithm + PPO (GAPPO) training.
    fn train_gappo(
        &mut self,
        experiences: &[Experience],
        total_loss: &mut f64,
        policy_updates: &mut u32,
    ) -> Result<(), RLError> {
        // GAPPO: Combine genetic algorithm evolution with PPO
        
        // Phase 1: Genetic selection of best policies
        let best_experiences = self.select_best_experiences(experiences)?;
        
        // Phase 2: PPO update on selected experiences
        self.train_ppo(&best_experiences, total_loss, policy_updates)?;
        
        Ok(())
    }

    /// Transfer Learning Model (TLM) training.
    fn train_tlm(
        &mut self,
        experiences: &[Experience],
        total_loss: &mut f64,
        policy_updates: &mut u32,
    ) -> Result<(), RLError> {
        // TLM: Leverage pre-trained knowledge for faster convergence
        
        if self.config.enable_transfer_learning {
            // Load pre-trained weights (simplified)
            self.apply_transfer_weights()?;
        }
        
        // Standard PPO training with transfer initialization
        self.train_ppo(experiences, total_loss, policy_updates)?;
        
        Ok(())
    }

    /// Store experience in replay buffer.
    pub fn store_experience(&mut self, experience: Experience) {
        self.replay_buffer.push(experience);
    }

    /// Train from replay buffer.
    pub async fn train_from_buffer(&mut self) -> Result<TrainingMetrics, RLError> {
        let batch_size = self.config.batch_size.min(self.replay_buffer.len());
        let experiences = self.replay_buffer.sample(batch_size);
        self.train(experiences).await
    }

    /// Record episode reward.
    pub fn record_episode_reward(&mut self, reward: f64) {
        self.episode_rewards.push(reward);
        
        // Keep only last 100 episodes for memory efficiency
        if self.episode_rewards.len() > 100 {
            self.episode_rewards.remove(0);
        }
    }

    /// Get training statistics.
    pub fn get_training_stats(&self) -> RLTrainingStats {
        let avg_reward = if self.episode_rewards.is_empty() {
            0.0
        } else {
            self.episode_rewards.iter().sum::<f64>() / self.episode_rewards.len() as f64
        };

        RLTrainingStats {
            training_steps: self.training_steps,
            episode_count: self.episode_rewards.len() as u32,
            average_reward: avg_reward,
            best_reward: self.episode_rewards.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            replay_buffer_size: self.replay_buffer.len(),
            algorithm: self.config.algorithm,
        }
    }

    // Helper methods (simplified implementations)
    fn estimate_value(&self, state: &Array1<f64>) -> Result<f64, RLError> {
        // Simplified value estimation
        Ok(state.iter().sum::<f64>() * 0.1)
    }

    fn estimate_q(&self, state: &Array1<f64>, action: &RLAction) -> Result<f64, RLError> {
        // Simplified Q-value estimation
        Ok(state.iter().sum::<f64>() * 0.1 + action.confidence)
    }

    fn estimate_max_q(&self, state: &Array1<f64>) -> Result<f64, RLError> {
        self.estimate_value(state)
    }

    fn compute_probability_ratio(
        &self,
        state: &Array1<f64>,
        action: &RLAction,
    ) -> Result<f64, RLError> {
        // Simplified probability ratio
        Ok(1.0 + action.confidence * 0.1