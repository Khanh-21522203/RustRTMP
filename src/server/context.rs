use crate::{Error, Result};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::net::IpAddr;
use crate::server::config::ServerConfig;
use crate::server::registry::PublisherRegistry;

pub struct ServerContext {
    /// Server configuration
    config: Arc<ServerConfig>,

    /// Publisher registry
    publishers: Arc<PublisherRegistry>,

    /// Connection ID counter
    connection_counter: AtomicU64,

    /// IP connection counts
    ip_counts: Arc<RwLock<HashMap<IpAddr, usize>>>,
}

impl ServerContext {
    /// Create new context
    pub fn new(config: Arc<ServerConfig>) -> Self {
        ServerContext {
            config,
            publishers: Arc::new(PublisherRegistry::new()),
            connection_counter: AtomicU64::new(0),
            ip_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get publisher registry
    pub fn publishers(&self) -> Arc<PublisherRegistry> {
        self.publishers.clone()
    }

    /// Generate unique connection ID
    pub fn generate_connection_id(&self) -> String {
        let id = self.connection_counter.fetch_add(1, Ordering::SeqCst);
        format!("conn-{}", id)
    }

    /// Check if can accept from IP
    pub async fn can_accept_from_ip(&self, ip: IpAddr) -> bool {
        let counts = self.ip_counts.read().await;
        let count = counts.get(&ip).copied().unwrap_or(0);
        count < self.config.max_connections_per_ip
    }

    /// Increment IP connection count
    pub async fn increment_ip_count(&self, ip: IpAddr) {
        let mut counts = self.ip_counts.write().await;
        *counts.entry(ip).or_insert(0) += 1;
    }

    /// Decrement IP connection count
    pub async fn decrement_ip_count(&self, ip: IpAddr) {
        let mut counts = self.ip_counts.write().await;
        if let Some(count) = counts.get_mut(&ip) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                counts.remove(&ip);
            }
        }
    }
}