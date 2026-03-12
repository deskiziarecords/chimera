//! ChimeraOS Core
//!
//! Foundational primitives, transforms, and orchestration engine for ChimeraOS.
//!
//! Provides:
//! - Core cryptographic primitives
//! - Differentiable transform system
//! - Mining orchestration via Alchemist

pub mod primitives;
pub mod transforms;
pub mod alchemist;



// ==========================
// Re-exports
// ==========================

// Primitives
pub use primitives::{
    AtomicNonce,
    Hash,
    NodeId,
    Nonce,
    OpCost,
    ThermalState,
};

// Transforms
pub use transforms::{
    Transform,
    Grad,
    TransformChain,
    HashTransform,
    NonceTransform,
    OpCostTransform,
    ActivationFn,
    VMap,
};

// Alchemist
pub use alchemist::{
    Alchemist,
    AlchemistConfig,
    MiningResult,
};



// ==========================
// Crate Information
// ==========================

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const EDITION: &str = "2021";



/// Build metadata for ChimeraOS.
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub rustc: String,
    pub git_hash: String,
}



/// Returns build information.
pub fn build_info() -> BuildInfo {
    BuildInfo {
        name: NAME.to_string(),
        version: VERSION.to_string(),
        edition: EDITION.to_string(),
        rustc: option_env!("VERGEN_RUSTC_SEMVER")
            .unwrap_or("unknown")
            .to_string(),
        git_hash: option_env!("VERGEN_GIT_SHA")
            .unwrap_or("unknown")
            .to_string(),
    }
}



impl std::fmt::Display for BuildInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} v{} (Rust {}, Git: {})",
            self.name, self.version, self.rustc, self.git_hash
        )
    }
}



// ==========================
// Prelude
// ==========================

/// Convenient imports for downstream crates.
pub mod prelude {

    pub use crate::primitives::{
        Hash,
        Nonce,
        NodeId,
        OpCost,
        AtomicNonce,
    };

    pub use crate::transforms::{
        Transform,
        Grad,
        TransformChain,
        HashTransform,
        NonceTransform,
        OpCostTransform,
    };

    pub use crate::alchemist::{
        Alchemist,
        AlchemistConfig,
        MiningResult,
    };
}



// ==========================
// Initialization
// ==========================

/// Initialize ChimeraOS core.
pub fn init() -> Result<(), CoreError> {

    let _ = tracing_subscriber::fmt::try_init();

    tracing::info!("ChimeraOS Core initialized (v{})", VERSION);

    Ok(())
}



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

    #[test]
    fn test_build_info() {
        let info = build_info();
        assert_eq!(info.version, VERSION);
    }

    #[test]
    fn test_primitives() {
        let _hash: Hash = Hash::zero();
        let _nonce: Nonce = Nonce::zero();
        let _cost: OpCost = OpCost::zero();
    }

    #[test]
    fn test_core_init() {
        assert!(init().is_ok());
    }

}
