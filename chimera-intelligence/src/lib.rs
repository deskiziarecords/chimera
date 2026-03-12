//! Chimera Intelligence
//!
//! Adaptive optimization layer for ChimeraOS.
//! Observes cluster behavior and applies differentiable
//! optimization strategies to improve scheduling,
//! resource usage, and distributed compute performance.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use tokio::sync::RwLock;

use chimera_core::primitives::NodeId;
use chimera_scheduler::{Scheduler, SchedulerMetrics};
use chimera_fabric::topology::FabricTopology;
use chimera_jax::{DifferentiableHash, OptimizationMetrics};



/// Errors produced by the intelligence layer
#[derive(Error, Debug)]
pub enum IntelligenceError {

    #[error("Scheduler unavailable")]
    SchedulerUnavailable,

    #[error("Optimization failed")]
    OptimizationFailed,

    #[error("Node not found")]
    NodeNotFound,
}



/// Strategy parameters that guide optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {

    /// learning rate for optimization
    pub learning_rate: f64,

    /// maximum gradient norm
    pub gradient_clip: f64,

    /// scheduling bias factor
    pub scheduling_bias: f64,
}



impl Default for Strategy {

    fn default() -> Self {

        Self {
            learning_rate: 0.01,
            gradient_clip: 10.0,
            scheduling_bias: 1.0,
        }
    }
}



/// Node-level telemetry used for optimization.
#[derive(Debug, Clone, Default)]
pub struct NodeTelemetry {

    pub cpu_load: f64,
    pub active_tasks: usize,
    pub latency: f64,
}



/// Intelligence engine
pub struct IntelligenceEngine {

    scheduler: Arc<Scheduler>,

    topology: Arc<RwLock<FabricTopology>>,

    optimizer: Arc<RwLock<DifferentiableHash>>,

    strategy: Arc<RwLock<Strategy>>,

    telemetry: Arc<RwLock<HashMap<NodeId, NodeTelemetry>>>,

    history: Arc<RwLock<Vec<OptimizationMetrics>>>,
}



impl IntelligenceEngine {

    pub fn new(
        scheduler: Arc<Scheduler>,
        topology: Arc<RwLock<FabricTopology>>,
    ) -> Self {

        Self {

            scheduler,

            topology,

            optimizer: Arc::new(RwLock::new(
                DifferentiableHash::new()
            )),

            strategy: Arc::new(RwLock::new(
                Strategy::default()
            )),

            telemetry: Arc::new(RwLock::new(HashMap::new())),

            history: Arc::new(RwLock::new(Vec::new())),
        }
    }



    /// Update node telemetry
    pub async fn update_telemetry(
        &self,
        node_id: NodeId,
        cpu_load: f64,
        active_tasks: usize,
        latency: f64,
    ) {

        let mut telemetry = self.telemetry.write().await;

        telemetry.insert(
            node_id,
            NodeTelemetry {
                cpu_load,
                active_tasks,
                latency,
            },
        );
    }



    /// Analyze cluster metrics
    pub async fn analyze_cluster(
        &self,
    ) -> Result<ClusterAnalysis, IntelligenceError> {

        let metrics = self.scheduler.metrics().await;

        let telemetry = self.telemetry.read().await;

        let node_count = telemetry.len();

        let avg_load = if node_count == 0 {
            0.0
        } else {
            telemetry
                .values()
                .map(|n| n.cpu_load)
                .sum::<f64>() / node_count as f64
        };

        Ok(ClusterAnalysis {

            pending_tasks: metrics.pending,
            running_tasks: metrics.running,
            completed_tasks: metrics.completed,
            avg_cpu_load: avg_load,
            node_count,
        })
    }



    /// Run differentiable optimization step
    pub async fn optimize(
        &self,
        parameters: Vec<f64>,
    ) -> Result<OptimizationMetrics, IntelligenceError> {

        let optimizer = self.optimizer.read().await;

        let (loss, gradient) =
            optimizer.compute_gradient(&parameters);

        let metrics =
            OptimizationMetrics::new(&gradient, loss, 1);

        let mut history = self.history.write().await;

        history.push(metrics.clone());

        Ok(metrics)
    }



    /// Apply optimization updates to strategy
    pub async fn update_strategy(
        &self,
        metrics: OptimizationMetrics,
    ) {

        let mut strategy = self.strategy.write().await;

        if metrics.gradient_norm > strategy.gradient_clip {

            strategy.learning_rate *= 0.9;
        } else {

            strategy.learning_rate *= 1.05;
        }

        strategy.learning_rate =
            strategy.learning_rate.clamp(0.0001, 0.1);
    }



    /// Intelligence scheduling cycle
    pub async fn control_loop(
        &self,
    ) -> Result<(), IntelligenceError> {

        let analysis = self.analyze_cluster().await?;

        let params = vec![
            analysis.pending_tasks as f64,
            analysis.running_tasks as f64,
            analysis.avg_cpu_load,
        ];

        let metrics = self.optimize(params).await?;

        self.update_strategy(metrics).await;

        Ok(())
    }



    /// Retrieve current strategy
    pub async fn strategy(
        &self,
    ) -> Strategy {

        self.strategy.read().await.clone()
    }



    /// Optimization history
    pub async fn history(
        &self,
    ) -> Vec<OptimizationMetrics> {

        self.history.read().await.clone()
    }
}



/// Cluster state summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterAnalysis {

    pub pending_tasks: usize,

    pub running_tasks: usize,

    pub completed_tasks: usize,

    pub avg_cpu_load: f64,

    pub node_count: usize,
}



#[cfg(test)]
mod tests {

    use super::*;
    use tokio::sync::mpsc;

    use chimera_bus::BusManager;
    use chimera_scheduler::Scheduler;
    use chimera_fabric::topology::FabricTopology;

    #[tokio::test]
    async fn test_intelligence_cycle() {

        let bus = Arc::new(BusManager::new());

        let topology =
            Arc::new(RwLock::new(FabricTopology::default()));

        let scheduler =
            Arc::new(Scheduler::new(bus, topology.clone()));

        let engine =
            IntelligenceEngine::new(scheduler, topology);

        let metrics =
            engine.optimize(vec![1.0,2.0,3.0]).await.unwrap();

        assert!(metrics.loss_value > 0.0);
    }
}
