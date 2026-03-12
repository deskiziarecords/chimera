//! Optimized SHA-256 implementation.
//! High-performance hashing engine for ChimeraOS mining and verification.

use chimera_core::primitives::{Hash, Nonce};
use crate::{CryptoError, CryptographicTransform};

use sha2::{Digest, Sha256};

use std::sync::Arc;
use tokio::sync::Mutex;



/// High-performance SHA-256 engine.
///
/// Design goals:
/// - minimal allocations
/// - reusable buffers
/// - batch hashing support
/// - future GPU / FPGA backend compatibility
pub struct Sha256Engine {
    buffer_pool: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl Sha256Engine {

    pub fn new() -> Self {

        Self {
            buffer_pool: Arc::new(Mutex::new(Vec::with_capacity(128))),
        }
    }



    /// Acquire buffer from pool or allocate new.
    async fn acquire_buffer(&self, size: usize) -> Vec<u8> {

        let mut pool = self.buffer_pool.lock().await;

        if let Some(mut buf) = pool.pop() {

            buf.clear();
            buf.reserve(size);
            buf

        } else {

            Vec::with_capacity(size)
        }
    }



    /// Return buffer to pool
    async fn release_buffer(&self, mut buf: Vec<u8>) {

        buf.clear();

        let mut pool = self.buffer_pool.lock().await;

        if pool.len() < 256 {
            pool.push(buf);
        }
    }



    /// Compute SHA-256(data || nonce)
    ///
    /// Target: extremely low allocation overhead for mining loops.
    pub async fn compute(
        &self,
        nonce: Nonce,
        data: &[u8],
    ) -> Result<Hash, CryptoError> {

        let mut buffer =
            self.acquire_buffer(data.len() + 8).await;

        buffer.extend_from_slice(data);
        buffer.extend_from_slice(&nonce.0.to_le_bytes());

        let mut hasher = Sha256::new();
        hasher.update(&buffer);

        let result = hasher.finalize();

        self.release_buffer(buffer).await;

        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&result);

        Ok(Hash(hash_bytes))
    }



    /// Compute hashes for multiple nonces.
    ///
    /// Optimized for batch mining strategies.
    pub async fn compute_batch(
        &self,
        nonces: &[Nonce],
        data: &[u8],
    ) -> Result<Vec<Hash>, CryptoError> {

        let mut results =
            Vec::with_capacity(nonces.len());

        for nonce in nonces {

            let hash =
                self.compute(*nonce, data).await?;

            results.push(hash);
        }

        Ok(results)
    }
}



/// Verify hash meets difficulty target.
///
/// Uses lexicographic comparison.
/// Future phases may replace with big-int comparison.
pub fn verify_difficulty(
    hash: &Hash,
    target: &[u8],
) -> bool {

    let h = &hash.0;

    let len = target.len().min(h.len());

    for i in 0..len {

        if h[i] < target[i] {
            return true;
        }

        if h[i] > target[i] {
            return false;
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

        let hash =
            engine.compute(nonce, data).await.unwrap();

        assert_eq!(hash.0.len(), 32);

        assert!(
            hash.0.iter().any(|&b| b != 0)
        );
    }



    #[tokio::test]
    async fn test_batch_computation() {

        let engine = Sha256Engine::new();

        let nonces =
            vec![Nonce(1), Nonce(2), Nonce(3)];

        let data = b"batch_test";

        let hashes =
            engine.compute_batch(&nonces, data)
                .await
                .unwrap();

        assert_eq!(hashes.len(), 3);
    }
}
