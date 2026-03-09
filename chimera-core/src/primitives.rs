<<<<<<< HEAD
//! ChimeraOS Core Primitives
//!
//! Defines fundamental types (Hash, Nonce, NodeId) and operational metrics (OpCost, ThermalState).
//! Used globally across all crates to ensure type safety and consistency.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::fmt;
use std::hash::Hash as StdHash;
use thiserror::Error;

/// 32-byte cryptographic hash (SHA-256 output).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, StdHash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// Create a zero hash (all bytes set to 0).
    pub fn zero() -> Self {
        Hash([0u8; 32])
    }

    /// Check if hash is all zeros.
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    /// Get hash as hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Create hash from hex string.
    pub fn from_hex(s: &str) -> Result<Self, PrimitiveError> {
        let bytes = hex::decode(s).map_err(|e| PrimitiveError::InvalidHex(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(PrimitiveError::InvalidHashLength(bytes.len()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Hash(arr))
    }

    /// Get leading zeros count (for difficulty comparison).
    pub fn leading_zeros(&self) -> u32 {
        self.0.iter().map(|&b| b.leading_zeros()).find(|&z| z < 8).unwrap_or(8) * 8
            + self.0.iter().skip_while(|&&b| b == 0).next().map(|&b| b.leading_zeros()).unwrap_or(8)
    }

    /// Compare with target difficulty (returns true if hash <= target).
    pub fn meets_difficulty(&self, target: &Hash) -> bool {
        self <= target
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for Hash {
    fn from(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }
}

impl From<Hash> for [u8; 32] {
    fn from(hash: Hash) -> Self {
        hash.0
    }
}

/// 64-bit nonce for mining operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, StdHash, Serialize, Deserialize)]
pub struct Nonce(pub u64);

impl Nonce {
    /// Create a zero nonce.
    pub fn zero() -> Self {
        Nonce(0)
    }

    /// Increment nonce by 1.
    pub fn increment(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(1);
        self.0
    }

    /// Get nonce value.
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Create nonce from value.
    pub fn from_value(v: u64) -> Self {
        Nonce(v)
    }

    /// Check if nonce is zero.
    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl Default for Nonce {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for Nonce {
    fn from(v: u64) -> Self {
        Nonce(v)
    }
}

impl From<Nonce> for u64 {
    fn from(nonce: Nonce) -> Self {
        nonce.0
    }
}

/// Unique identifier for compute nodes in the fabric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, StdHash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 32]);

impl NodeId {
    /// Create a zero NodeId.
    pub fn zero() -> Self {
        NodeId([0u8; 32])
    }

    /// Generate a random NodeId.
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut bytes = [0u8; 32];
        rng.fill(&mut bytes);
        NodeId(bytes)
    }

    /// Get NodeId as hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Create NodeId from hex string.
    pub fn from_hex(s: &str) -> Result<Self, PrimitiveError> {
        let bytes = hex::decode(s).map_err(|e| PrimitiveError::InvalidHex(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(PrimitiveError::InvalidNodeIdLength(bytes.len()));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(NodeId(arr))
    }

    /// Check if NodeId is zero.
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }

    /// Get short identifier (first 8 hex chars).
    pub fn short_id(&self) -> String {
        self.to_hex()[..8].to_string()
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_id())
    }
}

impl From<[u8; 32]> for NodeId {
    fn from(bytes: [u8; 32]) -> Self {
        NodeId(bytes)
    }
}

impl From<NodeId> for [u8; 32] {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

/// Operational cost metrics for mining tasks.
/// Tracks energy, time, and financial cost for optimization.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct OpCost {
    /// Energy consumption in joules.
    pub joules: f64,
    /// Time duration in seconds.
    pub seconds: f64,
    /// Financial cost in dollars (USD).
    pub dollars: f64,
}

impl OpCost {
    /// Create a zero OpCost.
    pub fn zero() -> Self {
        Self::default()
    }

    /// Create OpCost with specified values.
    pub fn new(joules: f64, seconds: f64, dollars: f64) -> Self {
        OpCost {
            joules,
            seconds,
            dollars,
        }
    }

    /// Check if all costs are zero.
    pub fn is_zero(&self) -> bool {
        self.joules == 0.0 && self.seconds == 0.0 && self.dollars == 0.0
    }

    /// Add another OpCost to this one.
    pub fn add(&mut self, other: &OpCost) {
        self.joules += other.joules;
        self.seconds += other.seconds;
        self.dollars += other.dollars;
    }

    /// Calculate efficiency score (lower is better).
    pub fn efficiency_score(&self) -> f64 {
        // Weighted sum: prioritize time, then energy, then cost
        self.seconds * 0.5 + self.joules * 0.3 + self.dollars * 0.2
    }

    /// Calculate hashes per joule (energy efficiency).
    pub fn hashes_per_joule(&self, hashes: u64) -> f64 {
        if self.joules == 0.0 {
            return 0.0;
        }
        hashes as f64 / self.joules
    }

    /// Calculate hashes per second (hashrate).
    pub fn hashrate(&self, hashes: u64) -> f64 {
        if self.seconds == 0.0 {
            return 0.0;
        }
        hashes as f64 / self.seconds
    }

    /// Calculate cost per hash (dollars).
    pub fn cost_per_hash(&self, hashes: u64) -> f64 {
        if hashes == 0 {
            return 0.0;
        }
        self.dollars / hashes as f64
    }

    /// Scale OpCost by a factor.
    pub fn scale(&mut self, factor: f64) {
        self.joules *= factor;
        self.seconds *= factor;
        self.dollars *= factor;
    }
}

impl std::ops::Add for OpCost {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        OpCost {
            joules: self.joules + other.joules,
            seconds: self.seconds + other.seconds,
            dollars: self.dollars + other.dollars,
        }
    }
}

impl std::ops::AddAssign for OpCost {
    fn add_assign(&mut self, other: Self) {
        self.add(&other);
    }
}

impl fmt::Display for OpCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "OpCost({:.2}J, {:.4}s, ${:.6})",
            self.joules, self.seconds, self.dollars
        )
    }
}

/// Thermal state monitoring for device health.
/// Used by `chimera-fabric` for hardware abstraction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct ThermalState {
    /// Current temperature in Celsius.
    pub temperature_celsius: f32,
    /// Whether thermal throttling is active.
    pub thermal_throttling: bool,
    /// Cooling efficiency (0.0 - 1.0).
    pub cooling_efficiency: f32,
    /// Maximum safe temperature in Celsius.
    pub max_safe_temperature: f32,
}

impl ThermalState {
    /// Create a default thermal state.
    pub fn new() -> Self {
        Self {
            temperature_celsius: 25.0, // Room temperature
            thermal_throttling: false,
            cooling_efficiency: 1.0,
            max_safe_temperature: 85.0, // Typical CPU/GPU max
        }
    }

    /// Check if thermal state is safe.
    pub fn is_safe(&self) -> bool {
        self.temperature_celsius < self.max_safe_temperature && !self.thermal_throttling
    }

    /// Calculate health score (0.0 - 1.0, higher is better).
    pub fn health_score(&self) -> f32 {
        if self.temperature_celsius >= self.max_safe_temperature {
            return 0.0;
        }
        let temp_ratio = self.temperature_celsius / self.max_safe_temperature;
        (1.0 - temp_ratio) * self.cooling_efficiency
    }

    /// Check if throttling is likely (>90% of max temp).
    pub fn is_throttling_likely(&self) -> bool {
        self.temperature_celsius > self.max_safe_temperature * 0.9
    }

    /// Update temperature and check for throttling.
    pub fn update_temperature(&mut self, temp: f32) {
        self.temperature_celsius = temp;
        self.thermal_throttling = self.is_throttling_likely();
    }

    /// Set cooling efficiency.
    pub fn set_cooling_efficiency(&mut self, efficiency: f32) {
        self.cooling_efficiency = efficiency.clamp(0.0, 1.0);
    }

    /// Set maximum safe temperature.
    pub fn set_max_safe_temperature(&mut self, max_temp: f32) {
        self.max_safe_temperature = max_temp;
    }
}

impl fmt::Display for ThermalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThermalState({:.1}°C, throttling={}, health={:.2})",
            self.temperature_celsius,
            self.thermal_throttling,
            self.health_score()
        )
    }
}

/// Mining difficulty target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Difficulty(pub [u8; 32]);

impl Difficulty {
    /// Create difficulty from target hash.
    pub fn from_target(target: Hash) -> Self {
        Difficulty(target.0)
    }

    /// Get target as Hash.
    pub fn to_target(&self) -> Hash {
        Hash(self.0)
    }

    /// Create difficulty from bits (compact representation).
    pub fn from_bits(bits: u32) -> Self {
        // Simplified difficulty from bits (Bitcoin-style)
        let exponent = (bits >> 24) as usize;
        let mantissa = bits & 0x00ffffff;

        let mut target = [0u8; 32];
        if exponent <= 32 {
            let shift = 8 * (32 - exponent);
            if shift < 256 {
                let byte_idx = shift / 8;
                let bit_idx = shift % 8;
                if byte_idx < 32 {
                    target[byte_idx] = (mantissa << bit_idx) as u8;
                }
            }
        }

        Difficulty(target)
    }

    /// Get bits (compact representation).
    pub fn to_bits(&self) -> u32 {
        // Simplified bits from difficulty
        0x1d00ffff // Default difficulty bits
    }

    /// Check if hash meets difficulty.
    pub fn meets_difficulty(&self, hash: &Hash) -> bool {
        hash.meets_difficulty(&self.to_target())
    }
}

impl Default for Difficulty {
    fn default() -> Self {
        // Default difficulty (easy for testing)
        let mut target = [0xffu8; 32];
        target[0] = 0x00; // At least one leading zero
        Difficulty(target)
    }
}

impl From<Hash> for Difficulty {
    fn from(hash: Hash) -> Self {
        Difficulty::from_target(hash)
    }
}

impl From<Difficulty> for Hash {
    fn from(diff: Difficulty) -> Self {
        diff.to_target()
    }
}

/// Block header for mining operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Previous block hash.
    pub prev_hash: Hash,
    /// Merkle root of transactions.
    pub merkle_root: Hash,
    /// Block timestamp (Unix epoch).
    pub timestamp: u64,
    /// Current difficulty target.
    pub difficulty: Difficulty,
    /// Current nonce being tested.
    pub nonce: Nonce,
    /// Version number.
    pub version: u32,
}

impl BlockHeader {
    pub fn new(
        prev_hash: Hash,
        merkle_root: Hash,
        timestamp: u64,
        difficulty: Difficulty,
    ) -> Self {
        Self {
            prev_hash,
            merkle_root,
            timestamp,
            difficulty,
            nonce: Nonce::zero(),
            version: 1,
        }
    }

    /// Serialize header for hashing.
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.prev_hash.0);
        bytes.extend_from_slice(&self.merkle_root.0);
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.difficulty.0);
        bytes.extend_from_slice(&self.nonce.0.to_le_bytes());
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes
    }

    /// Increment nonce for next hash attempt.
    pub fn increment_nonce(&mut self) -> u64 {
        self.nonce.increment()
    }

    /// Reset nonce to zero.
    pub fn reset_nonce(&mut self) {
        self.nonce = Nonce::zero();
    }
}

/// Mining result from a successful hash computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningResult {
    /// The hash that met difficulty.
    pub hash: Hash,
    /// The nonce that produced the hash.
    pub nonce: Nonce,
    /// Time taken to find the hash (seconds).
    pub time_seconds: f64,
    /// Number of hash attempts.
    pub attempts: u64,
    /// Node that found the hash.
    pub node_id: NodeId,
}

impl MiningResult {
    pub fn new(hash: Hash, nonce: Nonce, time_seconds: f64, attempts: u64, node_id: NodeId) -> Self {
        Self {
            hash,
            nonce,
            time_seconds,
            attempts,
            node_id,
        }
    }

    /// Calculate effective hashrate.
    pub fn effective_hashrate(&self) -> f64 {
        if self.time_seconds == 0.0 {
            return 0.0;
        }
        self.attempts as f64 / self.time_seconds
    }

    /// Calculate energy per hash (if OpCost is known).
    pub fn energy_per_hash(&self, joules: f64) -> f64 {
        if self.attempts == 0 {
            return 0.0;
        }
        joules / self.attempts as f64
    }
}

/// Atomic counter for thread-safe nonce generation.
pub struct AtomicNonce {
    counter: AtomicU64,
}

impl AtomicNonce {
    pub fn new(initial: u64) -> Self {
        Self {
            counter: AtomicU64::new(initial),
        }
    }

    pub fn next(&self) -> Nonce {
        Nonce(self.counter.fetch_add(1, Ordering::Relaxed))
    }

    pub fn current(&self) -> Nonce {
        Nonce(self.counter.load(Ordering::Relaxed))
    }

    pub fn reset(&self, value: u64) {
        self.counter.store(value, Ordering::Relaxed);
    }
}

impl Default for AtomicNonce {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Error types for primitive operations.
#[derive(Error, Debug)]
pub enum PrimitiveError {
    #[error("Invalid hex string: {0}")]
    InvalidHex(String),
    #[error("Invalid hash length: expected 32 bytes, got {0}")]
    InvalidHashLength(usize),
    #[error("Invalid NodeId length: expected 32 bytes, got {0}")]
    InvalidNodeIdLength(usize),
    #[error("Difficulty not met: hash {hash} > target {target}")]
    DifficultyNotMet { hash: Hash, target: Hash },
    #[error("Nonce overflow")]
    NonceOverflow,
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Mining strategy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningStrategy {
    /// Unique strategy identifier.
    pub id: String,
    /// Target algorithm (e.g., "SHA-256", "Equihash", "Grover").
    pub algorithm: String,
    /// Target hashrate (hashes/sec).
    pub target_hashrate: f64,
    /// Maximum power draw (watts).
    pub max_power_watts: f64,
    /// Maximum temperature (Celsius).
    pub max_temperature: f32,
    /// Priority level (higher = more resources).
    pub priority: u32,
    /// Enabled subsystems.
    pub subsystems: Vec<String>,
}

impl MiningStrategy {
    pub fn new(id: &str, algorithm: &str) -> Self {
        Self {
            id: id.to_string(),
            algorithm: algorithm.to_string(),
            target_hashrate: 10_000_000.0, // 10M hashes/sec/core target
            max_power_watts: 500.0,
            max_temperature: 85.0,
            priority: 1,
            subsystems: Vec::new(),
        }
    }

    /// Check if strategy meets constraints.
    pub fn validate(&self) -> Result<(), PrimitiveError> {
        if self.target_hashrate <= 0.0 {
            return Err(PrimitiveError::InvalidParameter(
                "Target hashrate must be > 0".to_string()
            ));
        }
        if self.max_power_watts <= 0.0 {
            return Err(PrimitiveError::InvalidParameter(
                "Max power must be > 0".to_string()
            ));
        }
        if self.max_temperature <= 0.0 {
            return Err(PrimitiveError::InvalidParameter(
                "Max temperature must be > 0".to_string()
            ));
        }
        Ok(())
    }
}

impl Default for MiningStrategy {
    fn default() -> Self {
        Self::new("default", "SHA-256")
    }
}

/// Fleet-wide statistics for dashboard telemetry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FleetStats {
    /// Total hashrate across all nodes (hashes/sec).
    pub total_hashrate: f64,
    /// Total power draw (watts).
    pub total_power_watts: f64,
    /// Average temperature (Celsius).
    pub avg_temperature: f32,
    /// Number of online nodes.
    pub online_nodes: u32,
    /// Number of offline nodes.
    pub offline_nodes: u32,
    /// Total hashes computed (lifetime).
    pub total_hashes: u64,
    /// Total energy consumed (joules).
    pub total_energy_joules: f64,
    /// Total cost (dollars).
    pub total_cost_dollars: f64,
    /// Average efficiency (hashes/joule).
    pub avg_efficiency: f64,
    /// Last update timestamp (Unix epoch).
    pub last_update: u64,
}

impl FleetStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate fleet efficiency.
    pub fn efficiency(&self) -> f64 {
        if self.total_energy_joules == 0.0 {
            return 0.0;
        }
        self.total_hashes as f64 / self.total_energy_joules
    }

    /// Calculate cost per hash.
    pub fn cost_per_hash(&self) -> f64 {
        if self.total_hashes == 0 {
            return 0.0;
        }
        self.total_cost_dollars / self.total_hashes as f64
    }

    /// Get hashrate in TH/s.
    pub defn hashrate_th_s(&self) -> f64 {
        self.total_hashrate / 1e12
    }

    /// Get power in kW.
    pub fn power_kw(&self) -> f64 {
        self.total_power_watts / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_zero() {
        let hash = Hash::zero();
        assert!(hash.is_zero());
        assert_eq!(hash.to_hex(), "0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_hash_from_hex() {
        let hex = "0000000000000000000000000000000000000000000000000000000000000001";
        let hash = Hash::from_hex(hex).unwrap();
        assert!(!hash.is_zero());
        assert_eq!(hash.to_hex(), hex);
    }

    #[test]
    fn test_nonce_increment() {
        let mut nonce = Nonce::zero();
        assert_eq!(nonce.value(), 0);
        
        nonce.increment();
        assert_eq!(nonce.value(), 1);
        
        nonce.increment();
        assert_eq!(nonce.value(), 2);
    }

    #[test]
    fn test_node_id_random() {
        let id1 = NodeId::random();
        let id2 = NodeId::random();
        
        assert!(!id1.is_zero());
        assert!(!id2.is_zero());
        assert_ne!(id1, id2); // Should be unique
    }

    #[test]
    fn test_opcost_operations() {
        let mut cost1 = OpCost::new(100.0, 1.0, 0.01);
        let cost2 = OpCost::new(50.0, 0.5, 0.005);
        
        cost1.add(&cost2);
        
        assert_eq!(cost1.joules, 150.0);
        assert_eq!(cost1.seconds, 1.5);
        assert_eq!(cost1.dollars, 0.015);
    }

    #[test]
    fn test_thermal_state() {
        let mut thermal = ThermalState::new();
        assert!(thermal.is_safe());
        assert!(thermal.health_score() > 0.5);
        
        thermal.update_temperature(90.0);
        assert!(!thermal.is_safe());
        assert!(thermal.thermal_throttling);
    }

    #[test]
    fn test_difficulty() {
        let diff = Difficulty::default();
        let target = diff.to_target();
        
        // Easy target should be met by most hashes
        let easy_hash = Hash::zero();
        assert!(diff.meets_difficulty(&easy_hash));
    }

    #[test]
    fn test_block_header() {
        let mut header = BlockHeader::new(
            Hash::zero(),
            Hash::zero(),
            0,
            Difficulty::default(),
        );
        
        assert_eq!(header.nonce.value(), 0);
        
        header.increment_nonce();
        assert_eq!(header.nonce.value(), 1);
        
        header.reset_nonce();
        assert_eq!(header.nonce.value(), 0);
    }

    #[test]
    fn test_mining_result() {
        let result = MiningResult::new(
            Hash::zero(),
            Nonce(12345),
            1.0,
            1000000,
            NodeId::default(),
        );
        
        assert_eq!(result.effective_hashrate(), 1_000_000.0);
    }

    #[test]
    fn test_atomic_nonce() {
        let atomic = AtomicNonce::new(100);
        
        let nonce1 = atomic.next();
        let nonce2 = atomic.next();
        
        assert_eq!(nonce1.value(), 100);
        assert_eq!(nonce2.value(), 101);
        assert_eq!(atomic.current().value(), 102);
    }

    #[test]
    fn test_mining_strategy() {
        let strategy = MiningStrategy::new("test", "SHA-256");
        assert!(strategy.validate().is_ok());
        
        let invalid = MiningStrategy {
            target_hashrate: 0.0,
            ..strategy.clone()
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_fleet_stats() {
        let mut stats = FleetStats::new();
        stats.total_hashrate = 1e12; // 1 TH/s
        stats.total_power_watts = 1000.0; // 1 kW
        stats.total_hashes = 1_000_000;
        stats.total_energy_joules = 100.0;
        
        assert_eq!(stats.hashrate_th_s(), 1.0);
        assert_eq!(stats.power_kw(), 1.0);
        assert_eq!(stats.efficiency(), 10_000.0);
    }

    #[test]
    fn test_hash_display() {
        let hash = Hash::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        let display = format!("{}", hash);
        assert_eq!(display.len(), 64); // 64 hex chars
    }

    #[test]
    fn test_opcost_display() {
        let cost = OpCost::new(100.0, 1.0, 0.01);
        let display = format!("{}", cost);
        assert!(display.contains("J"));
        assert!(display.contains("s"));
        assert!(display.contains("$"));
    }
}
=======

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

// --- Cryptographic Primitives ---

/// A 256-bit cryptographic hash result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    /// Returns a zeroed hash (often used as a null value).
    pub const fn zero() -> Self {
        Hash([0u8; 32])
    }

    /// Converts the hash to a hexadecimal string representation.
    pub fn to_hex_string(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex_string())
    }
}

/// A 64-bit mining nonce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Nonce(pub u64);

impl Nonce {
    /// Increments the nonce by one, wrapping around on overflow.
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }
}

/// A thread-safe wrapper for Nonce, allowing concurrent mining threads
/// to claim unique nonces without lock contention.
pub struct AtomicNonce {
    inner: AtomicU64,
}

impl AtomicNonce {
    pub fn new(start: u64) -> Self {
        Self {
            inner: AtomicU64::new(start),
        }
    }

    /// Fetches the current nonce and increments it by `step` atomically.
    /// Returns the value *before* the increment.
    pub fn fetch_add(&self, step: u64) -> Nonce {
        Nonce(self.inner.fetch_add(step, Ordering::Relaxed))
    }
}

// --- Network & Identity ---

/// Unique identifier for a node in the Chimera mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 16]); // UUID v4 compatible

impl NodeId {
    pub fn generate() -> Self {
        // In a real impl, this would use a proper UUID crate or RNG
        Self([0u8; 16]) 
    }
}

// --- Operational Metrics ---

/// Represents the operational cost of a mining cycle or strategy.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct OpCost {
    pub joules: f64,
    pub seconds: f64,
    pub dollars: f64,
}

impl OpCost {
    /// Calculates efficiency metric: Hashes per Joule (if provided) or Joules per Second (Power).
    pub fn power_draw(&self) -> f64 {
        if self.seconds > 0.0 {
            self.joules / self.seconds
        } else {
            0.0
        }
    }
}

/// Represents the thermal status of a hardware device.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ThermalState {
    pub celsius: f64,
    pub critical_threshold: f64,
    pub throttling: bool,
}

impl ThermalState {
    /// Checks if the device is approaching critical thermal limits.
    pub fn is_critical(&self) -> bool {
        self.celsius >= self.critical_threshold * 0.95
    }
}
>>>>>>> b1c3fa6ecf5982d921dbc44b3f253667a676f19b
