use crate::{Error, Result};
use crate::amf::{Amf0Value, Amf0Encoder, Amf0Decoder};
use crate::ByteBuffer;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RtmpData {
    pub data_type: String,
    pub values: Vec<Amf0Value>,
}

impl RtmpData {
    /// Create new data message
    pub fn new(data_type: String) -> Self {
        RtmpData {
            data_type,
            values: Vec::new(),
        }
    }

    /// Create onMetaData message
    pub fn on_metadata(metadata: HashMap<String, Amf0Value>) -> Self {
        let mut data = RtmpData::new("onMetaData".to_string());
        data.values.push(Amf0Value::Object(metadata));
        data
    }

    /// Create setDataFrame message
    pub fn set_data_frame(key: &str, value: Amf0Value) -> Self {
        let mut data = RtmpData::new("@setDataFrame".to_string());
        data.values.push(Amf0Value::String(key.to_string()));
        data.values.push(value);
        data
    }

    /// Add common metadata fields
    pub fn with_stream_metadata(
        width: f64,
        height: f64,
        video_codec: &str,
        audio_codec: &str,
        fps: f64,
    ) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("width".to_string(), Amf0Value::Number(width));
        metadata.insert("height".to_string(), Amf0Value::Number(height));
        metadata.insert("videocodecid".to_string(), Amf0Value::String(video_codec.to_string()));
        metadata.insert("audiocodecid".to_string(), Amf0Value::String(audio_codec.to_string()));
        metadata.insert("framerate".to_string(), Amf0Value::Number(fps));

        RtmpData::on_metadata(metadata)
    }

    /// Encode data message to bytes
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut encoder = Amf0Encoder::new();

        // Encode data type
        encoder.encode(&Amf0Value::String(self.data_type.clone()))?;

        // Encode values
        for value in &self.values {
            encoder.encode(value)?;
        }

        Ok(encoder.get_bytes())
    }

    /// Decode data message from bytes
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut buffer = ByteBuffer::new(data.to_vec());
        let mut decoder = Amf0Decoder::new(&mut buffer);

        // Decode data type
        let type_val = decoder.decode()?;
        let data_type = type_val.as_string()
            .ok_or_else(|| Error::amf_decode("Data type must be string"))?
            .to_string();

        // Decode remaining values
        let mut values = Vec::new();
        while decoder.has_remaining() {
            values.push(decoder.decode()?);
        }

        Ok(RtmpData {
            data_type,
            values,
        })
    }

    /// Get metadata object if this is onMetaData
    pub fn get_metadata(&self) -> Option<&HashMap<String, Amf0Value>> {
        if self.data_type == "onMetaData" && !self.values.is_empty() {
            self.values[0].as_object()
        } else {
            None
        }
    }
}