use std::collections::HashMap;
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct StreamInfo {
    /// Stream ID
    pub id: u32,

    /// Stream name (for publish/play)
    pub name: Option<String>,

    /// Stream type
    pub stream_type: StreamType,

    /// Creation timestamp
    pub created_at: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamType {
    /// Command stream (ID 0)
    Command,

    /// Network stream
    Network,

    /// Publishing stream
    Publishing,

    /// Playing stream
    Playing,
}

pub struct StreamManager {
    /// Next stream ID to allocate
    next_stream_id: u32,

    /// Active streams
    streams: HashMap<u32, StreamInfo>,
}

impl StreamManager {
    /// Create new stream manager
    pub fn new() -> Self {
        let mut manager = StreamManager {
            next_stream_id: 1, // 0 is reserved for commands
            streams: HashMap::new(),
        };

        // Add command stream
        manager.streams.insert(0, StreamInfo {
            id: 0,
            name: None,
            stream_type: StreamType::Command,
            created_at: 0,
        });

        manager
    }

    /// Create new stream
    pub fn create_stream(&mut self) -> u32 {
        let id = self.next_stream_id;
        self.next_stream_id += 1;

        self.streams.insert(id, StreamInfo {
            id,
            name: None,
            stream_type: StreamType::Network,
            created_at: crate::utils::current_timestamp(),
        });

        id
    }

    /// Delete stream
    pub fn delete_stream(&mut self, id: u32) -> Result<()> {
        if id == 0 {
            return Err(Error::stream("Cannot delete command stream"));
        }

        self.streams.remove(&id)
            .ok_or_else(|| Error::stream(format!("Stream {} not found", id)))?;

        Ok(())
    }

    /// Set stream as publishing
    pub fn set_publishing(&mut self, id: u32, name: String) -> Result<()> {
        let stream = self.streams.get_mut(&id)
            .ok_or_else(|| Error::stream(format!("Stream {} not found", id)))?;

        stream.name = Some(name);
        stream.stream_type = StreamType::Publishing;

        Ok(())
    }

    /// Set stream as playing
    pub fn set_playing(&mut self, id: u32, name: String) -> Result<()> {
        let stream = self.streams.get_mut(&id)
            .ok_or_else(|| Error::stream(format!("Stream {} not found", id)))?;

        stream.name = Some(name);
        stream.stream_type = StreamType::Playing;

        Ok(())
    }

    /// Get stream info
    pub fn get_stream(&self, id: u32) -> Option<&StreamInfo> {
        self.streams.get(&id)
    }

    /// Get all streams
    pub fn get_streams(&self) -> Vec<&StreamInfo> {
        self.streams.values().collect()
    }
}