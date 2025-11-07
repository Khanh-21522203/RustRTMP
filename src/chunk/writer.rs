use crate::{ByteBuffer, Error, Result, DEFAULT_CHUNK_SIZE};
use crate::protocol::{RtmpPacket, RtmpHeader};
use std::collections::HashMap;
use tokio::io::{AsyncWrite, AsyncWriteExt};

pub struct ChunkWriter {
    /// Previous headers for each chunk stream
    prev_headers: HashMap<u32, RtmpHeader>,

    /// Current chunk size for writing
    chunk_size_out: usize,
}

impl ChunkWriter {
    /// Create new chunk writer
    pub fn new() -> Self {
        ChunkWriter {
            prev_headers: HashMap::new(),
            chunk_size_out: DEFAULT_CHUNK_SIZE as usize,
        }
    }

    /// Set outgoing chunk size
    pub fn set_chunk_size(&mut self, size: usize) {
        self.chunk_size_out = size;
    }

    /// Write packet as chunks
    pub async fn write_packet<W: AsyncWrite + Unpin>(
        &mut self,
        packet: &RtmpPacket,
        writer: &mut W
    ) -> Result<()> {
        let cs_id = packet.header.chunk_stream_id;
        let chunks = self.create_chunks(packet)?;

        writer.write_all(&chunks).await
            .map_err(|e| Error::chunk(format!("Failed to write chunks: {}", e)))?;

        writer.flush().await
            .map_err(|e| Error::chunk(format!("Failed to flush: {}", e)))?;

        // Store header for delta encoding
        self.prev_headers.insert(cs_id, packet.header.clone());

        Ok(())
    }

    /// Create chunks from packet
    pub fn create_chunks(&mut self, packet: &RtmpPacket) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        let cs_id = packet.header.chunk_stream_id;

        // Determine chunk format type
        let (fmt, header_bytes) = self.get_header_bytes(packet)?;

        // Calculate number of chunks needed
        let payload_len = packet.payload.len();
        let num_chunks = (payload_len + self.chunk_size_out - 1) / self.chunk_size_out;

        // Write first chunk with full header
        result.extend_from_slice(&self.encode_basic_header(fmt, cs_id));
        result.extend_from_slice(&header_bytes);

        // Write first chunk data
        let first_chunk_size = payload_len.min(self.chunk_size_out);
        result.extend_from_slice(&packet.payload[0..first_chunk_size]);

        // Write continuation chunks (type 3)
        let mut offset = first_chunk_size;
        while offset < payload_len {
            // Type 3 header (no message header)
            result.extend_from_slice(&self.encode_basic_header(3, cs_id));

            // Chunk data
            let chunk_end = (offset + self.chunk_size_out).min(payload_len);
            result.extend_from_slice(&packet.payload[offset..chunk_end]);

            offset = chunk_end;
        }

        Ok(result)
    }

    /// Get header bytes and format type
    fn get_header_bytes(&self, packet: &RtmpPacket) -> Result<(u8, Vec<u8>)> {
        let cs_id = packet.header.chunk_stream_id;

        // Check if we have previous header
        if let Some(prev) = self.prev_headers.get(&cs_id) {
            // Can we use type 1, 2, or 3?
            if prev.message_stream_id == packet.header.message_stream_id &&
                prev.message_type == packet.header.message_type &&
                prev.message_length == packet.header.message_length {
                // Type 3: No header needed (continuation)
                if packet.header.timestamp == prev.timestamp {
                    return Ok((3, vec![]));
                }
                // Type 2: Timestamp delta only
                let delta = packet.header.timestamp - prev.timestamp;
                return Ok((2, self.encode_type2_header(delta)));
            }

            if prev.message_stream_id == packet.header.message_stream_id {
                // Type 1: Same stream ID
                let delta = packet.header.timestamp - prev.timestamp;
                return Ok((1, self.encode_type1_header(delta, packet)?));
            }
        }

        // Type 0: Full header
        Ok((0, self.encode_type0_header(packet)?))
    }

    /// Encode basic header
    fn encode_basic_header(&self, fmt: u8, cs_id: u32) -> Vec<u8> {
        let mut result = Vec::new();

        if cs_id <= 63 {
            // 1-byte header
            result.push((fmt << 6) | (cs_id as u8));
        } else if cs_id <= 319 {
            // 2-byte header
            result.push((fmt << 6) | 0);
            result.push((cs_id - 64) as u8);
        } else {
            // 3-byte header
            result.push((fmt << 6) | 1);
            let id = cs_id - 64;
            result.push((id & 0xFF) as u8);
            result.push((id >> 8) as u8);
        }

        result
    }

    /// Encode type 0 header (11 bytes + optional extended timestamp)
    fn encode_type0_header(&self, packet: &RtmpPacket) -> Result<Vec<u8>> {
        let mut buffer = ByteBuffer::with_capacity(15);

        // Timestamp (3 bytes) or 0xFFFFFF for extended
        if packet.header.timestamp >= 0xFFFFFF {
            buffer.write_u8(0xFF)?;
            buffer.write_u8(0xFF)?;
            buffer.write_u8(0xFF)?;
        } else {
            buffer.write_u8((packet.header.timestamp >> 16) as u8)?;
            buffer.write_u8((packet.header.timestamp >> 8) as u8)?;
            buffer.write_u8(packet.header.timestamp as u8)?;
        }

        // Message length (3 bytes)
        let len = packet.payload.len() as u32;
        buffer.write_u8((len >> 16) as u8)?;
        buffer.write_u8((len >> 8) as u8)?;
        buffer.write_u8(len as u8)?;

        // Message type (1 byte)
        buffer.write_u8(packet.header.message_type)?;

        // Message stream ID (4 bytes, little endian)
        let stream_id = packet.header.message_stream_id.to_le_bytes();
        buffer.write_bytes(&stream_id)?;

        // Extended timestamp if needed
        if packet.header.timestamp >= 0xFFFFFF {
            buffer.write_u32_be(packet.header.timestamp)?;
        }

        Ok(buffer.to_vec())
    }

    /// Encode type 1 header (7 bytes + optional extended timestamp)
    fn encode_type1_header(&self, timestamp_delta: u32, packet: &RtmpPacket) -> Result<Vec<u8>> {
        let mut buffer = ByteBuffer::with_capacity(11);

        // Timestamp delta (3 bytes)
        if timestamp_delta >= 0xFFFFFF {
            buffer.write_u8(0xFF)?;
            buffer.write_u8(0xFF)?;
            buffer.write_u8(0xFF)?;
        } else {
            buffer.write_u8((timestamp_delta >> 16) as u8)?;
            buffer.write_u8((timestamp_delta >> 8) as u8)?;
            buffer.write_u8(timestamp_delta as u8)?;
        }

        // Message length (3 bytes)
        let len = packet.payload.len() as u32;
        buffer.write_u8((len >> 16) as u8)?;
        buffer.write_u8((len >> 8) as u8)?;
        buffer.write_u8(len as u8)?;

        // Message type (1 byte)
        buffer.write_u8(packet.header.message_type)?;

        // Extended timestamp if needed
        if timestamp_delta >= 0xFFFFFF {
            buffer.write_u32_be(timestamp_delta)?;
        }

        Ok(buffer.to_vec())
    }

    /// Encode type 2 header (3 bytes + optional extended timestamp)
    fn encode_type2_header(&self, timestamp_delta: u32) -> Vec<u8> {
        let mut buffer = ByteBuffer::with_capacity(7);

        // Timestamp delta (3 bytes)
        if timestamp_delta >= 0xFFFFFF {
            buffer.write_u8(0xFF).unwrap();
            buffer.write_u8(0xFF).unwrap();
            buffer.write_u8(0xFF).unwrap();
            buffer.write_u32_be(timestamp_delta).unwrap();
        } else {
            buffer.write_u8((timestamp_delta >> 16) as u8).unwrap();
            buffer.write_u8((timestamp_delta >> 8) as u8).unwrap();
            buffer.write_u8(timestamp_delta as u8).unwrap();
        }

        buffer.to_vec()
    }
}