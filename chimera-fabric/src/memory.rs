//! Hardware-level memory abstraction.
//! Manages allocation across CPU RAM, GPU VRAM, and FPGA HBM.

use serde::{Deserialize, Serialize};

use chimera_core::primitives::{NodeId, OpCost};

use crate::topology::{Topology, DeviceType};
use crate::FabricError;

use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;


/// Types of memory regions available in the fabric.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryRegionType {
    /// System RAM
    Host,

    /// GPU VRAM / FPGA HBM
    Device,

    /// Shared host-device mapped memory
    Shared,
}


/// Represents an allocated memory region.
#[derive(Debug)]
pub struct MemoryHandle {

    /// Pointer to allocated region
    pub ptr: NonNull<u8>,

    /// Size in bytes
    pub size: usize,

    /// Region type
    pub region_type: MemoryRegionType,

    /// Device/node where memory resides
    pub node_id: NodeId,

    /// Backing storage to keep allocation alive
    buffer: Arc<Vec<u8>>,
}


/// Safe wrapper around raw pointers
unsafe impl Send for MemoryHandle {}
unsafe impl Sync for MemoryHandle {}


/// Telemetry snapshot for memory usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {

    pub total_allocated: usize,
    pub capacity: usize,
    pub utilization: f64,
}


/// Central memory manager for Chimera Fabric.
pub struct MemoryManager {

    /// Current allocated bytes
    total_allocated: AtomicUsize,

    /// Maximum memory capacity across fabric
    max_capacity: usize,

    /// Optional topology reference for placement decisions
    topology_nodes: usize,
}

impl MemoryManager {

    /// Initialize memory manager based on detected topology.
    pub fn new(topology: &Topology) -> Result<Self, FabricError> {

        let mut capacity = 0usize;

        for device in topology.nodes.values() {

            let device_capacity = match device.device_type {
                DeviceType::CPU => device.compute_units as usize * 2 * 1024 * 1024 * 1024,
                DeviceType::GPU => device.compute_units as usize * 4 * 1024 * 1024 * 1024,
                DeviceType::FPGA => device.compute_units as usize * 1024 * 1024 * 1024,
                DeviceType::ASIC => device.compute_units as usize * 512 * 1024 * 1024,
                DeviceType::Unknown => 512 * 1024 * 1024,
            };

            capacity += device_capacity;
        }

        if capacity == 0 {
            capacity = 1024 * 1024 * 1024; // fallback 1GB
        }

        Ok(Self {
            total_allocated: AtomicUsize::new(0),
            max_capacity: capacity,
            topology_nodes: topology.nodes.len(),
        })
    }


    /// Allocate memory region.
    pub async fn allocate(
        &self,
        size: usize,
        region_type: MemoryRegionType,
        node_id: NodeId,
    ) -> Result<MemoryHandle, FabricError> {

        let current = self.total_allocated.load(Ordering::Relaxed);

        if current + size > self.max_capacity {
            return Err(FabricError::AllocationFailed(
                "Fabric memory exhausted".into(),
            ));
        }

        let buffer = vec![0u8; size];
        let arc_buf = Arc::new(buffer);

        let ptr = NonNull::new(arc_buf.as_ptr() as *mut u8)
            .ok_or_else(|| FabricError::AllocationFailed("Null allocation".into()))?;

        self.total_allocated
            .fetch_add(size, Ordering::SeqCst);

        Ok(MemoryHandle {
            ptr,
            size,
            region_type,
            node_id,
            buffer: arc_buf,
        })
    }


    /// Free memory region.
    pub fn deallocate(&self, handle: MemoryHandle) {

        self.total_allocated
            .fetch_sub(handle.size, Ordering::SeqCst);

        // Arc<Vec<u8>> automatically drops
    }


    /// Zero-copy clone of handle.
    pub fn clone_handle(&self, handle: &MemoryHandle) -> MemoryHandle {

        MemoryHandle {
            ptr: handle.ptr,
            size: handle.size,
            region_type: handle.region_type,
            node_id: handle.node_id,
            buffer: Arc::clone(&handle.buffer),
        }
    }


    /// Estimate transfer cost between memory regions.
    pub fn transfer_cost(
        &self,
        size: usize,
        transfers: u32,
    ) -> OpCost {

        let bandwidth = 25_000_000_000.0; // 25 GB/s typical PCIe

        let seconds =
            (size as f64 / bandwidth) * transfers as f64;

        let joules =
            (size as f64 * 0.00005) + transfers as f64 * 0.002;

        let dollars = joules * 0.0001;

        OpCost {
            joules,
            seconds,
            dollars,
        }
    }


    /// Estimate compute locality cost (used by scheduler).
    pub fn locality_penalty(
        &self,
        region: MemoryRegionType,
    ) -> f64 {

        match region {
            MemoryRegionType::Host => 1.0,
            MemoryRegionType::Shared => 1.5,
            MemoryRegionType::Device => 0.8,
        }
    }


    /// Get memory usage statistics.
    pub fn stats(&self) -> MemoryStats {

        let allocated = self.total_allocated.load(Ordering::Relaxed);

        MemoryStats {
            total_allocated: allocated,
            capacity: self.max_capacity,
            utilization: allocated as f64 / self.max_capacity as f64,
        }
    }


    /// Estimate maximum parallel allocations supported.
    pub fn max_parallel_buffers(&self, average_size: usize) -> usize {

        if average_size == 0 {
            return 0;
        }

        self.max_capacity / average_size
    }
}
