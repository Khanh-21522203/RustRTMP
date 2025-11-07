use crate::processing::audio::AudioCodec;
use crate::processing::video::VideoCodec;

mod audio;
mod video;
mod metadata;

pub fn detect_audio_codec(data: &[u8]) -> AudioCodec {
    if data.is_empty() {
        return AudioCodec::Reserved;
    }

    let sound_format = (data[0] >> 4) & 0x0F;
    AudioCodec::from_sound_format(sound_format)
}

pub fn detect_video_codec(data: &[u8]) -> VideoCodec {
    if data.is_empty() {
        return VideoCodec::Unknown(0);
    }

    let codec_id = data[0] & 0x0F;
    VideoCodec::from_codec_id(codec_id)
}

pub fn is_keyframe(video_data: &[u8]) -> bool {
    if video_data.is_empty() {
        return false;
    }

    let frame_type = (video_data[0] >> 4) & 0x0F;
    frame_type == 1 || frame_type == 4 // Keyframe or Generated keyframe
}