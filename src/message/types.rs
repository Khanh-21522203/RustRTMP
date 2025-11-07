use crate::protocol::constants::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// Control messages
    Control(ControlType),

    /// Audio data
    Audio,

    /// Video data
    Video,

    /// Command (AMF0/AMF3)
    Command,

    /// Data (AMF0/AMF3)
    Data,

    /// Aggregate message
    Aggregate,

    /// Shared object (AMF0/AMF3)
    SharedObject,

    /// Unknown type
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlType {
    SetChunkSize,
    Abort,
    Acknowledgement,
    WindowAcknowledgement,
    SetPeerBandwidth,
}

impl MessageType {
    /// Create from message type ID
    pub fn from_id(id: u8) -> Self {
        match id {
            MSG_TYPE_SET_CHUNK_SIZE => MessageType::Control(ControlType::SetChunkSize),
            MSG_TYPE_ABORT => MessageType::Control(ControlType::Abort),
            MSG_TYPE_ACK => MessageType::Control(ControlType::Acknowledgement),
            MSG_TYPE_WINDOW_ACK => MessageType::Control(ControlType::WindowAcknowledgement),
            MSG_TYPE_SET_PEER_BW => MessageType::Control(ControlType::SetPeerBandwidth),
            MSG_TYPE_AUDIO => MessageType::Audio,
            MSG_TYPE_VIDEO => MessageType::Video,
            MSG_TYPE_COMMAND_AMF0 | MSG_TYPE_COMMAND_AMF3 => MessageType::Command,
            MSG_TYPE_DATA_AMF0 | MSG_TYPE_DATA_AMF3 => MessageType::Data,
            MSG_TYPE_AGGREGATE => MessageType::Aggregate,
            MSG_TYPE_SHARED_OBJECT_AMF0 | MSG_TYPE_SHARED_OBJECT_AMF3 => MessageType::SharedObject,
            _ => MessageType::Unknown(id),
        }
    }

    /// Check if this is a control message
    pub fn is_control(&self) -> bool {
        matches!(self, MessageType::Control(_))
    }

    /// Check if this is a media message (audio/video)
    pub fn is_media(&self) -> bool {
        matches!(self, MessageType::Audio | MessageType::Video)
    }

    /// Check if this is a command message
    pub fn is_command(&self) -> bool {
        matches!(self, MessageType::Command)
    }

    /// Get processing priority
    pub fn priority(&self) -> u8 {
        match self {
            MessageType::Control(_) => 10,
            MessageType::Command => 8,
            MessageType::Data => 6,
            MessageType::Audio => 4,
            MessageType::Video => 2,
            _ => 1,
        }
    }
}

/// Additional message type constants
pub mod constants {
    pub const MSG_TYPE_AGGREGATE: u8 = 22;
    pub const MSG_TYPE_SHARED_OBJECT_AMF0: u8 = 19;
    pub const MSG_TYPE_SHARED_OBJECT_AMF3: u8 = 16;
}