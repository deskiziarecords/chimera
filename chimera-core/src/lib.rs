//! # ChimeraOS Core
//!
//! The foundational library for the ChimeraOS orchestration system.
//! 
//! This crate provides:
//! - **Primitives**: Core data types (`Hash`, `Nonce`, `OpCost`) for cryptographic operations.
//! - **Transforms**: JAX-style differentiable programming traits and structures.
//! - **Alchemist**: The LLM-driven engine for parsing natural language into mining strategies.
//!
//! ## Example
//!
//! ```rust
//! use chimera_core::{Hash, Nonce, Alchemist};
//!
//! // Basic primitive usage
//! let hash = Hash::zero();
//! println!("Zero hash: {}", hash);
//! ```

// --- Public Module Declarations ---
pub mod alchemist;
pub mod primitives;
pub mod transforms;

// --- Convenience Re-exports ---
// This allows users to use `chimera_core::Hash` instead of `chimera_core::primitives::Hash`.

// Primitives
pub use primitives::{AtomicNonce, Hash, NodeId, Nonce, OpCost, ThermalState};

// Transforms
pub use transforms::{BoxedFunction, Grad, Transform, VMap};

// Alchemist Engine
pub use alchemist::{Alchemist, AlchemistConfig, AlchemistError, LanguageModel, MiningStrategy};

// External Re-exports (optional, commonly used by consumers)
pub use serde::{Deserialize, Serialize};
pub use thiserror::Error;

// --- Crate Information ---
pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");