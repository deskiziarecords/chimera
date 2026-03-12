//! Chimera Bus
//!
//! Distributed messaging fabric for ChimeraOS.
//! Provides async node communication, RPC-style messaging,
//! and pub/sub topic routing for compute workloads.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use chimera_core::primitives::NodeId;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{RwLock, mpsc};


/// Errors produced by the messaging fabric.
#[derive(Error, Debug)]
pub enum BusError {

    #[error("Node not registered: {0}")]
    NodeNotFound(NodeId),

    #[error("Channel send failed")]
    SendFailed,

    #[error("Channel receive failed")]
    ReceiveFailed,

    #[error("Topic not found: {0}")]
    TopicNotFound(String),
}



/// Types of messages flowing through the Chimera bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusMessage {

    /// Task distribution
    ComputeTask {
        task_id: String,
        payload: Vec<u8>,
    },

    /// Task result
    ComputeResult {
        task_id: String,
        result: Vec<u8>,
    },

    /// Node health ping
    Heartbeat {
        node_id: NodeId,
        timestamp: u64,
    },

    /// Generic message
    Data {
        topic: String,
        payload: Vec<u8>,
    },
}



/// Internal channel wrapper.
#[derive(Clone)]
struct NodeChannel {

    sender: mpsc::Sender<BusMessage>,
}



/// Distributed messaging manager.
pub struct BusManager {

    /// Registered node channels
    nodes: Arc<RwLock<HashMap<NodeId, NodeChannel>>>,

    /// Topic subscribers
    topics: Arc<RwLock<HashMap<String, Vec<NodeId>>>>,
}



impl BusManager {

    /// Create a new messaging fabric.
    pub fn new() -> Self {

        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            topics: Arc::new(RwLock::new(HashMap::new())),
        }
    }



    /// Register a node with the bus.
    pub async fn register_node(
        &self,
        node_id: NodeId,
        sender: mpsc::Sender<BusMessage>,
    ) {

        let mut nodes = self.nodes.write().await;

        nodes.insert(
            node_id,
            NodeChannel { sender },
        );
    }



    /// Remove node from bus.
    pub async fn unregister_node(
        &self,
        node_id: NodeId,
    ) {

        let mut nodes = self.nodes.write().await;

        nodes.remove(&node_id);
    }



    /// Send message to a specific node.
    pub async fn send(
        &self,
        target: NodeId,
        message: BusMessage,
    ) -> Result<(), BusError> {

        let nodes = self.nodes.read().await;

        let channel = nodes
            .get(&target)
            .ok_or(BusError::NodeNotFound(target))?;

        channel
            .sender
            .send(message)
            .await
            .map_err(|_| BusError::SendFailed)
    }



    /// Broadcast message to all nodes.
    pub async fn broadcast(
        &self,
        message: BusMessage,
    ) {

        let nodes = self.nodes.read().await;

        for channel in nodes.values() {

            let _ = channel.sender.send(message.clone()).await;
        }
    }



    /// Subscribe a node to a topic.
    pub async fn subscribe(
        &self,
        topic: String,
        node_id: NodeId,
    ) {

        let mut topics = self.topics.write().await;

        topics
            .entry(topic)
            .or_insert_with(Vec::new)
            .push(node_id);
    }



    /// Publish a message to a topic.
    pub async fn publish(
        &self,
        topic: &str,
        payload: Vec<u8>,
    ) -> Result<(), BusError> {

        let topics = self.topics.read().await;

        let subscribers =
            topics.get(topic)
                .ok_or(BusError::TopicNotFound(topic.into()))?;

        let nodes = self.nodes.read().await;

        for node in subscribers {

            if let Some(channel) = nodes.get(node) {

                let msg = BusMessage::Data {
                    topic: topic.to_string(),
                    payload: payload.clone(),
                };

                let _ = channel.sender.send(msg).await;
            }
        }

        Ok(())
    }



    /// Send compute task to node.
    pub async fn dispatch_task(
        &self,
        node_id: NodeId,
        task_id: String,
        payload: Vec<u8>,
    ) -> Result<(), BusError> {

        self.send(
            node_id,
            BusMessage::ComputeTask {
                task_id,
                payload,
            },
        )
        .await
    }



    /// Report compute result.
    pub async fn send_result(
        &self,
        node_id: NodeId,
        task_id: String,
        result: Vec<u8>,
    ) -> Result<(), BusError> {

        self.send(
            node_id,
            BusMessage::ComputeResult {
                task_id,
                result,
            },
        )
        .await
    }



    /// Heartbeat broadcast.
    pub async fn heartbeat(
        &self,
        node_id: NodeId,
        timestamp: u64,
    ) {

        let msg = BusMessage::Heartbeat {
            node_id,
            timestamp,
        };

        self.broadcast(msg).await;
    }
}



/// Create a local node channel pair.
pub fn create_node_channel(
    buffer: usize,
) -> (mpsc::Sender<BusMessage>, mpsc::Receiver<BusMessage>) {

    mpsc::channel(buffer)
}



#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_node_registration() {

        let bus = BusManager::new();

        let node_id = NodeId::default();

        let (tx, mut rx) = create_node_channel(16);

        bus.register_node(node_id, tx).await;

        bus.send(
            node_id,
            BusMessage::Data {
                topic: "test".into(),
                payload: vec![1, 2, 3],
            },
        )
        .await
        .unwrap();

        let msg = rx.recv().await.unwrap();

        match msg {
            BusMessage::Data { topic, .. } => {
                assert_eq!(topic, "test");
            }
            _ => panic!("unexpected message"),
        }
    }
}
