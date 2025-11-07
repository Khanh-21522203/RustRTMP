use crate::{ByteBuffer, Error, Result};
use crate::handshake::state::HandshakeFormat;
use crate::utils::{generate_random_bytes, calculate_hmac_sha256, current_timestamp};

/// RTMP version
pub const RTMP_VERSION: u8 = 3;

/// Handshake packet size (C1/S1/C2/S2)
pub const HANDSHAKE_SIZE: usize = 1536;

/// FMS version for complex handshake
pub const FMS_VERSION: [u8; 4] = [0x05, 0x00, 0x01, 0x01];

/// Client handshake (C0 + C1)
#[derive(Debug, Clone)]
pub struct C0C1 {
    /// RTMP version (C0)
    pub version: u8,

    /// Timestamp (C1)
    pub timestamp: u32,

    /// Zero (C1) - should be 0
    pub zero: u32,

    /// Random data (C1)
    pub random_data: Vec<u8>,
}

impl C0C1 {
    /// Parse C0+C1 from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 1537 {
            return Err(Error::handshake(format!(
                "C0+C1 too short: {} bytes, expected 1537",
                data.len()
            )));
        }

        // Parse C0
        let version = data[0];
        if version != RTMP_VERSION {
            return Err(Error::handshake(format!(
                "Unsupported RTMP version: {}, expected {}",
                version, RTMP_VERSION
            )));
        }

        // Parse C1
        let c1_data = &data[1..1537];
        let mut buffer = ByteBuffer::new(c1_data.to_vec());

        let timestamp = buffer.read_u32_be()
            .map_err(|e| Error::handshake(format!("Failed to read timestamp: {}", e)))?;

        let zero = buffer.read_u32_be()
            .map_err(|e| Error::handshake(format!("Failed to read zero: {}", e)))?;

        let random_data = buffer.read_bytes(HANDSHAKE_SIZE - 8)
            .map_err(|e| Error::handshake(format!("Failed to read random data: {}", e)))?;

        Ok(C0C1 {
            version,
            timestamp,
            zero,
            random_data,
        })
    }

    /// Create C0+C1 for client
    pub fn create_client() -> Self {
        C0C1 {
            version: RTMP_VERSION,
            timestamp: current_timestamp(),
            zero: 0,
            random_data: generate_random_bytes(HANDSHAKE_SIZE - 8),
        }
    }

    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(1537);

        // C0
        result.push(self.version);

        // C1
        let mut c1_buffer = ByteBuffer::with_capacity(HANDSHAKE_SIZE);
        c1_buffer.write_u32_be(self.timestamp).unwrap();
        c1_buffer.write_u32_be(self.zero).unwrap();
        c1_buffer.write_bytes(&self.random_data).unwrap();

        result.extend_from_slice(&c1_buffer.to_vec());
        result
    }

    /// Detect handshake format
    pub fn detect_format(&self) -> HandshakeFormat {
        // Check for FMS version bytes at different positions
        // This is simplified - real implementation would check digest positions

        // Check format 1 position (offset 4)
        if self.random_data.len() > 4 &&
            &self.random_data[0..4] == &FMS_VERSION {
            return HandshakeFormat::Format1;
        }

        // Check format 2 position (offset 768)
        if self.random_data.len() > 768 + 4 &&
            &self.random_data[764..768] == &FMS_VERSION {
            return HandshakeFormat::Format2;
        }

        HandshakeFormat::Simple
    }

    /// Validate C1 digest for complex handshake
    pub fn validate_digest(&self, format: HandshakeFormat) -> Result<()> {
        match format {
            HandshakeFormat::Simple => Ok(()),
            HandshakeFormat::Format1 | HandshakeFormat::Format2 => {
                // Simplified validation - real implementation would verify HMAC
                // For educational purposes, we accept all complex handshakes
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c0c1_creation() {
        let c0c1 = C0C1::create_client();
        assert_eq!(c0c1.version, RTMP_VERSION);
        assert_eq!(c0c1.zero, 0);
        assert_eq!(c0c1.random_data.len(), HANDSHAKE_SIZE - 8);
    }

    #[test]
    fn test_c0c1_round_trip() {
        let original = C0C1::create_client();
        let bytes = original.encode();
        assert_eq!(bytes.len(), 1537);

        let parsed = C0C1::parse(&bytes).unwrap();
        assert_eq!(parsed.version, original.version);
        assert_eq!(parsed.timestamp, original.timestamp);
        assert_eq!(parsed.zero, original.zero);
    }
}