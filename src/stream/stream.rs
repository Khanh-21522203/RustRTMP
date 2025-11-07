use crate::{Error, Result};
use crate::protocol::{RtmpPacket, RtmpData};
use crate::amf::Amf0Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StreamInfo {
    /// Stream ID
    pub id: u32,

    /// Stream name
    pub name: String,

    /// Stream type
    pub stream_type: StreamType,

    /// Creation timestamp
    pub created_at: u32,

    /// Metadata
    pub metadata: Option<StreamMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamType {
    Live,
    Record,
    Append,
    PlayOnly,
}

#[derive(Debug, Clone)]
pub struct StreamMetadata {
    /// Video codec
    pub video_codec: Option<String>,

    /// Audio codec
    pub audio_codec: Option<String>,

    /// Video width
    pub width: Option<f64>,

    /// Video height
    pub height: Option<f64>,

    /// Frame rate
    pub framerate: Option<f64>,

    /// Video bitrate
    pub video_bitrate: Option<f64>,

    /// Audio bitrate
    pub audio_bitrate: Option<f64>,

    /// Audio sample rate
    pub audio_sample_rate: Option<f64>,

    /// Audio channels
    pub audio_channels: Option<f64>,

    /// Duration for VOD
    pub duration: Option<f64>,

    /// Custom properties
    pub custom: HashMap<String, Amf0Value>,
}

impl StreamMetadata {
    /// Create from AMF metadata
    pub fn from_amf(data: &HashMap<String, Amf0Value>) -> Self {
        let mut metadata = StreamMetadata {
            video_codec: data.get("videocodecid").and_then(|v| v.as_string()).map(String::from),
            audio_codec: data.get("audiocodecid").and_then(|v| v.as_string()).map(String::from),
            width: data.get("width").and_then(|v| v.as_number()),
            height: data.get("height").and_then(|v| v.as_number()),
            framerate: data.get("framerate").and_then(|v| v.as_number()),
            video_bitrate: data.get("videodatarate").and_then(|v| v.as_number()),
            audio_bitrate: data.get("audiodatarate").and_then(|v| v.as_number()),
            audio_sample_rate: data.get("audiosamplerate").and_then(|v| v.as_number()),
            audio_channels: data.get("audiochannels").and_then(|v| v.as_number()),
            duration: data.get("duration").and_then(|v| v.as_number()),
            custom: HashMap::new(),
        };

        // Store other properties as custom
        for (key, value) in data {
            if !is_standard_metadata_key(key) {
                metadata.custom.insert(key.clone(), value.clone());
            }
        }

        metadata
    }

    /// Convert to AMF for sending
    pub fn to_amf(&self) -> HashMap<String, Amf0Value> {
        let mut data = HashMap::new();

        if let Some(ref codec) = self.video_codec {
            data.insert("videocodecid".to_string(), Amf0Value::String(codec.clone()));
        }
        if let Some(ref codec) = self.audio_codec {
            data.insert("audiocodecid".to_string(), Amf0Value::String(codec.clone()));
        }
        if let Some(width) = self.width {
            data.insert("width".to_string(), Amf0Value::Number(width));
        }
        if let Some(height) = self.height {
            data.insert("height".to_string(), Amf0Value::Number(height));
        }
        if let Some(fps) = self.framerate {
            data.insert("framerate".to_string(), Amf0Value::Number(fps));
        }
        if let Some(bitrate) = self.video_bitrate {
            data.insert("videodatarate".to_string(), Amf0Value::Number(bitrate));
        }
        if let Some(bitrate) = self.audio_bitrate {
            data.insert("audiodatarate".to_string(), Amf0Value::Number(bitrate));
        }
        if let Some(rate) = self.audio_sample_rate {
            data.insert("audiosamplerate".to_string(), Amf0Value::Number(rate));
        }
        if let Some(channels) = self.audio_channels {
            data.insert("audiochannels".to_string(), Amf0Value::Number(channels));
        }
        if let Some(duration) = self.duration {
            data.insert("duration".to_string(), Amf0Value::Number(duration));
        }

        // Add custom properties
        for (key, value) in &self.custom {
            data.insert(key.clone(), value.clone());
        }

        data
    }
}

fn is_standard_metadata_key(key: &str) -> bool {
    matches!(key,
        "videocodecid" | "audiocodecid" | "width" | "height" | "framerate" |
        "videodatarate" | "audiodatarate" | "audiosamplerate" | "audiochannels" | "duration"
    )
}

pub struct Stream {
    /// Stream info
    info: Arc<RwLock<StreamInfo>>,

    /// Stream statistics
    stats: Arc<RwLock<StreamStats>>,
}

#[derive(Debug, Default, Clone)]
pub struct StreamStats {
    /// Bytes received
    pub bytes_in: u64,

    /// Bytes sent
    pub bytes_out: u64,

    /// Audio packets
    pub audio_packets: u64,

    /// Video packets
    pub video_packets: u64,

    /// Data packets
    pub data_packets: u64,

    /// Last audio timestamp
    pub last_audio_timestamp: u32,

    /// Last video timestamp
    pub last_video_timestamp: u32,
}

impl Stream {
    /// Create new stream
    pub fn new(id: u32, name: String, stream_type: StreamType) -> Self {
        let info = StreamInfo {
            id,
            name,
            stream_type,
            created_at: crate::utils::current_timestamp(),
            metadata: None,
        };

        Stream {
            info: Arc::new(RwLock::new(info)),
            stats: Arc::new(RwLock::new(StreamStats::default())),
        }
    }

    /// Get stream info
    pub async fn info(&self) -> StreamInfo {
        self.info.read().await.clone()
    }

    /// Update metadata
    pub async fn set_metadata(&self, metadata: StreamMetadata) {
        let mut info = self.info.write().await;
        info.metadata = Some(metadata);
    }

    /// Update statistics
    pub async fn update_stats<F>(&self, updater: F)
    where
        F: FnOnce(&mut StreamStats),
    {
        let mut stats = self.stats.write().await;
        updater(&mut stats);
    }

    /// Get statistics
    pub async fn stats(&self) -> StreamStats {
        (*self.stats.read().await).clone()
    }
}