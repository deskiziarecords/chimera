<<<<<<< HEAD
//! JAX-Style Differentiable Transforms
//!
//! Implements gradient-based optimization for mining algorithms.
//! Enables ML-driven gradient descent on cryptographic structures (Phase 2).

use crate::primitives::{Hash, Nonce, OpCost};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

/// Transform-specific error types.
#[derive(Error, Debug)]
pub enum TransformError {
    #[error("Gradient computation failed: {0}")]
    GradientComputationFailed(String),
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Transform not differentiable: {0}")]
    NotDifferentiable(String),
    #[error("Numerical instability detected: {0}")]
    NumericalInstability(String),
}

/// Core trait for all transformations in ChimeraOS.
/// Aligns with JAX-style functional programming for differentiable optimization.
pub trait Transform<Input>: Send + Sync {
    /// Output type of the transformation.
    type Output;

    /// Apply the transformation to input.
    fn apply(&self, input: Input) -> Self::Output;

    /// Human-readable name for telemetry and debugging.
    fn name(&self) -> &'static str;

    /// Optional: compute gradient with respect to input.
    /// Returns None if transform is not differentiable.
    fn gradient(&self, _input: &Input) -> Option<Grad<Input, Self::Output>> {
        None
    }

    /// Optional: compute operational cost of this transform.
    fn cost(&self) -> OpCost {
        OpCost::default()
    }
}

/// Gradient structure for differentiable transforms.
/// Stores the function and argument indices for automatic differentiation.
pub struct Grad<Input, Output> {
    /// Boxed function that computes the gradient.
    pub f: Arc<dyn Fn(&Input) -> Output + Send + Sync>,
    /// Argument indices to compute gradients with respect to.
    pub argnums: Vec<usize>,
    /// Gradient values (computed on demand).
    pub values: Option<Vec<f64>>,
}

impl<Input, Output> Grad<Input, Output> {
    /// Create a new gradient structure.
    pub fn new(
        f: Arc<dyn Fn(&Input) -> Output + Send + Sync>,
        argnums: Vec<usize>,
    ) -> Self {
        Self {
            f,
            argnums,
            values: None,
        }
    }

    /// Compute gradient values.
    pub fn compute(&mut self, input: &Input) -> Result<(), TransformError> {
        // Placeholder - actual implementation would use autodiff
        self.values = Some(vec![1.0; self.argnums.len()]);
        Ok(())
    }

    /// Get computed gradient values.
    pub fn values(&self) -> Option<&Vec<f64>> {
        self.values.as_ref()
    }

    /// Get argument indices.
    pub fn argnums(&self) -> &Vec<usize> {
        &self.argnums
    }
}

impl<Input, Output> Clone for Grad<Input, Output> {
    fn clone(&self) -> Self {
        Self {
            f: Arc::clone(&self.f),
            argnums: self.argnums.clone(),
            values: self.values.clone(),
        }
    }
}

/// Differentiable function wrapper for JAX-style value_and_grad.
pub struct DifferentiableFn<Input, Output> {
    /// Function to compute value.
    value_fn: Arc<dyn Fn(&Input) -> Output + Send + Sync>,
    /// Function to compute gradient.
    grad_fn: Option<Arc<dyn Fn(&Input) -> Vec<f64> + Send + Sync>>,
    /// Number of input dimensions.
    input_dim: usize,
}

impl<Input, Output> DifferentiableFn<Input, Output> {
    /// Create a new differentiable function.
    pub fn new(
        value_fn: Arc<dyn Fn(&Input) -> Output + Send + Sync>,
        grad_fn: Option<Arc<dyn Fn(&Input) -> Vec<f64> + Send + Sync>>,
        input_dim: usize,
    ) -> Self {
        Self {
            value_fn,
            grad_fn,
            input_dim,
        }
    }

    /// Compute function value.
    pub fn value(&self, input: &Input) -> Output {
        (self.value_fn)(input)
    }

    /// Compute gradient (if available).
    pub fn gradient(&self, input: &Input) -> Option<Vec<f64>> {
        self.grad_fn.as_ref().map(|f| f(input))
    }

    /// Compute both value and gradient in one call (efficient).
    pub fn value_and_grad(&self, input: &Input) -> (Output, Option<Vec<f64>>) {
        let value = (self.value_fn)(input);
        let grad = self.grad_fn.as_ref().map(|f| f(input));
        (value, grad)
    }

    /// Get input dimension.
    pub fn input_dim(&self) -> usize {
        self.input_dim
    }
}

impl<Input, Output> Clone for DifferentiableFn<Input, Output> {
    fn clone(&self) -> Self {
        Self {
            value_fn: Arc::clone(&self.value_fn),
            grad_fn: self.grad_fn.as_ref().map(Arc::clone),
            input_dim: self.input_dim,
        }
    }
}

/// Composable transform chain for building complex pipelines.
pub struct TransformChain<Input> {
    transforms: Vec<Arc<dyn Transform<Input, Output = Input> + Send + Sync>>,
}

impl<Input> TransformChain<Input> {
    /// Create a new empty transform chain.
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }

    /// Add a transform to the chain.
    pub fn add<T>(&mut self, transform: T)
    where
        T: Transform<Input, Output = Input> + Send + Sync + 'static,
    {
        self.transforms.push(Arc::new(transform));
    }

    /// Apply all transforms in sequence.
    pub fn apply(&self, mut input: Input) -> Input {
        for transform in &self.transforms {
            input = transform.apply(input);
        }
        input
    }

    /// Get number of transforms in chain.
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Check if chain is empty.
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }

    /// Get transform names for telemetry.
    pub fn get_names(&self) -> Vec<&str> {
        self.transforms.iter().map(|t| t.name()).collect()
    }
}

impl<Input> Default for TransformChain<Input> {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash transformation for differentiable hash approximation (Phase 2).
pub struct HashTransform {
    /// Transformation weights (learnable parameters).
    weights: Vec<f64>,
    /// Bias term.
    bias: f64,
    /// Activation function type.
    activation: ActivationFn,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ActivationFn {
    Linear,
    ReLU,
    Sigmoid,
    Tanh,
}

impl ActivationFn {
    pub fn apply(&self, x: f64) -> f64 {
        match self {
            ActivationFn::Linear => x,
            ActivationFn::ReLU => x.max(0.0),
            ActivationFn::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            ActivationFn::Tanh => x.tanh(),
        }
    }

    pub fn derivative(&self, x: f64) -> f64 {
        match self {
            ActivationFn::Linear => 1.0,
            ActivationFn::ReLU => if x > 0.0 { 1.0 } else { 0.0 },
            ActivationFn::Sigmoid => {
                let s = 1.0 / (1.0 + (-x).exp());
                s * (1.0 - s)
            }
            ActivationFn::Tanh => 1.0 - x.tanh().powi(2),
        }
    }
}

impl HashTransform {
    /// Create a new hash transform with random weights.
    pub fn new(dim: usize, activation: ActivationFn) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        Self {
            weights: (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect(),
            bias: rng.gen_range(-0.1..0.1),
            activation,
        }
    }

    /// Create hash transform with specified weights.
    pub fn with_weights(weights: Vec<f64>, bias: f64, activation: ActivationFn) -> Self {
        Self {
            weights,
            bias,
            activation,
        }
    }

    /// Get weights as slice.
    pub fn weights(&self) -> &[f64] {
        &self.weights
    }

    /// Update weights (for gradient descent).
    pub fn update_weights(&mut self, gradients: &[f64], learning_rate: f64) {
        for (i, weight) in self.weights.iter_mut().enumerate() {
            if i < gradients.len() {
                *weight -= learning_rate * gradients[i];
            }
        }
    }

    /// Compute differentiable approximation of hash.
    pub fn compute_approx(&self, hash: &Hash) -> Vec<f64> {
        let mut output = Vec::with_capacity(self.weights.len());
        
        for (i, &weight) in self.weights.iter().enumerate() {
            let idx = i % hash.0.len();
            let x = (hash.0[idx] as f64 / 255.0) * weight + self.bias;
            output.push(self.activation.apply(x));
        }
        
        output
    }

    /// Compute gradient with respect to weights.
    pub fn compute_gradient(&self, hash: &Hash, target: &[f64]) -> Vec<f64> {
        let output = self.compute_approx(hash);
        let mut gradients = vec![0.0; self.weights.len()];
        
        for i in 0..self.weights.len() {
            let idx = i % hash.0.len();
            let x = (hash.0[idx] as f64 / 255.0) * self.weights[i] + self.bias;
            let activation_deriv = self.activation.derivative(x);
            let error = output[i] - target.get(i).copied().unwrap_or(0.0);
            
            gradients[i] = error * activation_deriv * (hash.0[idx] as f64 / 255.0);
        }
        
        gradients
    }
}

impl Transform<Hash> for HashTransform {
    type Output = Vec<f64>;

    fn apply(&self, input: Hash) -> Self::Output {
        self.compute_approx(&input)
    }

    fn name(&self) -> &'static str {
        "HashTransform"
    }

    fn gradient(&self, input: &Hash) -> Option<Grad<Hash, Self::Output>> {
        // Create gradient function
        let weights = self.weights.clone();
        let bias = self.bias;
        let activation = self.activation;
        
        let grad_fn = Arc::new(move |hash: &Hash| -> Vec<f64> {
            let mut grad = vec![0.0; weights.len()];
            for i in 0..weights.len() {
                let idx = i % hash.0.len();
                let x = (hash.0[idx] as f64 / 255.0) * weights[i] + bias;
                grad[i] = activation.derivative(x) * (hash.0[idx] as f64 / 255.0);
            }
            grad
        });
        
        Some(Grad::new(grad_fn, (0..self.weights.len()).collect()))
    }

    fn cost(&self) -> OpCost {
        OpCost {
            joules: self.weights.len() as f64 * 0.0001,
            seconds: self.weights.len() as f64 * 0.000001,
            dollars: self.weights.len() as f64 * 0.0000001,
        }
    }
}

/// OpCost transformation for optimization metrics.
pub struct OpCostTransform {
    /// Weight for joules in optimization.
    pub joules_weight: f64,
    /// Weight for seconds in optimization.
    pub seconds_weight: f64,
    /// Weight for dollars in optimization.
    pub dollars_weight: f64,
}

impl OpCostTransform {
    pub fn new(joules_weight: f64, seconds_weight: f64, dollars_weight: f64) -> Self {
        Self {
            joules_weight,
            seconds_weight,
            dollars_weight,
        }
    }

    /// Compute weighted score from OpCost.
    pub fn score(&self, cost: &OpCost) -> f64 {
        cost.joules * self.joules_weight
            + cost.seconds * self.seconds_weight
            + cost.dollars * self.dollars_weight
    }
}

impl Transform<OpCost> for OpCostTransform {
    type Output = f64;

    fn apply(&self, input: OpCost) -> Self::Output {
        self.score(&input)
    }

    fn name(&self) -> &'static str {
        "OpCostTransform"
    }

    fn cost(&self) -> OpCost {
        OpCost::zero()
    }
}

/// Nonce transformation for mining optimization.
pub struct NonceTransform {
    /// Stride for nonce increment.
    pub stride: u64,
    /// Maximum nonce value.
    pub max_nonce: u64,
}

impl NonceTransform {
    pub fn new(stride: u64, max_nonce: u64) -> Self {
        Self { stride, max_nonce }
    }

    /// Apply stride to nonce.
    pub fn apply_stride(&self, nonce: Nonce) -> Nonce {
        Nonce((nonce.0 + self.stride) % self.max_nonce)
    }
}

impl Transform<Nonce> for NonceTransform {
    type Output = Nonce;

    fn apply(&self, input: Nonce) -> Self::Output {
        self.apply_stride(input)
    }

    fn name(&self) -> &'static str {
        "NonceTransform"
    }

    fn cost(&self) -> OpCost {
        OpCost {
            joules: 0.0001,
            seconds: 0.0000001,
            dollars: 0.00000001,
        }
    }
}

/// Gradient descent optimizer for transform parameters.
pub struct GradientDescent {
    /// Learning rate.
    pub learning_rate: f64,
    /// Momentum coefficient.
    pub momentum: f64,
    /// Maximum iterations.
    pub max_iterations: u32,
    /// Convergence threshold.
    pub convergence_threshold: f64,
}

impl GradientDescent {
    pub fn new(learning_rate: f64, momentum: f64, max_iterations: u32, convergence_threshold: f64) -> Self {
        Self {
            learning_rate,
            momentum,
            max_iterations,
            convergence_threshold,
        }
    }

    /// Optimize transform parameters using gradient descent.
    pub fn optimize<F, G>(
        &self,
        mut params: Vec<f64>,
        loss_fn: F,
        grad_fn: G,
    ) -> Result<OptimizationResult, TransformError>
    where
        F: Fn(&[f64]) -> f64 + Send + Sync,
        G: Fn(&[f64]) -> Vec<f64> + Send + Sync,
    {
        let mut velocity = vec![0.0; params.len()];
        let mut prev_loss = loss_fn(&params);
        let mut best_params = params.clone();
        let mut best_loss = prev_loss;

        for iteration in 0..self.max_iterations {
            let gradients = grad_fn(&params);
            
            // Update with momentum
            for i in 0..params.len() {
                velocity[i] = self.momentum * velocity[i] - self.learning_rate * gradients[i];
                params[i] += velocity[i];
            }

            let loss = loss_fn(&params);
            
            if loss < best_loss {
                best_loss = loss;
                best_params = params.clone();
            }

            // Check convergence
            if (prev_loss - loss).abs() < self.convergence_threshold {
                return Ok(OptimizationResult {
                    optimal_parameters: best_params,
                    best_loss,
                    iterations: iteration + 1,
                    converged: true,
                });
            }

            prev_loss = loss;
        }

        Ok(OptimizationResult {
            optimal_parameters: best_params,
            best_loss,
            iterations: self.max_iterations,
            converged: false,
        })
    }
}

impl Default for GradientDescent {
    fn default() -> Self {
        Self {
            learning_rate: 0.001,
            momentum: 0.9,
            max_iterations: 1000,
            convergence_threshold: 1e-6,
        }
    }
}

/// Result of gradient descent optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub optimal_parameters: Vec<f64>,
    pub best_loss: f64,
    pub iterations: u32,
    pub converged: bool,
}

/// Trait for differentiable transforms with gradient support.
pub trait DifferentiableTransform<Input>: Transform<Input> {
    /// Compute gradient with respect to input.
    fn compute_gradient(&self, input: &Input) -> Result<Vec<f64>, TransformError>;
    
    /// Compute Hessian matrix (second-order derivatives).
    fn compute_hessian(&self, _input: &Input) -> Result<Vec<Vec<f64>>, TransformError> {
        Err(TransformError::NotDifferentiable(
            "Hessian not implemented".to_string()
        ))
    }
}

impl DifferentiableTransform<Hash> for HashTransform {
    fn compute_gradient(&self, input: &Hash) -> Result<Vec<f64>, TransformError> {
        // Simplified gradient computation
        let target = vec![0.5; self.weights.len()]; // Target value
        Ok(self.compute_gradient(input, &target))
    }
}

/// Jacobian matrix computation for vector-valued functions.
pub struct Jacobian {
    /// Number of outputs.
    pub output_dim: usize,
    /// Number of inputs.
    pub input_dim: usize,
}

impl Jacobian {
    pub fn new(output_dim: usize, input_dim: usize) -> Self {
        Self { output_dim, input_dim }
    }

    /// Compute Jacobian matrix using finite differences.
    pub fn compute<F>(&self, f: F, input: &[f64], epsilon: f64) -> Vec<Vec<f64>>
    where
        F: Fn(&[f64]) -> Vec<f64> + Send + Sync,
    {
        let mut jacobian = vec![vec![0.0; self.input_dim]; self.output_dim];
        let base_output = f(input);

        for i in 0..self.input_dim {
            let mut perturbed = input.to_vec();
            perturbed[i] += epsilon;
            let perturbed_output = f(&perturbed);

            for j in 0..self.output_dim {
                jacobian[j][i] = (perturbed_output[j] - base_output[j]) / epsilon;
            }
        }

        jacobian
    }
}

/// Chain rule for composing gradients.
pub fn chain_rule<const N: usize>(
    gradients: [&[f64]; N],
) -> Vec<f64> {
    if N == 0 {
        return vec![];
    }

    let mut result = gradients[0].to_vec();
    
    for &grad in &gradients[1..] {
        for (i, val) in result.iter_mut().enumerate() {
            if i < grad.len() {
                *val *= grad[i];
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_transform() {
        let transform = HashTransform::new(32, ActivationFn::ReLU);
        let hash = Hash::zero();
        
        let output = transform.apply(hash);
        assert_eq!(output.len(), 32);
    }

    #[test]
    fn test_transform_chain() {
        let mut chain = TransformChain::new();
        chain.add(HashTransform::new(32, ActivationFn::Linear));
        chain.add(HashTransform::new(32, ActivationFn::ReLU));
        
        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());
        
        let names = chain.get_names();
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn test_gradient_descent() {
        let optimizer = GradientDescent::default();
        
        // Simple quadratic loss: f(x) = x^2
        let loss_fn = |params: &[f64]| params.iter().map(|x| x.powi(2)).sum::<f64>();
        let grad_fn = |params: &[f64]| params.iter().map(|x| 2.0 * x).collect::<Vec<_>>();
        
        let result = optimizer.optimize(vec![10.0, 10.0], loss_fn, grad_fn);
        
        assert!(result.is_ok());
        let opt = result.unwrap();
        assert!(opt.converged || opt.iterations == 1000);
        assert!(opt.best_loss < 100.0); // Should improve from initial 200
    }

    #[test]
    fn test_opcost_transform() {
        let transform = OpCostTransform::new(0.5, 0.3, 0.2);
        let cost = OpCost::new(100.0, 1.0, 0.01);
        
        let score = transform.apply(cost);
        assert!(score > 0.0);
    }

    #[test]
    fn test_nonce_transform() {
        let transform = NonceTransform::new(100, 1000);
        let nonce = Nonce(50);
        
        let result = transform.apply(nonce);
        assert_eq!(result.0, 150);
    }

    #[test]
    fn test_jacobian() {
        let jacobian = Jacobian::new(2, 3);
        let f = |x: &[f64]| vec![x[0] + x[1], x[1] + x[2]];
        let input = vec![1.0, 2.0, 3.0];
        
        let j = jacobian.compute(f, &input, 1e-5);
        assert_eq!(j.len(), 2);
        assert_eq!(j[0].len(), 3);
    }

    #[test]
    fn test_activation_functions() {
        let relu = ActivationFn::ReLU;
        assert_eq!(relu.apply(5.0), 5.0);
        assert_eq!(relu.apply(-5.0), 0.0);
        
        let sigmoid = ActivationFn::Sigmoid;
        let sig_val = sigmoid.apply(0.0);
        assert!((sig_val - 0.5).abs() < 0.001);
        
        let tanh = ActivationFn::Tanh;
        assert_eq!(tanh.apply(0.0), 0.0);
    }

    #[test]
    fn test_differentiable_hash_transform() {
        let transform = HashTransform::new(32, ActivationFn::Sigmoid);
        let hash = Hash::from([1u8; 32]);
        
        let grad = transform.compute_gradient(&hash);
        assert!(grad.is_ok());
        assert_eq!(grad.unwrap().len(), 32);
    }

    #[test]
    fn test_chain_rule() {
        let g1 = [1.0, 2.0, 3.0];
        let g2 = [0.5, 0.5, 0.5];
        
        let result = chain_rule([&g1, &g2]);
        assert_eq!(result, vec![0.5, 1.0, 1.5]);
    }
}
=======

use std::sync::Arc;

/// A generic transformation trait (similar to JAX transformations).
/// Applies a function to an input to produce an output.
pub trait Transform<Input> {
    type Output;
    
    /// Applies the transformation to the input.
    fn apply(&self, input: Input) -> Self::Output;
    
    /// Returns the name of the transformation for logging/debugging.
    fn name(&self) -> &'static str;
}

/// Type alias for a thread-safe, boxed function.
pub type BoxedFunction<Input, Output> = Box<dyn Fn(Input) -> Output + Send + Sync>;

/// Represents a Gradient Transformation.
/// In the context of ChimeraOS, this is used for "Differentiable Hash Approximation"
/// to optimize nonce selection or energy efficiency.
pub struct Grad<Input, Output> {
    /// The function to differentiate or the gradient approximation logic itself.
    pub f: BoxedFunction<Input, Output>,
    /// Arguments indices to differentiate with respect to (if input is a tuple/struct).
    pub argnums: Vec<usize>,
}

// Implementation for Grad allowing it to act as a Transform.
impl<Input, Output> Transform<Input> for Grad<Input, Output> 
where 
    Input: Clone + Send + Sync,
{
    type Output = Output;

    fn apply(&self, input: Input) -> Self::Output {
        // In a real implementation, this would perform automatic differentiation.
        // For chimera-core, we assume 'f' encapsulates the gradient logic 
        // (e.g., a surrogate model from chimera-intelligence).
        (self.f)(input)
    }

    fn name(&self) -> &'static str {
        "grad_transform"
    }
}

/// Represents a Vectorizing Map (vmap in JAX).
/// Used to parallelize mining operations across multiple cores or SIMD lanes.
pub struct VMap<Input, Output> {
    /// The function to apply over a batch of inputs.
    pub f: Arc<dyn Fn(Input) -> Output + Send + Sync>,
}

impl<Input, Output> Transform<Vec<Input>> for VMap<Input, Output>
where
    Input: Clone + Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    type Output = Vec<Output>;

    fn apply(&self, inputs: Vec<Input>) -> Self::Output {
        // Uses parallel iterator (conceptually) or hardware batching.
        // In a real Phase 5 optimization, this would target <100ns latency via SIMD or GPU.
        inputs.into_iter().map(|x| (self.f)(x)).collect()
    }

    fn name(&self) -> &'static str {
        "vmap_transform"
    }
}

/// Helper builder for creating Grad instances.
impl<Input, Output> Grad<Input, Output> {
    pub fn new(f: BoxedFunction<Input, Output>) -> Self {
        Self { f, argnums: vec![0] }
    }

    pub fn with_argnums(mut self, argnums: Vec<usize>) -> Self {
        self.argnums = argnums;
        self
    }
}
>>>>>>> b1c3fa6ecf5982d921dbc44b3f253667a676f19b
