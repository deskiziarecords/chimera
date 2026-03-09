//! Chimera Fabric
//!
//! Hardware and memory abstraction layer for ChimeraOS.
//! Handles physical device topology mapping and hardware-level memory abstraction.

pub mod topology;
pub mod memory;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

use chimera_core::primitives::{NodeId, OpCost};
use topology::{Topology, Device};
use memory::MemoryManager;

#[derive(Error, Debug)]
pub enum FabricError {
    #[error("Hardware detection failed: {0}")]
    DetectionFailed(String),
    #[error("Memory allocation failed: {0}")]
    AllocationFailed(String),
    #[error("Topology error: {0}")]
    TopologyError(String),
    #[error("Device offline: {0}")]
    DeviceOffline(NodeId),
}

/// Central manager for hardware resources.
/// Referenced by the Alchemist engine in chimera-core.
pub struct FabricManager {
    topology: Arc<RwLock<Topology>>,
    memory_manager: Arc<MemoryManager>,
}

impl FabricManager {
    pub async fn new() -> Result<Self, FabricError> {
        let topology = Topology::detect().await?;
        let memory_manager = MemoryManager::new(&topology)?;
        
        Ok(Self {
            topology: Arc::new(RwLock::new(topology)),
            memory_manager: Arc::new(memory_manager),
        })
    }

    pub async fn get_topology(&self) -> Arc<RwLock<Topology>> {
        Arc::clone(&self.topology)
    }

    pub async fn get_memory_manager(&self) -> Arc<MemoryManager> {
        Arc::clone(&self.memory_manager)
    }

    /// Assess available hardware for a specific task
    pub async fn assess_capabilities(&self, node_id: NodeId) -> Result<DeviceCapabilities, FabricError> {
        let topo = self.topology.read().await;
        let device = topo.get_device(node_id)
            .ok_or_else(|| FabricError::DeviceOffline(node_id))?;
        
        Ok(DeviceCapabilities {
            compute_units: device.compute_units,
            memory_bandwidth: device.memory_bandwidth_gb_s,
            latency_ns: device.avg_latency_ns,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub compute_units: u32,
    pub memory_bandwidth: f64, // GB/s
    pub latency_ns: f64,       // Average latency
}