//! Swarm Intelligence Optimization Module
//!
//! Implements metaheuristic algorithms for mining parameter optimization.
//! Supports GWO, WOA, ACO, SSA, and hybrid variants from the Algorithm Compendium.

use chimera_core::primitives::{OpCost, NodeId};
use chimera_core::transforms::Transform;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use rand::Rng;
use ndarray::{Array1, Array2};

#[derive(Error, Debug)]
pub enum SwarmError {
    #[error("Convergence failed after {0} iterations")]
    ConvergenceFailed(u32),
    #[error("Invalid parameter space: {0}")]
    InvalidParameterSpace(String),
    #[error("Optimization dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("Swarm initialization failed: {0}")]
    InitializationFailed(String),
}

/// Configuration for swarm optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    pub population_size: usize,
    pub max_iterations: u32,
    pub convergence_threshold: f64,
    pub algorithm: SwarmAlgorithm,
    pub exploration_weight: f64,
    pub exploitation_weight: f64,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            population_size: 50,
            max_iterations: 1000,
            convergence_threshold: 1e-6,
            algorithm: SwarmAlgorithm::GWO,
            exploration_weight: 0.5,
            exploitation_weight: 0.5,
        }
    }
}

/// Available swarm intelligence algorithms from the Compendium.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwarmAlgorithm {
    GWO,      // Grey Wolf Optimizer
    WOA,      // Whale Optimization Algorithm
    ACO,      // Ant Colony Optimization
    SSA,      // Salp Swarm Algorithm
    ISO,      // Improved Snake Optimization (SO + RIME)
    HHO,      // Harris Hawks Optimization
    Hybrid,   // Adaptive hybrid of multiple algorithms
}

/// Represents a single agent/particle in the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub position: Array1<f64>,
    pub fitness: f64,
    pub velocity: Option<Array1<f64>>,
    pub best_position: Array1<f64>,
    pub best_fitness: f64,
}

impl Agent {
    pub fn new(dimension: usize) -> Self {
        let mut rng = rand::thread_rng();
        let position = Array1::from_vec(
            (0..dimension).map(|_| rng.gen_range(0.0..1.0)).collect()
        );
        
        Self {
            position,
            fitness: f64::INFINITY,
            velocity: None,
            best_position: position.clone(),
            best_fitness: f64::INFINITY,
        }
    }

    pub fn evaluate<F>(&mut self, fitness_fn: &F)
    where
        F: Fn(&Array1<f64>) -> f64,
    {
        self.fitness = fitness_fn(&self.position);
        
        if self.fitness < self.best_fitness {
            self.best_fitness = self.fitness;
            self.best_position = self.position.clone();
        }
    }
}

/// Central swarm optimization engine.
/// Referenced by `chimera-intelligence` for parameter tuning.
pub struct SwarmOptimizer {
    config: SwarmConfig,
    agents: Vec<Agent>,
    global_best_position: Array1<f64>,
    global_best_fitness: f64,
    iteration_history: Vec<f64>,
    dimension: usize,
}

impl SwarmOptimizer {
    pub fn new(config: SwarmConfig, dimension: usize) -> Result<Self, SwarmError> {
        if dimension == 0 {
            return Err(SwarmError::InvalidParameterSpace(
                "Dimension must be > 0".to_string()
            ));
        }

        let mut agents = Vec::with_capacity(config.population_size);
        for _ in 0..config.population_size {
            agents.push(Agent::new(dimension));
        }

        Ok(Self {
            config,
            agents,
            global_best_position: Array1::zeros(dimension),
            global_best_fitness: f64::INFINITY,
            iteration_history: Vec::new(),
            dimension,
        })
    }

    /// Run swarm optimization to find optimal parameters.
    pub async fn optimize<F>(&mut self, fitness_fn: F) -> Result<OptimizationResult, SwarmError>
    where
        F: Fn(&Array1<f64>) -> f64 + Send + Sync + Clone + 'static,
    {
        let fitness_fn = Arc::new(RwLock::new(fitness_fn));
        let mut prev_best = f64::INFINITY;
        let mut stagnation_counter = 0;

        for iteration in 0..self.config.max_iterations {
            // Evaluate all agents
            for agent in &mut self.agents {
                let fitness_fn_clone = Arc::clone(&fitness_fn);
                let fitness = {
                    let fn_lock = fitness_fn_clone.read().await;
                    fn_lock(&agent.position)
                };
                agent.evaluate(&|p| fitness);
                
                // Update global best
                if agent.fitness < self.global_best_fitness {
                    self.global_best_fitness = agent.fitness;
                    self.global_best_position = agent.position.clone();
                }
            }

            // Record iteration history
            self.iteration_history.push(self.global_best_fitness);

            // Check convergence
            let improvement = (prev_best - self.global_best_fitness).abs();
            if improvement < self.config.convergence_threshold {
                stagnation_counter += 1;
                if stagnation_counter > 10 {
                    tracing::info!("Converged after {} iterations", iteration);
                    break;
                }
            } else {
                stagnation_counter = 0;
            }
            prev_best = self.global_best_fitness;

            // Update agent positions based on algorithm
            self.update_positions(iteration, Arc::clone(&fitness_fn)).await?;
        }

        if self.global_best_fitness == f64::INFINITY {
            return Err(SwarmError::ConvergenceFailed(self.config.max_iterations));
        }

        Ok(OptimizationResult {
            optimal_parameters: self.global_best_position.to_vec(),
            best_fitness: self.global_best_fitness,
            iterations: self.iteration_history.len() as u32,
            convergence_history: self.iteration_history.clone(),
            algorithm: self.config.algorithm,
        })
    }

    /// Update agent positions based on selected algorithm.
    async fn update_positions<F>(&mut self, iteration: u32, fitness_fn: Arc<RwLock<F>>)
    where
        F: Fn(&Array1<f64>) -> f64 + Send + Sync,
    {
        match self.config.algorithm {
            SwarmAlgorithm::GWO => self.update_gwo(iteration),
            SwarmAlgorithm::WOA => self.update_woa(iteration),
            SwarmAlgorithm::ACO => self.update_aco(iteration),
            SwarmAlgorithm::SSA => self.update_ssa(iteration),
            SwarmAlgorithm::ISO => self.update_iso(iteration),
            SwarmAlgorithm::HHO => self.update_hho(iteration),
            SwarmAlgorithm::Hybrid => self.update_hybrid(iteration).await,
        }
    }

    /// Grey Wolf Optimizer (GWO) implementation.
    fn update_gwo(&mut self, iteration: u32) {
        // GWO uses alpha, beta, delta wolves to guide search
        let mut sorted_agents: Vec<&mut Agent> = self.agents.iter_mut().collect();
        sorted_agents.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());

        let alpha = &sorted_agents[0];
        let beta = &sorted_agents[1];
        let delta = &sorted_agents[2];

        let a = 2.0 - (iteration as f64 / self.config.max_iterations as f64) * 2.0;

        for agent in &mut self.agents {
            let mut new_position = Array1::zeros(self.dimension);
            
            for i in 0..self.dimension {
                let r1 = rand::thread_rng().gen_range(0.0..1.0);
                let r2 = rand::thread_rng().gen_range(0.0..1.0);
                
                let A = 2.0 * a * r1 - a;
                let C = 2.0 * r2;
                
                let D_alpha = (C * alpha.position[i] - agent.position[i]).abs();
                let D_beta = (C * beta.position[i] - agent.position[i]).abs();
                let D_delta = (C * delta.position[i] - agent.position[i]).abs();
                
                let X1 = alpha.position[i] - A * D_alpha;
                let X2 = beta.position[i] - A * D_beta;
                let X3 = delta.position[i] - A * D_delta;
                
                new_position[i] = (X1 + X2 + X3) / 3.0;
            }
            
            agent.position = new_position;
        }
    }

    /// Whale Optimization Algorithm (WOA) implementation.
    fn update_woa(&mut self, iteration: u32) {
        let best_agent = self.agents
            .iter()
            .min_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
            .unwrap();
        
        let best_position = best_agent.position.clone();
        let a = 2.0 - (iteration as f64 / self.config.max_iterations as f64) * 2.0;

        for agent in &mut self.agents {
            let r = rand::thread_rng().gen_range(0.0..1.0);
            
            if r < 0.5 {
                // Encircling prey
                for i in 0..self.dimension {
                    let A = 2.0 * a * rand::thread_rng().gen_range(0.0..1.0) - a;
                    let C = 2.0 * rand::thread_rng().gen_range(0.0..1.0);
                    let D = (C * best_position[i] - agent.position[i]).abs();
                    agent.position[i] = best_position[i] - A * D;
                }
            } else {
                // Spiral updating position
                for i in 0..self.dimension {
                    let l = rand::thread_rng().gen_range(-1.0..1.0);
                    let b = 1.0;
                    let D = (best_position[i] - agent.position[i]).abs();
                    agent.position[i] = D * (2.718f64.powf(b * l)).cos() + best_position[i];
                }
            }
        }
    }

    /// Ant Colony Optimization (ACO) implementation.
    fn update_aco(&mut self, iteration: u32) {
        // Simplified ACO for continuous optimization
        let pheromone_decay = 0.9;
        let best_agent = self.agents
            .iter()
            .min_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
            .unwrap();

        for agent in &mut self.agents {
            for i in 0..self.dimension {
                let r = rand::thread_rng().gen_range(0.0..1.0);
                if r < 0.3 {
                    // Follow best path with probability
                    agent.position[i] = best_agent.position[i] + 
                        rand::thread_rng().gen_range(-0.1..0.1);
                } else {
                    // Explore
                    agent.position[i] += rand::thread_rng().gen_range(-0.5..0.5);
                }
                
                // Clamp to valid range
                agent.position[i] = agent.position[i].clamp(0.0, 1.0);
            }
        }
    }

    /// Salp Swarm Algorithm (SSA) implementation.
    fn update_ssa(&mut self, iteration: u32) {
        let best_agent = self.agents
            .iter()
            .min_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
            .unwrap();
        
        let c1 = 2.0 * (-4.0 * iteration as f64 / self.config.max_iterations as f64).exp();

        for (i, agent) in self.agents.iter_mut().enumerate() {
            if i == 0 {
                // Leader update
                for j in 0..self.dimension {
                    let c2 = rand::thread_rng().gen_range(0.0..1.0);
                    let c3 = rand::thread_rng().gen_range(0.0..1.0);
                    
                    if c3 < 0.5 {
                        agent.position[j] = best_agent.position[j] + 
                            c1 * ((c2 * best_agent.position[j]).abs());
                    } else {
                        agent.position[j] = best_agent.position[j] - 
                            c1 * ((c2 * best_agent.position[j]).abs());
                    }
                }
            } else {
                // Follower update
                for j in 0..self.dimension {
                    agent.position[j] = (agent.position[j] + 
                        self.agents[i - 1].position[j]) / 2.0;
                }
            }
        }
    }

    /// Improved Snake Optimization (ISO) implementation.
    fn update_iso(&mut self, iteration: u32) {
        // ISO combines SO with RIME and escape mechanism
        let temperature = 1.0 - (iteration as f64 / self.config.max_iterations as f64);
        
        for agent in &mut self.agents {
            if temperature > 0.6 {
                // Exploration phase (snake searching for food)
                for i in 0..self.dimension {
                    agent.position[i] += rand::thread_rng().gen_range(-0.5..0.5) * temperature;
                }
            } else {
                // Exploitation phase (snake fighting or mating)
                let fight_or_mate = rand::thread_rng().gen_range(0.0..1.0);
                if fight_or_mate < 0.5 {
                    // Fighting mode
                    let best_agent = self.agents
                        .iter()
                        .min_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
                        .unwrap();
                    for i in 0..self.dimension {
                        agent.position[i] = best_agent.position[i] + 
                            rand::thread_rng().gen_range(-0.1..0.1);
                    }
                } else {
                    // Mating mode
                    let random_agent = &self.agents[rand::thread_rng().gen_range(0..self.agents.len())];
                    for i in 0..self.dimension {
                        agent.position[i] = (agent.position[i] + random_agent.position[i]) / 2.0;
                    }
                }
            }
            
            // Escape mechanism (RIME-inspired)
            if rand::thread_rng().gen_range(0.0..1.0) < 0.1 {
                for i in 0..self.dimension {
                    agent.position[i] = rand::thread_rng().gen_range(0.0..1.0);
                }
            }
        }
    }

    /// Harris Hawks Optimization (HHO) implementation.
    fn update_hho(&mut self, iteration: u32) {
        let best_agent = self.agents
            .iter()
            .min_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
            .unwrap();
        
        let escaping_energy = 2.0 * (1.0 - iteration as f64 / self.config.max_iterations as f64);

        for agent in &mut self.agents {
            if escaping_energy.abs() >= 1.0 {
                // Exploration phase
                let random_agent = &self.agents[rand::thread_rng().gen_range(0..self.agents.len())];
                for i in 0..self.dimension {
                    agent.position[i] = random_agent.position[i] - 
                        rand::thread_rng().gen_range(0.0..1.0) * 
                        (random_agent.position[i] - 2.0 * rand::thread_rng().gen_range(0.0..1.0) * agent.position[i]).abs();
                }
            } else {
                // Exploitation phase
                let r = rand::thread_rng().gen_range(0.0..1.0);
                if r >= 0.5 {
                    // Soft besiege
                    for i in 0..self.dimension {
                        agent.position[i] = best_agent.position[i] - 
                            escaping_energy * (best_agent.position[i] - agent.position[i]).abs();
                    }
                } else {
                    // Hard besiege
                    for i in 0..self.dimension {
                        agent.position[i] = best_agent.position[i] - 
                            escaping_energy * agent.position[i].abs();
                    }
                }
            }
        }
    }

    /// Hybrid adaptive algorithm selection.
    async fn update_hybrid<F>(&mut self, iteration: u32, fitness_fn: Arc<RwLock<F>>)
    where
        F: Fn(&Array1<f64>) -> f64 + Send + Sync,
    {
        // Dynamically select algorithm based on convergence progress
        let progress = iteration as f64 / self.config.max_iterations as f64;
        
        if progress < 0.3 {
            // Early phase: Use exploration-heavy algorithms
            self.config.algorithm = SwarmAlgorithm::SSA;
        } else if progress < 0.7 {
            // Mid phase: Balanced exploration/exploitation
            self.config.algorithm = SwarmAlgorithm::GWO;
        } else {
            // Late phase: Exploitation-heavy algorithms
            self.config.algorithm = SwarmAlgorithm::WOA;
        }
        
        self.update_positions(iteration, fitness_fn).await;
    }

    /// Get current optimization state.
    pub fn get_state(&self) -> SwarmState {
        SwarmState {
            global_best_fitness: self.global_best_fitness,
            global_best_position: self.global_best_position.to_vec(),
            iteration_count: self.iteration_history.len() as u32,
            population_size: self.agents.len(),
            algorithm: self.config.algorithm,
        }
    }
}

/// Result of swarm optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub optimal_parameters: Vec<f64>,
    pub best_fitness: f64,
    pub iterations: u32,
    pub convergence_history: Vec<f64>,
    pub algorithm: SwarmAlgorithm,
}

/// Current state of the swarm optimizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmState {
    pub global_best_fitness: f64,
    pub global_best_position: Vec<f64>,
    pub iteration_count: u32,
    pub population_size: usize,
    pub algorithm: SwarmAlgorithm,
}

/// Mining-specific parameter optimization using swarm intelligence.
pub struct MiningParameterOptimizer {
    swarm: SwarmOptimizer,
}

impl MiningParameterOptimizer {
    pub fn new(config: SwarmConfig) -> Result<Self, SwarmError> {
        // Mining parameters: [thread_count, memory_mb, batch_size, power_limit]
        let dimension = 4;
        let swarm = SwarmOptimizer::new(config, dimension)?;
        
        Ok(Self { swarm })
    }

    /// Optimize mining parameters for best OpCost.
    pub async fn optimize_for_opcost(
        &mut self,
        base_opcost: OpCost,
    ) -> Result<OptimizationResult, SwarmError> {
        let fitness_fn = move |params: &Array1<f64>| -> f64 {
            // Simulate OpCost based on parameters
            let thread_count = (params[0] * 32.0).max(1.0) as u32;
            let memory_mb = (params[1] * 4096.0).max(512.0);
            let batch_size = (params[2] * 1000.0).max(100.0) as u32;
            let power_limit = (params[3] * 500.0).max(50.0);
            
            // Estimated OpCost model
            let joules = power_limit * 0.001 * thread_count as f64;
            let seconds = 1.0 / (thread_count as f64 * batch_size as f64);
            let dollars = joules * 0.0001;
            
            // Fitness: minimize OpCost (weighted sum)
            joules * 0.5 + seconds * 0.3 + dollars * 0.2
        };

        self.swarm.optimize(fitness_fn).await
    }

    /// Optimize for hashrate with power constraints.
    pub async fn optimize_for_hashrate(
        &mut self,
        max_power_watts: f64,
    ) -> Result<OptimizationResult, SwarmError> {
        let fitness_fn = move |params: &Array1<f64>| -> f64 {
            let thread_count = (params[0] * 32.0).max(1.0) as u32;
            let power_limit = (params[3] * 500.0).max(50.0);
            
            // Penalize exceeding power limit
            let power_penalty = if power_limit > max_power_watts {
                (power_limit - max_power_watts) * 10.0
            } else {
                0.0
            };
            
            // Maximize hashrate (minimize inverse)
            let hashrate = thread_count as f64 * 10_000_000.0; // 10M hashes/sec/core target
            let inverse_hashrate = 1.0 / hashrate;
            
            inverse_hashrate + power_penalty
        };

        self.swarm.optimize(fitness_fn).await
    }
}

/// Trait for swarm-based transformations.
/// Aligns with chimera-core transforms for differentiable optimization.
pub trait SwarmTransform: Send + Sync {
    fn optimize(&self, params: &[f64]) -> Result<OptimizationResult, SwarmError>;
    fn get_state(&self) -> SwarmState;
    fn cost(&self) -> OpCost;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gwo_optimization() {
        let config = SwarmConfig {
            algorithm: SwarmAlgorithm::GWO,
            population_size: 20,
            max_iterations: 100,
            ..Default::default()
        };
        
        let mut optimizer = SwarmOptimizer::new(config, 4).unwrap();
        let fitness_fn = |params: &Array1<f64>| -> f64 {
            params.iter().map(|x| x.powi(2)).sum()
        };
        
        let result = optimizer.optimize(fitness_fn).await;
        assert!(result.is_ok());
        assert!(result.unwrap().iterations <= 100);
    }

    #[tokio::test]
    async fn test_mining_parameter_optimizer() {
        let config = SwarmConfig::default();
        let mut optimizer = MiningParameterOptimizer::new(config).unwrap();
        let base_opcost = OpCost::default();
        
        let result = optimizer.optimize_for_opcost(base_opcost).await;
        assert!(result.is_ok());
        
        let opt_result = result.unwrap();
        assert_eq!(opt_result.optimal_parameters.len(), 4);
    }

    #[tokio::test]
    async fn test_swarm_state() {
        let config = SwarmConfig::default();
        let optimizer = SwarmOptimizer::new(config, 4).unwrap();
        let state = optimizer.get_state();
        
        assert_eq!(state.population_size, 50);
        assert_eq!(state.iteration_count, 0);
    }
}