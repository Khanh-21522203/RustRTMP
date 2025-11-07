use crate::protocol::constants::*;

#[derive(Debug, Clone)]
pub struct RtmpPacket {
    pub header: RtmpHeader,
    pub payload: Vec<u8>,
}

impl RtmpPacket {
    /// Create new packet
    pub fn new(header: RtmpHeader, payload: Vec<u8>) -> Self {
        RtmpPacket { header, payload }
    }

    /// Get message type
    pub fn message_type(&self) -> u8 {
        self.header.message_type
    }

    /// Get message stream ID
    pub fn message_stream_id(&self) -> u32 {
        self.header.message_stream_id
    }

    /// Get timestamp
    pub fn timestamp(&self) -> u32 {
        self.header.timestamp
    }

    /// Check if this is an audio packet
    pub fn is_audio(&self) -> bool {
        self.header.message_type == MSG_TYPE_AUDIO
    }

    /// Check if this is a video packet
    pub fn is_video(&self) -> bool {
        self.header.message_type == MSG_TYPE_VIDEO
    }

    /// Check if this is a command message
    pub fn is_command(&self) -> bool {
        self.header.message_type == MSG_TYPE_COMMAND_AMF0 ||
            self.header.message_type == MSG_TYPE_COMMAND_AMF3
    }

    /// Check if this is a data message
    pub fn is_data(&self) -> bool {
        self.header.message_type == MSG_TYPE_DATA_AMF0 ||
            self.header.message_type == MSG_TYPE_DATA_AMF3
    }

    /// Check if this is a control message
    pub fn is_control(&self) -> bool {
        matches!(self.header.message_type,
            MSG_TYPE_SET_CHUNK_SIZE |
            MSG_TYPE_ABORT |
            MSG_TYPE_ACK |
            MSG_TYPE_WINDOW_ACK |
            MSG_TYPE_SET_PEER_BW)
    }

    /// Create chunks for sending (will be refined in Phase 4)
    pub fn create_chunks(&self, chunk_size: usize) -> Vec<u8> {
        // Simplified version - Phase 4 will implement proper chunking
        let mut result = Vec::new();

        // Basic header (fmt 0, cs_id 3)
        result.push(0x03);

        // Message header (11 bytes)
        // Timestamp (3 bytes)
        result.push((self.header.timestamp >> 16) as u8);
        result.push((self.header.timestamp >> 8) as u8);
        result.push(self.header.timestamp as u8);

        // Message length (3 bytes)
        let len = self.payload.len();
        result.push((len >> 16) as u8);
        result.push((len >> 8) as u8);
        result.push(len as u8);

        // Message type (1 byte)
        result.push(self.header.message_type);

        // Message stream ID (4 bytes, little endian)
        let stream_id = self.header.message_stream_id;
        result.push(stream_id as u8);
        result.push((stream_id >> 8) as u8);
        result.push((stream_id >> 16) as u8);
        result.push((stream_id >> 24) as u8);

        // Payload
        result.extend_from_slice(&self.payload);

        result
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RtmpHeader {
    pub timestamp: u32,
    pub message_length: u32,
    pub message_type: u8,
    pub message_stream_id: u32,
    pub chunk_stream_id: u32,
}

impl RtmpHeader {
    /// Create new header
    pub fn new(
        timestamp: u32,
        message_length: u32,
        message_type: u8,
        message_stream_id: u32,
        chunk_stream_id: u32,
    ) -> Self {
        RtmpHeader {
            timestamp,
            message_length,
            message_type,
            message_stream_id,
            chunk_stream_id,
        }
    }

    /// Create header for audio message
    pub fn audio(timestamp: u32, length: u32, stream_id: u32) -> Self {
        RtmpHeader::new(timestamp, length, MSG_TYPE_AUDIO, stream_id, CHUNK_STREAM_AUDIO)
    }

    /// Create header for video message
    pub fn video(timestamp: u32, length: u32, stream_id: u32) -> Self {
        RtmpHeader::new(timestamp, length, MSG_TYPE_VIDEO, stream_id, CHUNK_STREAM_VIDEO)
    }

    /// Create header for command message
    pub fn command(timestamp: u32, length: u32, stream_id: u32) -> Self {
        RtmpHeader::new(timestamp, length, MSG_TYPE_COMMAND_AMF0, stream_id, CHUNK_STREAM_COMMAND)
    }

    /// Create header for data message
    pub fn data(timestamp: u32, length: u32, stream_id: u32) -> Self {
        RtmpHeader::new(timestamp, length, MSG_TYPE_DATA_AMF0, stream_id, CHUNK_STREAM_DATA)
    }

    /// Check if timestamp is extended (>= 0xFFFFFF)
    pub fn has_extended_timestamp(&self) -> bool {
        self.timestamp >= 0xFFFFFF
    }

    /// Get timestamp for wire format
    pub fn wire_timestamp(&self) -> u32 {
        if self.has_extended_timestamp() {
            0xFFFFFF
        } else {
            self.timestamp
        }
    }
}

pub fn make_audio_packet(data: Vec<u8>, timestamp: u32, stream_id: u32) -> RtmpPacket {
    let header = RtmpHeader::audio(timestamp, data.len() as u32, stream_id);
    RtmpPacket::new(header, data)
}

pub fn make_video_packet(data: Vec<u8>, timestamp: u32, stream_id: u32) -> RtmpPacket {
    let header = RtmpHeader::video(timestamp, data.len() as u32, stream_id);
    RtmpPacket::new(header, data)
}

pub fn parse_basic_header(byte: u8) -> (u8, u32) {
    let fmt = (byte >> 6) & 0x03;
    let chunk_stream_id = match byte & 0x3F {
        0 => {
            // 2-byte header form
            // Next byte is CS ID - 64
            0 // Will be updated by caller
        }
        1 => {
            // 3-byte header form
            // Next 2 bytes are CS ID - 64
            1 // Will be updated by caller
        }
        n => n as u32,
    };

    (fmt, chunk_stream_id)
}

pub fn encode_basic_header(fmt: u8, chunk_stream_id: u32) -> Vec<u8> {
    let mut result = Vec::new();

    if chunk_stream_id <= 63 {
        // 1-byte header
        result.push((fmt << 6) | (chunk_stream_id as u8));
    } else if chunk_stream_id <= 319 {
        // 2-byte header
        result.push((fmt << 6) | 0);
        result.push((chunk_stream_id - 64) as u8);
    } else {
        // 3-byte header
        result.push((fmt << 6) | 1);
        let id = chunk_stream_id - 64;
        result.push((id & 0xFF) as u8);
        result.push((id >> 8) as u8);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_creation() {
        let header = RtmpHeader {
            timestamp: 1000,
            message_length: 100,
            message_type: MSG_TYPE_AUDIO,
            message_stream_id: 1,
            chunk_stream_id: 4,
        };

        let payload = vec![0x01, 0x02, 0x03];
        let packet = RtmpPacket::new(header, payload);

        assert!(packet.is_audio());
        assert!(!packet.is_video());
        assert_eq!(packet.timestamp(), 1000);
        assert_eq!(packet.message_stream_id(), 1);
    }
}