use std::time::Duration;
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Connection timeout
    pub connect_timeout: Duration,

    /// Read timeout
    pub read_timeout: Duration,

    /// Write timeout
    pub write_timeout: Duration,

    /// Chunk size
    pub chunk_size: u32,

    /// Window acknowledgement size
    pub window_ack_size: u32,

    /// Reconnect on failure
    pub auto_reconnect: bool,

    /// Maximum reconnect attempts
    pub max_reconnect_attempts: usize,

    /// Reconnect delay
    pub reconnect_delay: Duration,

    /// Enable audio
    pub enable_audio: bool,

    /// Enable video
    pub enable_video: bool,

    /// Buffer time in milliseconds
    pub buffer_time: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            chunk_size: 4096,
            window_ack_size: 2500000,
            auto_reconnect: false,
            max_reconnect_attempts: 3,
            reconnect_delay: Duration::from_secs(5),
            enable_audio: true,
            enable_video: true,
            buffer_time: 1000,
        }
    }
}

impl ClientConfig {
    /// Create config builder
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::new()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.chunk_size < 128 {
            return Err(Error::config("Chunk size must be at least 128"));
        }

        if self.chunk_size > 65536 {
            return Err(Error::config("Chunk size must not exceed 65536"));
        }

        Ok(())
    }
}

/// Builder for ClientConfig
pub struct ClientConfigBuilder {
    config: ClientConfig,
}

impl ClientConfigBuilder {
    /// Create new builder
    pub fn new() -> Self {
        ClientConfigBuilder {
            config: ClientConfig::default(),
        }
    }

    /// Set connect timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Set chunk size
    pub fn chunk_size(mut self, size: u32) -> Self {
        self.config.chunk_size = size;
        self
    }

    /// Enable auto-reconnect
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.config.auto_reconnect = enabled;
        self
    }

    /// Set buffer time
    pub fn buffer_time(mut self, ms: u32) -> Self {
        self.config.buffer_time = ms;
        self
    }

    /// Build configuration
    pub fn build(self) -> Result<ClientConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}