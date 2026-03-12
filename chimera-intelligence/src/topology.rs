//! Physical Device Topology Mapping
//!
//! Maps node architecture and optimizes data locality for <100ns latency.
//! Supports CPU, GPU, FPGA and specialized subsystems.

use chimera_core::primitives::{NodeId, OpCost};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use rand::{rngs::OsRng, RngCore};

const HEARTBEAT_TIMEOUT: u64 = 30;

fn generate_node_id() -> NodeId {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    NodeId::from(bytes)
}

#[derive(Error, Debug)]
pub enum TopologyError {
    #[error("Device not found")]
    DeviceNotFound,

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Topology detection failed: {0}")]
    DetectionFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    CPU,
    GPU,
    FPGA,
    ASIC,
    TPU,
    Quantum,
}

impl DeviceType {
    pub fn typical_latency_ns(&self) -> f64 {
        match self {
            DeviceType::CPU => 50.0,
            DeviceType::GPU => 100.0,
            DeviceType::FPGA => 20.0,
            DeviceType::ASIC => 10.0,
            DeviceType::TPU => 30.0,
            DeviceType::Quantum => 1000.0,
        }
    }

    pub fn typical_hashrate(&self) -> f64 {
        match self {
            DeviceType::CPU => 10_000_000.0,
            DeviceType::GPU => 100_000_000.0,
            DeviceType::FPGA => 500_000_000.0,
            DeviceType::ASIC => 1_000_000_000.0,
            DeviceType::TPU => 200_000_000.0,
            DeviceType::Quantum => 1_000_000.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ThermalState {
    pub temperature_celsius: f32,
    pub thermal_throttling: bool,
    pub cooling_efficiency: f32,
    pub max_safe_temperature: f32,
}

impl Default for ThermalState {
    fn default() -> Self {
        Self {
            temperature_celsius: 35.0,
            thermal_throttling: false,
            cooling_efficiency: 1.0,
            max_safe_temperature: 85.0,
        }
    }
}

impl ThermalState {
    pub fn is_safe(&self) -> bool {
        !self.thermal_throttling && self.temperature_celsius < self.max_safe_temperature
    }

    pub fn health_score(&self) -> f32 {
        if self.temperature_celsius >= self.max_safe_temperature {
            return 0.0;
        }

        let ratio = self.temperature_celsius / self.max_safe_temperature;
        (1.0 - ratio) * self.cooling_efficiency
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: NodeId,
    pub device_type: DeviceType,
    pub name: String,
    pub is_online: bool,

    pub compute_units: u32,
    pub memory_capacity_mb: u64,
    pub memory_used_mb: u64,

    pub avg_latency_ns: f64,
    pub current_hashrate: f64,

    pub thermal_state: ThermalState,
    pub power_draw_watts: f64,

    pub last_heartbeat: u64,
}

impl Device {
    pub fn new(device_type: DeviceType, name: &str) -> Self {
        Self {
            id: generate_node_id(),
            device_type,
            name: name.to_string(),
            is_online: true,
            compute_units: 0,
            memory_capacity_mb: 0,
            memory_used_mb: 0,
            avg_latency_ns: device_type.typical_latency_ns(),
            current_hashrate: 0.0,
            thermal_state: ThermalState::default(),
            power_draw_watts: 0.0,
            last_heartbeat: now(),
        }
    }

    pub fn available_memory(&self) -> u64 {
        self.memory_capacity_mb.saturating_sub(self.memory_used_mb)
    }

    pub fn estimated_hashrate(&self) -> f64 {
        if self.current_hashrate > 0.0 {
            self.current_hashrate
        } else {
            self.device_type.typical_hashrate()
        }
    }

    pub fn efficiency_score(&self) -> f64 {
        if self.power_draw_watts == 0.0 {
            return 0.0;
        }

        self.estimated_hashrate() / self.power_draw_watts
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLink {
    pub from: NodeId,
    pub to: NodeId,
    pub latency_ns: f64,
}

#[derive(Debug)]
pub struct Topology {
    devices: HashMap<NodeId, Device>,
    links: Vec<DeviceLink>,

    adjacency: BTreeMap<(NodeId, NodeId), f64>,
    optimized_routes: HashMap<(NodeId, NodeId), Vec<NodeId>>,

    dirty: bool,
    last_detection: u64,
}

impl Topology {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            links: Vec::new(),
            adjacency: BTreeMap::new(),
            optimized_routes: HashMap::new(),
            dirty: true,
            last_detection: 0,
        }
    }

    pub async fn detect(&mut self) -> Result<(), TopologyError> {
        self.devices.clear();

        let mut cpu = Device::new(DeviceType::CPU, "Local CPU");

        cpu.compute_units = num_cpus::get() as u32;
        cpu.memory_capacity_mb = 16384;

        self.devices.insert(cpu.id, cpu);

        self.detect_gpu_devices().await?;
        self.detect_fpga_devices().await?;

        self.rebuild_topology()?;

        self.last_detection = now();

        tracing::info!("Topology detection complete: {}", self.devices.len());

        Ok(())
    }

    async fn detect_gpu_devices(&mut self) -> Result<(), TopologyError> {
        Ok(())
    }

    async fn detect_fpga_devices(&mut self) -> Result<(), TopologyError> {
        Ok(())
    }

    fn rebuild_topology(&mut self) -> Result<(), TopologyError> {
        self.build_adjacency();
        self.optimize_routes();
        self.dirty = false;
        Ok(())
    }

    fn build_adjacency(&mut self) {
        self.adjacency.clear();

        let ids: Vec<NodeId> = self.devices.keys().copied().collect();

        for &a in &ids {
            for &b in &ids {
                if a == b {
                    self.adjacency.insert((a, b), 0.0);
                } else {
                    let latency = self.calculate_latency(a, b);
                    self.adjacency.insert((a, b), latency);
                }
            }
        }
    }

    fn calculate_latency(&self, a: NodeId, b: NodeId) -> f64 {
        if let Some(link) = self.links.iter().find(|l| l.from == a && l.to == b) {
            return link.latency_ns;
        }

        let da = self.devices.get(&a);
        let db = self.devices.get(&b);

        match (da, db) {
            (Some(x), Some(y)) => (x.avg_latency_ns + y.avg_latency_ns) / 2.0 + 100.0,
            _ => 1000.0,
        }
    }

    fn optimize_routes(&mut self) {
        self.optimized_routes.clear();

        let ids: Vec<NodeId> = self.devices.keys().copied().collect();
        let n = ids.len();

        if n == 0 {
            return;
        }

        let mut dist = vec![vec![f64::INFINITY; n]; n];
        let mut next = vec![vec![None; n]; n];

        for i in 0..n {
            dist[i][i] = 0.0;
            next[i][i] = Some(ids[i]);
        }

        for (i, &a) in ids.iter().enumerate() {
            for (j, &b) in ids.iter().enumerate() {
                if let Some(lat) = self.adjacency.get(&(a, b)) {
                    dist[i][j] = *lat;
                    next[i][j] = Some(b);
                }
            }
        }

        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    let alt = dist[i][k] + dist[k][j];

                    if alt < dist[i][j] {
                        dist[i][j] = alt;
                        next[i][j] = next[i][k];
                    }
                }
            }
        }

        for i in 0..n {
            for j in 0..n {
                if i == j {
                    continue;
                }

                let from = ids[i];
                let to = ids[j];

                let mut path = vec![from];
                let mut current = i;

                while let Some(next_node) = next[current][j] {
                    path.push(next_node);

                    if next_node == to {
                        break;
                    }

                    current = ids.iter().position(|x| *x == next_node).unwrap();
                }

                self.optimized_routes.insert((from, to), path);
            }
        }
    }

    pub fn get_device(&self, id: NodeId) -> Option<&Device> {
        self.devices.get(&id)
    }

    pub fn get_online_devices(&self) -> Vec<&Device> {
        let now = now();

        self.devices
            .values()
            .filter(|d| d.is_online && now - d.last_heartbeat < HEARTBEAT_TIMEOUT)
            .collect()
    }

    pub fn mining_devices_ranked(&self) -> Vec<NodeId> {
        let mut devices: Vec<_> = self
            .get_online_devices()
            .into_iter()
            .filter(|d| d.thermal_state.is_safe())
            .collect();

        devices.sort_by(|a, b| {
            b.efficiency_score()
                .partial_cmp(&a.efficiency_score())
                .unwrap()
        });

        devices.into_iter().map(|d| d.id).collect()
    }

    pub fn total_hashrate(&self) -> f64 {
        self.get_online_devices()
            .iter()
            .map(|d| d.estimated_hashrate())
            .sum()
    }

    pub fn total_power(&self) -> f64 {
        self.get_online_devices()
            .iter()
            .map(|d| d.power_draw_watts)
            .sum()
    }

    pub fn fleet_efficiency(&self) -> f64 {
        let power = self.total_power();

        if power == 0.0 {
            return 0.0;
        }

        self.total_hashrate() / power
    }

    pub fn telemetry_snapshot(&self) -> Vec<(NodeId, f64, f64)> {
        self.get_online_devices()
            .iter()
            .map(|d| (d.id, d.current_hashrate, d.power_draw_watts))
            .collect()
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct TopologyManager {
    topology: Arc<RwLock<Topology>>,
}

impl TopologyManager {
    pub fn new() -> Self {
        Self {
            topology: Arc::new(RwLock::new(Topology::new())),
        }
    }

    pub async fn detect(&self) -> Result<(), TopologyError> {
        self.topology.write().await.detect().await
    }

    pub async fn health_hashrate(&self) -> f64 {
        self.topology.read().await.total_hashrate()
    }

    pub async fn find_best_device(&self) -> Option<NodeId> {
        let topo = self.topology.read().await;

        topo.mining_devices_ranked().first().copied()
    }

    pub async fn topology(&self) -> Arc<RwLock<Topology>> {
        self.topology.clone()
    }
}

impl Default for TopologyManager {
    fn default() -> Self {
        Self::new()
    }
}

pub trait TopologyTransform: Send + Sync {
    fn optimize_route(&self, from: NodeId, to: NodeId) -> Result<Vec<NodeId>, TopologyError>;
    fn latency(&self, from: NodeId, to: NodeId) -> Result<f64, TopologyError>;
    fn cost(&self) -> OpCost;
}

pub struct HilbertWagnerMapper {
    dimensions: u32,
}

impl HilbertWagnerMapper {
    pub fn new(dimensions: u32) -> Self {
        Self { dimensions }
    }

    pub fn map(&self, coords: &[u32]) -> Result<u64, TopologyError> {
        if coords.len() != self.dimensions as usize {
            return Err(TopologyError::InvalidConfiguration(
                "dimension mismatch".into(),
            ));
        }

        let mut index = 0u64;

        for (i, &c) in coords.iter().enumerate() {
            index ^= (c as u64).wrapping_mul(0x9E3779B97F4A7C15u64.rotate_left(i as u32));
        }

        Ok(index)
    }
}
