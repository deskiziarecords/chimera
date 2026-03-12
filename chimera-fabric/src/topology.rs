//! Physical device topology mapping.
//! Optimizes data locality to support <100ns latency constraints
//! across distributed Chimera fabric nodes.

use serde::{Deserialize, Serialize};
use chimera_core::primitives::NodeId;
use crate::FabricError;

use std::collections::HashMap;
use tokio::sync::RwLock;

use std::sync::Arc;

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
    pub thermal_state: f32, // 0.0–1.0
    pub is_online: bool,
}

impl Device {
    pub fn new(id: NodeId, device_type: DeviceType) -> Self {
        Self {
            id,
            device_type,
            compute_units: 0,
            memory_bandwidth_gb_s: 0.0,
            avg_latency_ns: 1000.0,
            thermal_state: 0.0,
            is_online: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topology {
    pub nodes: HashMap<NodeId, Device>,

    /// Latency matrix (ns)
    pub adjacency_matrix: Vec<Vec<f64>>,

    /// Cached optimized routes
    pub optimized_routes: HashMap<(NodeId, NodeId), Vec<NodeId>>,
}

impl Topology {
    pub async fn detect() -> Result<Self, FabricError> {

        let mut nodes = HashMap::new();

        let local_id = NodeId::default();

        let mut cpu = Device::new(local_id, DeviceType::CPU);

        cpu.compute_units = num_cpus::get() as u32;
        cpu.memory_bandwidth_gb_s = 50.0;
        cpu.avg_latency_ns = 50.0;
        cpu.is_online = true;

        nodes.insert(local_id, cpu);

        Ok(Self {
            nodes,
            adjacency_matrix: Vec::new(),
            optimized_routes: HashMap::new(),
        })
    }

    pub fn get_device(&self, id: NodeId) -> Option<&Device> {
        self.nodes.get(&id)
    }

    pub fn register_device(&mut self, device: Device) {
        self.nodes.insert(device.id, device);
    }

    pub fn remove_device(&mut self, id: NodeId) {
        self.nodes.remove(&id);
    }

    pub fn build_latency_matrix(&mut self) {

        let ids: Vec<NodeId> = self.nodes.keys().copied().collect();
        let n = ids.len();

        self.adjacency_matrix = vec![vec![f64::INFINITY; n]; n];

        for i in 0..n {
            self.adjacency_matrix[i][i] = 0.0;
        }

        for i in 0..n {
            for j in 0..n {

                if i == j {
                    continue;
                }

                let a = self.nodes.get(&ids[i]).unwrap();
                let b = self.nodes.get(&ids[j]).unwrap();

                self.adjacency_matrix[i][j] =
                    (a.avg_latency_ns + b.avg_latency_ns) / 2.0 + 100.0;
            }
        }
    }

    pub fn optimize_data_locality(&mut self) {

        let ids: Vec<NodeId> = self.nodes.keys().copied().collect();
        let n = ids.len();

        if n == 0 {
            return;
        }

        let mut dist = self.adjacency_matrix.clone();

        for k in 0..n {
            for i in 0..n {
                for j in 0..n {

                    let alt = dist[i][k] + dist[k][j];

                    if alt < dist[i][j] {
                        dist[i][j] = alt;

                        self.optimized_routes.insert(
                            (ids[i], ids[j]),
                            vec![ids[i], ids[k], ids[j]],
                        );
                    }
                }
            }
        }

        tracing::info!(
            "Fabric topology optimized: {} nodes, {} routes",
            n,
            self.optimized_routes.len()
        );
    }

    pub fn get_route(
        &self,
        from: NodeId,
        to: NodeId,
    ) -> Option<&Vec<NodeId>> {
        self.optimized_routes.get(&(from, to))
    }

    pub fn total_hashrate_capacity(&self) -> u64 {

        self.nodes
            .values()
            .filter(|d| d.is_online)
            .map(|d| d.compute_units as u64 * 10_000_000)
            .sum()
    }
}

#[derive(Clone)]
pub struct TopologyManager {
    topology: Arc<RwLock<Topology>>,
}

impl TopologyManager {

    pub fn new(topology: Topology) -> Self {
        Self {
            topology: Arc::new(RwLock::new(topology)),
        }
    }

    pub async fn get_device(&self, id: NodeId) -> Option<Device> {

        let topo = self.topology.read().await;

        topo.get_device(id).cloned()
    }

    pub async fn register_device(&self, device: Device) {

        let mut topo = self.topology.write().await;

        topo.register_device(device);
    }

    pub async fn optimize(&self) {

        let mut topo = self.topology.write().await;

        topo.build_latency_matrix();
        topo.optimize_data_locality();
    }

    pub async fn total_hashrate(&self) -> u64 {

        let topo = self.topology.read().await;

        topo.total_hashrate_capacity()
    }
}
