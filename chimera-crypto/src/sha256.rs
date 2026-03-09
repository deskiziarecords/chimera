//! Optimized SHA-256 implementation.
//! Wraps high-performance primitives while adhering to ChimeraOS type safety.

use chimera_core::primitives::{Hash, Nonce};
use crate::{CryptoError, CryptographicTransform};
use sha2::{Sha256, Digest};
use std::sync::Arc;

/// Internal high-performance SHA-256 engine.
/// In Phase 3+, this will swap implementations based on SST (FPGA) or GPU availability.
pub struct Sha256Engine {
    // Pre-allocated buffer for performance to reduce allocations during mining loops
    buffer_pool: Arc<tokio::sync::Mutex<Vec<Vec<u8>>>>,
}

impl Sha256Engine {
    pub fn new() -> Self {
        Self {
            buffer_pool: Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(100))),
        }
    }

    /// Computes SHA-256 hash of data + nonce.
    /// Target: <100ns latency per hash on modern CPU.
    pub fn compute(&self, nonce: Nonce, data: &[u8]) -> Result<Hash, CryptoError> {
        // Prepare input: Data + Nonce
        // Optimization: Avoid unnecessary allocations in tight loops
        let mut input = Vec::with_capacity(data.len() + 8);
        input.extend_from_slice(data);
        input.extend_from_slice(&nonce.0.to_le_bytes());

        let mut hasher = Sha256::new();
        hasher.update(&input);
        let result = hasher.finalize();

        // Convert to Chimera Core Hash primitive
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&result);
        
        Ok(Hash(hash_bytes))
    }

    /// Batch computation for parallel mining strategies.
    pub fn compute_batch(&self, nonces: &[Nonce], data: &[u8]) -> Result<Vec<Hash>, CryptoError> {
        let mut results = Vec::with_capacity(nonces.len());
        
        for nonce in nonces {
            results.push(self.compute(*nonce, data)?);
        }
        
        Ok(results)
    }
}

/// Verifies if a hash meets a specific difficulty target.
/// Used by the Alchemist engine to validate mining strategies.
pub fn verify_difficulty(hash: &Hash, target: &[u8]) -> bool {
    // Simple leading zero comparison for Phase 1
    // Phase 3+ will use full big-int comparison
    for (i, &byte) in hash.0.iter().enumerate() {
        if i >= target.len() {
            break;
        }
        if byte > target[i] {
            return false;
        }
        if byte < target[i] {
            return true;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_core::primitives::Nonce;

    #[tokio::test]
    async fn test_sha256_computation() {
        let engine = Sha256Engine::new();
        let nonce = Nonce(12345);
        let data = b"chimera_block_data";
        
        let hash = engine.compute(nonce, data).unwrap();
        
        // Ensure hash is 32 bytes
        assert_eq!(hash.0.len(), 32);
        // Ensure hash is not all zeros (probabilistic)
        assert!(hash.0.iter().any(|&b| b != 0));
    }

    #[tokio::test]
    async fn test_batch_computation() {
        let engine = Sha256Engine::new();
        let nonces = vec![Nonce(1), Nonce(2), Nonce(3)];
        let data = b"batch_test";
        
        let hashes = engine.compute_batch(&nonces, data).unwrap();
        assert_eq!(hashes.len(), 3);
    }
}