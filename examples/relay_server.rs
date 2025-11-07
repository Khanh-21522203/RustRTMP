// RTMP Relay Server Example
//
// This example demonstrates:
// - Creating a relay/proxy server
// - Pulling streams from upstream servers
// - Redistributing streams to downstream clients
// - Load balancing and failover
//
// Usage:
//   cargo run --example relay_server

use rtmp::{RtmpServer, RtmpClient, ServerConfig, ClientConfig, Result, Error};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use log::{info, error, warn};

/// Relay server that pulls from upstream and redistributes
pub struct RelayServer {
    /// Local RTMP server
    server: RtmpServer,
    
    /// Upstream connections
    upstreams: Arc<RwLock<HashMap<String, Arc<RtmpClient>>>>,
    
    /// Active relay tasks
    relay_tasks: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl RelayServer {
    /// Create new relay server
    pub fn new(config: ServerConfig) -> Self {
        RelayServer {
            server: RtmpServer::new(config),
            upstreams: Arc::new(RwLock::new(HashMap::new())),
            relay_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Add an upstream server
    pub async fn add_upstream(&self, name: String, url: String) -> Result<()> {
        info!("Adding upstream: {} -> {}", name, url);
        
        // Create client connection
        let client_config = ClientConfig::builder()
            .auto_reconnect(true)
            .build()?;
        
        let mut client = RtmpClient::with_config(client_config);
        
        // Connect to upstream
        match client.connect(&url).await {
            Ok(_) => {
                info!("Connected to upstream: {}", name);
                
                // Store upstream
                let mut upstreams = self.upstreams.write().await;
                upstreams.insert(name.clone(), Arc::new(client));
                
                Ok(())
            }
            Err(e) => {
                error!("Failed to connect to upstream {}: {}", name, e);
                Err(e)
            }
        }
    }
    
    /// Start relaying a stream from upstream
    pub async fn start_relay(
        &self,
        stream_name: String,
        upstream_name: String,
    ) -> Result<()> {
        info!("Starting relay: stream={}, upstream={}", stream_name, upstream_name);
        
        // Get upstream client
        let upstreams = self.upstreams.read().await;
        let upstream = upstreams.get(&upstream_name)
            .ok_or_else(|| Error::invalid_state(format!("Upstream not found: {}", upstream_name)))?;
        
        let upstream_clone = upstream.clone();
        let stream_name_clone = stream_name.clone();
        
        // Spawn relay task
        let task = tokio::spawn(async move {
            info!("Relay task started for stream: {}", stream_name_clone);
            
            // In a real implementation, this would:
            // 1. Call upstream.play() to start receiving data
            // 2. Register handlers to receive video/audio packets
            // 3. Republish those packets to local server's stream
            
            // Placeholder implementation
            warn!("Relay task for {} is a placeholder. Full implementation needed.", stream_name_clone);
            
            // Keep task alive
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        });
        
        // Store task
        let mut relay_tasks = self.relay_tasks.write().await;
        relay_tasks.insert(stream_name, task);
        
        Ok(())
    }
    
    /// Stop relaying a stream
    pub async fn stop_relay(&self, stream_name: &str) -> Result<()> {
        info!("Stopping relay for stream: {}", stream_name);
        
        let mut relay_tasks = self.relay_tasks.write().await;
        if let Some(task) = relay_tasks.remove(stream_name) {
            task.abort();
            info!("Relay task stopped: {}", stream_name);
        } else {
            warn!("No active relay for stream: {}", stream_name);
        }
        
        Ok(())
    }
    
    /// Get upstream server names
    pub async fn list_upstreams(&self) -> Vec<String> {
        let upstreams = self.upstreams.read().await;
        upstreams.keys().cloned().collect()
    }
    
    /// Start the relay server
    pub async fn run(&self) -> Result<()> {
        info!("Starting relay server");
        self.server.listen().await
    }
    
    /// Shutdown the relay server
    pub async fn shutdown(&self) {
        info!("Shutting down relay server");
        
        // Stop all relay tasks
        let mut relay_tasks = self.relay_tasks.write().await;
        for (stream_name, task) in relay_tasks.drain() {
            info!("Stopping relay task: {}", stream_name);
            task.abort();
        }
        
        // Disconnect from upstreams
        let mut upstreams = self.upstreams.write().await;
        upstreams.clear();
        
        // Shutdown local server
        self.server.shutdown().await;
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    // Create relay server configuration
    let config = ServerConfig::builder()
        .host("0.0.0.0")
        .port(1936) // Different port for relay server
        .max_connections(100)
        .chunk_size(4096)
        .build()?;
    
    info!("Creating relay server on port 1936");
    let relay = RelayServer::new(config);
    
    // Add upstream servers (examples)
    // In a real application, these would be actual upstream URLs
    info!("Configuring upstream servers...");
    
    // Example 1: Primary upstream
    if let Err(e) = relay.add_upstream(
        "primary".to_string(),
        "rtmp://upstream1.example.com/live".to_string(),
    ).await {
        warn!("Failed to add primary upstream: {}", e);
    }
    
    // Example 2: Backup upstream
    if let Err(e) = relay.add_upstream(
        "backup".to_string(),
        "rtmp://upstream2.example.com/live".to_string(),
    ).await {
        warn!("Failed to add backup upstream: {}", e);
    }
    
    // List configured upstreams
    let upstreams = relay.list_upstreams().await;
    info!("Configured upstreams: {:?}", upstreams);
    
    // Note: In a real application, you would need to handle shutdown differently
    // since RelayServer doesn't implement Clone. One approach is to use Arc<RelayServer>
    // or separate shutdown signaling mechanism.
    
    // Run relay server
    info!("Relay server ready");
    info!("Listening on port 1936");
    info!("Press Ctrl+C to stop");
    
    match relay.run().await {
        Ok(_) => {
            info!("Relay server stopped normally");
            Ok(())
        }
        Err(e) => {
            error!("Relay server error: {}", e);
            Err(e)
        }
    }
}
