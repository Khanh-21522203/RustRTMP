
mod stream;
mod reader;
mod writer;

pub use stream::*;
pub use reader::*;
pub use writer::*;
pub use stream::*;

use crate::{Error, Result, RtmpPacket};

pub fn split_into_chunks(packet: &RtmpPacket, chunk_size: usize) -> Vec<u8> {
    let mut writer = ChunkWriter::new();
    writer.set_chunk_size(chunk_size);
    writer.create_chunks(packet).unwrap_or_default()
}

pub fn parse_chunk_header(bytes: &[u8]) -> Result<(ChunkHeader, usize)> {
    if bytes.is_empty() {
        return Err(Error::chunk("Empty chunk header"));
    }

    let first_byte = bytes[0];
    let fmt = (first_byte >> 6) & 0x03;
    let cs_id_part = first_byte & 0x3F;

    let (cs_id, offset) = match cs_id_part {
        0 if bytes.len() > 1 => ((bytes[1] as u32) + 64, 2),
        1 if bytes.len() > 2 => {
            let id = u16::from_le_bytes([bytes[1], bytes[2]]) as u32;
            (id + 64, 3)
        }
        n => (n as u32, 1),
    };

    let header = ChunkHeader { fmt, cs_id };
    Ok((header, offset))
}

#[derive(Debug, Clone, Copy)]
pub struct ChunkHeader {
    pub fmt: u8,
    pub cs_id: u32,
}