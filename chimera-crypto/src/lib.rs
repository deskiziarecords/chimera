//! Chimera Crypto
//!
//! Cryptographic hash implementations for ChimeraOS.
//! Provides optimized SHA-256 and interfaces for hardware-accelerated hashing (GPU/FPGA).

pub mod sha256;

pub use sha256::{Sha256Engine, verify_difficulty};

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
///
/// Aligns with `chimera-core` transforms and distributed execution.
#[async_trait::async_trait]
pub trait CryptographicTransform: Send + Sync {

    /// Compute hash for a given nonce and data.
    async fn compute(
        &self,
        nonce: Nonce,
        data: &[u8],
    ) -> Result<Hash, CryptoError>;



    /// Batch compute for parallel processing (CPU/GPU).
    async fn compute_batch(
        &self,
        nonces: &[Nonce],
        data: &[u8],
    ) -> Result<Vec<Hash>, CryptoError>;



    /// Estimate operational cost (Energy/Time).
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

            // Phase 1: CPU hashing
            use_hardware_acceleration: false,

            preferred_memory_region: MemoryRegionType::Host,

            // Phase 5 target
            target_latency_ns: 100.0,
        }
    }
}



/// Central manager for cryptographic operations.
pub struct CryptoEngine {

    config: CryptoConfig,

    sha256_impl: Sha256Engine,
}



impl CryptoEngine {

    pub fn new(config: CryptoConfig) -> Self {

        Self {

            config,

            sha256_impl: Sha256Engine::new(),
        }
    }



    /// Select the best hashing backend based on Fabric capabilities.
    ///
    /// Future versions will auto-switch between:
    /// - CPU SIMD
    /// - GPU kernels
    /// - FPGA SST pipelines
    pub async fn optimize_for_fabric(
        &mut self,
        _fabric: &chimera_fabric::FabricManager,
    ) {

        tracing::info!(
            "Optimizing crypto engine for available fabric..."
        );

        // Phase 1: CPU
        self.config.use_hardware_acceleration = false;
    }
}



#[async_trait::async_trait]
impl CryptographicTransform for CryptoEngine {

    async fn compute(
        &self,
        nonce: Nonce,
        data: &[u8],
    ) -> Result<Hash, CryptoError> {

        self.sha256_impl
            .compute(nonce, data)
            .await
    }



    async fn compute_batch(
        &self,
        nonces: &[Nonce],
        data: &[u8],
    ) -> Result<Vec<Hash>, CryptoError> {

        self.sha256_impl
            .compute_batch(nonces, data)
            .await
    }



    fn estimate_cost(
        &self,
        iterations: u64,
    ) -> OpCost {

        // Placeholder until telemetry feedback loop exists

        let joules =
            iterations as f64 * 0.0000001;

        let seconds =
            iterations as f64 * 0.0000000001;

        let dollars =
            joules * 0.0001;

        OpCost {
            joules,
            seconds,
            dollars,
        }
    }
}
