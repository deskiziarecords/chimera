//! SIMD hash batching for Chimera miners.
//!
//! Provides CPU vectorized hashing pipelines for evaluating many nonces
//! simultaneously.

use crate::primitives::{Hash, Nonce};


/// Maximum batch size for SIMD evaluation.
pub const SIMD_BATCH: usize = 8;



/// SIMD nonce batch.
#[derive(Debug, Clone)]
pub struct NonceBatch {
    pub nonces: [Nonce; SIMD_BATCH],
}



impl NonceBatch {

    pub fn new(start: Nonce) -> Self {

        let mut arr = [Nonce(0); SIMD_BATCH];

        for i in 0..SIMD_BATCH {
            arr[i] = Nonce(start.0 + i as u64);
        }

        Self { nonces: arr }
    }
}



/// SIMD hash batch result.
#[derive(Debug, Clone)]
pub struct HashBatch {
    pub hashes: [Hash; SIMD_BATCH],
}



/// Generic SIMD batch hash executor.
pub struct SimdHasher<F>
where
    F: Fn(Nonce) -> Hash + Send + Sync,
{
    hash_fn: F,
}



impl<F> SimdHasher<F>
where
    F: Fn(Nonce) -> Hash + Send + Sync,
{

    pub fn new(hash_fn: F) -> Self {
        Self { hash_fn }
    }



    pub fn hash_batch(&self, batch: &NonceBatch) -> HashBatch {

        let mut hashes = [Hash::zero(); SIMD_BATCH];

        for i in 0..SIMD_BATCH {
            hashes[i] = (self.hash_fn)(batch.nonces[i]);
        }

        HashBatch { hashes }
    }
}
