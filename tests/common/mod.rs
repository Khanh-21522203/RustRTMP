// Common test utilities and helper functions
//
// This module provides reusable test utilities for integration and unit tests

use rtmp::{RtmpPacket, RtmpHeader};

/// Create a test video packet with specified timestamp
pub fn create_test_video_packet(timestamp: u32, is_keyframe: bool) -> RtmpPacket {
    let message_type = 9; // Video message type
    let stream_id = 1;
    let chunk_stream_id = 6; // Video chunk stream
    
    // Create video data
    let mut payload = Vec::new();
    if is_keyframe {
        // Keyframe: 0x17 (AVC keyframe)
        payload.push(0x17);
    } else {
        // Inter-frame: 0x27 (AVC inter-frame)
        payload.push(0x27);
    }
    
    // AVC NALU
    payload.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
    
    let header = RtmpHeader::new(
        timestamp,
        payload.len() as u32,
        message_type,
        stream_id,
        chunk_stream_id,
    );
    
    RtmpPacket::new(header, payload)
}

/// Create a test audio packet with specified timestamp
pub fn create_test_audio_packet(timestamp: u32) -> RtmpPacket {
    let message_type = 8; // Audio message type
    let stream_id = 1;
    let chunk_stream_id = 4; // Audio chunk stream
    
    // Create audio data
    let mut payload = Vec::new();
    // AAC audio: 0xAF (AAC, 44.1kHz, 16-bit, stereo)
    payload.push(0xAF);
    // AAC packet type (1 = raw)
    payload.push(0x01);
    
    let header = RtmpHeader::new(
        timestamp,
        payload.len() as u32,
        message_type,
        stream_id,
        chunk_stream_id,
    );
    
    RtmpPacket::new(header, payload)
}

/// Create a test data/metadata packet
pub fn create_test_metadata_packet() -> RtmpPacket {
    let message_type = 18; // AMF0 data message
    let stream_id = 1;
    let chunk_stream_id = 3; // Command chunk stream
    
    // Simple metadata payload (would normally be AMF0 encoded)
    let payload = vec![0x02, 0x00, 0x0A]; // String marker + length
    
    let header = RtmpHeader::new(
        0,
        payload.len() as u32,
        message_type,
        stream_id,
        chunk_stream_id,
    );
    
    RtmpPacket::new(header, payload)
}

/// Compare two RTMP packets for equality
pub fn assert_packet_equal(a: &RtmpPacket, b: &RtmpPacket) {
    assert_eq!(a.header().timestamp, b.header().timestamp, "Timestamps don't match");
    assert_eq!(a.header().message_type, b.header().message_type, "Message types don't match");
    assert_eq!(a.header().message_stream_id, b.header().message_stream_id, "Stream IDs don't match");
    assert_eq!(a.payload(), b.payload(), "Payloads don't match");
}

/// Generate test video frame data
pub fn generate_h264_keyframe() -> Vec<u8> {
    vec![
        0x17, // Frame type (1=keyframe) + codec (7=AVC)
        0x01, // AVC packet type (1=NALU)
        0x00, 0x00, 0x00, // Composition time
        // Simplified NALU data
        0x00, 0x00, 0x00, 0x01, // Start code
        0x67, // SPS NAL unit type
    ]
}

/// Generate test video frame data (inter-frame)
pub fn generate_h264_interframe() -> Vec<u8> {
    vec![
        0x27, // Frame type (2=inter) + codec (7=AVC)
        0x01, // AVC packet type (1=NALU)
        0x00, 0x00, 0x00, // Composition time
        // Simplified NALU data
        0x00, 0x00, 0x00, 0x01, // Start code
        0x41, // Coded slice NAL unit type
    ]
}

/// Generate test AAC audio data
pub fn generate_aac_audio() -> Vec<u8> {
    vec![
        0xAF, // Sound format (10=AAC) + rate + size + type
        0x01, // AAC packet type (1=raw)
        // AAC data would follow
        0x00, 0x00,
    ]
}

/// Create a simple test server configuration for testing
pub fn test_server_config(port: u16) -> rtmp::ServerConfig {
    rtmp::ServerConfig::builder()
        .host("127.0.0.1")
        .port(port)
        .max_connections(10)
        .chunk_size(4096)
        .build()
        .expect("Failed to create test server config")
}

/// Create a simple test client configuration
pub fn test_client_config() -> rtmp::ClientConfig {
    rtmp::ClientConfig::builder()
        .chunk_size(4096)
        .buffer_time(1000)
        .build()
        .expect("Failed to create test client config")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_video_packet() {
        let packet = create_test_video_packet(1000, true);
        assert_eq!(packet.header().timestamp, 1000);
        assert_eq!(packet.header().message_type, 9);
        assert!(packet.payload().len() > 0);
        assert_eq!(packet.payload()[0], 0x17); // Keyframe marker
    }

    #[test]
    fn test_create_audio_packet() {
        let packet = create_test_audio_packet(2000);
        assert_eq!(packet.header().timestamp, 2000);
        assert_eq!(packet.header().message_type, 8);
        assert!(packet.payload().len() > 0);
        assert_eq!(packet.payload()[0], 0xAF); // AAC marker
    }

    #[test]
    fn test_packet_equality() {
        let packet1 = create_test_video_packet(1000, true);
        let packet2 = create_test_video_packet(1000, true);
        assert_packet_equal(&packet1, &packet2);
    }

    #[test]
    fn test_h264_generation() {
        let keyframe = generate_h264_keyframe();
        assert_eq!(keyframe[0], 0x17);
        
        let interframe = generate_h264_interframe();
        assert_eq!(interframe[0], 0x27);
    }

    #[test]
    fn test_aac_generation() {
        let audio = generate_aac_audio();
        assert_eq!(audio[0], 0xAF);
    }
}
