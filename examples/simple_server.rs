// Simple RTMP Server Example
// 
// This example demonstrates:
// - Creating a basic RTMP server
// - Accepting connections
// - Handling publish/play requests
// - Graceful shutdown
//
// Usage:
//   cargo run --example simple_server

use rtmp::{RtmpServer, ServerConfig, Result};
use log::info;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    // Create server configuration
    let config = ServerConfig::builder()
        .host("0.0.0.0")
        .port(1935)
        .max_connections(100)
        .chunk_size(4096)
        .build()?;
    
    info!("Starting RTMP server on {}:{}", config.host, config.port);
    info!("Configuration:");
    info!("  - Max connections: {}", config.max_connections);
    info!("  - Chunk size: {}", config.chunk_size);
    info!("  - GOP cache enabled: {}", config.gop_cache_enabled);
    info!("  - GOP cache size: {}", config.gop_cache_size);
    
    // Create server
    let server = Arc::new(RtmpServer::new(config));
    
    // Setup graceful shutdown
    let server_clone = server.clone();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, shutting down server...");
                server_clone.shutdown().await;
            }
            Err(err) => {
                eprintln!("Error setting up signal handler: {}", err);
            }
        }
    });
    
    // Start server
    info!("Server ready to accept connections");
    info!("Press Ctrl+C to stop");
    server.listen().await?;
    
    info!("Server stopped");
    Ok(())
}
