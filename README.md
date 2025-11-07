# RustRTMP

Pure Rust implementation of the RTMP (Real-Time Messaging Protocol) for both server and client applications.

## Features

- ✅ **RTMP Server** - Accept incoming connections and handle publish/play requests
- ✅ **RTMP Client** - Connect to servers, publish and play streams
- ✅ **AMF0 Codec** - Encode/decode Action Message Format 0
- ✅ **H.264/AAC Support** - Handle video and audio codecs
- ✅ **GOP Cache** - Cache Group of Pictures for faster playback start
- ✅ **Async/Await** - Built on Tokio for high-performance async I/O
- ✅ **Handshake** - Complete RTMP handshake implementation (C0/C1/C2, S0/S1/S2)
- ✅ **Chunking** - Message chunking and reassembly
- ✅ **Stream Processing** - Video/audio/metadata processing

## Quick Start

### Server

```rust
use rtmp::{RtmpServer, ServerConfig, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let config = ServerConfig::builder()
        .host("0.0.0.0")
        .port(1935)
        .max_connections(100)
        .build()?;
    
    let server = RtmpServer::new(config);
    server.listen().await?;
    
    Ok(())
}
```

### Client

```rust
use rtmp::{RtmpClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = RtmpClient::new();
    
    // Connect to server
    client.connect("rtmp://localhost/live").await?;
    
    // Publish a stream
    client.publish("mystream", "live").await?;
    
    // Send video data
    client.send_video(video_data, timestamp).await?;
    
    // Send audio data
    client.send_audio(audio_data, timestamp).await?;
    
    Ok(())
}
```

## Examples

The `examples/` directory contains comprehensive examples:

- **`simple_server`** - Basic RTMP server
- **`simple_client`** - RTMP client for publishing and playing
- **`relay_server`** - Relay/proxy server implementation

See [examples/README.md](examples/README.md) for detailed usage instructions.

### Running Examples

Start a server:
```bash
cargo run --example simple_server
```

Publish test data:
```bash
cargo run --example simple_client -- rtmp://localhost/live publish mystream
```

Play a stream:
```bash
cargo run --example simple_client -- rtmp://localhost/live play mystream
```

## Testing with FFmpeg

Publish a video file:
```bash
ffmpeg -re -i video.mp4 -c:v libx264 -c:a aac -f flv rtmp://localhost/live/stream
```

Play a stream:
```bash
ffplay rtmp://localhost/live/stream
```


## Project Structure

```
src/
├── amf/              # AMF0 encoding/decoding
├── chunk/            # Chunk protocol implementation
├── client/           # RTMP client
├── connection/       # Connection management
├── handlers/         # Message handlers
├── handshake/        # RTMP handshake
├── message/          # RTMP messages
├── processing/       # Media processing
├── protocol/         # Core protocol types
├── server/           # RTMP server
├── stream/           # Stream management
├── utils/            # Utilities (error, buffer, crypto)
└── lib.rs           # Library entry point
```

## Development

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run with Logging

```bash
RUST_LOG=debug cargo run --example simple_server
```

## Configuration

### Server Configuration

```rust
let config = ServerConfig::builder()
    .host("0.0.0.0")
    .port(1935)
    .max_connections(100)
    .max_connections_per_ip(10)
    .chunk_size(4096)
    .gop_cache_enabled(true)
    .gop_cache_size(10)
    .build()?;
```

### Client Configuration

```rust
let config = ClientConfig::builder()
    .chunk_size(4096)
    .buffer_time(1000)
    .auto_reconnect(true)
    .build()?;
```

## Use Cases

- **Live Streaming Servers** - Build your own RTMP server
- **Stream Relay/Proxy** - Create relay servers for load balancing
- **Recording** - Record RTMP streams to disk
- **Transcoding** - Convert between different formats
- **Analytics** - Monitor and analyze streaming metrics
- **Testing Tools** - Test RTMP implementations

## Compatibility

Tested with:
- ✅ FFmpeg
- ✅ OBS Studio
- ✅ VLC Media Player
- ✅ ffplay

## Requirements

- Rust 1.70 or higher
- Tokio runtime
- OpenSSL (for crypto operations)

## Dependencies

Main dependencies:
- `tokio` - Async runtime
- `byteorder` - Byte order handling
- `log` - Logging facade
- `thiserror` - Error handling
- `async-trait` - Async traits

## Roadmap

- [ ] RTMPS (RTMP over TLS) support
- [ ] Enhanced relay capabilities
- [ ] Stream recording to disk
- [ ] HLS/DASH output
- [ ] WebRTC integration
- [ ] Comprehensive test suite
- [ ] Performance benchmarks
- [ ] Documentation improvements

## Performance

The server is built on Tokio and uses async I/O throughout, allowing it to handle thousands of concurrent connections efficiently.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT OR Apache-2.0 license.

## Resources

- [RTMP Specification](https://rtmp.veriskope.com/docs/spec/)
- [Adobe RTMP Specification](https://www.adobe.com/devnet/rtmp.html)
- [FFmpeg RTMP](https://trac.ffmpeg.org/wiki/EncodingForStreamingSites)

## Support

For questions, issues, or contributions, please open an issue on the project repository.
