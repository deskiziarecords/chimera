//! Multi-Objective Optimization Module
//!
//! Implements evolutionary algorithms for balancing conflicting mining objectives.
//! Supports NSGA-II, MOEA/D, Pareto Front analysis from the Algorithm Compendium.

use chimera_core::primitives::{OpCost, NodeId};
use chimera_core::transforms::Transform;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use rand::Rng;
use ndarray::{Array1, Array2};

#[derive(Error, Debug)]
pub enum MultiObjectiveError {
    #[error("Pareto front computation failed: {0}")]
    ParetoFrontFailed(String),
    #[error("Dominance comparison error: {0}")]
    DominanceError(String),
    #[error("Invalid objective space: {0}")]
    InvalidObjectiveSpace(String),
    #[error("Population initialization failed: {0}")]
    PopulationInitializationFailed(String),
    #[error("Convergence failed after {0} generations")]
    ConvergenceFailed(u32),
    #[error("Crowding distance computation failed: {0}")]
    CrowdingDistanceFailed(String),
}

/// Configuration for multi-objective optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MOConfig {
    pub algorithm: MOAlgorithm,
    pub population_size: usize,
    pub max_generations: u32,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub archive_size: usize,
    pub convergence_threshold: f64,
}

impl Default for MOConfig {
    fn default() -> Self {
        Self {
            algorithm: MOAlgorithm::NSGA2,
            population_size: 100,
            max_generations: 500,
            mutation_rate: 0.1,
            crossover_rate: 0.9,
            archive_size: 50,
            convergence_threshold: 1e-6,
        }
    }
}

/// Available multi-objective algorithms from the Compendium.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MOAlgorithm {
    NSGA2,      // Non-dominated Sorting Genetic Algorithm II
    MOEAD,      // Multi-Objective Evolutionary Algorithm based on Decomposition
    SPEA2,      // Strength Pareto Evolutionary Algorithm 2
    PESA2,      // Pareto Envelope-based Selection Algorithm II
    Hybrid,     // Adaptive hybrid of multiple algorithms
}

/// Represents a solution in the multi-objective space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MOSolution {
    pub decision_variables: Array1<f64>,    // Decision space (parameters)
    pub objectives: Array1<f64>,            // Objective space (metrics to optimize)
    pub rank: usize,                        // Non-domination rank
    pub crowding_distance: f64,             // Diversity metric
    pub feasibility: bool,                  // Constraint satisfaction
    pub node_id: Option<NodeId>,            // Associated node (if applicable)
}

impl MOSolution {
    pub fn new(num_decision_vars: usize, num_objectives: usize) -> Self {
        let mut rng = rand::thread_rng();
        
        Self {
            decision_variables: Array1::from_vec(
                (0..num_decision_vars).map(|_| rng.gen_range(0.0..1.0)).collect()
            ),
            objectives: Array1::zeros(num_objectives),
            rank: 0,
            crowding_distance: 0.0,
            feasibility: true,
            node_id: None,
        }
    }

    /// Check if this solution dominates another.
    pub fn dominates(&self, other: &MOSolution) -> bool {
        // Self dominates other if:
        // 1. Self is no worse than other in all objectives
        // 2. Self is strictly better in at least one objective
        
        let mut at_least_one_better = false;
        
        for i in 0..self.objectives.len().min(other.objectives.len()) {
            if self.objectives[i] > other.objectives[i] {
                // Worse in this objective (assuming minimization)
                return false;
            }
            if self.objectives[i] < other.objectives[i] {
                at_least_one_better = true;
            }
        }
        
        at_least_one_better
    }
}

/// Represents a point on the Pareto front.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoPoint {
    pub objectives: Vec<f64>,
    pub decision_variables: Vec<f64>,
    pub trade_off_ratios: Vec<f64>,
}

/// Central multi-objective optimizer.
/// Referenced by `chimera-intelligence` for trade-off analysis.
pub struct MultiObjectiveOptimizer {
    config: MOConfig,
    population: Vec<MOSolution>,
    archive: Vec<MOSolution>,  // External archive for Pareto solutions
    num_decision_vars: usize,
    num_objectives: usize,
    generation_history: Vec<GenerationStats>,
}

impl MultiObjectiveOptimizer {
    pub fn new(
        config: MOConfig,
        num_decision_vars: usize,
        num_objectives: usize,
    ) -> Result<Self, MultiObjectiveError> {
        if num_decision_vars == 0 {
            return Err(MultiObjectiveError::InvalidObjectiveSpace(
                "Decision variables must be > 0".to_string()
            ));
        }
        if num_objectives == 0 {
            return Err(MultiObjectiveError::InvalidObjectiveSpace(
                "Objectives must be > 0".to_string()
            ));
        }

        let mut population = Vec::with_capacity(config.population_size);
        for _ in 0..config.population_size {
            population.push(MOSolution::new(num_decision_vars, num_objectives));
        }

        Ok(Self {
            config,
            population,
            archive: Vec::with_capacity(config.archive_size),
            num_decision_vars,
            num_objectives,
            generation_history: Vec::new(),
        })
    }

    /// Run multi-objective optimization to find Pareto-optimal solutions.
    pub async fn optimize<F>(&mut self, objective_fn: F) -> Result<ParetoFront, MultiObjectiveError>
    where
        F: Fn(&Array1<f64>) -> Array1<f64> + Send + Sync + Clone + 'static,
    {
        let objective_fn = Arc::new(RwLock::new(objective_fn));
        let mut prev_hypervolume = 0.0;
        let mut stagnation_counter = 0;

        for generation in 0..self.config.max_generations {
            // Evaluate all solutions
            for solution in &mut self.population {
                let obj_fn_clone = Arc::clone(&objective_fn);
                let objectives = {
                    let fn_lock = obj_fn_clone.read().await;
                    fn_lock(&solution.decision_variables)
                };
                solution.objectives = objectives;
            }

            // Non-dominated sorting
            self.non_dominated_sort()?;

            // Calculate crowding distance
            self.calculate_crowding_distance()?;

            // Record generation statistics
            let stats = self.get_generation_stats(generation);
            self.generation_history.push(stats.clone());

            // Check convergence via hypervolume
            let hypervolume = self.calculate_hypervolume();
            let improvement = (hypervolume - prev_hypervolume).abs();
            
            if improvement < self.config.convergence_threshold {
                stagnation_counter += 1;
                if stagnation_counter > 20 {
                    tracing::info!("Converged after {} generations", generation);
                    break;
                }
            } else {
                stagnation_counter = 0;
            }
            prev_hypervolume = hypervolume;

            // Create offspring
            let offspring = self.create_offspring(Arc::clone(&objective_fn)).await?;

            // Combine parent and offspring
            let mut combined = self.population.clone();
            combined.extend(offspring);

            // Environmental selection
            self.environmental_selection(&mut combined)?;

            // Update archive with non-dominated solutions
            self.update_archive()?;
        }

        // Extract Pareto front from archive
        self.extract_pareto_front()
    }

    /// Non-dominated sorting (NSGA-II style).
    fn non_dominated_sort(&mut self) -> Result<(), MultiObjectiveError> {
        let n = self.population.len();
        let mut domination_count = vec![0; n];
        let mut dominated_solutions: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut fronts: Vec<Vec<usize>> = Vec::new();

        // Compare all pairs
        for i in 0..n {
            for j in (i + 1)..n {
                if self.population[i].dominates(&self.population[j]) {
                    dominated_solutions[i].push(j);
                    domination_count[j] += 1;
                } else if self.population[j].dominates(&self.population[i]) {
                    dominated_solutions[j].push(i);
                    domination_count[i] += 1;
                }
            }
        }

        // Build fronts
        let mut first_front = Vec::new();
        for i in 0..n {
            if domination_count[i] == 0 {
                self.population[i].rank = 0;
                first_front.push(i);
            }
        }
        fronts.push(first_front);

        let mut front_idx = 0;
        while !fronts[front_idx].is_empty() {
            let mut next_front = Vec::new();
            
            for &i in &fronts[front_idx] {
                for &j in &dominated_solutions[i] {
                    domination_count[j] -= 1;
                    if domination_count[j] == 0 {
                        self.population[j].rank = front_idx + 1;
                        next_front.push(j);
                    }
                }
            }
            
            front_idx += 1;
            if !next_front.is_empty() {
                fronts.push(next_front);
            }
        }

        Ok(())
    }

    /// Calculate crowding distance for diversity preservation.
    fn calculate_crowding_distance(&mut self) -> Result<(), MultiObjectiveError> {
        let n = self.population.len();
        
        // Reset distances
        for solution in &mut self.population {
            solution.crowding_distance = 0.0;
        }

        // Calculate per objective
        for obj_idx in 0..self.num_objectives {
            // Sort by this objective
            let mut indices: Vec<usize> = (0..n).collect();
            indices.sort_by(|&i, &j| {
                self.population[i].objectives[obj_idx]
                    .partial_cmp(&self.population[j].objectives[obj_idx])
                    .unwrap()
            });

            // Boundary points get infinite distance
            self.population[indices[0]].crowding_distance = f64::INFINITY;
            self.population[indices[n - 1]].crowding_distance = f64::INFINITY;

            // Calculate range
            let obj_range = self.population[indices[n - 1]].objectives[obj_idx]
                - self.population[indices[0]].objectives[obj_idx];

            if obj_range == 0.0 {
                continue;
            }

            // Interior points
            for i in 1..(n - 1) {
                let distance = self.population[indices[i + 1]].objectives[obj_idx]
                    - self.population[indices[i - 1]].objectives[obj_idx];
                self.population[indices[i]].crowding_distance += distance / obj_range;
            }
        }

        Ok(())
    }

    /// Create offspring through crossover and mutation.
    async fn create_offspring<F>(
        &mut self,
        objective_fn: Arc<RwLock<F>>,
    ) -> Result<Vec<MOSolution>, MultiObjectiveError>
    where
        F: Fn(&Array1<f64>) -> Array1<f64> + Send + Sync,
    {
        let mut offspring = Vec::with_capacity(self.config.population_size);
        let mut rng = rand::thread_rng();

        while offspring.len() < self.config.population_size {
            // Tournament selection
            let parent1 = self.tournament_select()?;
            let parent2 = self.tournament_select()?;

            // Crossover
            let (child1, child2) = self.simulated_binary_crossover(&parent1, &parent2)?;

            // Mutation
            let mut child1 = self.polynomial_mutation(child1)?;
            let mut child2 = self.polynomial_mutation(child2)?;

            // Evaluate offspring
            for child in [&mut child1, &mut child2] {
                let obj_fn_clone = Arc::clone(&objective_fn);
                let objectives = {
                    let fn_lock = obj_fn_clone.read().await;
                    fn_lock(&child.decision_variables)
                };
                child.objectives = objectives;
                offspring.push(child.clone());
                
                if offspring.len() >= self.config.population_size {
                    break;
                }
            }
        }

        Ok(offspring)
    }

    /// Tournament selection based on rank and crowding distance.
    fn tournament_select(&self) -> Result<MOSolution, MultiObjectiveError> {
        let mut rng = rand::thread_rng();
        let tournament_size = 3;
        
        let mut best_idx = rng.gen_range(0..self.population.len());
        
        for _ in 1..tournament_size {
            let candidate_idx = rng.gen_range(0..self.population.len());
            
            // Compare by rank first, then crowding distance
            if self.population[candidate_idx].rank < self.population[best_idx].rank {
                best_idx = candidate_idx;
            } else if self.population[candidate_idx].rank == self.population[best_idx].rank {
                if self.population[candidate_idx].crowding_distance > self.population[best_idx].crowding_distance {
                    best_idx = candidate_idx;
                }
            }
        }

        Ok(self.population[best_idx].clone())
    }

    /// Simulated Binary Crossover (SBX).
    fn simulated_binary_crossover(
        &self,
        parent1: &MOSolution,
        parent2: &MOSolution,
    ) -> Result<(MOSolution, MOSolution), MultiObjectiveError> {
        let mut rng = rand::thread_rng();
        let mut child1 = parent1.clone();
        let mut child2 = parent2.clone();

        for i in 0..self.num_decision_vars {
            if rng.gen_range(0.0..1.0) < self.config.crossover_rate {
                let y1 = parent1.decision_variables[i].min(parent2.decision_variables[i]);
                let y2 = parent1.decision_variables[i].max(parent2.decision_variables[i]);
                
                let beta = 1.0 + (2.0 * (y1 / (y2 - y1 + 1e-10)));
                let alpha = 2.0 - beta.powf(-2.0);
                
                let u = rng.gen_range(0.0..1.0);
                let beta_q = if u <= 1.0 / alpha {
                    (u * alpha).powf(1.0 / 3.0)
                } else {
                    (1.0 / (alpha * (1.0 - u))).powf(1.0 / 3.0)
                };

                child1.decision_variables[i] = (y1 + y2 - beta_q * (y2 - y1)).clamp(0.0, 1.0);
                child2.decision_variables[i] = (y1 + y2 + beta_q * (y2 - y1)).clamp(0.0, 1.0);
            }
        }

        Ok((child1, child2))
    }

    /// Polynomial mutation.
    fn polynomial_mutation(&self, mut solution: MOSolution) -> Result<MOSolution, MultiObjectiveError> {
        let mut rng = rand::thread_rng();
        let mutation_probability = self.config.mutation_rate / self.num_decision_vars as f64;
        let eta_m = 20.0; // Distribution index

        for i in 0..self.num_decision_vars {
            if rng.gen_range(0.0..1.0) < mutation_probability {
                let y = solution.decision_variables[i];
                let delta1 = y;
                let delta2 = 1.0 - y;
                
                let u = rng.gen_range(0.0..1.0);
                let mut delta = 0.0;

                if u < 0.5 {
                    delta = (2.0 * u + (1.0 - 2.0 * u) * (1.0 - delta1).powf(eta_m + 1.0)).powf(1.0 / (eta_m + 1.0)) - 1.0;
                } else {
                    delta = 1.0 - (2.0 * (1.0 - u) + 2.0 * (u - 0.5) * (1.0 - delta2).powf(eta_m + 1.0)).powf(1.0 / (eta_m + 1.0));
                }

                solution.decision_variables[i] = (y + delta).clamp(0.0, 1.0);
            }
        }

        Ok(solution)
    }

    /// Environmental selection (NSGA-II style).
    fn environmental_selection(
        &mut self,
        combined: &mut Vec<MOSolution>,
    ) -> Result<(), MultiObjectiveError> {
        // Sort combined population by rank and crowding distance
        combined.sort_by(|a, b| {
            if a.rank != b.rank {
                a.rank.cmp(&b.rank)
            } else {
                b.crowding_distance.partial_cmp(&a.crowding_distance).unwrap()
            }
        });

        // Select top N solutions
        self.population = combined.iter().take(self.config.population_size).cloned().collect();
        
        Ok(())
    }

    /// Update external archive with non-dominated solutions.
    fn update_archive(&mut self) -> Result<(), MultiObjectiveError> {
        // Add non-dominated solutions from population to archive
        for solution in &self.population {
            if solution.rank == 0 {
                // Check if already in archive
                let already_exists = self.archive.iter().any(|archived| {
                    archived.decision_variables.iter()
                        .zip(solution.decision_variables.iter())
                        .all(|(a, b)| (a - b).abs() < 1e-6)
                });

                if !already_exists && self.archive.len() < self.config.archive_size {
                    self.archive.push(solution.clone());
                }
            }
        }

        // Trim archive if too large (keep most diverse)
        if self.archive.len() > self.config.archive_size {
            self.archive.sort_by(|a, b| {
                b.crowding_distance.partial_cmp(&a.crowding_distance).unwrap()
            });
            self.archive.truncate(self.config.archive_size);
        }

        Ok(())
    }

    /// Extract final Pareto front from archive.
    fn extract_pareto_front(&self) -> Result<ParetoFront, MultiObjectiveError> {
        let mut points = Vec::new();

        for solution in &self.archive {
            let trade_off_ratios = self.calculate_trade_offs(&solution.objectives)?;
            
            points.push(ParetoPoint {
                objectives: solution.objectives.to_vec(),
                decision_variables: solution.decision_variables.to_vec(),
                trade_off_ratios,
            });
        }

        Ok(ParetoFront {
            points,
            num_objectives: self.num_objectives,
            hypervolume: self.calculate_hypervolume(),
            spread: self.calculate_spread(),
        })
    }

    /// Calculate trade-off ratios between objectives.
    fn calculate_trade_offs(&self, objectives: &Array1<f64>) -> Result<Vec<f64>, MultiObjectiveError> {
        let mut ratios = Vec::new();
        
        for i in 0..objectives.len() {
            for j in (i + 1)..objectives.len() {
                if objectives[j] != 0.0 {
                    ratios.push(objectives[i] / objectives[j]);
                } else {
                    ratios.push(f64::INFINITY);
                }
            }
        }

        Ok(ratios)
    }

    /// Calculate hypervolume indicator (simplified 2D version).
    fn calculate_hypervolume(&self) -> f64 {
        // Simplified hypervolume calculation for monitoring convergence
        let reference_point = Array1::from_elem(self.num_objectives, 1.0);
        let mut hypervolume = 0.0;

        for solution in &self.archive {
            let mut volume = 1.0;
            for i in 0..self.num_objectives {
                let diff = reference_point[i] - solution.objectives[i].min(reference_point[i]);
                volume *= diff.max(0.0);
            }
            hypervolume += volume;
        }

        hypervolume
    }

    /// Calculate spread metric for diversity assessment.
    fn calculate_spread(&self) -> f64 {
        if self.archive.len() < 2 {
            return 0.0;
        }

        let mut distances = Vec::new();
        for i in 0..(self.archive.len() - 1) {
            let mut dist = 0.0;
            for j in 0..self.num_objectives {
                let diff = self.archive[i].objectives[j] - self.archive[i + 1].objectives[j];
                dist += diff * diff;
            }
            distances.push(dist.sqrt());
        }

        let mean_dist = distances.iter().sum::<f64>() / distances.len() as f64;
        let variance = distances.iter().map(|d| (d - mean_dist).powi(2)).sum::<f64>() / distances.len() as f64;
        
        variance.sqrt() / mean_dist.max(1e-10)
    }

    /// Get generation statistics for telemetry.
    fn get_generation_stats(&self, generation: u32) -> GenerationStats {
        let avg_objectives: Array1<f64> = if self.population.is_empty() {
            Array1::zeros(self.num_objectives)
        } else {
            let mut sum = Array1::zeros(self.num_objectives);
            for solution in &self.population {
                sum += &solution.objectives;
            }
            sum / self.population.len() as f64
        };

        let best_objectives: Array1<f64> = if self.population.is_empty() {
            Array1::zeros(self.num_objectives)
        } else {
            let mut best = self.population[0].objectives.clone();
            for solution in &self.population {
                for i in 0..self.num_objectives {
                    if solution.objectives[i] < best[i] {
                        best[i] = solution.objectives[i];
                    }
                }
            }
            best
        };

        GenerationStats {
            generation,
            average_objectives: avg_objectives.to_vec(),
            best_objectives: best_objectives.to_vec(),
            population_size: self.population.len(),
            archive_size: self.archive.len(),
            hypervolume: self.calculate_hypervolume(),
        }
    }

    /// Get current optimization state.
    pub fn get_state(&self) -> MOState {
        MOState {
            current_generation: self.generation_history.len() as u32,
            archive_size: self.archive.len(),
            population_size: self.population.len(),
            best_hypervolume: self.generation_history.last().map(|s| s.hypervolume).unwrap_or(0.0),
            algorithm: self.config.algorithm,
        }
    }
}

/// Pareto front representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParetoFront {
    pub points: Vec<ParetoPoint>,
    pub num_objectives: usize,
    pub hypervolume: f64,
    pub spread: f64,
}

/// Statistics for a single generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    pub generation: u32,
    pub average_objectives: Vec<f64>,
    pub best_objectives: Vec<f64>,
    pub population_size: usize,
    pub archive_size: usize,
    pub hypervolume: f64,
}

/// Current state of the multi-objective optimizer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MOState {
    pub current_generation: u32,
    pub archive_size: usize,
    pub population_size: usize,
    pub best_hypervolume: f64,
    pub algorithm: MOAlgorithm,
}

/// Mining-specific multi-objective optimizer for OpCost trade-offs.
pub struct MiningMOOptimizer {
    optimizer: MultiObjectiveOptimizer,
}

impl MiningMOOptimizer {
    pub fn new(config: MOConfig) -> Result<Self, MultiObjectiveError> {
        // Decision variables: [thread_count, memory_mb, batch_size, power_limit]
        let num_decision_vars = 4;
        // Objectives: [joules, seconds, dollars, thermal]
        let num_objectives = 4;
        
        let optimizer = MultiObjectiveOptimizer::new(config, num_decision_vars, num_objectives)?;
        
        Ok(Self { optimizer })
    }

    /// Optimize mining parameters for Pareto-optimal OpCost trade-offs.
    pub async fn optimize_opcost_tradeoffs(
        &mut self,
        base_opcost: OpCost,
    ) -> Result<ParetoFront, MultiObjectiveError> {
        let objective_fn = move |params: &Array1<f64>| -> Array1<f64> {
            let thread_count = (params[0] * 32.0).max(1.0) as u32;
            let memory_mb = (params[1] * 4096.0).max(512.0);
            let batch_size = (params[2] * 1000.0).max(100.0) as u32;
            let power_limit = (params[3] * 500.0).max(50.0);
            
            // Objective 1: Energy (joules)
            let joules = power_limit * 0.001 * thread_count as f64;
            
            // Objective 2: Time (seconds per batch)
            let seconds = 1.0 / (thread_count as f64 * batch_size as f64);
            
            // Objective 3: Cost (dollars)
            let dollars = joules * 0.0001;
            
            // Objective 4: Thermal stress (normalized)
            let thermal = (power_limit / 500.0).powi(2);
            
            Array1::from_vec(vec![joules, seconds, dollars, thermal])
        };

        self.optimizer.optimize(objective_fn).await
    }

    /// Select best solution from Pareto front based on preference weights.
    pub fn select_by_preference(
        &self,
        front: &ParetoFront,
        weights: &[f64],
    ) -> Option<ParetoPoint> {
        if front.points.is_empty() || weights.len() != front.num_objectives {
            return None;
        }

        let mut best_score = f64::INFINITY;
        let mut best_point = None;

        for point in &front.points {
            let mut score = 0.0;
            for (i, &obj) in point.objectives.iter().enumerate() {
                score += weights[i] * obj;
            }
            
            if score < best_score {
                best_score = score;
                best_point = Some(point.clone());
            }
        }

        best_point
    }

    /// Get optimization history for dashboard visualization.
    pub fn get_history(&self) -> Vec<GenerationStats> {
        self.optimizer.generation_history.clone()
    }
}

/// Trait for multi-objective transformations.
/// Aligns with chimera-core transforms for differentiable optimization.
pub trait MultiObjectiveTransform: Send + Sync {
    fn optimize(&self, weights: &[f64]) -> Result<ParetoFront, MultiObjectiveError>;
    fn get_state(&self) -> MOState;
    fn cost(&self) -> OpCost;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nsga2_optimization() {
        let config = MOConfig {
            algorithm: MOAlgorithm::NSGA2,
            population_size: 50,
            max_generations: 100,
            ..Default::default()
        };
        
        let mut optimizer = MultiObjectiveOptimizer::new(config, 4, 4).unwrap();
        let objective_fn = |params: &Array1<f64>| -> Array1<f64> {
            params.clone() // Simple identity function for testing
        };
        
        let front = optimizer.optimize(objective_fn).await;
        assert!(front.is_ok());
        assert!(front.unwrap().points.len() > 0);
    }

    #[tokio::test]
    async fn test_mining_mo_optimizer() {
        let config = MOConfig::default();
        let mut optimizer = MiningMOOptimizer::new(config).unwrap();
        let base_opcost = OpCost::default();
        
        let front = optimizer.optimize_opcost_tradeoffs(base_opcost).await;
        assert!(front.is_ok());
        
        let pareto_front = front.unwrap();
        assert!(pareto_front.points.len() > 0);
        assert_eq!(pareto_front.num_objectives, 4);
    }

    #[test]
    fn test_pareto_dominance() {
        let mut sol1 = MOSolution::new(4, 4);
        sol1.objectives = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.0]);
        
        let mut sol2 = MOSolution::new(4, 4);
        sol2.objectives = Array1::from_vec(vec![2.0, 3.0, 4.0, 5.0]);
        
        // sol1 should dominate sol2 (all objectives better)
        assert!(sol1.dominates(&sol2));
        assert!(!sol2.dominates(&sol1));
    }

    #[test]
    fn test_preference_selection() {
        let config = MOConfig::default();
        let optimizer = MiningMOOptimizer::new(config).unwrap();
        
        let front = ParetoFront {
            points: vec![
                ParetoPoint {
                    objectives: vec![1.0, 2.0, 3.0, 4.0],
                    decision_variables: vec![0.5, 0.5, 0.5, 0.5],
                    trade_off_ratios: vec![0.5, 0.33, 0.25],
                },
                ParetoPoint {
                    objectives: vec![2.0, 1.0, 4.0, 3.0],
                    decision_variables: vec![0.6, 0.4, 0.6, 0.4],
                    trade_off_ratios: vec![2.0, 0.5, 0.67],
                },
            ],
            num_objectives: 4,
            hypervolume: 0.5,
            spread: 0.1,
        };
        
        let weights = vec![1.0, 0.0, 0.0, 0.0]; // Prioritize first objective
        let selected = optimizer.select_by_preference(&front, &weights);
        
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().objectives[0], 1.0); // Best for first objective
    }

    #[test]
    fn test_mo_state() {
        let config = MOConfig::default();
        let optimizer = MultiObjectiveOptimizer::new(config, 4, 4).unwrap();
        let state = optimizer.get_state();
        
        assert_eq!(state.current_generation, 0);
        assert_eq!(state.population_size, 100);
        assert_eq!(state.algorithm, MOAlgorithm::NSGA2);
    }
}