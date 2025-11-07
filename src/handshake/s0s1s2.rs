use crate::{ByteBuffer, Error, Result};
use crate::handshake::c0c1::{C0C1, RTMP_VERSION, HANDSHAKE_SIZE};
use crate::handshake::state::HandshakeFormat;
use crate::utils::{generate_random_bytes, calculate_hmac_sha256, current_timestamp};

/// Server handshake (S0 + S1 + S2)
#[derive(Debug, Clone)]
pub struct S0S1S2 {
    /// RTMP version (S0)
    pub version: u8,

    /// S1 timestamp
    pub s1_timestamp: u32,

    /// S1 zero (should be 0)
    pub s1_zero: u32,

    /// S1 random data
    pub s1_random: Vec<u8>,

    /// S2 timestamp (echo of C1 timestamp)
    pub s2_timestamp: u32,

    /// S2 timestamp2 (current server time)
    pub s2_timestamp2: u32,

    /// S2 random echo (echo of C1 random)
    pub s2_random_echo: Vec<u8>,
}

impl S0S1S2 {
    /// Generate S0+S1+S2 response for C0+C1
    pub fn generate(c0c1: &C0C1) -> Result<Self> {
        // Validate client version
        if c0c1.version != RTMP_VERSION {
            return Err(Error::handshake(format!(
                "Unsupported client version: {}",
                c0c1.version
            )));
        }

        // Generate S1 random data
        let s1_random = generate_random_bytes(HANDSHAKE_SIZE - 8);

        Ok(S0S1S2 {
            version: RTMP_VERSION,
            s1_timestamp: current_timestamp(),
            s1_zero: 0,
            s1_random,
            s2_timestamp: c0c1.timestamp,
            s2_timestamp2: current_timestamp(),
            s2_random_echo: c0c1.random_data.clone(),
        })
    }

    /// Generate with complex handshake (HMAC-SHA256)
    pub fn generate_complex(c0c1: &C0C1, format: HandshakeFormat) -> Result<Self> {
        // First generate simple response
        let mut response = Self::generate(c0c1)?;

        // Add digest for complex handshake
        match format {
            HandshakeFormat::Simple => {}
            HandshakeFormat::Format1 | HandshakeFormat::Format2 => {
                // For educational purposes, we generate valid-looking data
                // Real implementation would calculate proper HMAC digest
                response.add_digest(format)?;
            }
        }

        Ok(response)
    }

    /// Add digest for complex handshake
    fn add_digest(&mut self, format: HandshakeFormat) -> Result<()> {
        // Simplified digest generation
        // Real implementation would:
        // 1. Calculate digest position based on format
        // 2. Generate HMAC-SHA256 digest
        // 3. Insert digest at correct position

        let digest_key = b"Genuine Adobe Flash Media Server 001";
        let digest = calculate_hmac_sha256(digest_key, &self.s1_random[0..32]);

        // Insert digest at appropriate position
        match format {
            HandshakeFormat::Format1 => {
                // Digest at offset 8
                self.s1_random[0..32].copy_from_slice(&digest);
            }
            HandshakeFormat::Format2 => {
                // Digest at offset 772
                if self.s1_random.len() >= 772 + 32 {
                    self.s1_random[772..772+32].copy_from_slice(&digest);
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(1 + HANDSHAKE_SIZE * 2);

        // S0
        result.push(self.version);

        // S1
        let mut s1_buffer = ByteBuffer::with_capacity(HANDSHAKE_SIZE);
        s1_buffer.write_u32_be(self.s1_timestamp).unwrap();
        s1_buffer.write_u32_be(self.s1_zero).unwrap();
        s1_buffer.write_bytes(&self.s1_random).unwrap();
        result.extend_from_slice(&s1_buffer.to_vec());

        // S2
        let mut s2_buffer = ByteBuffer::with_capacity(HANDSHAKE_SIZE);
        s2_buffer.write_u32_be(self.s2_timestamp).unwrap();
        s2_buffer.write_u32_be(self.s2_timestamp2).unwrap();
        s2_buffer.write_bytes(&self.s2_random_echo).unwrap();
        result.extend_from_slice(&s2_buffer.to_vec());

        result
    }

    /// Parse S0+S1+S2 from bytes (for client side)
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 1 + HANDSHAKE_SIZE * 2 {
            return Err(Error::handshake(format!(
                "S0+S1+S2 too short: {} bytes",
                data.len()
            )));
        }

        // Parse S0
        let version = data[0];

        // Parse S1
        let s1_data = &data[1..1537];
        let mut s1_buffer = ByteBuffer::new(s1_data.to_vec());
        let s1_timestamp = s1_buffer.read_u32_be()?;
        let s1_zero = s1_buffer.read_u32_be()?;
        let s1_random = s1_buffer.read_bytes(HANDSHAKE_SIZE - 8)?;

        // Parse S2
        let s2_data = &data[1537..3073];
        let mut s2_buffer = ByteBuffer::new(s2_data.to_vec());
        let s2_timestamp = s2_buffer.read_u32_be()?;
        let s2_timestamp2 = s2_buffer.read_u32_be()?;
        let s2_random_echo = s2_buffer.read_bytes(HANDSHAKE_SIZE - 8)?;

        Ok(S0S1S2 {
            version,
            s1_timestamp,
            s1_zero,
            s1_random,
            s2_timestamp,
            s2_timestamp2,
            s2_random_echo,
        })
    }
}

/// C2 packet for completing handshake
#[derive(Debug, Clone)]
pub struct C2 {
    pub timestamp: u32,
    pub timestamp2: u32,
    pub random_echo: Vec<u8>,
}

impl C2 {
    /// Create C2 from S0+S1+S2
    pub fn create_from_s1(s0s1s2: &S0S1S2) -> Self {
        C2 {
            timestamp: s0s1s2.s1_timestamp,
            timestamp2: current_timestamp(),
            random_echo: s0s1s2.s1_random.clone(),
        }
    }

    /// Parse C2 from bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < HANDSHAKE_SIZE {
            return Err(Error::handshake(format!(
                "C2 too short: {} bytes",
                data.len()
            )));
        }

        let mut buffer = ByteBuffer::new(data.to_vec());
        let timestamp = buffer.read_u32_be()?;
        let timestamp2 = buffer.read_u32_be()?;
        let random_echo = buffer.read_bytes(HANDSHAKE_SIZE - 8)?;

        Ok(C2 {
            timestamp,
            timestamp2,
            random_echo,
        })
    }

    /// Validate C2 against S1
    pub fn validate(&self, s0s1s2: &S0S1S2) -> Result<()> {
        // Verify timestamp echo
        if self.timestamp != s0s1s2.s1_timestamp {
            return Err(Error::handshake("C2 timestamp mismatch"));
        }

        // Verify random echo
        if self.random_echo != s0s1s2.s1_random {
            return Err(Error::handshake("C2 random echo mismatch"));
        }

        Ok(())
    }

    /// Encode to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = ByteBuffer::with_capacity(HANDSHAKE_SIZE);
        buffer.write_u32_be(self.timestamp).unwrap();
        buffer.write_u32_be(self.timestamp2).unwrap();
        buffer.write_bytes(&self.random_echo).unwrap();
        buffer.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_flow() {
        // Client creates C0+C1
        let c0c1 = C0C1::create_client();

        // Server generates S0+S1+S2
        let s0s1s2 = S0S1S2::generate(&c0c1).unwrap();
        assert_eq!(s0s1s2.version, RTMP_VERSION);
        assert_eq!(s0s1s2.s2_timestamp, c0c1.timestamp);

        // Client creates C2
        let c2 = C2::create_from_s1(&s0s1s2);

        // Server validates C2
        c2.validate(&s0s1s2).unwrap();
    }
}