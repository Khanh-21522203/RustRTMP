use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{Error, Result};

#[derive(Clone)]
pub struct PublisherInfo {
    /// Connection ID
    pub connection_id: String,

    /// Stream name
    pub stream_name: String,

    /// Stream ID
    pub stream_id: u32,

    /// Publishing start time
    pub started_at: u32,

    /// Metadata
    pub metadata: Option<HashMap<String, crate::amf::Amf0Value>>,

    /// Subscriber count
    pub subscriber_count: Arc<RwLock<usize>>,
}

pub struct PublisherRegistry {
    /// Publishers by stream name
    publishers: Arc<RwLock<HashMap<String, PublisherInfo>>>,
}

impl PublisherRegistry {
    /// Create new registry
    pub fn new() -> Self {
        PublisherRegistry {
            publishers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register publisher
    pub async fn register(
        &self,
        stream_name: String,
        connection_id: String,
        stream_id: u32,
    ) -> Result<()> {
        let mut publishers = self.publishers.write().await;

        // Check if already publishing
        if publishers.contains_key(&stream_name) {
            return Err(Error::stream(format!(
                "Stream '{}' is already being published",
                stream_name
            )));
        }

        // Add publisher
        publishers.insert(stream_name.clone(), PublisherInfo {
            connection_id,
            stream_name,
            stream_id,
            started_at: crate::utils::current_timestamp(),
            metadata: None,
            subscriber_count: Arc::new(RwLock::new(0)),
        });

        Ok(())
    }

    /// Unregister publisher
    pub async fn unregister(&self, stream_name: &str) -> Result<()> {
        let mut publishers = self.publishers.write().await;
        publishers.remove(stream_name)
            .ok_or_else(|| Error::stream(format!("Stream '{}' not found", stream_name)))?;
        Ok(())
    }

    /// Get publisher info
    pub async fn get(&self, stream_name: &str) -> Option<PublisherInfo> {
        let publishers = self.publishers.read().await;
        publishers.get(stream_name).cloned()
    }

    /// Check if stream is being published
    pub async fn is_publishing(&self, stream_name: &str) -> bool {
        let publishers = self.publishers.read().await;
        publishers.contains_key(stream_name)
    }

    /// Update metadata
    pub async fn update_metadata(
        &self,
        stream_name: &str,
        metadata: HashMap<String, crate::amf::Amf0Value>,
    ) -> Result<()> {
        let mut publishers = self.publishers.write().await;
        let publisher = publishers.get_mut(stream_name)
            .ok_or_else(|| Error::stream(format!("Stream '{}' not found", stream_name)))?;
        publisher.metadata = Some(metadata);
        Ok(())
    }

    /// Get all publishers
    pub async fn get_all(&self) -> Vec<PublisherInfo> {
        let publishers = self.publishers.read().await;
        publishers.values().cloned().collect()
    }

    /// Increment subscriber count
    pub async fn increment_subscribers(&self, stream_name: &str) -> Result<()> {
        let publishers = self.publishers.read().await;
        let publisher = publishers.get(stream_name)
            .ok_or_else(|| Error::stream(format!("Stream '{}' not found", stream_name)))?;

        let mut count = publisher.subscriber_count.write().await;
        *count += 1;
        Ok(())
    }

    /// Decrement subscriber count
    pub async fn decrement_subscribers(&self, stream_name: &str) -> Result<()> {
        let publishers = self.publishers.read().await;
        let publisher = publishers.get(stream_name)
            .ok_or_else(|| Error::stream(format!("Stream '{}' not found", stream_name)))?;

        let mut count = publisher.subscriber_count.write().await;
        *count = count.saturating_sub(1);
        Ok(())
    }
}