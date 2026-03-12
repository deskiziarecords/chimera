//! ChimeraOS Core Primitives
//!
//! Defines fundamental types used across the Chimera system.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash as StdHash;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

//
// HASH
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, StdHash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn zero() -> Self {
        Hash([0u8; 32])
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self, PrimitiveError> {
        let bytes = hex::decode(s).map_err(|e| PrimitiveError::InvalidHex(e.to_string()))?;

        if bytes.len() != 32 {
            return Err(PrimitiveError::InvalidHashLength(bytes.len()));
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);

        Ok(Hash(arr))
    }

    /// Count leading zero bits.
    pub fn leading_zeros(&self) -> u32 {
        let mut count = 0;

        for byte in &self.0 {
            if *byte == 0 {
                count += 8;
            } else {
                count += byte.leading_zeros();
                break;
            }
        }

        count
    }

    pub fn meets_difficulty(&self, target: &Hash) -> bool {
        self <= target
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }
}

impl From<Hash> for [u8; 32] {
    fn from(hash: Hash) -> Self {
        hash.0
    }
}

//
// NONCE
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, StdHash, Serialize, Deserialize)]
pub struct Nonce(pub u64);

impl Nonce {
    pub fn zero() -> Self {
        Nonce(0)
    }

    pub fn increment(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(1);
        self.0
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Default for Nonce {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Nonce {
    fn from(v: u64) -> Self {
        Nonce(v)
    }
}

impl From<Nonce> for u64 {
    fn from(n: Nonce) -> Self {
        n.0
    }
}

//
// NODE ID
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, StdHash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 32]);

impl NodeId {
    pub fn zero() -> Self {
        NodeId([0u8; 32])
    }

    pub fn random() -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];

        rng.fill(&mut bytes);

        NodeId(bytes)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex(s: &str) -> Result<Self, PrimitiveError> {
        let bytes = hex::decode(s).map_err(|e| PrimitiveError::InvalidHex(e.to_string()))?;

        if bytes.len() != 32 {
            return Err(PrimitiveError::InvalidNodeIdLength(bytes.len()));
        }

        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);

        Ok(NodeId(arr))
    }

    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    pub fn short_id(&self) -> String {
        let hex = self.to_hex();
        hex.get(0..8).unwrap_or(&hex).to_string()
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_id())
    }
}

//
// DIFFICULTY
//

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Difficulty(pub [u8; 32]);

impl Difficulty {
    pub fn from_target(target: Hash) -> Self {
        Difficulty(target.0)
    }

    pub fn to_target(&self) -> Hash {
        Hash(self.0)
    }

    pub fn meets_difficulty(&self, hash: &Hash) -> bool {
        hash.meets_difficulty(&self.to_target())
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        let mut target = [0xffu8; 32];
        target[0] = 0;

        Difficulty(target)
    }
}

//
// BLOCK HEADER
//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub prev_hash: Hash,
    pub merkle_root: Hash,
    pub timestamp: u64,
    pub difficulty: Difficulty,
    pub nonce: Nonce,
    pub version: u32,
}

impl BlockHeader {
    pub fn new(
        prev_hash: Hash,
        merkle_root: Hash,
        timestamp: u64,
        difficulty: Difficulty,
    ) -> Self {
        Self {
            prev_hash,
            merkle_root,
            timestamp,
            difficulty,
            nonce: Nonce::zero(),
            version: 1,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(116);

        bytes.extend_from_slice(&self.prev_hash.0);
        bytes.extend_from_slice(&self.merkle_root.0);
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.difficulty.0);
        bytes.extend_from_slice(&self.nonce.0.to_le_bytes());
        bytes.extend_from_slice(&self.version.to_le_bytes());

        bytes
    }

    pub fn increment_nonce(&mut self) -> u64 {
        self.nonce.increment()
    }

    pub fn reset_nonce(&mut self) {
        self.nonce = Nonce::zero();
    }
}

//
// MINING RESULT
//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningResult {
    pub hash: Hash,
    pub nonce: Nonce,
    pub time_seconds: f64,
    pub attempts: u64,
    pub node_id: NodeId,
}

impl MiningResult {
    pub fn new(hash: Hash, nonce: Nonce, time_seconds: f64, attempts: u64, node_id: NodeId) -> Self {
        Self {
            hash,
            nonce,
            time_seconds,
            attempts,
            node_id,
        }
    }

    pub fn effective_hashrate(&self) -> f64 {
        if self.time_seconds == 0.0 {
            return 0.0;
        }

        self.attempts as f64 / self.time_seconds
    }
}

//
// ATOMIC NONCE
//

pub struct AtomicNonce {
    counter: AtomicU64,
}

impl AtomicNonce {
    pub fn new(initial: u64) -> Self {
        Self {
            counter: AtomicU64::new(initial),
        }
    }

    pub fn next(&self) -> Nonce {
        Nonce(self.counter.fetch_add(1, Ordering::SeqCst))
    }

    pub fn current(&self) -> Nonce {
        Nonce(self.counter.load(Ordering::SeqCst))
    }

    pub fn reset(&self, value: u64) {
        self.counter.store(value, Ordering::SeqCst);
    }
}

impl Default for AtomicNonce {
    fn default() -> Self {
        Self::new(0)
    }
}

//
// ERRORS
//

#[derive(Error, Debug)]
pub enum PrimitiveError {
    #[error("Invalid hex string: {0}")]
    InvalidHex(String),

    #[error("Invalid hash length: expected 32 bytes, got {0}")]
    InvalidHashLength(usize),

    #[error("Invalid NodeId length: expected 32 bytes, got {0}")]
    InvalidNodeIdLength(usize),

    #[error("Difficulty not met: hash {hash} > target {target}")]
    DifficultyNotMet { hash: Hash, target: Hash },

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}
