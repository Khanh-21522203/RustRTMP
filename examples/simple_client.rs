// Simple RTMP Client Example
//
// This example demonstrates:
// - Connecting to an RTMP server
// - Publishing a stream
// - Playing a stream
// - Sending audio/video data
//
// Usage:
//   # Publish mode
//   cargo run --example simple_client -- rtmp://localhost/live publish mystream
//
//   # Play mode
//   cargo run --example simple_client -- rtmp://localhost/live play mystream

use rtmp::{RtmpClient, ClientConfig, Result};
use std::env;
use log::{info, error};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <url> <publish|play> [stream_name]", args[0]);
        eprintln!("Examples:");
        eprintln!("  {} rtmp://localhost/live publish mystream", args[0]);
        eprintln!("  {} rtmp://localhost/live play mystream", args[0]);
        std::process::exit(1);
    }
    
    let url = &args[1];
    let mode = &args[2];
    let stream_name = args.get(3).map(|s| s.as_str()).unwrap_or("stream");
    
    // Create client configuration
    let config = ClientConfig::builder()
        .chunk_size(4096)
        .buffer_time(1000)
        .auto_reconnect(false)
        .build()?;
    
    let mut client = RtmpClient::with_config(config);
    
    // Connect to server
    info!("Connecting to {}", url);
    client.connect(url).await?;
    info!("Connected successfully!");
    
    match mode.as_str() {
        "publish" => {
            // Start publishing
            info!("Publishing to stream: {}", stream_name);
            client.publish(stream_name, "live").await?;
            info!("Publishing started");
            
            // Send metadata
            let mut metadata = HashMap::new();
            metadata.insert("width".to_string(), rtmp::Amf0Value::Number(1920.0));
            metadata.insert("height".to_string(), rtmp::Amf0Value::Number(1080.0));
            metadata.insert("videocodecid".to_string(), rtmp::Amf0Value::Number(7.0)); // H.264
            metadata.insert("audiocodecid".to_string(), rtmp::Amf0Value::Number(10.0)); // AAC
            metadata.insert("framerate".to_string(), rtmp::Amf0Value::Number(25.0));
            
            client.send_metadata(metadata).await?;
            info!("Metadata sent");
            
            // Send test data for 30 seconds
            let mut timestamp = 0u32;
            let duration_ms = 30_000;
            let frame_duration_ms = 40; // 25 fps
            
            info!("Sending test video/audio data for 30 seconds...");
            while timestamp < duration_ms {
                // Generate test video frame
                let video_data = generate_test_video_frame(timestamp);
                client.send_video(video_data, timestamp).await?;
                
                // Generate test audio frame
                let audio_data = generate_test_audio_frame();
                client.send_audio(audio_data, timestamp).await?;
                
                timestamp += frame_duration_ms;
                tokio::time::sleep(tokio::time::Duration::from_millis(frame_duration_ms as u64)).await;
                
                if timestamp % 1000 == 0 {
                    info!("Publishing... {}s / 30s", timestamp / 1000);
                }
            }
            
            info!("Publishing completed");
        }
        "play" => {
            // Start playing
            info!("Playing stream: {}", stream_name);
            client.play(stream_name, 0.0, -1.0, true).await?;
            info!("Playing started");
            
            // Wait for Ctrl+C
            info!("Receiving stream data. Press Ctrl+C to stop");
            tokio::signal::ctrl_c().await?;
            info!("Stopping playback");
        }
        _ => {
            error!("Invalid mode: {}. Use 'publish' or 'play'", mode);
            std::process::exit(1);
        }
    }
    
    // Disconnect
    info!("Disconnecting...");
    client.disconnect().await?;
    info!("Disconnected");
    
    Ok(())
}

/// Generate a test video frame
/// In a real application, this would come from an encoder
fn generate_test_video_frame(timestamp: u32) -> Vec<u8> {
    // Simple test pattern: inter-frame (P-frame)
    // Real implementation would use actual H.264 encoded data
    let is_keyframe = timestamp % 2000 == 0; // Keyframe every 2 seconds
    
    if is_keyframe {
        // AVC keyframe header (0x17 = keyframe + AVC)
        vec![0x17, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x67]
    } else {
        // AVC inter-frame header (0x27 = inter-frame + AVC)
        vec![0x27, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x41]
    }
}

/// Generate a test audio frame
/// In a real application, this would come from audio input
fn generate_test_audio_frame() -> Vec<u8> {
    // AAC audio header (0xAF = AAC, 44.1kHz, 16-bit, stereo)
    // 0x01 = AAC raw data
    vec![0xAF, 0x01, 0x00, 0x00]
}
