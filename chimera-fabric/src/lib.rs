//! Chimera Fabric
//!
//! Hardware and memory abstraction layer for ChimeraOS.
//! Handles physical device topology mapping, cluster routing,
//! and hardware-level memory abstraction.

pub mod topology;
pub mod memory;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::sync::Arc;
use tokio::sync::RwLock;

use chimera_core::primitives::NodeId;

use topology::{Topology, TopologyManager, Device};
use memory::MemoryManager;


/// Errors originating from the Chimera Fabric layer.
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

    #[error("Device already exists: {0}")]
    DeviceExists(NodeId),

    #[error("Routing failure: {0}")]
    RoutingFailure(String),
}

/// Central manager for hardware resources across the Chimera fabric.
///
/// This manager coordinates:
/// - hardware topology
/// - memory allocation
/// - device capability inspection
/// - cluster routing optimization
pub struct FabricManager {

    topology: Arc<RwLock<Topology>>,

    topology_manager: TopologyManager,

    memory_manager: Arc<MemoryManager>,
}

impl FabricManager {

    /// Initialize the Chimera Fabric layer.
    pub async fn new() -> Result<Self, FabricError> {

        let topology = Topology::detect().await?;

        let memory_manager = MemoryManager::new(&topology)?;

        let topology_manager = TopologyManager::new(topology.clone());

        Ok(Self {
            topology: Arc::new(RwLock::new(topology)),
            topology_manager,
            memory_manager: Arc::new(memory_manager),
        })
    }

    /// Access the topology directly.
    pub fn topology(&self) -> Arc<RwLock<Topology>> {
        Arc::clone(&self.topology)
    }

    /// Access the topology manager.
    pub fn topology_manager(&self) -> TopologyManager {
        self.topology_manager.clone()
    }

    /// Access the memory manager.
    pub fn memory_manager(&self) -> Arc<MemoryManager> {
        Arc::clone(&self.memory_manager)
    }

    /// Register a new compute device into the fabric.
    pub async fn register_device(&self, device: Device) -> Result<(), FabricError> {

        let mut topo = self.topology.write().await;

        if topo.nodes.contains_key(&device.id) {
            return Err(FabricError::DeviceExists(device.id));
        }

        topo.register_device(device);

        Ok(())
    }

    /// Remove a device from the fabric.
    pub async fn remove_device(&self, id: NodeId) {

        let mut topo = self.topology.write().await;

        topo.remove_device(id);
    }

    /// Trigger a topology optimization pass.
    ///
    /// This recomputes latency routes and data locality.
    pub async fn optimize_topology(&self) {

        self.topology_manager.optimize().await;
    }

    /// Assess hardware capabilities for a specific node.
    pub async fn assess_capabilities(
        &self,
        node_id: NodeId,
    ) -> Result<DeviceCapabilities, FabricError> {

        let topo = self.topology.read().await;

        let device = topo
            .get_device(node_id)
            .ok_or(FabricError::DeviceOffline(node_id))?;

        Ok(DeviceCapabilities {
            compute_units: device.compute_units,
            memory_bandwidth: device.memory_bandwidth_gb_s,
            latency_ns: device.avg_latency_ns,
        })
    }

    /// Estimate the total compute capacity of the fabric.
    pub async fn total_hashrate_capacity(&self) -> u64 {

        let topo = self.topology.read().await;

        topo.total_hashrate_capacity()
    }

    /// Query routing path between two nodes.
    pub async fn route(
        &self,
        from: NodeId,
        to: NodeId,
    ) -> Option<Vec<NodeId>> {

        let topo = self.topology.read().await;

        topo.get_route(from, to).cloned()
    }
}


/// Summary of device hardware capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {

    /// Number of parallel compute units.
    pub compute_units: u32,

    /// Memory bandwidth (GB/s).
    pub memory_bandwidth: f64,

    /// Average communication latency (nanoseconds).
    pub latency_ns: f64,
}
