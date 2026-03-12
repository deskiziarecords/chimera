//! Chimera Scheduler
//!
//! Intelligent workload scheduler for ChimeraOS.
//! Responsible for task placement, node load balancing,
//! and distributed compute orchestration.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use tokio::sync::RwLock;

use chimera_core::primitives::NodeId;
use chimera_bus::{BusManager, BusMessage};
use chimera_fabric::topology::FabricTopology;



/// Scheduler errors
#[derive(Error, Debug)]
pub enum SchedulerError {

    #[error("No nodes available for scheduling")]
    NoNodesAvailable,

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Dispatch error")]
    DispatchError,
}



/// Task state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskState {

    Pending,
    Running,
    Completed,
    Failed,
}



/// Compute task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {

    pub id: String,

    pub payload: Vec<u8>,

    pub assigned_node: Option<NodeId>,

    pub state: TaskState,
}



/// Node load metrics
#[derive(Debug, Clone, Default)]
pub struct NodeLoad {

    pub active_tasks: usize,

    pub last_heartbeat: u64,
}



/// Core scheduler
pub struct Scheduler {

    /// Messaging fabric
    bus: Arc<BusManager>,

    /// Fabric topology
    topology: Arc<RwLock<FabricTopology>>,

    /// Task registry
    tasks: Arc<RwLock<HashMap<String, Task>>>,

    /// Node load tracking
    node_load: Arc<RwLock<HashMap<NodeId, NodeLoad>>>,

    /// Pending queue
    queue: Arc<RwLock<VecDeque<String>>>,
}



impl Scheduler {

    pub fn new(
        bus: Arc<BusManager>,
        topology: Arc<RwLock<FabricTopology>>,
    ) -> Self {

        Self {
            bus,
            topology,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            node_load: Arc::new(RwLock::new(HashMap::new())),
            queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }



    /// Submit new compute task
    pub async fn submit_task(
        &self,
        id: String,
        payload: Vec<u8>,
    ) {

        let task = Task {
            id: id.clone(),
            payload,
            assigned_node: None,
            state: TaskState::Pending,
        };

        let mut tasks = self.tasks.write().await;
        tasks.insert(id.clone(), task);

        let mut queue = self.queue.write().await;
        queue.push_back(id);
    }



    /// Register node for scheduling
    pub async fn register_node(
        &self,
        node_id: NodeId,
    ) {

        let mut nodes = self.node_load.write().await;

        nodes.insert(
            node_id,
            NodeLoad::default(),
        );
    }



    /// Update heartbeat timestamp
    pub async fn update_heartbeat(
        &self,
        node_id: NodeId,
        timestamp: u64,
    ) {

        let mut loads = self.node_load.write().await;

        if let Some(load) = loads.get_mut(&node_id) {

            load.last_heartbeat = timestamp;
        }
    }



    /// Select least loaded node
    async fn select_node(&self) -> Result<NodeId, SchedulerError> {

        let loads = self.node_load.read().await;

        loads
            .iter()
            .min_by_key(|(_, load)| load.active_tasks)
            .map(|(id, _)| *id)
            .ok_or(SchedulerError::NoNodesAvailable)
    }



    /// Main scheduling loop
    pub async fn schedule(&self) -> Result<(), SchedulerError> {

        let mut queue = self.queue.write().await;

        if let Some(task_id) = queue.pop_front() {

            let mut tasks = self.tasks.write().await;

            let task =
                tasks.get_mut(&task_id)
                    .ok_or(SchedulerError::TaskNotFound(task_id.clone()))?;

            let node = self.select_node().await?;

            self.bus
                .dispatch_task(
                    node,
                    task.id.clone(),
                    task.payload.clone(),
                )
                .await
                .map_err(|_| SchedulerError::DispatchError)?;

            task.assigned_node = Some(node);
            task.state = TaskState::Running;

            let mut loads = self.node_load.write().await;

            if let Some(load) = loads.get_mut(&node) {

                load.active_tasks += 1;
            }
        }

        Ok(())
    }



    /// Handle result returned from worker node
    pub async fn handle_result(
        &self,
        node_id: NodeId,
        task_id: String,
    ) -> Result<(), SchedulerError> {

        let mut tasks = self.tasks.write().await;

        let task =
            tasks.get_mut(&task_id)
                .ok_or(SchedulerError::TaskNotFound(task_id.clone()))?;

        task.state = TaskState::Completed;

        let mut loads = self.node_load.write().await;

        if let Some(load) = loads.get_mut(&node_id) {

            if load.active_tasks > 0 {
                load.active_tasks -= 1;
            }
        }

        Ok(())
    }



    /// Get task status
    pub async fn task_status(
        &self,
        task_id: &str,
    ) -> Option<TaskState> {

        let tasks = self.tasks.read().await;

        tasks.get(task_id).map(|t| t.state.clone())
    }



    /// Scheduler metrics
    pub async fn metrics(&self) -> SchedulerMetrics {

        let tasks = self.tasks.read().await;

        let mut pending = 0;
        let mut running = 0;
        let mut completed = 0;

        for task in tasks.values() {

            match task.state {

                TaskState::Pending => pending += 1,
                TaskState::Running => running += 1,
                TaskState::Completed => completed += 1,
                TaskState::Failed => {}
            }
        }

        SchedulerMetrics {
            pending,
            running,
            completed,
        }
    }
}



/// Scheduler statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerMetrics {

    pub pending: usize,
    pub running: usize,
    pub completed: usize,
}



#[cfg(test)]
mod tests {

    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_scheduler_queue() {

        let bus = Arc::new(BusManager::new());

        let topology =
            Arc::new(RwLock::new(FabricTopology::default()));

        let scheduler = Scheduler::new(bus.clone(), topology);

        let node = NodeId::default();

        let (tx, _rx) = mpsc::channel(8);

        bus.register_node(node, tx).await;

        scheduler.register_node(node).await;

        scheduler
            .submit_task(
                "task1".into(),
                vec![1,2,3],
            )
            .await;

        scheduler.schedule().await.unwrap();

        let status =
            scheduler.task_status("task1").await.unwrap();

        match status {

            TaskState::Running => {}
            _ => panic!("task not running"),
        }
    }
}
