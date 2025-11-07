use std::time::Duration;
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host to bind
    pub host: String,

    /// Port to bind
    pub port: u16,

    /// Maximum connections
    pub max_connections: usize,

    /// Maximum connections per IP
    pub max_connections_per_ip: usize,

    /// Chunk size
    pub chunk_size: u32,

    /// Window acknowledgement size
    pub window_ack_size: u32,

    /// Peer bandwidth
    pub peer_bandwidth: u32,

    /// Ping interval
    pub ping_interval: Duration,

    /// Timeout for idle connections
    pub idle_timeout: Duration,

    /// GOP cache size
    pub gop_cache_size: usize,

    /// Enable GOP cache
    pub gop_cache_enabled: bool,

    /// Allow publishing
    pub allow_publish: bool,

    /// Allow playing
    pub allow_play: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 1935,
            max_connections: 1000,
            max_connections_per_ip: 10,
            chunk_size: 4096,
            window_ack_size: 2500000,
            peer_bandwidth: 2500000,
            ping_interval: Duration::from_secs(60),
            idle_timeout: Duration::from_secs(300),
            gop_cache_size: 10,
            gop_cache_enabled: true,
            allow_publish: true,
            allow_play: true,
        }
    }
}

impl ServerConfig {
    /// Create config builder
    pub fn builder() -> ServerConfigBuilder {
        ServerConfigBuilder::new()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            return Err(Error::config("Invalid port: 0"));
        }

        if self.max_connections == 0 {
            return Err(Error::config("Invalid max_connections: 0"));
        }

        if self.chunk_size < 128 {
            return Err(Error::config("Chunk size must be at least 128"));
        }

        if self.chunk_size > 65536 {
            return Err(Error::config("Chunk size must not exceed 65536"));
        }

        Ok(())
    }
}

/// Builder for ServerConfig
pub struct ServerConfigBuilder {
    config: ServerConfig,
}

impl ServerConfigBuilder {
    /// Create new builder
    pub fn new() -> Self {
        ServerConfigBuilder {
            config: ServerConfig::default(),
        }
    }

    /// Set host
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.config.host = host.into();
        self
    }

    /// Set port
    pub fn port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Set max connections
    pub fn max_connections(mut self, max: usize) -> Self {
        self.config.max_connections = max;
        self
    }

    /// Set chunk size
    pub fn chunk_size(mut self, size: u32) -> Self {
        self.config.chunk_size = size;
        self
    }

    /// Build configuration
    pub fn build(self) -> Result<ServerConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}