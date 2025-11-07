use crate::{Error, Result};
use crate::protocol::RtmpPacket;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioCodec {
    /// Linear PCM, platform endian
    PCM,
    /// ADPCM
    ADPCM,
    /// MP3
    MP3,
    /// Linear PCM, little endian  
    PCMLittleEndian,
    /// Nellymoser 16kHz mono
    Nellymoser16kHz,
    /// Nellymoser 8kHz mono
    Nellymoser8kHz,
    /// Nellymoser
    Nellymoser,
    /// G.711 A-law
    G711ALaw,
    /// G.711 mu-law
    G711MuLaw,
    /// Reserved
    Reserved,
    /// AAC
    AAC,
    /// Speex
    Speex,
    /// MP3 8kHz
    MP38kHz,
    /// Device specific
    DeviceSpecific,
}

impl AudioCodec {
    /// Parse from sound format field
    pub fn from_sound_format(format: u8) -> Self {
        match format {
            0 => AudioCodec::PCM,
            1 => AudioCodec::ADPCM,
            2 => AudioCodec::MP3,
            3 => AudioCodec::PCMLittleEndian,
            4 => AudioCodec::Nellymoser16kHz,
            5 => AudioCodec::Nellymoser8kHz,
            6 => AudioCodec::Nellymoser,
            7 => AudioCodec::G711ALaw,
            8 => AudioCodec::G711MuLaw,
            9 => AudioCodec::Reserved,
            10 => AudioCodec::AAC,
            11 => AudioCodec::Speex,
            14 => AudioCodec::MP38kHz,
            15 => AudioCodec::DeviceSpecific,
            _ => AudioCodec::Reserved,
        }
    }

    /// Get codec name
    pub fn name(&self) -> &str {
        match self {
            AudioCodec::PCM => "PCM",
            AudioCodec::ADPCM => "ADPCM",
            AudioCodec::MP3 => "MP3",
            AudioCodec::PCMLittleEndian => "PCM-LE",
            AudioCodec::Nellymoser16kHz => "Nellymoser-16kHz",
            AudioCodec::Nellymoser8kHz => "Nellymoser-8kHz",
            AudioCodec::Nellymoser => "Nellymoser",
            AudioCodec::G711ALaw => "G.711-A",
            AudioCodec::G711MuLaw => "G.711-μ",
            AudioCodec::Reserved => "Reserved",
            AudioCodec::AAC => "AAC",
            AudioCodec::Speex => "Speex",
            AudioCodec::MP38kHz => "MP3-8kHz",
            AudioCodec::DeviceSpecific => "Device",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoundRate {
    Rate5_5kHz,
    Rate11kHz,
    Rate22kHz,
    Rate44kHz,
}

impl SoundRate {
    pub fn from_bits(bits: u8) -> Self {
        match bits {
            0 => SoundRate::Rate5_5kHz,
            1 => SoundRate::Rate11kHz,
            2 => SoundRate::Rate22kHz,
            3 => SoundRate::Rate44kHz,
            _ => SoundRate::Rate44kHz,
        }
    }

    pub fn as_hz(&self) -> u32 {
        match self {
            SoundRate::Rate5_5kHz => 5500,
            SoundRate::Rate11kHz => 11000,
            SoundRate::Rate22kHz => 22000,
            SoundRate::Rate44kHz => 44000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoundSize {
    Bits8,
    Bits16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SoundType {
    Mono,
    Stereo,
}

pub struct AudioProcessor {
    /// Current codec
    codec: Option<AudioCodec>,

    /// Sample rate
    sample_rate: Option<SoundRate>,

    /// Sample size
    sample_size: Option<SoundSize>,

    /// Sound type
    sound_type: Option<SoundType>,

    /// AAC specific config
    aac_config: Option<AACAudioConfig>,
}

#[derive(Debug, Clone)]
pub struct AACAudioConfig {
    /// Audio object type
    pub object_type: u8,

    /// Sampling frequency index
    pub sampling_index: u8,

    /// Channel configuration
    pub channel_config: u8,
}

impl AudioProcessor {
    /// Create new audio processor
    pub fn new() -> Self {
        AudioProcessor {
            codec: None,
            sample_rate: None,
            sample_size: None,
            sound_type: None,
            aac_config: None,
        }
    }

    /// Process audio packet
    pub fn process(&mut self, packet: &RtmpPacket) -> Result<AudioInfo> {
        if packet.payload.is_empty() {
            return Err(Error::protocol("Empty audio packet"));
        }

        let tag_header = packet.payload[0];

        // Parse audio tag header
        let sound_format = (tag_header >> 4) & 0x0F;
        let sound_rate = (tag_header >> 2) & 0x03;
        let sound_size = (tag_header >> 1) & 0x01;
        let sound_type = tag_header & 0x01;

        let codec = AudioCodec::from_sound_format(sound_format);
        let rate = SoundRate::from_bits(sound_rate);
        let size = if sound_size == 0 { SoundSize::Bits8 } else { SoundSize::Bits16 };
        let sound = if sound_type == 0 { SoundType::Mono } else { SoundType::Stereo };

        // Update state
        self.codec = Some(codec);
        self.sample_rate = Some(rate);
        self.sample_size = Some(size);
        self.sound_type = Some(sound);

        // Handle AAC specific
        if codec == AudioCodec::AAC && packet.payload.len() > 1 {
            let aac_packet_type = packet.payload[1];

            if aac_packet_type == 0 {
                // AAC sequence header
                self.parse_aac_config(&packet.payload[2..])?;
            }
        }

        Ok(AudioInfo {
            codec,
            sample_rate: rate,
            sample_size: size,
            sound_type: sound,
            is_sequence_header: codec == AudioCodec::AAC &&
                packet.payload.len() > 1 &&
                packet.payload[1] == 0,
        })
    }

    /// Parse AAC audio specific config
    fn parse_aac_config(&mut self, data: &[u8]) -> Result<()> {
        if data.len() < 2 {
            return Err(Error::protocol("AAC config too short"));
        }

        // AudioSpecificConfig ISO 14496-3
        let byte1 = data[0];
        let byte2 = data[1];

        let object_type = byte1 >> 3;
        let sampling_index = ((byte1 & 0x07) << 1) | (byte2 >> 7);
        let channel_config = (byte2 >> 3) & 0x0F;

        self.aac_config = Some(AACAudioConfig {
            object_type,
            sampling_index,
            channel_config,
        });

        Ok(())
    }

    /// Get current codec
    pub fn codec(&self) -> Option<AudioCodec> {
        self.codec
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> Option<u32> {
        self.sample_rate.map(|r| r.as_hz())
    }
}

pub struct AudioInfo {
    pub codec: AudioCodec,
    pub sample_rate: SoundRate,
    pub sample_size: SoundSize,
    pub sound_type: SoundType,
    pub is_sequence_header: bool,
}