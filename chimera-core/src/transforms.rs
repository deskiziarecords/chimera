//! JAX-Style Differentiable Transforms for ChimeraOS
//!
//! Provides composable, differentiable transformations for mining,
//! optimization, and ML-assisted cryptographic exploration.

use crate::primitives::{Hash, Nonce, OpCost};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;


/// =========================
/// Errors
/// =========================

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
}



/// =========================
/// Core Transform Trait
/// =========================

pub trait Transform<Input>: Send + Sync {
    type Output;

    fn apply(&self, input: Input) -> Self::Output;

    fn name(&self) -> &'static str;

    fn gradient(&self, _input: &Input) -> Option<Grad<Input, Self::Output>> {
        None
    }

    fn cost(&self) -> OpCost {
        OpCost::default()
    }
}



/// =========================
/// Gradient Structure
/// =========================

pub struct Grad<Input, Output> {
    pub f: Arc<dyn Fn(&Input) -> Output + Send + Sync>,
    pub argnums: Vec<usize>,
    pub values: Option<Vec<f64>>,
}

impl<Input, Output> Grad<Input, Output> {
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

    pub fn compute(&mut self, _input: &Input) -> Result<(), TransformError> {
        self.values = Some(vec![1.0; self.argnums.len()]);
        Ok(())
    }

    pub fn values(&self) -> Option<&Vec<f64>> {
        self.values.as_ref()
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



/// =========================
/// Vectorized Map (JAX vmap)
/// =========================
///
/// Enables batch execution of transforms.
/// Useful for parallel mining or GPU batching.

pub struct VMap<Input, Output> {
    pub f: Arc<dyn Fn(Input) -> Output + Send + Sync>,
}

impl<Input, Output> Transform<Vec<Input>> for VMap<Input, Output>
where
    Input: Clone + Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    type Output = Vec<Output>;

    fn apply(&self, inputs: Vec<Input>) -> Self::Output {
        inputs.into_iter().map(|x| (self.f)(x)).collect()
    }

    fn name(&self) -> &'static str {
        "vmap_transform"
    }
}



/// =========================
/// Transform Chain
/// =========================

pub struct TransformChain<Input> {
    transforms: Vec<Arc<dyn Transform<Input, Output = Input> + Send + Sync>>,
}

impl<Input> TransformChain<Input> {
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }

    pub fn add<T>(&mut self, transform: T)
    where
        T: Transform<Input, Output = Input> + Send + Sync + 'static,
    {
        self.transforms.push(Arc::new(transform));
    }

    pub fn apply(&self, mut input: Input) -> Input {
        for t in &self.transforms {
            input = t.apply(input);
        }
        input
    }

    pub fn len(&self) -> usize {
        self.transforms.len()
    }
}

impl<Input> Default for TransformChain<Input> {
    fn default() -> Self {
        Self::new()
    }
}



/// =========================
/// Activation Functions
/// =========================

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



/// =========================
/// Hash Transform
/// =========================

pub struct HashTransform {
    weights: Vec<f64>,
    bias: f64,
    activation: ActivationFn,
}

impl HashTransform {
    pub fn new(dim: usize, activation: ActivationFn) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        Self {
            weights: (0..dim).map(|_| rng.gen_range(-1.0..1.0)).collect(),
            bias: rng.gen_range(-0.1..0.1),
            activation,
        }
    }

    pub fn compute_approx(&self, hash: &Hash) -> Vec<f64> {
        let mut out = Vec::with_capacity(self.weights.len());

        for (i, &w) in self.weights.iter().enumerate() {
            let idx = i % hash.0.len();
            let x = (hash.0[idx] as f64 / 255.0) * w + self.bias;
            out.push(self.activation.apply(x));
        }

        out
    }
}

impl Transform<Hash> for HashTransform {
    type Output = Vec<f64>;

    fn apply(&self, input: Hash) -> Self::Output {
        self.compute_approx(&input)
    }

    fn name(&self) -> &'static str {
        "hash_transform"
    }
}



/// =========================
/// Nonce Transform
/// =========================

pub struct NonceTransform {
    pub stride: u64,
    pub max_nonce: u64,
}

impl NonceTransform {
    pub fn new(stride: u64, max_nonce: u64) -> Self {
        Self { stride, max_nonce }
    }
}

impl Transform<Nonce> for NonceTransform {
    type Output = Nonce;

    fn apply(&self, input: Nonce) -> Self::Output {
        Nonce((input.0 + self.stride) % self.max_nonce)
    }

    fn name(&self) -> &'static str {
        "nonce_transform"
    }
}



/// =========================
/// OpCost Transform
/// =========================

pub struct OpCostTransform {
    pub joules_weight: f64,
    pub seconds_weight: f64,
    pub dollars_weight: f64,
}

impl OpCostTransform {
    pub fn new(j: f64, s: f64, d: f64) -> Self {
        Self {
            joules_weight: j,
            seconds_weight: s,
            dollars_weight: d,
        }
    }

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
        "opcost_transform"
    }
}
