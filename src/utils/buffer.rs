use std::io::{Cursor, Result as IoResult, Error as IoError, ErrorKind};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub struct ByteBuffer {
    buffer: Vec<u8>,
    cursor: usize,
}

impl ByteBuffer {
    /// Create a new ByteBuffer from bytes
    pub fn new(data: Vec<u8>) -> Self {
        ByteBuffer {
            buffer: data,
            cursor: 0,
        }
    }

    /// Create an empty ByteBuffer with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        ByteBuffer {
            buffer: Vec::with_capacity(capacity),
            cursor: 0,
        }
    }

    /// Get current cursor position
    pub fn position(&self) -> usize {
        self.cursor
    }

    /// Set cursor position
    pub fn set_position(&mut self, pos: usize) -> IoResult<()> {
        if pos > self.buffer.len() {
            return Err(IoError::new(ErrorKind::InvalidInput, "Position out of bounds"));
        }
        self.cursor = pos;
        Ok(())
    }

    /// Get remaining bytes from current position
    pub fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.cursor)
    }

    /// Check if buffer has at least n bytes remaining
    pub fn has_remaining(&self, n: usize) -> bool {
        self.remaining() >= n
    }

    /// Read bytes into buffer
    pub fn read_bytes(&mut self, len: usize) -> IoResult<Vec<u8>> {
        if !self.has_remaining(len) {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
        }
        let bytes = self.buffer[self.cursor..self.cursor + len].to_vec();
        self.cursor += len;
        Ok(bytes)
    }

    /// Write bytes to buffer
    pub fn write_bytes(&mut self, data: &[u8]) -> IoResult<()> {
        self.buffer.extend_from_slice(data);
        Ok(())
    }

    /// Read u8
    pub fn read_u8(&mut self) -> IoResult<u8> {
        if !self.has_remaining(1) {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
        }
        let value = self.buffer[self.cursor];
        self.cursor += 1;
        Ok(value)
    }

    /// Write u8
    pub fn write_u8(&mut self, value: u8) -> IoResult<()> {
        self.buffer.push(value);
        Ok(())
    }

    /// Read u16 (big endian)
    pub fn read_u16_be(&mut self) -> IoResult<u16> {
        if !self.has_remaining(2) {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
        }
        let mut cursor = Cursor::new(&self.buffer[self.cursor..]);
        let value = cursor.read_u16::<BigEndian>()?;
        self.cursor += 2;
        Ok(value)
    }

    /// Write u16 (big endian)
    pub fn write_u16_be(&mut self, value: u16) -> IoResult<()> {
        let mut bytes = vec![];
        bytes.write_u16::<BigEndian>(value)?;
        self.buffer.extend_from_slice(&bytes);
        Ok(())
    }

    /// Read i16 (big endian) - for signed integers
    pub fn read_i16_be(&mut self) -> IoResult<i16> {
        if !self.has_remaining(2) {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
        }
        let mut cursor = Cursor::new(&self.buffer[self.cursor..]);
        let value = cursor.read_i16::<BigEndian>()?;
        self.cursor += 2;
        Ok(value)
    }

    /// Write i16 (big endian) - for signed integers
    pub fn write_i16_be(&mut self, value: i16) -> IoResult<()> {
        let mut bytes = vec![];
        bytes.write_i16::<BigEndian>(value)?;
        self.buffer.extend_from_slice(&bytes);
        Ok(())
    }

    /// Read u32 (big endian)
    pub fn read_u32_be(&mut self) -> IoResult<u32> {
        if !self.has_remaining(4) {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
        }
        let mut cursor = Cursor::new(&self.buffer[self.cursor..]);
        let value = cursor.read_u32::<BigEndian>()?;
        self.cursor += 4;
        Ok(value)
    }

    /// Write u32 (big endian)
    pub fn write_u32_be(&mut self, value: u32) -> IoResult<()> {
        let mut bytes = vec![];
        bytes.write_u32::<BigEndian>(value)?;
        self.buffer.extend_from_slice(&bytes);
        Ok(())
    }

    /// Read f64 (big endian)
    pub fn read_f64_be(&mut self) -> IoResult<f64> {
        if !self.has_remaining(8) {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "Not enough bytes"));
        }
        let mut cursor = Cursor::new(&self.buffer[self.cursor..]);
        let value = cursor.read_f64::<BigEndian>()?;
        self.cursor += 8;
        Ok(value)
    }

    /// Write f64 (big endian)
    pub fn write_f64_be(&mut self, value: f64) -> IoResult<()> {
        let mut bytes = vec![];
        bytes.write_f64::<BigEndian>(value)?;
        self.buffer.extend_from_slice(&bytes);
        Ok(())
    }

    /// Get all bytes as Vec
    pub fn to_vec(&self) -> Vec<u8> {
        self.buffer.clone()
    }

    /// Get slice of underlying buffer
    pub fn as_slice(&self) -> &[u8] {
        &self.buffer
    }

    /// Clear buffer and reset cursor
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    /// Get length of buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_u8() {
        let mut buffer = ByteBuffer::with_capacity(10);
        buffer.write_u8(0x42).unwrap();
        buffer.write_u8(0x84).unwrap();

        buffer.set_position(0).unwrap();
        assert_eq!(buffer.read_u8().unwrap(), 0x42);
        assert_eq!(buffer.read_u8().unwrap(), 0x84);
    }

    #[test]
    fn test_read_write_u16() {
        let mut buffer = ByteBuffer::with_capacity(10);
        buffer.write_u16_be(0x1234).unwrap();

        buffer.set_position(0).unwrap();
        assert_eq!(buffer.read_u16_be().unwrap(), 0x1234);
    }

    #[test]
    fn test_remaining_bytes() {
        let data = vec![1, 2, 3, 4, 5];
        let mut buffer = ByteBuffer::new(data);

        assert_eq!(buffer.remaining(), 5);
        buffer.read_u8().unwrap();
        assert_eq!(buffer.remaining(), 4);
    }

    #[test]
    fn test_boundary_checks() {
        let data = vec![1, 2];
        let mut buffer = ByteBuffer::new(data);

        // Should succeed
        assert!(buffer.read_u16_be().is_ok());

        // Should fail - not enough bytes
        assert!(buffer.read_u32_be().is_err());
    }
}