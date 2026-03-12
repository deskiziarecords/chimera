// chimera-bus/src/lib.rs
use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Event {
    pub topic: String,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

pub struct EventBus {
    tx: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(10000);
        Self { tx }
    }

    pub fn publish(&self, topic: &str, payload: Vec<u8>) {
        let _ = self.tx.send(Event {
            topic: topic.to_string(),
            payload,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        });
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}