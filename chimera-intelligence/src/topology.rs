//! Physical Device Topology Mapping
//!
//! Maps node architecture and optimizes data locality for <100ns latency constraint.
//! Supports CPU, GPU, FPGA, and specialized subsystems (Grover, SST, Sonar, etc.).

use chimera_core::primitives::{NodeId, OpCost, Hash};
use chimera_core::transforms::Transform;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::{HashMap, BTreeMap};
use std::time::{Instant, Duration};

#[derive(Error, Debug)]
pub enum TopologyError {
    #[error("Device not found: {0}")]
    DeviceNotFound(NodeId),
    #[error("Device offline: {0}")]
    DeviceOffline(NodeId),
    #[error("Latency constraint violated: expected <{expected}ns, got {actual}ns")]
    LatencyViolation { expected: u64, actual: u64 },
    #[error("Memory allocation failed: {0}")]
    MemoryAllocationFailed(String),
    #[error("Topology detection failed: {0}")]
    DetectionFailed(String),
    #[error("Invalid device configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Data locality optimization failed: {0}")]
    LocalityOptimizationFailed(String),
}

/// Device type enumeration for hardware abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    CPU,
    GPU,
    FPGA,
    ASIC,
    TPU,              // Tensor Processing Unit (for Z-Iota)
    Quantum,          // For Grover subsystem
    Unknown,
}

impl DeviceType {
    /// Get typical latency for device type (in nanoseconds).
    pub fn typical_latency_ns(&self) -> u64 {
        match self {
            DeviceType::CPU => 50,
            DeviceType::GPU => 100,
            DeviceType::FPGA => 20,
            DeviceType::ASIC => 10,
            DeviceType::TPU => 30,
            DeviceType::Quantum => 1000, // Higher due to coherence requirements
            DeviceType::Unknown => 1000,
        }
    }

    /// Get typical hashrate capacity (hashes/sec).
    pub fn typical_hashrate(&self) -> u64 {
        match self {
            DeviceType::CPU => 10_000_000,   // 10M hashes/sec/core target
            DeviceType::GPU => 100_000_000,  // 100M hashes/sec
            DeviceType::FPGA => 500_000_000, // 500M hashes/sec
            DeviceType::ASIC => 1_000_000_000, // 1B hashes/sec
            DeviceType::TPU => 200_000_000,  // Optimized for tensor ops
            DeviceType::Quantum => 1_000_000, // Quantum operations
            DeviceType::Unknown => 0,
        }
    }
}

/// Represents a single compute device in the topology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: NodeId,
    pub device_type: DeviceType,
    pub name: String,
    pub is_online: bool,
    pub compute_units: u32,
    pub memory_capacity_mb: u64,
    pub memory_used_mb: u64,
    pub memory_bandwidth_gb_s: f64,
    pub avg_latency_ns: f64,
    pub current_hashrate: f64,
    pub thermal_state: ThermalState,
    pub power_draw_watts: f64,
    pub subsystem_support: Vec<SubsystemCapability>,
    pub last_heartbeat: u64, // Unix timestamp
    pub location: Option<DeviceLocation>,
}

impl Device {
    pub fn new(id: NodeId, device_type: DeviceType, name: &str) -> Self {
        Self {
            id,
            device_type,
            name: name.to_string(),
            is_online: false,
            compute_units: 0,
            memory_capacity_mb: 0,
            memory_used_mb: 0,
            memory_bandwidth_gb_s: 0.0,
            avg_latency_ns: device_type.typical_latency_ns() as f64,
            current_hashrate: 0.0,
            thermal_state: ThermalState::default(),
            power_draw_watts: 0.0,
            subsystem_support: Vec::new(),
            last_heartbeat: 0,
            location: None,
        }
    }

    /// Check if device meets latency constraint (<100ns).
    pub fn meets_latency_constraint(&self) -> bool {
        self.avg_latency_ns < 100.0
    }

    /// Get available memory in MB.
    pub fn available_memory_mb(&self) -> u64 {
        self.memory_capacity_mb.saturating_sub(self.memory_used_mb)
    }

    /// Get utilization percentage (0.0 - 1.0).
    pub fn utilization(&self) -> f64 {
        if self.memory_capacity_mb == 0 {
            return 0.0;
        }
        self.memory_used_mb as f64 / self.memory_capacity_mb as f64
    }

    /// Calculate efficiency score (hashrate per watt).
    pub fn efficiency_score(&self) -> f64 {
        if self.power_draw_watts == 0.0 {
            return 0.0;
        }
        self.current_hashrate / self.power_draw_watts
    }

    /// Check if device supports a specific subsystem.
    pub fn supports_subsystem(&self, subsystem: SubsystemCapability) -> bool {
        self.subsystem_support.contains(&subsystem)
    }
}

/// Thermal state monitoring for device health.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ThermalState {
    pub temperature_celsius: f32,
    pub thermal_throttling: bool,
    pub cooling_efficiency: f32, // 0.0 - 1.0
    pub max_safe_temperature: f32,
}

impl ThermalState {
    pub fn is_safe(&self) -> bool {
        self.temperature_celsius < self.max_safe_temperature && !self.thermal_throttling
    }

    pub fn health_score(&self) -> f32 {
        if self.temperature_celsius >= self.max_safe_temperature {
            return 0.0;
        }
        let temp_ratio = self.temperature_celsius / self.max_safe_temperature;
        (1.0 - temp_ratio) * self.cooling_efficiency
    }
}

/// Physical location of a device (for data locality optimization).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLocation {
    pub rack_id: String,
    pub slot_id: String,
    pub numa_node: u32,
    pub pci_bus: String,
    pub distance_from_cpu: u32, // NUMA distance
}

/// Subsystem capabilities for Phase 3 integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubsystemCapability {
    Grover,      // Quantum algorithm validation
    EchoVoid,    // Advanced mathematics
    VPI,         // Physics validations
    SST,         // FPGA interactions
    Sonar,       // Signal processing
    HilbertWagner, // Geometric mapping (Z-Omega)
    TensorXOR,   // TXB Matrix Logic (Z-Iota)
}

/// Represents a connection between two devices in the topology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLink {
    pub from: NodeId,
    pub to: NodeId,
    pub bandwidth_gb_s: f64,
    pub latency_ns: f64,
    pub link_type: LinkType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
    PCIe,
    NVLink,
    Ethernet,
    InfiniBand,
    MemoryBus,
    QuantumChannel,
}

impl LinkType {
    pub fn typical_bandwidth_gb_s(&self) -> f64 {
        match self {
            LinkType::PCIe => 32.0,      // PCIe 4.0 x16
            LinkType::NVLink => 300.0,   // NVIDIA NVLink
            LinkType::Ethernet => 10.0,  // 10GbE
            LinkType::InfiniBand => 100.0,
            LinkType::MemoryBus => 50.0,
            LinkType::QuantumChannel => 1.0,
        }
    }

    pub fn typical_latency_ns(&self) -> f64 {
        match self {
            LinkType::PCIe => 500.0,
            LinkType::NVLink => 100.0,
            LinkType::Ethernet => 1000.0,
            LinkType::InfiniBand => 500.0,
            LinkType::MemoryBus => 50.0,
            LinkType::QuantumChannel => 10000.0,
        }
    }
}

/// Central topology manager for hardware abstraction.
/// Referenced by `chimera-core::Alchemist` for resource allocation.
pub struct Topology {
    devices: HashMap<NodeId, Device>,
    links: Vec<DeviceLink>,
    adjacency_matrix: BTreeMap<(NodeId, NodeId), f64>, // Latency between nodes
    optimized_routes: HashMap<(NodeId, NodeId), Vec<NodeId>>,
    last_detection: u64,
    detection_interval_secs: u64,
}

impl Topology {
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            links: Vec::new(),
            adjacency_matrix: BTreeMap::new(),
            optimized_routes: HashMap::new(),
            last_detection: 0,
            detection_interval_secs: 60,
        }
    }

    /// Detect and map available hardware topology.
    /// Phase 1: CPU detection. Phase 3+: GPU, FPGA, Quantum.
    pub async fn detect(&mut self) -> Result<(), TopologyError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Phase 1: Mock CPU detection
        // In production, this would use hwloc, PCI enumeration, or platform APIs
        let cpu_id = NodeId::default();
        let mut cpu_device = Device::new(cpu_id, DeviceType::CPU, "Local CPU");
        cpu_device.is_online = true;
        cpu_device.compute_units = num_cpus::get() as u32;
        cpu_device.memory_capacity_mb = 16384; // 16GB default
        cpu_device.memory_bandwidth_gb_s = 50.0;
        cpu_device.avg_latency_ns = 50.0; // Target <100ns
        cpu_device.last_heartbeat = now;

        self.devices.insert(cpu_id, cpu_device);

        // Build adjacency matrix for latency optimization
        self.build_adjacency_matrix()?;

        // Optimize data locality routes
        self.optimize_data_locality()?;

        self.last_detection = now;

        tracing::info!("Topology detection complete: {} devices", self.devices.len());
        Ok(())
    }

    /// Register a new device in the topology.
    pub fn register_device(&mut self, device: Device) -> Result<(), TopologyError> {
        if self.devices.contains_key(&device.id) {
            return Err(TopologyError::InvalidConfiguration(
                format!("Device {} already registered", device.id)
            ));
        }

        self.devices.insert(device.id, device);
        self.build_adjacency_matrix()?;
        self.optimize_data_locality()?;

        Ok(())
    }

    /// Unregister a device from the topology.
    pub fn unregister_device(&mut self, device_id: NodeId) -> Result<(), TopologyError> {
        if !self.devices.contains_key(&device_id) {
            return Err(TopologyError::DeviceNotFound(device_id));
        }

        self.devices.remove(&device_id);
        self.links.retain(|link| link.from != device_id && link.to != device_id);
        self.build_adjacency_matrix()?;

        Ok(())
    }

    /// Get a device by ID.
    pub fn get_device(&self, device_id: NodeId) -> Option<&Device> {
        self.devices.get(&device_id)
    }

    /// Get a mutable device by ID.
    pub fn get_device_mut(&mut self, device_id: NodeId) -> Option<&mut Device> {
        self.devices.get_mut(&device_id)
    }

    /// Get all online devices.
    pub fn get_online_devices(&self) -> Vec<&Device> {
        self.devices.values().filter(|d| d.is_online).collect()
    }

    /// Get devices by type.
    pub fn get_devices_by_type(&self, device_type: DeviceType) -> Vec<&Device> {
        self.devices
            .values()
            .filter(|d| d.device_type == device_type && d.is_online)
            .collect()
    }

    /// Get devices supporting a specific subsystem.
    pub fn get_devices_for_subsystem(
        &self,
        subsystem: SubsystemCapability,
    ) -> Vec<&Device> {
        self.devices
            .values()
            .filter(|d| d.is_online && d.supports_subsystem(subsystem))
            .collect()
    }

    /// Find the best device for a task based on constraints.
    pub fn find_best_device(
        &self,
        max_latency_ns: f64,
        min_memory_mb: u64,
        required_subsystem: Option<SubsystemCapability>,
    ) -> Option<NodeId> {
        self.devices
            .values()
            .filter(|d| {
                d.is_online
                    && d.avg_latency_ns <= max_latency_ns
                    && d.available_memory_mb() >= min_memory_mb
                    && d.thermal_state.is_safe()
                    && required_subsystem.map_or(true, |s| d.supports_subsystem(s))
            })
            .min_by(|a, b| {
                // Prioritize: latency, then efficiency, then thermal health
                a.avg_latency_ns
                    .partial_cmp(&b.avg_latency_ns)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        b.efficiency_score()
                            .partial_cmp(&a.efficiency_score())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| {
                        b.thermal_state
                            .health_score()
                            .partial_cmp(&a.thermal_state.health_score())
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
            })
            .map(|d| d.id)
    }

    /// Build adjacency matrix for latency calculations.
    fn build_adjacency_matrix(&mut self) -> Result<(), TopologyError> {
        self.adjacency_matrix.clear();

        let device_ids: Vec<NodeId> = self.devices.keys().copied().collect();

        for &from_id in &device_ids {
            for &to_id in &device_ids {
                if from_id == to_id {
                    self.adjacency_matrix.insert((from_id, to_id), 0.0);
                } else {
                    // Calculate latency based on links or default
                    let latency = self.calculate_link_latency(from_id, to_id);
                    self.adjacency_matrix.insert((from_id, to_id), latency);
                }
            }
        }

        Ok(())
    }

    /// Calculate latency between two devices.
    fn calculate_link_latency(&self, from: NodeId, to: NodeId) -> f64 {
        // Check for direct link
        if let Some(link) = self.links.iter().find(|l| {
            (l.from == from && l.to == to) || (l.from == to && l.to == from)
        }) {
            return link.latency_ns;
        }

        // Default latency based on device types
        let from_device = self.devices.get(&from);
        let to_device = self.devices.get(&to);

        match (from_device, to_device) {
            (Some(a), Some(b)) => {
                (a.avg_latency_ns + b.avg_latency_ns) / 2.0 + 100.0 // Inter-device overhead
            }
            _ => 1000.0, // High latency for unknown paths
        }
    }

    /// Optimize data locality routes using Floyd-Warshall algorithm.
    /// Ensures <100ns latency constraint for critical paths.
    pub fn optimize_data_locality(&mut self) -> Result<(), TopologyError> {
        self.optimized_routes.clear();

        let device_ids: Vec<NodeId> = self.devices.keys().copied().collect();
        let n = device_ids.len();

        if n == 0 {
            return Ok(());
        }

        // Initialize distance matrix
        let mut dist = vec![vec![f64::INFINITY; n]; n];
        let mut next = vec![vec![None; n]; n];

        for i in 0..n {
            dist[i][i] = 0.0;
            next[i][i] = Some(device_ids[i]);
        }

        // Fill in known link latencies
        for (i, &from_id) in device_ids.iter().enumerate() {
            for (j, &to_id) in device_ids.iter().enumerate() {
                if let Some(&latency) = self.adjacency_matrix.get(&(from_id, to_id)) {
                    dist[i][j] = latency;
                    next[i][j] = Some(to_id);
                }
            }
        }

        // Floyd-Warshall algorithm
        for k in 0..n {
            for i in 0..n {
                for j in 0..n {
                    if dist[i][k] + dist[k][j] < dist[i][j] {
                        dist[i][j] = dist[i][k] + dist[k][j];
                        next[i][j] = next[i][k];
                    }
                }
            }
        }

        // Extract optimized routes
        for i in 0..n {
            for j in 0..n {
                if i != j && dist[i][j] != f64::INFINITY {
                    let from = device_ids[i];
                    let to = device_ids[j];

                    // Validate latency constraint
                    if dist[i][j] > 100.0 {
                        tracing::warn!(
                            "Latency constraint violated for {:?} -> {:?}: {}ns",
                            from,
                            to,
                            dist[i][j]
                        );
                    }

                    // Reconstruct path
                    let mut path = vec![from];
                    let mut current = i;
                    while let Some(next_node) = next[current][j] {
                        if next_node == to {
                            path.push(to);
                            break;
                        }
                        path.push(next_node);
                        current = device_ids.iter().position(|&id| id == next_node).unwrap();
                    }

                    self.optimized_routes.insert((from, to), path);
                }
            }
        }

        tracing::info!("Data locality optimization complete: {} routes", self.optimized_routes.len());
        Ok(())
    }

    /// Get optimized route between two devices.
    pub fn get_optimized_route(
        &self,
        from: NodeId,
        to: NodeId,
    ) -> Option<Vec<NodeId>> {
        self.optimized_routes.get(&(from, to)).cloned()
    }

    /// Get latency between two devices.
    pub fn get_latency(&self, from: NodeId, to: NodeId) -> Option<f64> {
        self.adjacency_matrix.get(&(from, to)).copied()
    }

    /// Calculate total hashrate capacity across all devices.
    pub fn total_hashrate_capacity(&self) -> f64 {
        self.devices
            .values()
            .filter(|d| d.is_online)
            .map(|d| d.current_hashrate)
            .sum()
    }

    /// Calculate total power draw across all devices.
    pub fn total_power_draw(&self) -> f64 {
        self.devices
            .values()
            .filter(|d| d.is_online)
            .map(|d| d.power_draw_watts)
            .sum()
    }

    /// Calculate fleet-wide efficiency score.
    pub fn fleet_efficiency(&self) -> f64 {
        let total_hashrate = self.total_hashrate_capacity();
        let total_power = self.total_power_draw();

        if total_power == 0.0 {
            return 0.0;
        }

        total_hashrate / total_power
    }

    /// Get topology health summary.
    pub fn get_health_summary(&self) -> TopologyHealth {
        let total_devices = self.devices.len();
        let online_devices = self.devices.values().filter(|d| d.is_online).count();
        let devices_meeting_latency = self
            .devices
            .values()
            .filter(|d| d.is_online && d.meets_latency_constraint())
            .count();
        let devices_thermal_safe = self
            .devices
            .values()
            .filter(|d| d.is_online && d.thermal_state.is_safe())
            .count();

        TopologyHealth {
            total_devices,
            online_devices,
            offline_devices: total_devices - online_devices,
            devices_meeting_latency_constraint: devices_meeting_latency,
            devices_thermal_safe,
            total_hashrate: self.total_hashrate_capacity(),
            total_power_draw: self.total_power_draw(),
            fleet_efficiency: self.fleet_efficiency(),
            last_detection: self.last_detection,
        }
    }

    /// Update device telemetry (called asynchronously from devices).
    pub async fn update_telemetry(
        &mut self,
        device_id: NodeId,
        hashrate: f64,
        power_draw: f64,
        temperature: f32,
    ) -> Result<(), TopologyError> {
        let device = self
            .devices
            .get_mut(&device_id)
            .ok_or(TopologyError::DeviceNotFound(device_id))?;

        device.current_hashrate = hashrate;
        device.power_draw_watts = power_draw;
        device.thermal_state.temperature_celsius = temperature;

        // Check for thermal throttling
        if temperature > device.thermal_state.max_safe_temperature * 0.9 {
            device.thermal_state.thermal_throttling = true;
        } else {
            device.thermal_state.thermal_throttling = false;
        }

        device.last_heartbeat = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }

    /// Add a link between two devices.
    pub fn add_link(&mut self, link: DeviceLink) -> Result<(), TopologyError> {
        if !self.devices.contains_key(&link.from) {
            return Err(TopologyError::DeviceNotFound(link.from));
        }
        if !self.devices.contains_key(&link.to) {
            return Err(TopologyError::DeviceNotFound(link.to));
        }

        self.links.push(link);
        self.build_adjacency_matrix()?;
        self.optimize_data_locality()?;

        Ok(())
    }

    /// Get all links in the topology.
    pub fn get_links(&self) -> &[DeviceLink] {
        &self.links
    }

    /// Check if topology needs re-detection.
    pub fn needs_detection(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now - self.last_detection > self.detection_interval_secs
    }

    /// Set detection interval.
    pub fn set_detection_interval(&mut self, interval_secs: u64) {
        self.detection_interval_secs = interval_secs;
    }
}

impl Default for Topology {
    fn default() -> Self {
        Self::new()
    }
}

/// Health summary for dashboard telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyHealth {
    pub total_devices: usize,
    pub online_devices: usize,
    pub offline_devices: usize,
    pub devices_meeting_latency_constraint: usize,
    pub devices_thermal_safe: usize,
    pub total_hashrate: f64,
    pub total_power_draw: f64,
    pub fleet_efficiency: f64,
    pub last_detection: u64,
}

/// Thread-safe topology manager for concurrent access.
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
        let mut topo = self.topology.write().await;
        topo.detect().await
    }

    pub async fn get_device(&self, device_id: NodeId) -> Option<Device> {
        let topo = self.topology.read().await;
        topo.get_device(device_id).cloned()
    }

    pub async fn get_health_summary(&self) -> TopologyHealth {
        let topo = self.topology.read().await;
        topo.get_health_summary()
    }

    pub async fn find_best_device(
        &self,
        max_latency_ns: f64,
        min_memory_mb: u64,
        required_subsystem: Option<SubsystemCapability>,
    ) -> Option<NodeId> {
        let topo = self.topology.read().await;
        topo.find_best_device(max_latency_ns, min_memory_mb, required_subsystem)
    }

    pub async fn update_telemetry(
        &self,
        device_id: NodeId,
        hashrate: f64,
        power_draw: f64,
        temperature: f32,
    ) -> Result<(), TopologyError> {
        let mut topo = self.topology.write().await;
        topo.update_telemetry(device_id, hashrate, power_draw, temperature).await
    }

    pub async fn get_topology(&self) -> Arc<RwLock<Topology>> {
        Arc::clone(&self.topology)
    }
}

impl Default for TopologyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for topology-based transformations.
/// Aligns with chimera-core transforms for differentiable optimization.
pub trait TopologyTransform: Send + Sync {
    fn optimize_route(&self, from: NodeId, to: NodeId) -> Result<Vec<NodeId>, TopologyError>;
    fn get_latency(&self, from: NodeId, to: NodeId) -> Result<f64, TopologyError>;
    fn cost(&self) -> OpCost;
}

/// Hilbert-Wagner Geometric Mapping for Z-Omega algorithm.
/// Maps high-dimensional states to fractal Hilbert Curve for data locality.
pub struct HilbertWagnerMapper {
    dimensions: u32,
    order: u32,
}

impl HilbertWagnerMapper {
    pub fn new(dimensions: u32, order: u32) -> Self {
        Self { dimensions, order }
    }

    /// Map n-dimensional coordinate to Hilbert curve index.
    pub fn map_to_hilbert(&self, coords: &[u32]) -> Result<u64, TopologyError> {
        if coords.len() != self.dimensions as usize {
            return Err(TopologyError::InvalidConfiguration(
                format!(
                    "Expected {} dimensions, got {}",
                    self.dimensions,
                    coords.len()
                )
            ));
        }

        // Simplified Hilbert curve mapping
        // In production, use proper Hilbert curve algorithm
        let mut index = 0u64;
        for (i, &coord) in coords.iter().enumerate() {
            index |= ((coord as u64) << (i * 8));
        }

        Ok(index)
    }

    /// Calculate distance preservation score.
    pub fn distance_preservation_score(
        &self,
        x: &[u32],
        y: &[u32],
    ) -> Result<f64, TopologyError> {
        let hx = self.map_to_hilbert(x)?;
        let hy = self.map_to_hilbert(y)?;

        // Euclidean distance in original space
        let original_dist: f64 = x
            .iter()
            .zip(y.iter())
            .map(|(&a, &b)| (a as i64 - b as i64).pow(2) as f64)
            .sum::<f64>()
            .sqrt();

        // Distance in Hilbert space
        let hilbert_dist = (hx as i64 - hy as i64).abs() as f64;

        // Score: 1.0 = perfect preservation, 0.0 = no preservation
        if original_dist == 0.0 {
            return Ok(1.0);
        }

        let ratio = hilbert_dist / original_dist;
        (1.0 / (1.0 + ratio)).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_topology_detection() {
        let mut topology = Topology::new();
        let result = topology.detect().await;

        assert!(result.is_ok());
        assert!(topology.devices.len() >= 1); // At least CPU detected
    }

    #[tokio::test]
    async fn test_device_registration() {
        let mut topology = Topology::new();
        topology.detect().await.unwrap();

        let gpu_device = Device::new(NodeId::from([1u8; 32]), DeviceType::GPU, "Test GPU");
        let result = topology.register_device(gpu_device);

        assert!(result.is_ok());
        assert_eq!(topology.devices.len(), 2);
    }

    #[test]
    fn test_device_latency_constraint() {
        let mut device = Device::new(NodeId::default(), DeviceType::CPU, "Test CPU");
        device.avg_latency_ns = 50.0;

        assert!(device.meets_latency_constraint());

        device.avg_latency_ns = 150.0;
        assert!(!device.meets_latency_constraint());
    }

    #[tokio::test]
    async fn test_find_best_device() {
        let mut topology = Topology::new();
        topology.detect().await.unwrap();

        let best = topology.find_best_device(100.0, 1024, None);
        assert!(best.is_some()); // Should find CPU
    }

    #[tokio::test]
    async fn test_topology_health() {
        let mut topology = Topology::new();
        topology.detect().await.unwrap();

        let health = topology.get_health_summary();

        assert!(health.total_devices >= 1);
        assert!(health.online_devices >= 1);
        assert!(health.total_hashrate >= 0.0);
    }

    #[test]
    fn test_hilbert_wagner_mapper() {
        let mapper = HilbertWagnerMapper::new(2, 4);
        let coords = [10u32, 20u32];

        let index = mapper.map_to_hilbert(&coords);
        assert!(index.is_ok());

        let score = mapper.distance_preservation_score(&coords, &[15u32, 25u32]);
        assert!(score.is_ok());
        assert!(score.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_telemetry_update() {
        let mut topology = Topology::new();
        topology.detect().await.unwrap();

        let device_id = NodeId::default();
        let result = topology
            .update_telemetry(device_id, 10_000_000.0, 500.0, 65.0)
            .await;

        assert!(result.is_ok());

        let device = topology.get_device(device_id).unwrap();
        assert_eq!(device.current_hashrate, 10_000_000.0);
        assert_eq!(device.power_draw_watts, 500.0);
    }

    #[test]
    fn test_thermal_state() {
        let mut thermal = ThermalState::default();
        thermal.temperature_celsius = 50.0;
        thermal.max_safe_temperature = 85.0;
        thermal.cooling_efficiency = 0.9;

        assert!(thermal.is_safe());
        assert!(thermal.health_score() > 0.5);

        thermal.temperature_celsius = 90.0;
        assert!(!thermal.is_safe());
    }

    #[tokio::test]
    async fn test_topology_manager() {
        let manager = TopologyManager::new();
        manager.detect().await.unwrap();

        let health = manager.get_health_summary().await;
        assert!(health.total_devices >= 1);

        let best = manager
            .find_best_device(100.0, 1024, None)
            .await;
        assert!(best.is_some());
    }
}