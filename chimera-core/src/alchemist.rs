<<<<<<< HEAD
//! The Alchemist Engine
//!
//! Translates natural language intent into executable mining strategies.
//! Integrates LLM parsing, hardware validation, AI optimization, and WASM deployment.
//! "Hard-core" implementation: Async, typed, safety-guarded, telemetry-enabled.

use crate::primitives::{
    Hash, Nonce, NodeId, OpCost, ThermalState, MiningStrategy, MiningResult, FleetStats, Difficulty, BlockHeader
};
use crate::transforms::{Transform, Grad};
use chimera_fabric::{TopologyManager, DeviceType, SubsystemCapability, TopologyHealth};
use chimera_cell::{CellRegistry, CellConfig, WasmSandbox};
use chimera_intelligence::{
    SwarmOptimizer, SwarmConfig, 
    RLAgent, RLConfig, 
    MultiObjectiveOptimizer, MOConfig
};
use tokio::sync::{RwLock, mpsc, broadcast};
use tokio::task::JoinHandle;
use std::sync::Arc;
use std::time::{Instant, Duration};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn, error, instrument};

/// Alchemist-specific error types.
#[derive(Error, Debug)]
pub enum AlchemistError {
    #[error("Intent parsing failed: {0}")]
    IntentParsingFailed(String),
    #[error("Strategy validation failed: {0}")]
    StrategyValidationFailed(String),
    #[error("Hardware constraint violation: {0}")]
    HardwareConstraintViolation(String),
    #[error("WASM compilation failed: {0}")]
    WasmCompilationFailed(String),
    #[error("LLM service unavailable: {0}")]
    LLMServiceUnavailable(String),
    #[error("Safety guard triggered: {0}")]
    SafetyGuardTriggered(String),
    #[error("Resource allocation failed: {0}")]
    ResourceAllocationFailed(String),
    #[error("Telemetry channel closed")]
    TelemetryChannelClosed,
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),
}

/// Configuration for the Alchemist engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemistConfig {
    pub llm_endpoint: String,
    pub llm_model: String,
    pub max_strategy_latency_ms: u64,
    pub safety_thermal_threshold: f32,
    pub safety_power_threshold_watts: f64,
    pub enable_intent_cache: bool,
    pub cache_ttl_secs: u64,
    pub default_optimization_backend: OptimizationBackend,
}

impl Default for AlchemistConfig {
    fn default() -> Self {
        Self {
            llm_endpoint: "http://localhost:8080".to_string(),
            llm_model: "chimera-llama-70b".to_string(),
            max_strategy_latency_ms: 100, // Target <100ns for execution, 100ms for planning
            safety_thermal_threshold: 85.0,
            safety_power_threshold_watts: 1000.0,
            enable_intent_cache: true,
            cache_ttl_secs: 300,
            default_optimization_backend: OptimizationBackend::Swarm,
        }
    }
}

/// Available optimization backends from the Compendium.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationBackend {
    Swarm,          // GWO, WOA, ACO, etc.
    Reinforcement,  // PPO, DQN, A2C, etc.
    MultiObjective, // NSGA-II, MOEA/D
    Hybrid,         // Adaptive selection
}

/// Algorithm profiles including Zcash Synthesis and Compendium algorithms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgorithmProfile {
    // Standard
    SHA256,
    Equihash,
    // Zcash Synthesis
    ZOmega,         // Fractal Shielded Consensus
    ZSigma,         // Permutation-Aware Circuit Synthesis
    ZIota,          // Unitary Privacy Forensics
    // Compendium Specialized
    GroverQuantum,  // Quantum algorithm validation
    EchoVoidMath,   // Advanced mathematics
    VPIPhysics,     // Physics validations
    SSTFPGA,        // FPGA interactions
    SonarSignal,    // Signal processing
    // Optimization Algorithms
    SwarmGWO,
    SwarmWOA,
    RLPPO,
    MONSGA2,
}

/// Parsed intent structure from LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentSpec {
    pub goal: String,
    pub constraints: Vec<Constraint>,
    pub preferred_algorithm: Option<AlgorithmProfile>,
    pub optimization_priority: OptimizationPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constraint {
    MaxPower(f64),
    MaxTemperature(f32),
    MinHashrate(f64),
    MaxLatencyNs(u64),
    RequireSubsystem(SubsystemCapability),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OptimizationPriority {
    Performance,    // Maximize hashrate
    Efficiency,     // Maximize hashes/joule
    Cost,           // Minimize dollars
    Balance,        // Weighted mix
}

/// Safety guardrails for strategy deployment.
pub struct SafetyGuard {
    thermal_threshold: f32,
    power_threshold: f64,
    latency_threshold_ns: u64,
}

impl SafetyGuard {
    pub fn new(config: &AlchemistConfig) -> Self {
        Self {
            thermal_threshold: config.safety_thermal_threshold,
            power_threshold: config.safety_power_threshold_watts,
            latency_threshold_ns: config.max_strategy_latency_ms * 1_000_000, // ms to ns
        }
    }

    pub fn validate_strategy(&self, strategy: &MiningStrategy, health: &TopologyHealth) -> Result<(), AlchemistError> {
        if strategy.max_temperature > self.thermal_threshold {
            return Err(AlchemistError::SafetyGuardTriggered(
                format!("Temperature {}°C exceeds threshold {}°C", strategy.max_temperature, self.thermal_threshold)
            ));
        }
        if strategy.max_power_watts > self.power_threshold {
            return Err(AlchemistError::SafetyGuardTriggered(
                format!("Power {}W exceeds threshold {}W", strategy.max_power_watts, self.power_threshold)
            ));
        }
        if health.devices_meeting_latency_constraint == 0 {
            return Err(AlchemistError::HardwareConstraintViolation(
                "No devices meet latency constraint".to_string()
            ));
        }
        Ok(())
    }
}

/// Cached intent for faster repeated parsing.
#[derive(Debug, Clone)]
struct IntentCacheEntry {
    intent_hash: u64,
    spec: IntentSpec,
    created_at: Instant,
    ttl: Duration,
}

/// Central Alchemist engine.
/// Orchestrates intent parsing, validation, optimization, and deployment.
=======
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

>>>>>>> b1c3fa6ecf5982d921dbc44b3f253667a676f19b
pub struct Alchemist {
    config: AlchemistConfig,
    fabric_manager: Arc<TopologyManager>,
    cell_registry: Arc<CellRegistry>,
    safety_guard: SafetyGuard,
    intent_cache: Arc<RwLock<Vec<IntentCacheEntry>>>,
    telemetry_tx: broadcast::Sender<AlchemistTelemetry>,
    active_sessions: Arc<RwLock<Vec<MiningSessionHandle>>>,
}

impl Alchemist {
<<<<<<< HEAD
    /// Create a new Alchemist engine.
    pub fn new(
        config: AlchemistConfig,
        fabric_manager: Arc<TopologyManager>,
        cell_registry: Arc<CellRegistry>,
    ) -> Self {
        let (telemetry_tx, _) = broadcast::channel(1000);
        
        Self {
            config: config.clone(),
            fabric_manager,
            cell_registry,
            safety_guard: SafetyGuard::new(&config),
            intent_cache: Arc::new(RwLock::new(Vec::new())),
            telemetry_tx,
            active_sessions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get telemetry subscription channel.
    pub fn subscribe_telemetry(&self) -> broadcast::Receiver<AlchemistTelemetry> {
        self.telemetry_tx.subscribe()
    }

    /// Parse natural language intent into a structured spec.
    #[instrument(skip(self, intent), fields(intent_len = intent.len()))]
    pub async fn parse_intent(&self, intent: &str) -> Result<IntentSpec, AlchemistError> {
        // Check cache first
        if self.config.enable_intent_cache {
            if let Some(cached) = self.get_cached_intent(intent).await {
                info!("Intent cache hit");
                return Ok(cached);
            }
        }

        // Call LLM (mocked for now, would use HTTP/gRPC to LLM service)
        let spec = self.call_llm(intent).await?;

        // Cache result
        if self.config.enable_intent_cache {
            self.cache_intent(intent, &spec).await;
        }

        Ok(spec)
    }

    /// Generate a mining strategy from an intent spec.
    #[instrument(skip(self, spec))]
    pub async fn generate_strategy(&self, spec: IntentSpec) -> Result<MiningStrategy, AlchemistError> {
        // Get hardware health
        let health = self.fabric_manager.get_health_summary().await;

        // Create base strategy
        let mut strategy = MiningStrategy::new(
            &format!("strategy_{}", chrono::Utc::now().timestamp()),
            &spec.preferred_algorithm.map(|a| format!("{:?}", a)).unwrap_or("SHA256".to_string())
        );

        // Apply constraints
        for constraint in &spec.constraints {
            match constraint {
                Constraint::MaxPower(p) => strategy.max_power_watts = *p,
                Constraint::MaxTemperature(t) => strategy.max_temperature = *t,
                Constraint::MinHashrate(h) => strategy.target_hashrate = *h,
                Constraint::RequireSubsystem(sub) => {
                    strategy.subsystems.push(format!("{:?}", sub));
                }
                _ => {}
            }
        }

        // Apply optimization priority
        match spec.optimization_priority {
            OptimizationPriority::Performance => {
                strategy.priority = 10;
                strategy.target_hashrate *= 1.5; // Aggressive
            }
            OptimizationPriority::Efficiency => {
                strategy.priority = 5;
                strategy.max_power_watts *= 0.8; // Conservative
            }
            OptimizationPriority::Cost => {
                strategy.priority = 3;
                strategy.max_power_watts *= 0.6; // Very conservative
            }
            OptimizationPriority::Balance => {
                strategy.priority = 7;
            }
        }

        // Validate strategy
        strategy.validate().map_err(|e| AlchemistError::StrategyValidationFailed(e.to_string()))?;
        self.safety_guard.validate_strategy(&strategy, &health)?;

        Ok(strategy)
    }

    /// Optimize strategy parameters using AI/ML backends.
    #[instrument(skip(self, strategy))]
    pub async fn optimize_strategy(&self, strategy: &mut MiningStrategy) -> Result<OpCost, AlchemistError> {
        let start = Instant::now();
        
        match self.config.default_optimization_backend {
            OptimizationBackend::Swarm => {
                let mut optimizer = SwarmOptimizer::new(
                    SwarmConfig::default(),
                    4, // decision vars
                ).map_err(|e| AlchemistError::ResourceAllocationFailed(e.to_string()))?;
                
                // Optimize for OpCost
                // (Simplified - would integrate with intelligence module properly)
                info!("Optimizing strategy using Swarm Intelligence (GWO/WOA)");
            }
            OptimizationBackend::Reinforcement => {
                info!("Optimizing strategy using RL (PPO/DQN)");
                // Would load trained policy from intelligence module
            }
            OptimizationBackend::MultiObjective => {
                info!("Optimizing strategy using Multi-Objective (NSGA-II)");
                // Would compute Pareto front
            }
            OptimizationBackend::Hybrid => {
                info!("Optimizing strategy using Hybrid backend");
                // Adaptive selection
            }
        }

        let elapsed = start.elapsed();
        
        Ok(OpCost {
            joules: elapsed.as_secs_f64() * 10.0, // Estimate
            seconds: elapsed.as_secs_f64(),
            dollars: elapsed.as_secs_f64() * 0.0001,
        })
    }

    /// Deploy strategy to WASM cell registry.
    #[instrument(skip(self, strategy))]
    pub async fn deploy_strategy(&self, strategy: &MiningStrategy) -> Result<MiningSessionHandle, AlchemistError> {
        // Generate WASM module bytes (mocked - would compile from strategy)
        let wasm_bytes = self.compile_strategy_to_wasm(strategy).await?;

        // Load into cell registry
        let sandbox = self.cell_registry
            .load_module(&wasm_bytes)
            .await
            .map_err(|e| AlchemistError::WasmCompilationFailed(e.to_string()))?;

        // Create session handle
        let session = MiningSessionHandle::new(
            strategy.id.clone(),
            sandbox,
            self.telemetry_tx.clone(),
        );

        // Track active session
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.push(session.clone());
        }

        info!("Strategy {} deployed successfully", strategy.id);
        Ok(session)
    }

    /// Main entry point: Remix intent into execution.
    #[instrument(skip(self, intent))]
    pub async fn remix(&self, intent: &str) -> Result<MiningSessionHandle, AlchemistError> {
        let start = Instant::now();
        
        // 1. Parse Intent
        let spec = self.parse_intent(intent).await?;
        
        // 2. Generate Strategy
        let mut strategy = self.generate_strategy(spec).await?;
        
        // 3. Optimize Strategy
        let opt_cost = self.optimize_strategy(&mut strategy).await?;
        
        // 4. Deploy Strategy
        let session = self.deploy_strategy(&strategy).await?;
        
        let total_time = start.elapsed();
        
        // Emit telemetry
        let _ = self.telemetry_tx.send(AlchemistTelemetry {
            timestamp: chrono::Utc::now().timestamp(),
            event: "StrategyRemix".to_string(),
            intent_hash: intent.len() as u64,
            strategy_id: strategy.id,
            optimization_cost: opt_cost,
            deployment_time_ms: total_time.as_millis() as u64,
        });

        info!("Remix complete in {}ms", total_time.as_millis());
        Ok(session)
    }

    /// Stop all active mining sessions.
    pub async fn halt_all(&self) -> Result<(), AlchemistError> {
        let mut sessions = self.active_sessions.write().await;
        for session in sessions.iter() {
            session.stop().await?;
        }
        sessions.clear();
        info!("All mining sessions halted");
        Ok(())
    }

    /// Get active session count.
    pub async fn active_session_count(&self) -> usize {
        self.active_sessions.read().await.len()
    }

    // --- Internal Helper Methods ---

    async fn get_cached_intent(&self, intent: &str) -> Option<IntentSpec> {
        let cache = self.intent_cache.read().await;
        let hash = fxhash::hash64(intent);
        
        for entry in cache.iter() {
            if entry.intent_hash == hash && entry.created_at.elapsed() < entry.ttl {
                return Some(entry.spec.clone());
            }
        }
        None
    }

    async fn cache_intent(&self, intent: &str, spec: &IntentSpec) {
        let mut cache = self.intent_cache.write().await;
        let hash = fxhash::hash64(intent);
        
        cache.push(IntentCacheEntry {
            intent_hash: hash,
            spec: spec.clone(),
            created_at: Instant::now(),
            ttl: Duration::from_secs(self.config.cache_ttl_secs),
        });

        // Prune old entries
        cache.retain(|e| e.created_at.elapsed() < e.ttl);
    }

    async fn call_llm(&self, intent: &str) -> Result<IntentSpec, AlchemistError> {
        // Mock LLM call - in production, this would HTTP POST to LLM service
        // Expected response: JSON matching IntentSpec
        tokio::time::sleep(Duration::from_millis(50)).await; // Simulate latency
        
        // Default fallback if LLM fails to parse
        Ok(IntentSpec {
            goal: "Mine efficiently".to_string(),
            constraints: vec![],
            preferred_algorithm: Some(AlgorithmProfile::SHA256),
            optimization_priority: OptimizationPriority::Balance,
        })
    }

    async fn compile_strategy_to_wasm(&self, strategy: &MiningStrategy) -> Result<Vec<u8>, AlchemistError> {
        // Mock WASM compilation - in production, this would use WASM toolkit or precompiled templates
        // For now, return a minimal valid WASM module
        Ok(vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, // WASM header
            // ... rest of minimal module
        ])
    }
}

/// Handle for an active mining session.
/// Allows stopping, monitoring, and retrieving metrics.
#[derive(Clone)]
pub struct MiningSessionHandle {
    strategy_id: String,
    sandbox: Arc<WasmSandbox>,
    telemetry_tx: broadcast::Sender<AlchemistTelemetry>,
    stop_tx: Arc<RwLock<Option<mpsc::Sender<()>>>>,
    task_handle: Arc<RwLock<Option<JoinHandle<Result<(), AlchemistError>>>>>,
}

impl MiningSessionHandle {
    fn new(
        strategy_id: String,
        sandbox: Arc<WasmSandbox>,
        telemetry_tx: broadcast::Sender<AlchemistTelemetry>,
    ) -> Self {
        Self {
            strategy_id,
            sandbox,
            telemetry_tx,
            stop_tx: Arc::new(RwLock::new(None)),
            task_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the mining session.
    pub async fn start(&self, difficulty: Difficulty) -> Result<(), AlchemistError> {
        let (stop_tx, mut stop_rx) = mpsc::channel(1);
        
        {
            let mut lock = self.stop_tx.write().await;
            *lock = Some(stop_tx);
        }

        let sandbox = Arc::clone(&self.sandbox);
        let telemetry_tx = self.telemetry_tx.clone();
        let strategy_id = self.strategy_id.clone();

        let handle = tokio::spawn(async move {
            let mut nonce = Nonce::zero();
            let mut hashes = 0u64;
            let start = Instant::now();

            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {
                        // Execute hash
                        // (Simplified - would call sandbox.execute)
                        nonce.increment();
                        hashes += 1;

                        // Emit telemetry every 1000 hashes
                        if hashes % 1000 == 0 {
                            let elapsed = start.elapsed().as_secs_f64();
                            let _ = telemetry_tx.send(AlchemistTelemetry {
                                timestamp: chrono::Utc::now().timestamp(),
                                event: "MiningProgress".to_string(),
                                intent_hash: 0,
                                strategy_id: strategy_id.clone(),
                                optimization_cost: OpCost::zero(),
                                deployment_time_ms: 0,
                            });
                        }
                    }
                }
            }

            Ok(())
        });

        {
            let mut lock = self.task_handle.write().await;
            *lock = Some(handle);
        }

        Ok(())
    }

    /// Stop the mining session.
    pub async fn stop(&self) -> Result<(), AlchemistError> {
        if let Some(tx) = self.stop_tx.write().await.take() {
            let _ = tx.send(()).await;
        }
        
        if let Some(handle) = self.task_handle.write().await.take() {
            let _ = handle.await;
        }

        Ok(())
    }

    /// Get session status.
    pub async fn get_status(&self) -> SessionStatus {
        SessionStatus {
            strategy_id: self.strategy_id.clone(),
            is_running: self.task_handle.read().await.is_some(),
            sandbox_id: format!("{:p}", self.sandbox.as_ref()),
        }
    }
}

/// Session status for dashboard telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    pub strategy_id: String,
    pub is_running: bool,
    pub sandbox_id: String,
}

/// Telemetry event from Alchemist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlchemistTelemetry {
    pub timestamp: i64,
    pub event: String,
    pub intent_hash: u64,
    pub strategy_id: String,
    pub optimization_cost: OpCost,
    pub deployment_time_ms: u64,
}

/// Trait for Language Model abstraction.
#[async_trait::async_trait]
pub trait LanguageModel: Send + Sync {
    async fn generate_completion(&self, prompt: &str) -> Result<String, AlchemistError>;
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>, AlchemistError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chimera_fabric::TopologyManager;
    use chimera_cell::CellRegistry;

    #[tokio::test]
    async fn test_alchemist_remix() {
        let config = AlchemistConfig::default();
        let fabric = Arc::new(TopologyManager::new());
        let cell = Arc::new(CellRegistry::new(CellConfig::default()).unwrap());
        
        let alchemist = Alchemist::new(config, fabric, cell);
        
        // Initialize topology first
        alchemist.fabric_manager.detect().await.unwrap();
        
        let intent = "Maximize SHA-256 efficiency with power limit 500W";
        let session = alchemist.remix(intent).await;
        
        assert!(session.is_ok());
        assert_eq!(alchemist.active_session_count().await, 1);
    }

    #[tokio::test]
    async fn test_safety_guard() {
        let config = AlchemistConfig {
            safety_thermal_threshold: 50.0, // Very low for testing
            ..Default::default()
        };
        
        let guard = SafetyGuard::new(&config);
        let mut strategy = MiningStrategy::default();
        strategy.max_temperature = 85.0; // Exceeds threshold
        
        let health = TopologyHealth::default();
        let result = guard.validate_strategy(&strategy, &health);
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AlchemistError::SafetyGuardTriggered(_)));
    }

    #[tokio::test]
    async fn test_intent_cache() {
        let config = AlchemistConfig {
            enable_intent_cache: true,
            cache_ttl_secs: 60,
            ..Default::default()
        };
        
        let fabric = Arc::new(TopologyManager::new());
        let cell = Arc::new(CellRegistry::new(CellConfig::default()).unwrap());
        let alchemist = Alchemist::new(config, fabric, cell);
        
        let intent = "Test caching intent";
        let spec1 = alchemist.parse_intent(intent).await.unwrap();
        let spec2 = alchemist.parse_intent(intent).await.unwrap();
        
        // Should be same result (cached)
        assert_eq!(spec1.goal, spec2.goal);
    }

    #[tokio::test]
    async fn test_mining_session() {
        let config = AlchemistConfig::default();
        let fabric = Arc::new(TopologyManager::new());
        let cell = Arc::new(CellRegistry::new(CellConfig::default()).unwrap());
        let alchemist = Alchemist::new(config, fabric, cell);
        
        let session = alchemist.remix("Start mining").await.unwrap();
        session.start(Difficulty::default()).await.unwrap();
        
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let status = session.get_status().await;
        assert!(status.is_running);
        
        session.stop().await.unwrap();
        
        let status = session.get_status().await;
        assert!(!status.is_running);
    }
}
=======
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
>>>>>>> b1c3fa6ecf5982d921dbc44b3f253667a676f19b
