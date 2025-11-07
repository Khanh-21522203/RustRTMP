use crate::{Error, PublisherRegistry, Result};
use crate::protocol::RtmpPacket;
use crate::message::HandlerContext;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub struct ConnectionContext {
    /// Connection ID
    connection_id: String,

    /// Properties storage
    properties: Arc<RwLock<HashMap<String, String>>>,

    /// Outgoing packet sender
    packet_sender: mpsc::Sender<RtmpPacket>,

    /// Chunk size settings
    chunk_size_in: Arc<RwLock<usize>>,
    chunk_size_out: Arc<RwLock<usize>>,
}

impl ConnectionContext {
    /// Create new context
    pub fn new(
        connection_id: String,
        packet_sender: mpsc::Sender<RtmpPacket>,
    ) -> Self {
        ConnectionContext {
            connection_id,
            properties: Arc::new(RwLock::new(HashMap::new())),
            packet_sender,
            chunk_size_in: Arc::new(RwLock::new(128)),
            chunk_size_out: Arc::new(RwLock::new(128)),
        }
    }

    /// Get connection ID
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }

    /// Set chunk size for incoming
    pub async fn set_chunk_size_in(&self, size: usize) {
        let mut chunk_size = self.chunk_size_in.write().await;
        *chunk_size = size;
    }

    /// Set chunk size for outgoing
    pub async fn set_chunk_size_out(&self, size: usize) {
        let mut chunk_size = self.chunk_size_out.write().await;
        *chunk_size = size;
    }
}

#[async_trait::async_trait]
impl HandlerContext for ConnectionContext {
    async fn send_packet(&self, packet: RtmpPacket) -> Result<()> {
        self.packet_sender.send(packet).await
            .map_err(|_| Error::connection("Failed to send packet"))
    }

    async fn get_property(&self, key: &str) -> Option<String> {
        let props = self.properties.read().await;
        props.get(key).cloned()
    }

    async fn set_property(&self, key: String, value: String) {
        let mut props = self.properties.write().await;
        props.insert(key, value);
    }

    async fn remove_property(&self, key: &str) {
        let mut props = self.properties.write().await;
        props.remove(key);
    }

    fn get_publisher_registry(&self) -> Option<Arc<PublisherRegistry>> {
        // Note: This should be injected during ConnectionContext creation
        // For now, return None - will be implemented in Phase 10 (Stream Management)
        None
    }
}