
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
