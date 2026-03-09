//! Chimera Subsystems
//!
//! Specialized domain subsystems for ChimeraOS (Phase 3).
//! Provides validation for quantum algorithms, advanced mathematics, physics, FPGA, and signal processing.

pub mod grover;
pub mod echovoid;
pub mod vpi;
pub mod sst;
pub mod sonar;

use chimera_core::primitives::{Hash, Nonce, OpCost};
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Error, Debug)]
pub enum SubsystemError {
    #[error("Quantum validation failed: {0}")]
    QuantumValidationFailed(String),
    #[error("Mathematical validation failed: {0}")]
    MathValidationFailed(String),
    #[error("Physics validation failed: {0}")]
    PhysicsValidationFailed(String),
    #[error("FPGA communication failed: {0}")]
    FpgaCommunicationFailed(String),
    #[error("Signal processing error: {0}")]
    SignalProcessingError(String),
    #[error("Subsystem not enabled: {0}")]
    SubsystemNotEnabled(String),
}

/// Configuration for subsystem activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemConfig {
    pub enable_grover: bool,
    pub enable_echovoid: bool,
    pub enable_vpi: bool,
    pub enable_sst: bool,
    pub enable_sonar: bool,
}

impl Default for SubsystemConfig {
    fn default() -> Self {
        Self {
            enable_grover: false,    // Phase 3: Quantum
            enable_echovoid: false,  // Phase 3: Math
            enable_vpi: false,       // Phase 3: Physics
            enable_sst: false,       // Phase 3: FPGA
            enable_sonar: false,     // Phase 3: Signal
        }
    }
}

/// Central manager for all subsystems.
/// Referenced by the Alchemist engine for specialized task routing.
pub struct SubsystemRegistry {
    config: SubsystemConfig,
    grover_validator: Option<Arc<grover::GroverValidator>>,
    echovoid_engine: Option<Arc<echovoid::EchoVoidEngine>>,
    vpi_simulator: Option<Arc<vpi::PhysicsSimulator>>,
    sst_bridge: Option<Arc<sst::FpgaBridge>>,
    sonar_processor: Option<Arc<sonar::SignalProcessor>>,
}

impl SubsystemRegistry {
    pub fn new(config: SubsystemConfig) -> Self {
        Self {
            config: config.clone(),
            grover_validator: if config.enable_grover {
                Some(Arc::new(grover::GroverValidator::new()))
            } else {
                None
            },
            echovoid_engine: if config.enable_echovoid {
                Some(Arc::new(echovoid::EchoVoidEngine::new()))
            } else {
                None
            },
            vpi_simulator: if config.enable_vpi {
                Some(Arc::new(vpi::PhysicsSimulator::new()))
            } else {
                None
            },
            sst_bridge: if config.enable_sst {
                Some(Arc::new(sst::FpgaBridge::new()))
            } else {
                None
            },
            sonar_processor: if config.enable_sonar {
                Some(Arc::new(sonar::SignalProcessor::new()))
            } else {
                None
            },
        }
    }

    /// Route a task to the appropriate subsystem.
    pub async fn route_task(
        &self,
        task_type: &str,
        input: &[u8],
    ) -> Result<Hash, SubsystemError> {
        match task_type {
            "quantum" => {
                if let Some(ref validator) = self.grover_validator {
                    validator.validate(input).await
                } else {
                    Err(SubsystemError::SubsystemNotEnabled("Grover".to_string()))
                }
            }
            "math" => {
                if let Some(ref engine) = self.echovoid_engine {
                    engine.compute(input).await
                } else {
                    Err(SubsystemError::SubsystemNotEnabled("EchoVoid".to_string()))
                }
            }
            "physics" => {
                if let Some(ref simulator) = self.vpi_simulator {
                    simulator.simulate(input).await
                } else {
                    Err(SubsystemError::SubsystemNotEnabled("VPI".to_string()))
                }
            }
            "fpga" => {
                if let Some(ref bridge) = self.sst_bridge {
                    bridge.execute(input).await
                } else {
                    Err(SubsystemError::SubsystemNotEnabled("SST".to_string()))
                }
            }
            "signal" => {
                if let Some(ref processor) = self.sonar_processor {
                    processor.process(input).await
                } else {
                    Err(SubsystemError::SubsystemNotEnabled("Sonar".to_string()))
                }
            }
            _ => Err(SubsystemError::SubsystemNotEnabled(task_type.to_string())),
        }
    }

    /// Get subsystem health status.
    pub async fn get_health_status(&self) -> SubsystemHealth {
        SubsystemHealth {
            grover_active: self.grover_validator.is_some(),
            echovoid_active: self.echovoid_engine.is_some(),
            vpi_active: self.vpi_simulator.is_some(),
            sst_active: self.sst_bridge.is_some(),
            sonar_active: self.sonar_processor.is_some(),
        }
    }
}

/// Health status of all subsystems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubsystemHealth {
    pub grover_active: bool,
    pub echovoid_active: bool,
    pub vpi_active: bool,
    pub sst_active: bool,
    pub sonar_active: bool,
}

/// Trait for subsystem operations.
/// Aligns with chimera-core transforms for unified execution.
pub trait SubsystemOperation: Send + Sync {
    fn execute(&self, input: &[u8]) -> Result<Hash, SubsystemError>;
    fn cost(&self) -> OpCost;
    fn name(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subsystem_registry() {
        let config = SubsystemConfig::default();
        let registry = SubsystemRegistry::new(config);
        
        let health = registry.get_health_status().await;
        assert!(!health.grover_active); // Default disabled
    }
}

pub mod lir; // Add this line

// In SubsystemRegistry struct
pub lir_engine: Option<Arc<lir::LirEngine>>,

// In SubsystemConfig
pub enable_lir: bool,

// In route_task match
"lir" => {
    if let Some(ref engine) = self.lir_engine {
        engine.execute(input).await
    } else {
        Err(SubsystemError::SubsystemNotEnabled("LIR".to_string()))
    }
}