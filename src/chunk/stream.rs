use crate::{Error, Result};
use crate::protocol::{RtmpPacket, RtmpHeader};

#[derive(Debug, Clone)]
pub struct ChunkStreamContext {
    /// Previous header for this chunk stream
    pub prev_header: Option<RtmpHeader>,

    /// Partial message being assembled
    pub message_buffer: Vec<u8>,

    /// Bytes remaining for current message
    pub bytes_remaining: usize,

    /// Current message header being assembled
    pub current_header: Option<RtmpHeader>,

    /// Timestamp delta accumulator
    pub timestamp_delta: u32,
}

impl ChunkStreamContext {
    /// Create new chunk stream context
    pub fn new() -> Self {
        ChunkStreamContext {
            prev_header: None,
            message_buffer: Vec::new(),
            bytes_remaining: 0,
            current_header: None,
            timestamp_delta: 0,
        }
    }

    /// Check if currently assembling a message
    pub fn is_assembling(&self) -> bool {
        self.bytes_remaining > 0
    }

    /// Add chunk data to message buffer
    pub fn add_chunk_data(&mut self, data: Vec<u8>) -> Result<Option<RtmpPacket>> {
        self.message_buffer.extend_from_slice(&data);

        // Check if more chunks needed
        if data.len() >= self.bytes_remaining {
            self.bytes_remaining = 0;

            // Message complete
            if let Some(header) = self.current_header.take() {
                let packet = RtmpPacket::new(
                    header.clone(),
                    self.message_buffer.clone()
                );

                // Clear buffer for next message
                self.message_buffer.clear();

                // Save header for delta compression
                self.prev_header = Some(header);

                return Ok(Some(packet));
            }
        } else {
            self.bytes_remaining -= data.len();
        }

        Ok(None)
    }

    /// Start new message
    pub fn start_message(&mut self, header: RtmpHeader) {
        self.current_header = Some(header.clone());
        self.prev_header = Some(header.clone()); // Update for next chunk
        self.bytes_remaining = header.message_length as usize;
        self.message_buffer.clear();
        self.message_buffer.reserve(header.message_length as usize);
    }
}

#[derive(Debug, Clone)]
pub struct ChunkStream {
    /// Chunk stream ID
    pub id: u32,

    /// Stream context
    pub context: ChunkStreamContext,
}

impl ChunkStream {
    /// Create new chunk stream
    pub fn new(id: u32) -> Self {
        ChunkStream {
            id,
            context: ChunkStreamContext::new(),
        }
    }
}