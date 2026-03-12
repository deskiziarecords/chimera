// chimera-scheduler/src/lib.rs
use chimera_core::primitives::{NodeId, OpCost};
use tokio::sync::mpsc;

pub struct Task {
    pub id: u64,
    pub priority: u8,
    pub payload: Vec<u8>,
    pub target_node: Option<NodeId>,
}

pub struct Scheduler {
    tx: mpsc::Sender<Task>,
    rx: mpsc::Receiver<Task>,
}

impl Scheduler {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(1000);
        Self { tx, rx }
    }

    pub async fn submit(&self, task: Task) -> Result<(), SchedulerError> {
        self.tx.send(task).await.map_err(|_| SchedulerError::ChannelClosed)
    }

    pub async fn next(&mut self) -> Option<Task> {
        self.rx.recv().await
    }
}