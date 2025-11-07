use rand::{RngCore, rng};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

/// Generate cryptographically secure random bytes
pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    rng().fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes_length() {
        let bytes = generate_random_bytes(32);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_random_bytes_uniqueness() {
        let bytes1 = generate_random_bytes(32);
        let bytes2 = generate_random_bytes(32);
        // Very unlikely to be equal
        assert_ne!(bytes1, bytes2);
    }
}

type HmacSha256 = Hmac<Sha256>;

/// Calculate HMAC-SHA256
pub fn calculate_hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);

    let result = mac.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result.into_bytes());
    output
}

#[cfg(test)]
mod hmac_tests {
    use super::*;

    #[test]
    fn test_hmac_sha256() {
        let key = b"key";
        let data = b"The quick brown fox jumps over the lazy dog";
        let hmac = calculate_hmac_sha256(key, data);

        // Known test vector
        let expected = [
            0xf7, 0xbc, 0x83, 0xf4, 0x30, 0x53, 0x84, 0x24,
            0xb1, 0x32, 0x98, 0xe6, 0xaa, 0x6f, 0xb1, 0x43,
            0xef, 0x4d, 0x59, 0xa1, 0x49, 0x46, 0x17, 0x59,
            0x97, 0x47, 0x9d, 0xbc, 0x2d, 0x1a, 0x3c, 0xd8
        ];

        assert_eq!(hmac, expected);
    }
}