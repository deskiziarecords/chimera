//! ChimeraOS Alchemist
//!
//! The Alchemist is responsible for transmuting raw block headers into valid
//! proof-of-work results. It orchestrates hashing attempts, nonce management,
//! and difficulty validation for mining nodes.

use crate::{
    AtomicNonce, BlockHeader, Difficulty, Hash, MiningResult, NodeId, PrimitiveError,
};

use sha2::{Digest, Sha256};
use std::time::Instant;

/// Mining controller responsible for executing hash attempts.
pub struct Alchemist {
    /// Node performing the mining.
    pub node_id: NodeId,

    /// Global nonce generator (thread-safe).
    pub nonce_counter: AtomicNonce,
}

impl Alchemist {
    /// Create a new Alchemist instance.
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            nonce_counter: AtomicNonce::default(),
        }
    }

    /// Perform a single SHA-256 hash of a block header.
    fn hash_header(header: &BlockHeader) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(header.serialize());
        let result = hasher.finalize();

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);

        Hash(bytes)
    }

    /// Execute mining until a valid hash is found or attempt limit is reached.
    pub fn mine(
        &self,
        mut header: BlockHeader,
        max_attempts: u64,
    ) -> Result<MiningResult, PrimitiveError> {
        let start = Instant::now();

        let mut attempts: u64 = 0;

        while attempts < max_attempts {
            let nonce = self.nonce_counter.next();
            header.nonce = nonce;

            let hash = Self::hash_header(&header);

            attempts += 1;

            if header.difficulty.meets_difficulty(&hash) {
                let elapsed = start.elapsed().as_secs_f64();

                return Ok(MiningResult::new(
                    hash,
                    nonce,
                    elapsed,
                    attempts,
                    self.node_id,
                ));
            }
        }

        Err(PrimitiveError::DifficultyNotMet {
            hash: Hash::zero(),
            target: header.difficulty.to_target(),
        })
    }

    /// Verify that a mining result actually satisfies the block difficulty.
    pub fn verify_result(
        header: &BlockHeader,
        result: &MiningResult,
    ) -> Result<(), PrimitiveError> {
        let mut header = header.clone();
        header.nonce = result.nonce;

        let hash = Self::hash_header(&header);

        if hash != result.hash {
            return Err(PrimitiveError::InvalidParameter(
                "Hash mismatch".to_string(),
            ));
        }

        if !header.difficulty.meets_difficulty(&hash) {
            return Err(PrimitiveError::DifficultyNotMet {
                hash,
                target: header.difficulty.to_target(),
            });
        }

        Ok(())
    }

    /// Benchmark hashing performance for the current node.
    pub fn benchmark(&self, iterations: u64) -> f64 {
        let mut header = BlockHeader::new(
            Hash::zero(),
            Hash::zero(),
            0,
            Difficulty::default(),
        );

        let start = Instant::now();

        for _ in 0..iterations {
            header.nonce = self.nonce_counter.next();
            Self::hash_header(&header);
        }

        let elapsed = start.elapsed().as_secs_f64();

        if elapsed == 0.0 {
            return 0.0;
        }

        iterations as f64 / elapsed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Difficulty, Hash};

    #[test]
    fn test_hashing() {
        let header = BlockHeader::new(
            Hash::zero(),
            Hash::zero(),
            0,
            Difficulty::default(),
        );

        let hash = Alchemist::hash_header(&header);

        assert!(!hash.is_zero());
    }

    #[test]
    fn test_benchmark() {
        let alchemist = Alchemist::new(NodeId::default());

        let rate = alchemist.benchmark(1000);

        assert!(rate > 0.0);
    }

    #[test]
    fn test_mining_attempt() {
        let alchemist = Alchemist::new(NodeId::default());

        let header = BlockHeader::new(
            Hash::zero(),
            Hash::zero(),
            0,
            Difficulty::default(),
        );

        let result = alchemist.mine(header, 1_000_000);

        assert!(result.is_ok());
    }
}
