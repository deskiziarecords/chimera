//! Alchemist
//!
//! Core orchestrator for ChimeraOS mining pipelines.
//! Combines transforms, nonce exploration, and cost optimization
//! to discover optimal mining strategies.

use crate::primitives::{Hash, Nonce, OpCost};
use crate::transforms::{
    ActivationFn, HashTransform, NonceTransform, OpCostTransform, Transform, TransformChain,
};

use serde::{Deserialize, Serialize};
use std::sync::Arc;


/// Result returned by a mining attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningResult {
    pub nonce: Nonce,
    pub hash: Hash,
    pub score: f64,
    pub cost: OpCost,
}



/// Alchemist configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemistConfig {
    pub hash_dim: usize,
    pub nonce_stride: u64,
    pub max_nonce: u64,
    pub activation: ActivationFn,
}

impl Default for AlchemistConfig {
    fn default() -> Self {
        Self {
            hash_dim: 32,
            nonce_stride: 1,
            max_nonce: u64::MAX,
            activation: ActivationFn::ReLU,
        }
    }
}



/// Alchemist miner core.
pub struct Alchemist {

    /// Transform approximating hash behavior
    hash_transform: HashTransform,

    /// Nonce stepping strategy
    nonce_transform: NonceTransform,

    /// Cost scoring transform
    cost_transform: OpCostTransform,

    /// Hash transform pipeline
    hash_chain: TransformChain<Hash>,

    /// Nonce transform pipeline
    nonce_chain: TransformChain<Nonce>,
}

impl Alchemist {

    /// Create a new alchemist instance.
    pub fn new(config: AlchemistConfig) -> Self {

        let hash_transform =
            HashTransform::new(config.hash_dim, config.activation);

        let nonce_transform =
            NonceTransform::new(config.nonce_stride, config.max_nonce);

        let cost_transform =
            OpCostTransform::new(0.5, 0.3, 0.2);

        let mut hash_chain = TransformChain::new();
        let mut nonce_chain = TransformChain::new();

        hash_chain.add(hash_transform.clone());
        nonce_chain.add(nonce_transform.clone());

        Self {
            hash_transform,
            nonce_transform,
            cost_transform,
            hash_chain,
            nonce_chain,
        }
    }



    /// Compute approximate hash score.
    pub fn evaluate_hash(&self, hash: Hash) -> f64 {

        let approx = self.hash_chain.apply(hash);

        approx.iter().sum::<f64>() / approx.len() as f64
    }



    /// Evaluate operational cost score.
    pub fn evaluate_cost(&self, cost: OpCost) -> f64 {
        self.cost_transform.apply(cost)
    }



    /// Run a single mining step.
    pub fn step(
        &self,
        nonce: Nonce,
        hash_fn: Arc<dyn Fn(Nonce) -> Hash + Send + Sync>,
        cost_fn: Arc<dyn Fn() -> OpCost + Send + Sync>,
    ) -> MiningResult {

        let next_nonce = self.nonce_chain.apply(nonce);

        let hash = hash_fn(next_nonce);

        let score = self.evaluate_hash(hash.clone());

        let cost = cost_fn();

        MiningResult {
            nonce: next_nonce,
            hash,
            score,
            cost,
        }
    }



    /// Run a mining loop.
    pub fn mine(
        &self,
        start_nonce: Nonce,
        iterations: u64,
        hash_fn: Arc<dyn Fn(Nonce) -> Hash + Send + Sync>,
        cost_fn: Arc<dyn Fn() -> OpCost + Send + Sync>,
    ) -> MiningResult {

        let mut best: Option<MiningResult> = None;
        let mut nonce = start_nonce;

        for _ in 0..iterations {

            let result = self.step(nonce, hash_fn.clone(), cost_fn.clone());

            if let Some(ref best_res) = best {
                if result.score < best_res.score {
                    best = Some(result.clone());
                }
            } else {
                best = Some(result.clone());
            }

            nonce = result.nonce;
        }

        best.expect("Mining loop must produce at least one result")
    }



    /// Run batch mining (vectorized).
    pub fn mine_batch(
        &self,
        nonces: Vec<Nonce>,
        hash_fn: Arc<dyn Fn(Nonce) -> Hash + Send + Sync>,
    ) -> Vec<f64> {

        nonces
            .into_iter()
            .map(|n| {
                let h = hash_fn(n);
                self.evaluate_hash(h)
            })
            .collect()
    }
}



#[cfg(test)]
mod tests {

    use super::*;
    use crate::primitives::{Hash, Nonce, OpCost};

    fn dummy_hash(nonce: Nonce) -> Hash {
        let mut data = [0u8; 32];
        data[0] = (nonce.0 % 255) as u8;
        Hash::from(data)
    }

    fn dummy_cost() -> OpCost {
        OpCost::new(1.0, 0.1, 0.001)
    }

    #[test]
    fn test_alchemist_step() {

        let config = AlchemistConfig::default();
        let alchemist = Alchemist::new(config);

        let result = alchemist.step(
            Nonce(1),
            Arc::new(dummy_hash),
            Arc::new(dummy_cost),
        );

        assert!(result.score >= 0.0);
    }

    #[test]
    fn test_alchemist_mining() {

        let config = AlchemistConfig::default();
        let alchemist = Alchemist::new(config);

        let result = alchemist.mine(
            Nonce(1),
            100,
            Arc::new(dummy_hash),
            Arc::new(dummy_cost),
        );

        assert!(result.score >= 0.0);
    }

}
