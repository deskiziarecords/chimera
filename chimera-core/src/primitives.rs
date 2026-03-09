
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

// --- Cryptographic Primitives ---

/// A 256-bit cryptographic hash result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// Returns a zeroed hash (often used as a null value).
    pub const fn zero() -> Self {
        Hash([0u8; 32])
    }

    /// Converts the hash to a hexadecimal string representation.
    pub fn to_hex_string(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex_string())
    }
}

/// A 64-bit mining nonce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nonce(pub u64);

impl Nonce {
    /// Increments the nonce by one, wrapping around on overflow.
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

/// A thread-safe wrapper for Nonce, allowing concurrent mining threads
/// to claim unique nonces without lock contention.
pub struct AtomicNonce {
    inner: AtomicU64,
}

impl AtomicNonce {
    pub fn new(start: u64) -> Self {
        Self {
            inner: AtomicU64::new(start),
        }
    }

    /// Fetches the current nonce and increments it by `step` atomically.
    /// Returns the value *before* the increment.
    pub fn fetch_add(&self, step: u64) -> Nonce {
        Nonce(self.inner.fetch_add(step, Ordering::Relaxed))
    }
}

// --- Network & Identity ---

/// Unique identifier for a node in the Chimera mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 16]); // UUID v4 compatible

impl NodeId {
    pub fn generate() -> Self {
        // In a real impl, this would use a proper UUID crate or RNG
        Self([0u8; 16]) 
    }
}

// --- Operational Metrics ---

/// Represents the operational cost of a mining cycle or strategy.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct OpCost {
    pub joules: f64,
    pub seconds: f64,
    pub dollars: f64,
}

impl OpCost {
    /// Calculates efficiency metric: Hashes per Joule (if provided) or Joules per Second (Power).
    pub fn power_draw(&self) -> f64 {
        if self.seconds > 0.0 {
            self.joules / self.seconds
        } else {
            0.0
        }
    }
}

/// Represents the thermal status of a hardware device.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ThermalState {
    pub celsius: f64,
    pub critical_threshold: f64,
    pub throttling: bool,
}

impl ThermalState {
    /// Checks if the device is approaching critical thermal limits.
    pub fn is_critical(&self) -> bool {
        self.celsius >= self.critical_threshold * 0.95
    }
}
