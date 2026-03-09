//! Chimera Crypto
//!
//! Cryptographic hash implementations for ChimeraOS.
//! Provides optimized SHA-256 and interfaces for hardware-accelerated hashing (GPU/FPGA).

pub mod sha256;

use chimera_core::primitives::{Hash, Nonce, OpCost};
use chimera_fabric::memory::MemoryRegionType;
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Hash computation failed: {0}")]
    ComputationFailed(String),
    #[error("Hardware acceleration unavailable: {0}")]
    HardwareUnavailable(String),
    #[error("Invalid input size: {0}")]
    InvalidInputSize(String),
    #[error("Memory allocation failed in crypto context: {0}")]
    MemoryError(String),
}

/// Trait for cryptographic operations within the Chimera ecosystem.
/// Aligns with `chimera-core` transforms for differentiable optimization.
pub trait CryptographicTransform {
    /// Compute hash for a given nonce and data.
    fn compute(&self, nonce: Nonce, data: &[u8]) -> Result<Hash, CryptoError>;
    
    /// Batch compute for parallel processing (CPU/GPU).
    fn compute_batch(&self, nonces: &[Nonce], data: &[u8]) -> Result<Vec<Hash>, CryptoError>;
    
    /// Estimate operational cost (Energy/Time) for this operation.
    fn estimate_cost(&self, iterations: u64) -> OpCost;
}

/// Configuration for the crypto engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub use_hardware_acceleration: bool,
    pub preferred_memory_region: MemoryRegionType,
    pub target_latency_ns: f64,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            use_hardware_acceleration: false, // Phase 1: CPU only
            preferred_memory_region: MemoryRegionType::Host,
            target_latency_ns: 100.0, // Phase 5 Goal: <100ns
        }
    }
}

/// Central manager for cryptographic operations.
pub struct CryptoEngine {
    config: CryptoConfig,
    sha256_impl: sha256::Sha256Engine,
}

impl CryptoEngine {
    pub fn new(config: CryptoConfig) -> Self {
        Self {
            config,
            sha256_impl: sha256::Sha256Engine::new(),
        }
    }

    /// Selects the best available hashing strategy based on Fabric capabilities.
    pub async fn optimize_for_fabric(&mut self, fabric_caps: &chimera_fabric::FabricManager) {
        // Phase 3: Check for FPGA (SST) or GPU availability
        // For now, defaults to CPU implementation
        tracing::info!("Optimizing crypto engine for available fabric...");
        self.config.use_hardware_acceleration = false; 
    }
}

impl CryptographicTransform for CryptoEngine {
    fn compute(&self, nonce: Nonce, data: &[u8]) -> Result<Hash, CryptoError> {
        self.sha256_impl.compute(nonce, data)
    }

    fn compute_batch(&self, nonces: &[Nonce], data: &[u8]) -> Result<Vec<Hash>, CryptoError> {
        self.sha256_impl.compute_batch(nonces, data)
    }

    fn estimate_cost(&self, iterations: u64) -> OpCost {
        // Phase 5 Optimization: Fine-tune these metrics based on actual hardware
        let joules = iterations as f64 * 0.0000001; // Estimate per hash
        let seconds = iterations as f64 * 0.0000000001; // Estimate 10ns per hash
        let dollars = joules * 0.0001; // Energy cost

        OpCost {
            joules,
            seconds,
            dollars,
        }
    }
}