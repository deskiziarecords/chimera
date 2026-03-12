//! Chimera JAX
//!
//! Differentiable hash approximation layer for ChimeraOS.
//! Implements JAX-style gradient-based optimization primitives
//! for cryptographic and distributed compute workloads.

use chimera_core::primitives::{Hash, Nonce, OpCost};
use chimera_core::transforms::Transform;

use thiserror::Error;
use serde::{Deserialize, Serialize};

use ndarray::{Array1};


/// Errors produced by differentiable operations.
#[derive(Error, Debug)]
pub enum JaxError {

    #[error("Gradient computation failed: {0}")]
    GradientFailed(String),

    #[error("Dimension mismatch: {0}")]
    DimensionMismatch(String),

    #[error("Numerical instability detected")]
    NumericalInstability,
}



/// Convert Hash → tensor
fn hash_to_tensor(hash: &Hash) -> Array1<f64> {

    Array1::from_iter(hash.0.iter().map(|b| *b as f64 / 255.0))
}



/// Convert tensor → Hash
fn tensor_to_hash(tensor: &Array1<f64>) -> Hash {

    let mut bytes = [0u8; 32];

    for (i, v) in tensor.iter().take(32).enumerate() {

        let clamped = v.clamp(0.0, 1.0);

        bytes[i] = (clamped * 255.0) as u8;
    }

    Hash(bytes)
}



/// Differentiable approximation of a cryptographic hash.
///
/// This is NOT intended to replace secure hashes.
/// Instead it provides a smooth function usable for
/// optimization heuristics and strategy exploration.
pub struct DifferentiableHash {

    weights: Array1<f64>,
    bias: f64,

    /// gradient clipping threshold
    clip: f64,
}



impl DifferentiableHash {

    pub fn new() -> Self {

        let weights = Array1::from_vec(vec![1.0; 32]);

        Self {
            weights,
            bias: 0.0,
            clip: 10.0,
        }
    }



    /// Forward differentiable pass
    pub fn forward(&self, input: &Array1<f64>) -> Array1<f64> {

        input * &self.weights + self.bias
    }



    /// Compute loss and gradient
    pub fn compute_gradient(
        &self,
        params: &[f64],
    ) -> (f64, Vec<f64>) {

        let mut loss = 0.0;

        let mut grad = vec![0.0; params.len()];

        for (i, p) in params.iter().enumerate() {

            let w = self.weights[i % self.weights.len()];

            let val = p * w;

            loss += val * val;

            grad[i] = 2.0 * val * w;
        }

        loss += self.bias;

        self.clip_gradient(&mut grad);

        (loss, grad)
    }



    /// Clip gradients for numerical stability
    fn clip_gradient(&self, grad: &mut [f64]) {

        for g in grad.iter_mut() {

            if *g > self.clip {
                *g = self.clip;
            }

            if *g < -self.clip {
                *g = -self.clip;
            }
        }
    }



    /// Apply differentiable transform to a hash.
    pub fn apply_transform(
        &self,
        hash: &Hash,
    ) -> Result<Array1<f64>, JaxError> {

        let input = hash_to_tensor(hash);

        Ok(self.forward(&input))
    }



    /// Update internal weights using gradient descent.
    pub fn update_weights(
        &mut self,
        gradient: &[f64],
        learning_rate: f64,
    ) {

        for i in 0..self.weights.len() {

            self.weights[i] -=
                learning_rate * gradient[i % gradient.len()];
        }
    }
}



impl Default for DifferentiableHash {

    fn default() -> Self {
        Self::new()
    }
}



/// Gradient representation used by the intelligence layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradHash {

    pub value: Hash,
    pub gradient: Vec<f64>,
    pub argnums: Vec<usize>,
}



impl GradHash {

    pub fn new(
        value: Hash,
        gradient: Vec<f64>,
        argnums: Vec<usize>,
    ) -> Self {

        Self {
            value,
            gradient,
            argnums,
        }
    }
}



/// Trait for differentiable transforms.
pub trait DifferentiableTransform<Input>: Send + Sync {

    type Output;
    type Gradient;

    fn apply(&self, input: Input) -> Self::Output;

    fn gradient(
        &self,
        input: Input,
        argnums: &[usize],
    ) -> Result<Self::Gradient, JaxError>;

    fn name(&self) -> &'static str;
}



impl DifferentiableTransform<Hash> for DifferentiableHash {

    type Output = Array1<f64>;
    type Gradient = Vec<f64>;



    fn apply(&self, input: Hash) -> Self::Output {

        self.apply_transform(&input)
            .unwrap_or_else(|_| Array1::zeros(32))
    }



    fn gradient(
        &self,
        input: Hash,
        argnums: &[usize],
    ) -> Result<Self::Gradient, JaxError> {

        let tensor = hash_to_tensor(&input);

        let (_, grad) =
            self.compute_gradient(tensor.as_slice().unwrap());

        let filtered = argnums
            .iter()
            .filter_map(|i| grad.get(*i).copied())
            .collect();

        Ok(filtered)
    }



    fn name(&self) -> &'static str {
        "DifferentiableHash"
    }
}



/// Metrics produced by optimization runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationMetrics {

    pub gradient_norm: f64,
    pub loss_value: f64,
    pub convergence_rate: f64,
    pub iterations: u32,
}



impl OptimizationMetrics {

    pub fn new(
        gradient: &[f64],
        loss: f64,
        iterations: u32,
    ) -> Self {

        let norm =
            gradient.iter().map(|g| g * g).sum::<f64>().sqrt();

        Self {
            gradient_norm: norm,
            loss_value: loss,
            convergence_rate: 0.0,
            iterations,
        }
    }
}



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_gradient_computation() {

        let dh = DifferentiableHash::new();

        let params = vec![1.0, 2.0, 3.0];

        let (loss, grad) = dh.compute_gradient(&params);

        assert!(loss > 0.0);
        assert_eq!(grad.len(), params.len());
    }



    #[test]
    fn test_hash_transform() {

        let dh = DifferentiableHash::new();

        let hash = Hash([1u8; 32]);

        let out = dh.apply_transform(&hash).unwrap();

        assert_eq!(out.len(), 32);
    }
}
