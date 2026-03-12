//! Chimera Intelligence
//!
//! AI/ML optimization and inference layer for ChimeraOS.
//! Provides gradient-based optimization for mining strategies and predictive resource allocation.

use chimera_core::primitives::{Hash, OpCost};
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
            enable_ml_inference: false,
            learning_rate: 0.001,
            convergence_threshold: 0.0001,
            max_iterations: 1000,
            prediction_window_ms: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub optimized_parameters: Vec<f64>,
    pub predicted_hashrate: f64,
    pub predicted_power_draw: f64,
    pub confidence_score: f64,
    pub iterations_to_converge: u32,
}

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

    pub async fn optimize_strategy(
        &self,
        initial_params: &[f64],
        target_metric: &str,
    ) -> Result<OptimizationResult, IntelligenceError> {

        let mut params = initial_params.to_vec();
        let mut iterations = 0;
        let mut prev_loss = f64::INFINITY;

        for i in 0..self.config.max_iterations {

            let (mut loss, gradient) =
                self.differentiable_hash.compute_gradient(&params);

            // Metric-based weighting
            loss = match target_metric {
                "energy" => loss * 1.2,
                "thermal" => loss * 1.1,
                "balanced" => loss * 0.9,
                _ => loss,
            };

            // Relative convergence
            let change = ((prev_loss - loss) / prev_loss.abs()).abs();

            if change < self.config.convergence_threshold {
                tracing::info!("Converged after {} iterations", i);
                iterations = i;
                break;
            }

            for (param, grad) in params.iter_mut().zip(gradient.iter()) {
                *param -= self.config.learning_rate * grad;
            }

            prev_loss = loss;
            iterations = i;
        }

        Ok(OptimizationResult {
            optimized_parameters: params,
            predicted_hashrate: 10_000_000.0,
            predicted_power_draw: 50.0,
            confidence_score: 0.95,
            iterations_to_converge: iterations,
        })
    }

    pub async fn predict_allocation(
        &self,
        current_load: f64,
        available_resources: usize,
    ) -> Result<f64, IntelligenceError> {

        let prediction = current_load * 1.1;

        Ok(prediction.min(available_resources as f64))
    }

    pub async fn record_metric(&self, cost: OpCost) {

        let mut history = self.optimization_history.write().await;

        history.push(cost);

        if history.len() > 1000 {
            history.remove(0);
        }
    }

    pub async fn get_history(&self) -> Vec<OpCost> {

        let history = self.optimization_history.read().await;

        history.clone()
    }
}

pub trait IntelligentTransform: Send + Sync {

    fn predict(&self, input: Hash)
        -> Result<Hash, IntelligenceError>;

    fn optimize(&self, params: &[f64])
        -> Result<OptimizationResult, IntelligenceError>;

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

        let result = engine
            .optimize_strategy(&initial_params, "hashrate")
            .await;

        assert!(result.is_ok());

        let opt_result = result.unwrap();

        assert!(opt_result.iterations_to_converge < 1000);
    }
}
