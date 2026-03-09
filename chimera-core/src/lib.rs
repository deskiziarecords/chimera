//! ChimeraOS Core
//!
//! Foundational primitives, transforms, and orchestration engine for ChimeraOS.
//! Provides standard data structures, operational metrics, and intelligent strategy generation.

pub mod primitives;
pub mod transforms;
pub mod alchemist;

// Re-export commonly used types for convenience
pub use primitives::{
    Hash,
    Nonce,
    NodeId,
    OpCost,
    ThermalState,
    Difficulty,
    BlockHeader,
    MiningResult,
    MiningStrategy,
    FleetStats,
    AtomicNonce,
    PrimitiveError,
};

pub use transforms::{
    Transform,
    Grad,
    DifferentiableFn,
    TransformChain,
    HashTransform,
    OpCostTransform,
    NonceTransform,
    GradientDescent,
    DifferentiableTransform,
    Jacobian,
    ActivationFn,
    OptimizationResult,
    TransformError,
};

pub use alchemist::{
    Alchemist,
    AlchemistConfig,
    AlchemistError,
    AlchemistTelemetry,
    IntentSpec,
    Constraint,
    OptimizationPriority,
    OptimizationBackend,
    AlgorithmProfile,
    MiningSessionHandle,
    SessionStatus,
    SafetyGuard,
    LanguageModel,
};

/// ChimeraOS version information.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// ChimeraOS edition.
pub const EDITION: &str = "2021";

/// Get ChimeraOS build information.
pub fn build_info() -> BuildInfo {
    BuildInfo {
        version: VERSION.to_string(),
        edition: EDITION.to_string(),
        rustc: option_env!("VERGEN_RUSTC_SEMVER").unwrap_or("unknown").to_string(),
        git_hash: option_env!("VERGEN_GIT_SHA").unwrap_or("unknown").to_string(),
        build_timestamp: option_env!("VERGEN_BUILD_TIMESTAMP").unwrap_or("unknown").to_string(),
    }
}

/// Build information structure.
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub version: String,
    pub edition: String,
    pub rustc: String,
    pub git_hash: String,
    pub build_timestamp: String,
}

impl std::fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ChimeraOS v{} (Rust {}, Git: {})",
            self.version, self.rustc, self.git_hash
        )
    }
}

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::primitives::{
        Hash, Nonce, NodeId, OpCost, ThermalState, Difficulty, BlockHeader,
        MiningResult, MiningStrategy, FleetStats, AtomicNonce,
    };
    pub use crate::transforms::{
        Transform, Grad, TransformChain, HashTransform, GradientDescent,
    };
    pub use crate::alchemist::{
        Alchemist, AlchemistConfig, IntentSpec, Constraint, OptimizationPriority,
    };
}

/// Initialize ChimeraOS core with default configuration.
pub fn init() -> Result<(), CoreError> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    tracing::info!("ChimeraOS Core initialized (v{})", VERSION);
    Ok(())
}

/// Core-specific error types.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Module error: {0}")]
    ModuleError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Hash, Nonce, OpCost};
    use transforms::{HashTransform, ActivationFn};

    #[test]
    fn test_build_info() {
        let info = build_info();
        assert_eq!(info.version, VERSION);
        assert_eq!(info.edition, EDITION);
    }

    #[test]
    fn test_prelude_imports() {
        // Verify prelude exports work correctly
        let _hash: Hash = Hash::zero();
        let _nonce: Nonce = Nonce::zero();
        let _cost: OpCost = OpCost::zero();
    }

    #[test]
    fn test_core_init() {
        let result = init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_hash_transform_via_prelude() {
        use prelude::HashTransform;
        let transform = HashTransform::new(32, ActivationFn::ReLU);
        let hash = Hash::zero();
        let output = transform.apply(hash);
        assert_eq!(output.len(), 32);
    }

    #[test]
    fn test_version_constants() {
        assert!(!VERSION.is_empty());
        assert_eq!(EDITION, "2021");
    }
}