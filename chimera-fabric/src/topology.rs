//! Physical device topology mapping.
//! Optimizes data locality to support <100ns latency constraints.

use serde::{Deserialize, Serialize};
use chimera_core::primitives::NodeId;
use crate::FabricError;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    CPU,
    GPU,
    FPGA,
    ASIC,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: NodeId,
    pub device_type: DeviceType,
    pub compute_units: u32,
    pub memory_bandwidth_gb_s: f64,
    pub avg_latency_ns: f64,
    pub thermal_state: f32, // 0.0 to 1.0
    pub is_online: bool,
}

impl Device {
    pub fn new(id: NodeId, device_type: DeviceType) -> Self {
        Self {
            id,
            device_type,
            compute_units: 0,
            memory_bandwidth_gb_s: 0.0,
            avg_latency_ns: 1000.0, // Default high latency
            thermal_state: 0.0,
            is_online: false,
        }
    }
}

/// Represents the physical network and hardware layout of the fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub nodes: HashMap<NodeId, Device>,
    pub adjacency_matrix: Vec<Vec<f64>>, // Latency between nodes
    pub optimized_routes: HashMap<(NodeId, NodeId), Vec<NodeId>>,
}

impl Topology {
    /// Detects local hardware and initializes topology.
    /// In Phase 1, this mocks CPU detection. Phase 3+ adds FPGA/GPU.
    pub async fn detect() -> Result<Self, FabricError> {
        let mut nodes = HashMap::new();
        
        // Mock detection for Phase 1 (CPU)
        // In production, this would use hwloc or PCI enumeration
        let local_id = NodeId::default(); 
        let mut cpu_device = Device::new(local_id, DeviceType::CPU);
        cpu_device.compute_units = num_cpus::get() as u32;
        cpu_device.memory_bandwidth_gb_s = 50.0; // Approximate DDR4/5
        cpu_device.avg_latency_ns = 50.0; // Target <100ns
        cpu_device.is_online = true;
        
        nodes.insert(local_id, cpu_device);

        Ok(Topology {
            nodes,
            adjacency_matrix: vec![],
            optimized_routes: HashMap::new(),
        })
    }

    pub fn get_device(&self, id: NodeId) -> Option<&Device> {
        self.nodes.get(&id)
    }

    /// Optimizes data locality based on latency constraints.
    /// Ensures tasks are scheduled on nodes where data residency minimizes transfer time.
    pub fn optimize_data_locality(&mut self) {
        // Implementation of Phase 5 Optimization Goal
        // Calculates shortest paths for data movement between nodes
        tracing::info!("Optimizing data locality for {} nodes", self.nodes.len());
        
        // Placeholder for Floyd-Warshall or Dijkstra implementation
        // to populate optimized_routes based on adjacency_matrix
    }

    pub fn get_total_hashrate_capacity(&self) -> u64 {
        self.nodes.values()
            .filter(|d| d.is_online)
            .map(|d| d.compute_units as u64 * 10_000_000) // Estimate 10M hashes/sec/core
            .sum()
    }
}