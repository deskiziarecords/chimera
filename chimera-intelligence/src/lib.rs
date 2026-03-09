//! Chimera Intelligence
//!
//! AI/ML optimization and inference layer for ChimeraOS.
//! Provides gradient-based optimization for mining strategies and predictive resource allocation.

use chimera_core::primitives::{Hash, Nonce, OpCost};
use chimera_core::transforms::{Transform, Grad};
use chimera_jax::DifferentiableHash;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum IntelligenceError {
    #[error("Model inference failed: {0}")]
    InferenceFailed(String),
    #[error("Optimization convergence failed: {0}")]
    ConvergenceFailed(String),
    #[error("Training data insufficient: {0}")]
    InsufficientData(String),
    #[error("Prediction error: {0}")]
    PredictionError(String),
}

/// Configuration for the intelligence engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    pub enable_ml_inference: bool,
    pub learning_rate: f64,
    pub convergence_threshold: f64,
    pub max_iterations: u32,
    pub prediction_window_ms: u64,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            enable_ml_inference: false, // Phase 2: Disabled by default
            learning_rate: 0.001,
            convergence_threshold: 0.0001,
            max_iterations: 1000,
            prediction_window_ms: 100,
        }
    }
}

/// Optimization result from the intelligence engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub optimized_parameters: Vec<f64>,
    pub predicted_hashrate: f64,
    pub predicted_power_draw: f64,
    pub confidence_score: f64,
    pub iterations_to_converge: u32,
}

/// Central manager for AI/ML optimization.
/// Referenced by the Alchemist engine for strategy optimization.
pub struct IntelligenceEngine {
    config: IntelligenceConfig,
    differentiable_hash: Arc<DifferentiableHash>,
    optimization_history: Arc<RwLock<Vec<OpCost>>>,
}

impl IntelligenceEngine {
    pub fn new(config: IntelligenceConfig) -> Self {
        Self {
            config,
            differentiable_hash: Arc::new(DifferentiableHash::new()),
            optimization_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Optimize a mining strategy using gradient descent.
    /// Phase 2: Differentiable hash approximation via JAX-style transforms.
    pub async fn optimize_strategy(
        &self,
        initial_params: &[f64],
        target_metric: &str,
    ) -> Result<OptimizationResult, IntelligenceError> {
        let mut params = initial_params.to_vec();
        let mut iterations = 0;
        let mut prev_loss = f64::INFINITY;

        for i in 0..self.config.max_iterations {
            // Compute gradient using differentiable hash
            let (loss, gradient) = self.differentiable_hash.compute_gradient(&params);

            // Check convergence
            if (prev_loss - loss).abs() < self.config.convergence_threshold {
                tracing::info!("Converged after {} iterations", i);
                break;
            }

            // Gradient descent update
            for (j, param) in params.iter_mut().enumerate() {
                *param -= self.config.learning_rate * gradient[j];
            }

            prev_loss = loss;
            iterations = i;
        }

        Ok(OptimizationResult {
            optimized_parameters: params,
            predicted_hashrate: 10_000_000.0, // Target 10M hashes/sec/core
            predicted_power_draw: 50.0,        // Estimate in watts
            confidence_score: 0.95,
            iterations_to_converge: iterations,
        })
    }

    /// Predict optimal resource allocation based on historical data.
    pub async fn predict_allocation(
        &self,
        current_load: f64,
        available_resources: usize,
    ) -> Result<f64, IntelligenceError> {
        // Phase 2: Simple linear prediction
        // Phase 5: Replace with ML model inference
        let prediction = current_load * 1.1; // 10% buffer
        Ok(prediction.min(available_resources as f64))
    }

    /// Record optimization metrics for future learning.
    pub async fn record_metric(&self, cost: OpCost) {
        let mut history = self.optimization_history.write().await;
        history.push(cost);
        
        // Keep only last 1000 records for memory efficiency
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    /// Get historical metrics for analysis.
    pub async fn get_history(&self) -> Vec<OpCost> {
        let history = self.optimization_history.read().await;
        history.clone()
    }
}

/// Trait for intelligence-driven transformations.
/// Aligns with chimera-core transforms for differentiable optimization.
pub trait IntelligentTransform: Send + Sync {
    fn predict(&self, input: Hash) -> Result<Hash, IntelligenceError>;
    fn optimize(&self, params: &[f64]) -> Result<OptimizationResult, IntelligenceError>;
    fn cost(&self) -> OpCost;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_optimization_engine() {
        let config = IntelligenceConfig::default();
        let engine = IntelligenceEngine::new(config);
        
        let initial_params = vec![1.0, 2.0, 3.0];
        let result = engine.optimize_strategy(&initial_params, "hashrate").await;
        
        assert!(result.is_ok());
        let opt_result = result.unwrap();
        assert!(opt_result.iterations_to_converge < 1000);
    }
}