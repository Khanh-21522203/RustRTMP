use crate::{Error, Result};
use crate::amf::{Amf0Value, Amf0Decoder};
use crate::protocol::RtmpData;
use std::collections::HashMap;

pub struct MetadataProcessor {
    /// Cached metadata
    metadata_cache: HashMap<String, Amf0Value>,

    /// Last update timestamp
    last_update: Option<u32>,
}

impl MetadataProcessor {
    /// Create new metadata processor
    pub fn new() -> Self {
        MetadataProcessor {
            metadata_cache: HashMap::new(),
            last_update: None,
        }
    }

    /// Process metadata packet
    pub fn process(&mut self, data: &RtmpData, timestamp: u32) -> Result<Metadata> {
        // Check for onMetaData
        if data.data_type != "onMetaData" && data.data_type != "@setDataFrame" {
            return Err(Error::protocol("Not a metadata message"));
        }

        // Extract metadata object
        let metadata_obj = if data.data_type == "@setDataFrame" && data.values.len() > 1 {
            // @setDataFrame has key as first value
            data.values.get(1)
        } else {
            // onMetaData has metadata as first value
            data.values.first()
        };

        let metadata_obj = metadata_obj
            .and_then(|v| v.as_object())
            .ok_or_else(|| Error::protocol("Invalid metadata format"))?;

        // Update cache
        for (key, value) in metadata_obj {
            self.metadata_cache.insert(key.clone(), value.clone());
        }

        self.last_update = Some(timestamp);

        // Parse into structured metadata
        Ok(self.parse_metadata(metadata_obj))
    }

    /// Parse metadata object
    fn parse_metadata(&self, obj: &HashMap<String, Amf0Value>) -> Metadata {
        Metadata {
            // Video properties
            width: obj.get("width").and_then(|v| v.as_number()),
            height: obj.get("height").and_then(|v| v.as_number()),
            video_codec_id: obj.get("videocodecid")
                .and_then(|v| v.as_string())
                .map(String::from),
            video_data_rate: obj.get("videodatarate").and_then(|v| v.as_number()),
            framerate: obj.get("framerate").and_then(|v| v.as_number()),

            // Audio properties
            audio_codec_id: obj.get("audiocodecid")
                .and_then(|v| v.as_string())
                .map(String::from),
            audio_data_rate: obj.get("audiodatarate").and_then(|v| v.as_number()),
            audio_sample_rate: obj.get("audiosamplerate").and_then(|v| v.as_number()),
            audio_sample_size: obj.get("audiosamplesize").and_then(|v| v.as_number()),
            audio_channels: obj.get("audiochannels").and_then(|v| v.as_number()),
            stereo: obj.get("stereo").and_then(|v| v.as_boolean()),

            // File properties
            duration: obj.get("duration").and_then(|v| v.as_number()),
            file_size: obj.get("filesize").and_then(|v| v.as_number()),

            // Encoder info
            encoder: obj.get("encoder")
                .and_then(|v| v.as_string())
                .map(String::from),

            // Other properties
            can_seek_to_end: obj.get("canSeekToEnd").and_then(|v| v.as_boolean()),
        }
    }

    /// Get cached metadata
    pub fn get_cached(&self) -> &HashMap<String, Amf0Value> {
        &self.metadata_cache
    }

    /// Clear cache
    pub fn clear_cache(&mut self) {
        self.metadata_cache.clear();
        self.last_update = None;
    }
}

#[derive(Debug, Clone)]
pub struct Metadata {
    // Video properties
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub video_codec_id: Option<String>,
    pub video_data_rate: Option<f64>,
    pub framerate: Option<f64>,

    // Audio properties
    pub audio_codec_id: Option<String>,
    pub audio_data_rate: Option<f64>,
    pub audio_sample_rate: Option<f64>,
    pub audio_sample_size: Option<f64>,
    pub audio_channels: Option<f64>,
    pub stereo: Option<bool>,

    // File properties
    pub duration: Option<f64>,
    pub file_size: Option<f64>,

    // Encoder info
    pub encoder: Option<String>,

    // Other
    pub can_seek_to_end: Option<bool>,
}

impl Metadata {
    /// Check if has video
    pub fn has_video(&self) -> bool {
        self.video_codec_id.is_some() || self.width.is_some()
    }

    /// Check if has audio
    pub fn has_audio(&self) -> bool {
        self.audio_codec_id.is_some() || self.audio_sample_rate.is_some()
    }

    /// Get video resolution
    pub fn resolution(&self) -> Option<(u32, u32)> {
        match (self.width, self.height) {
            (Some(w), Some(h)) => Some((w as u32, h as u32)),
            _ => None,
        }
    }

    /// Create AMF object for sending
    pub fn to_amf(&self) -> HashMap<String, Amf0Value> {
        let mut obj = HashMap::new();

        if let Some(w) = self.width {
            obj.insert("width".to_string(), Amf0Value::Number(w));
        }
        if let Some(h) = self.height {
            obj.insert("height".to_string(), Amf0Value::Number(h));
        }
        if let Some(ref codec) = self.video_codec_id {
            obj.insert("videocodecid".to_string(), Amf0Value::String(codec.clone()));
        }
        if let Some(rate) = self.video_data_rate {
            obj.insert("videodatarate".to_string(), Amf0Value::Number(rate));
        }
        if let Some(fps) = self.framerate {
            obj.insert("framerate".to_string(), Amf0Value::Number(fps));
        }

        if let Some(ref codec) = self.audio_codec_id {
            obj.insert("audiocodecid".to_string(), Amf0Value::String(codec.clone()));
        }
        if let Some(rate) = self.audio_data_rate {
            obj.insert("audiodatarate".to_string(), Amf0Value::Number(rate));
        }
        if let Some(rate) = self.audio_sample_rate {
            obj.insert("audiosamplerate".to_string(), Amf0Value::Number(rate));
        }

        if let Some(duration) = self.duration {
            obj.insert("duration".to_string(), Amf0Value::Number(duration));
        }

        if let Some(ref encoder) = self.encoder {
            obj.insert("encoder".to_string(), Amf0Value::String(encoder.clone()));
        }

        obj
    }
}