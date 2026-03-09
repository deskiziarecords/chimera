use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

// --- Error Handling ---

#[derive(Debug, Error)]
pub enum AlchemistError {
    #[error("Failed to parse natural language intent: {0}")]
    ParseError(String),
    
    #[error("Strategy generation failed: {0}")]
    StrategyError(String),
    
    #[error("LLM inference error: {0}")]
    InferenceError(String),
}

// --- Domain Models & Primitives ---

/// Configuration for the Alchemist engine.
#[derive(Debug, Clone, Default)]
pub struct AlchemistConfig {
    pub target_latency_ns: u64, // From Phase 5 Roadmap
}

/// Internal specification parsed from natural language.
#[derive(Debug, Serialize, Deserialize)]
struct MiningSpec {
    algorithm: String,
    target_device: String, // e.g., "CPU", "FPGA", "GPU"
    optimization_goal: String,
}

/// The output strategy ready for execution.
#[derive(Debug)]
pub struct MiningStrategy {
    pub algorithm: String,
    pub instructions: String,
    pub estimated_joules: f64,
}

// --- External Dependencies (Stubs) ---

/// Registry for WASM modules (Phase 3/4)
pub struct CellRegistry;

/// Hardware topology manager (Phase 2)
pub struct FabricManager;

impl FabricManager {
    /// Checks if requested hardware is available in the topology.
    pub fn is_hardware_available(&self, _device: &str) -> bool {
        true // Stub implementation
    }
}

// --- LLM Trait Definition ---

#[async_trait]
pub trait LanguageModel: Send + Sync {
    /// Sends a prompt to the LLM and returns the raw string response.
    async fn infer(&self, prompt: &str) -> Result<String, AlchemistError>;
}

// --- Main Alchemist Implementation ---

pub struct Alchemist {
    config: AlchemistConfig,
    llm: Box<dyn LanguageModel + Send + Sync>,
    cell_registry: Arc<CellRegistry>,
    fabric_manager: Arc<FabricManager>,
}

impl Alchemist {
    /// Constructs a new Alchemist instance.
    pub fn new(
        config: AlchemistConfig,
        llm: Box<dyn LanguageModel + Send + Sync>,
        cell_registry: Arc<CellRegistry>,
        fabric_manager: Arc<FabricManager>,
    ) -> Self {
        Self { config, llm, cell_registry, fabric_manager }
    }

    /// Main orchestration loop: Intent -> Spec -> Strategy.
    pub async fn remix(&self, intent: &str) -> Result<MiningStrategy, AlchemistError> {
        let spec = self.parse_intent(intent).await?;
        let strategy = self.generate_strategy(spec).await?;
        Ok(strategy)
    }

    /// Phase 1: Uses LLM to translate natural language into a structured MiningSpec.
    async fn parse_intent(&self, intent: &str) -> Result<MiningSpec, AlchemistError> {
        // Construct the prompt for the LLM
        let system_prompt = r#"
            You are a mining orchestrator. Extract the algorithm, target device, and optimization goal.
            Respond ONLY with valid JSON: {"algorithm": "", "target_device": "", "optimization_goal": ""}
        "#;
        
        let full_prompt = format!("{}\n\nUser Intent: {}", system_prompt, intent);
        
        // Call LLM
        let response = self.llm.infer(&full_prompt).await?;
        
        // Parse JSON response
        let spec: MiningSpec = serde_json::from_str(&response)
            .map_err(|e| AlchemistError::ParseError(format!("JSON decode failed: {}", e)))?;

        Ok(spec)
    }

    /// Phase 2: Validates resources and constructs the execution strategy.
    async fn generate_strategy(&self, spec: MiningSpec) -> Result<MiningStrategy, AlchemistError> {
        // Validate hardware availability via FabricManager
        if !self.fabric_manager.is_hardware_available(&spec.target_device) {
            return Err(AlchemistError::StrategyError(format!(
                "Requested hardware '{}' is not available in fabric.", 
                spec.target_device
            )));
        }

        // In a full implementation, we would select a WASM module from CellRegistry here.
        
        Ok(MiningStrategy {
            algorithm: spec.algorithm,
            instructions: "wasm-module-hash-abc123".to_string(),
            estimated_joules: 0.05, // Placeholder
        })
    }
}
