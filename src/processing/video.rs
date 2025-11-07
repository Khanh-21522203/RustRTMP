use crate::{Error, Result};
use crate::protocol::RtmpPacket;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoCodec {
    /// Sorenson H.263
    H263,
    /// Screen video
    ScreenVideo,
    /// On2 VP6
    VP6,
    /// On2 VP6 with alpha
    VP6Alpha,
    /// Screen video v2
    ScreenVideo2,
    /// H.264 AVC
    H264,
    /// H.265 HEVC
    H265,
    /// AV1
    AV1,
    /// Unknown
    Unknown(u8),
}

impl VideoCodec {
    /// Parse from codec ID
    pub fn from_codec_id(id: u8) -> Self {
        match id {
            2 => VideoCodec::H263,
            3 => VideoCodec::ScreenVideo,
            4 => VideoCodec::VP6,
            5 => VideoCodec::VP6Alpha,
            6 => VideoCodec::ScreenVideo2,
            7 => VideoCodec::H264,
            12 => VideoCodec::H265,
            13 => VideoCodec::AV1,
            _ => VideoCodec::Unknown(id),
        }
    }

    /// Get codec name
    pub fn name(&self) -> &str {
        match self {
            VideoCodec::H263 => "H.263",
            VideoCodec::ScreenVideo => "Screen",
            VideoCodec::VP6 => "VP6",
            VideoCodec::VP6Alpha => "VP6-Alpha",
            VideoCodec::ScreenVideo2 => "Screen-v2",
            VideoCodec::H264 => "H.264",
            VideoCodec::H265 => "H.265",
            VideoCodec::AV1 => "AV1",
            VideoCodec::Unknown(_) => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FrameType {
    /// Keyframe (I-frame)
    Keyframe,
    /// Inter-frame (P-frame)
    InterFrame,
    /// Disposable inter-frame
    DisposableInterFrame,
    /// Generated keyframe
    GeneratedKeyframe,
    /// Video info/command frame
    VideoInfo,
}

impl FrameType {
    pub fn from_bits(bits: u8) -> Self {
        match bits {
            1 => FrameType::Keyframe,
            2 => FrameType::InterFrame,
            3 => FrameType::DisposableInterFrame,
            4 => FrameType::GeneratedKeyframe,
            5 => FrameType::VideoInfo,
            _ => FrameType::InterFrame,
        }
    }

    pub fn is_keyframe(&self) -> bool {
        matches!(self, FrameType::Keyframe | FrameType::GeneratedKeyframe)
    }
}

pub struct VideoProcessor {
    /// Current codec
    codec: Option<VideoCodec>,

    /// Last keyframe timestamp
    last_keyframe_timestamp: Option<u32>,

    /// Frame count since last keyframe
    frames_since_keyframe: u32,

    /// AVC/HEVC configuration
    avc_config: Option<AVCVideoConfig>,
}

#[derive(Debug, Clone)]
pub struct AVCVideoConfig {
    /// Configuration version
    pub version: u8,

    /// AVC profile
    pub profile: u8,

    /// AVC profile compatibility
    pub profile_compat: u8,

    /// AVC level
    pub level: u8,

    /// SPS (Sequence Parameter Sets)
    pub sps: Vec<Vec<u8>>,

    /// PPS (Picture Parameter Sets)
    pub pps: Vec<Vec<u8>>,
}

impl VideoProcessor {
    /// Create new video processor
    pub fn new() -> Self {
        VideoProcessor {
            codec: None,
            last_keyframe_timestamp: None,
            frames_since_keyframe: 0,
            avc_config: None,
        }
    }

    /// Process video packet
    pub fn process(&mut self, packet: &RtmpPacket) -> Result<VideoInfo> {
        if packet.payload.is_empty() {
            return Err(Error::protocol("Empty video packet"));
        }

        let tag_header = packet.payload[0];

        // Parse video tag header
        let frame_type = (tag_header >> 4) & 0x0F;
        let codec_id = tag_header & 0x0F;

        let frame = FrameType::from_bits(frame_type);
        let codec = VideoCodec::from_codec_id(codec_id);

        // Update state
        self.codec = Some(codec);

        if frame.is_keyframe() {
            self.last_keyframe_timestamp = Some(packet.timestamp());
            self.frames_since_keyframe = 0;
        } else {
            self.frames_since_keyframe += 1;
        }

        // Handle AVC/HEVC specific
        if (codec == VideoCodec::H264 || codec == VideoCodec::H265) && packet.payload.len() > 1 {
            let avc_packet_type = packet.payload[1];

            if avc_packet_type == 0 {
                // AVC sequence header
                self.parse_avc_config(&packet.payload[2..])?;
            }
        }

        Ok(VideoInfo {
            codec,
            frame_type: frame,
            is_sequence_header: (codec == VideoCodec::H264 || codec == VideoCodec::H265) &&
                packet.payload.len() > 1 &&
                packet.payload[1] == 0,
            is_keyframe: frame.is_keyframe(),
            frames_since_keyframe: self.frames_since_keyframe,
        })
    }

    /// Parse AVC video configuration
    fn parse_avc_config(&mut self, data: &[u8]) -> Result<()> {
        if data.len() < 7 {
            return Err(Error::protocol("AVC config too short"));
        }

        // AVCDecoderConfigurationRecord
        let version = data[0];
        let profile = data[1];
        let profile_compat = data[2];
        let level = data[3];

        let mut config = AVCVideoConfig {
            version,
            profile,
            profile_compat,
            level,
            sps: Vec::new(),
            pps: Vec::new(),
        };

        // Parse SPS
        let num_sps = data[5] & 0x1F;
        let mut offset = 6;

        for _ in 0..num_sps {
            if offset + 2 > data.len() {
                break;
            }

            let sps_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            if offset + sps_len <= data.len() {
                config.sps.push(data[offset..offset + sps_len].to_vec());
                offset += sps_len;
            }
        }

        // Parse PPS
        if offset < data.len() {
            let num_pps = data[offset];
            offset += 1;

            for _ in 0..num_pps {
                if offset + 2 > data.len() {
                    break;
                }

                let pps_len = u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
                offset += 2;

                if offset + pps_len <= data.len() {
                    config.pps.push(data[offset..offset + pps_len].to_vec());
                    offset += pps_len;
                }
            }
        }

        self.avc_config = Some(config);
        Ok(())
    }

    /// Check if GOP is too large
    pub fn gop_too_large(&self, max_gop_size: u32) -> bool {
        self.frames_since_keyframe > max_gop_size
    }

    /// Get current codec
    pub fn codec(&self) -> Option<VideoCodec> {
        self.codec
    }
}

pub struct VideoInfo {
    pub codec: VideoCodec,
    pub frame_type: FrameType,
    pub is_sequence_header: bool,
    pub is_keyframe: bool,
    pub frames_since_keyframe: u32,
}