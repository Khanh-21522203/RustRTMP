// Message types
pub const MSG_TYPE_SET_CHUNK_SIZE: u8 = 1;
pub const MSG_TYPE_ABORT: u8 = 2;
pub const MSG_TYPE_ACK: u8 = 3;
pub const MSG_TYPE_USER_CONTROL: u8 = 4;         // User control messages
pub const MSG_TYPE_WINDOW_ACK: u8 = 5;
pub const MSG_TYPE_SET_PEER_BW: u8 = 6;
pub const MSG_TYPE_AUDIO: u8 = 8;
pub const MSG_TYPE_VIDEO: u8 = 9;
pub const MSG_TYPE_DATA_AMF3: u8 = 15;
pub const MSG_TYPE_SHARED_OBJECT_AMF3: u8 = 16;  // Shared object (AMF3)
pub const MSG_TYPE_COMMAND_AMF3: u8 = 17;
pub const MSG_TYPE_DATA_AMF0: u8 = 18;
pub const MSG_TYPE_SHARED_OBJECT_AMF0: u8 = 19;  // Shared object (AMF0)
pub const MSG_TYPE_COMMAND_AMF0: u8 = 20;
pub const MSG_TYPE_AGGREGATE: u8 = 22;           // Aggregate message

// Chunk stream IDs
pub const CHUNK_STREAM_PROTOCOL: u32 = 2;
pub const CHUNK_STREAM_COMMAND: u32 = 3;
pub const CHUNK_STREAM_AUDIO: u32 = 4;
pub const CHUNK_STREAM_VIDEO: u32 = 6;
pub const CHUNK_STREAM_DATA: u32 = 8;

// Default values
pub const DEFAULT_CHUNK_SIZE: u32 = 128;
pub const DEFAULT_WINDOW_SIZE: u32 = 2500000;