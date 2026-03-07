use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nonce(pub u64);

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct OpCost {
    pub joules: f64,
    pub seconds: f64,
    pub dollars: f64,
}