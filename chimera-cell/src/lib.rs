//! Chimera Cell
//!
//! WASM execution sandbox for ChimeraOS.
//! Provides secure runtime environment for modular mining algorithms.

pub mod sandbox;

use chimera_core::primitives::{Hash, Nonce, OpCost};
use chimera_core::transforms::Transform;
use sandbox::{WasmSandbox, SandboxConfig};
use thiserror::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum CellError {
    #[error("WASM module loading failed: {0}")]
    ModuleLoadFailed(String),
    #[error("Execution timeout: {0}")]
    ExecutionTimeout(String),
    #[error("Memory limit exceeded: {0}")]
    MemoryLimitExceeded(String),
    #[error("Invalid module signature: {0}")]
    InvalidSignature(String),
    #[error("Sandbox initialization failed: {0}")]
    InitializationFailed(String),
}

/// Configuration for the WASM cell runtime.
#[derive(Debug, Clone)]
pub struct CellConfig {
    pub max_memory_mb: usize,
    pub execution_timeout_ms: u64,
    pub enable_profiling: bool,
    pub sandbox_isolation: bool,
}

impl Default for CellConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            execution_timeout_ms: 100, // Target <100ns latency per operation
            enable_profiling: false,
            sandbox_isolation: true,
        }
    }
}

/// Central manager for WASM execution cells.
/// Referenced by the Alchemist engine for strategy execution.
pub struct CellRegistry {
    config: CellConfig,
    sandboxes: Arc<RwLock<Vec<Arc<WasmSandbox>>>>,
    engine: wasmtime::Engine,
}

impl CellRegistry {
    pub fn new(config: CellConfig) -> Result<Self, CellError> {
        let mut engine_config = wasmtime::Config::new();
        engine_config
            .wasm_reference_types(true)
            .wasm_multi_value(true)
            .async_support(true)
            .consume_fuel(true)
            .max_wasm_stack(1024 * 1024); // 1MB stack limit

        if config.sandbox_isolation {
            engine_config
                .cranelift_opt_level(wasmtime::OptLevel::Speed)
                .wasm_simd(true);
        }

        let engine = wasmtime::Engine::new(&engine_config)
            .map_err(|e| CellError::InitializationFailed(e.to_string()))?;

        Ok(Self {
            config,
            sandboxes: Arc::new(RwLock::new(Vec::new())),
            engine,
        })
    }

    /// Load a new WASM module into a sandbox.
    pub async fn load_module(&self, module_bytes: &[u8]) -> Result<Arc<WasmSandbox>, CellError> {
        let sandbox = WasmSandbox::new(&self.engine, module_bytes, &self.config)?;
        let sandbox_arc = Arc::new(sandbox);
        
        let mut sandboxes = self.sandboxes.write().await;
        sandboxes.push(Arc::clone(&sandbox_arc));
        
        tracing::info!("Loaded new WASM module into cell registry");
        Ok(sandbox_arc)
    }

    /// Execute a mining strategy across all available sandboxes.
    pub async fn execute_strategy(
        &self,
        nonce: Nonce,
        data: &[u8],
    ) -> Result<Vec<Hash>, CellError> {
        let sandboxes = self.sandboxes.read().await;
        let mut results = Vec::new();

        for sandbox in sandboxes.iter() {
            let hash = sandbox.execute(nonce, data).await?;
            results.push(hash);
        }

        Ok(results)
    }

    /// Get execution metrics for optimization.
    pub async fn get_execution_metrics(&self) -> Vec<OpCost> {
        let sandboxes = self.sandboxes.read().await;
        sandboxes.iter()
            .map(|s| s.get_last_op_cost())
            .collect()
    }

    /// Clear all loaded sandboxes (for hot-reloading strategies).
    pub async fn clear(&self) {
        let mut sandboxes = self.sandboxes.write().await;
        sandboxes.clear();
        tracing::info!("Cleared all WASM sandboxes");
    }
}

/// Trait for cell-based transformations.
/// Aligns with chimera-core transforms for differentiable optimization.
pub trait CellTransform: Send + Sync {
    fn transform(&self, input: Hash) -> Result<Hash, CellError>;
    fn cost(&self) -> OpCost;
}