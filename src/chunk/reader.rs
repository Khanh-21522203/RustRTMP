use crate::{ByteBuffer, Error, Result, DEFAULT_CHUNK_SIZE};
use crate::protocol::{RtmpPacket, RtmpHeader};
use crate::chunk::stream::{ChunkStream, ChunkStreamContext};
use std::collections::HashMap;
use tokio::io::{AsyncRead, AsyncReadExt};

pub struct ChunkReader {
    /// Chunk streams by ID
    chunk_streams: HashMap<u32, ChunkStreamContext>,

    /// Current chunk size for reading
    chunk_size_in: usize,

    /// Buffer for reading
    read_buffer: Vec<u8>,
}

impl ChunkReader {
    /// Create new chunk reader
    pub fn new() -> Self {
        ChunkReader {
            chunk_streams: HashMap::new(),
            chunk_size_in: DEFAULT_CHUNK_SIZE as usize,
            read_buffer: Vec::with_capacity(4096),
        }
    }

    /// Set incoming chunk size
    pub fn set_chunk_size(&mut self, size: usize) {
        self.chunk_size_in = size;
    }

    /// Read next chunk from stream
    pub async fn read_chunk<R: AsyncRead + Unpin>(
        &mut self,
        reader: &mut R
    ) -> Result<Option<RtmpPacket>> {
        // Read basic header (1-3 bytes)
        let mut basic_header = [0u8; 1];
        reader.read_exact(&mut basic_header).await
            .map_err(|e| Error::chunk(format!("Failed to read basic header: {}", e)))?;

        let (fmt, cs_id) = self.parse_basic_header(basic_header[0], reader).await?;

        // Get previous header for delta calculations (if exists)
        let prev_header = self.chunk_streams.get(&cs_id).and_then(|ctx| ctx.prev_header.clone());

        // Read message header based on fmt
        let header = self.read_message_header(fmt, cs_id, prev_header, reader).await?;

        // Get or create chunk stream context
        let context = self.chunk_streams.entry(cs_id)
            .or_insert_with(ChunkStreamContext::new);

        // Start new message if not continuing
        if !context.is_assembling() {
            context.start_message(header.clone());
        }

        // Calculate chunk data size
        let chunk_data_size = if context.bytes_remaining > self.chunk_size_in {
            self.chunk_size_in
        } else {
            context.bytes_remaining
        };

        // Read chunk data
        let mut chunk_data = vec![0u8; chunk_data_size];
        reader.read_exact(&mut chunk_data).await
            .map_err(|e| Error::chunk(format!("Failed to read chunk data: {}", e)))?;

        // Add to message buffer
        context.add_chunk_data(chunk_data)
    }

    /// Parse basic header and get chunk stream ID
    async fn parse_basic_header<R: AsyncRead + Unpin>(
        &mut self,
        first_byte: u8,
        reader: &mut R
    ) -> Result<(u8, u32)> {
        let fmt = (first_byte >> 6) & 0x03;
        let cs_id = match first_byte & 0x3F {
            0 => {
                // 2-byte form
                let mut id_byte = [0u8; 1];
                reader.read_exact(&mut id_byte).await
                    .map_err(|e| Error::chunk(format!("Failed to read CS ID: {}", e)))?;
                (id_byte[0] as u32) + 64
            }
            1 => {
                // 3-byte form
                let mut id_bytes = [0u8; 2];
                reader.read_exact(&mut id_bytes).await
                    .map_err(|e| Error::chunk(format!("Failed to read CS ID: {}", e)))?;
                let id = u16::from_le_bytes(id_bytes) as u32;
                id + 64
            }
            n => n as u32,
        };

        Ok((fmt, cs_id))
    }

    /// Read message header based on format type
    async fn read_message_header<R: AsyncRead + Unpin>(
        &mut self,
        fmt: u8,
        cs_id: u32,
        prev_header: Option<RtmpHeader>,
        reader: &mut R
    ) -> Result<RtmpHeader> {
        match fmt {
            0 => {
                // Type 0: Full header (11 bytes)
                let mut header_bytes = [0u8; 11];
                reader.read_exact(&mut header_bytes).await
                    .map_err(|e| Error::chunk(format!("Failed to read type 0 header: {}", e)))?;

                let timestamp = u32::from_be_bytes([0, header_bytes[0], header_bytes[1], header_bytes[2]]);
                let message_length = u32::from_be_bytes([0, header_bytes[3], header_bytes[4], header_bytes[5]]);
                let message_type = header_bytes[6];
                let message_stream_id = u32::from_le_bytes([
                    header_bytes[7], header_bytes[8], header_bytes[9], header_bytes[10]
                ]);

                // Check for extended timestamp
                let final_timestamp = if timestamp == 0xFFFFFF {
                    let mut ext_bytes = [0u8; 4];
                    reader.read_exact(&mut ext_bytes).await
                        .map_err(|e| Error::chunk(format!("Failed to read extended timestamp: {}", e)))?;
                    u32::from_be_bytes(ext_bytes)
                } else {
                    timestamp
                };

                Ok(RtmpHeader::new(
                    final_timestamp,
                    message_length,
                    message_type,
                    message_stream_id,
                    cs_id,
                ))
            }
            1 => {
                // Type 1: Same stream ID (7 bytes)
                let mut header_bytes = [0u8; 7];
                reader.read_exact(&mut header_bytes).await
                    .map_err(|e| Error::chunk(format!("Failed to read type 1 header: {}", e)))?;

                let timestamp_delta = u32::from_be_bytes([0, header_bytes[0], header_bytes[1], header_bytes[2]]);
                let message_length = u32::from_be_bytes([0, header_bytes[3], header_bytes[4], header_bytes[5]]);
                let message_type = header_bytes[6];

                // Check for extended timestamp
                let final_timestamp_delta = if timestamp_delta == 0xFFFFFF {
                    let mut ext_bytes = [0u8; 4];
                    reader.read_exact(&mut ext_bytes).await
                        .map_err(|e| Error::chunk(format!("Failed to read extended timestamp: {}", e)))?;
                    u32::from_be_bytes(ext_bytes)
                } else {
                    timestamp_delta
                };

                // Calculate absolute timestamp from previous header
                let prev = prev_header.ok_or_else(|| Error::chunk("Type 1 header requires previous header"))?;
                let timestamp = prev.timestamp.wrapping_add(final_timestamp_delta);

                Ok(RtmpHeader::new(
                    timestamp,
                    message_length,
                    message_type,
                    prev.message_stream_id, // Reuse stream ID
                    cs_id,
                ))
            }
            2 => {
                // Type 2: Same length and stream ID (3 bytes)
                let mut header_bytes = [0u8; 3];
                reader.read_exact(&mut header_bytes).await
                    .map_err(|e| Error::chunk(format!("Failed to read type 2 header: {}", e)))?;

                let timestamp_delta = u32::from_be_bytes([0, header_bytes[0], header_bytes[1], header_bytes[2]]);

                // Check for extended timestamp
                let final_timestamp_delta = if timestamp_delta == 0xFFFFFF {
                    let mut ext_bytes = [0u8; 4];
                    reader.read_exact(&mut ext_bytes).await
                        .map_err(|e| Error::chunk(format!("Failed to read extended timestamp: {}", e)))?;
                    u32::from_be_bytes(ext_bytes)
                } else {
                    timestamp_delta
                };

                // Calculate absolute timestamp and reuse length, type, stream ID from previous
                let prev = prev_header.ok_or_else(|| Error::chunk("Type 2 header requires previous header"))?;
                let timestamp = prev.timestamp.wrapping_add(final_timestamp_delta);

                Ok(RtmpHeader::new(
                    timestamp,
                    prev.message_length,   // Reuse
                    prev.message_type,     // Reuse
                    prev.message_stream_id, // Reuse
                    cs_id,
                ))
            }
            3 => {
                // Type 3: No header - reuse everything from previous
                prev_header.ok_or_else(|| Error::chunk("Type 3 header requires previous header"))
            }
            _ => Err(Error::chunk(format!("Invalid chunk format: {}", fmt)))
        }
    }
}