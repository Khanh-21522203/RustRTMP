use crate::{Error, Result, RtmpPacket};

mod connection;
mod state;
mod context;
mod stream_manager;

pub use connection::*;
pub use state::*;
pub use context::*;
pub use stream_manager::*;

pub fn process_control_message(msg: &RtmpPacket) -> Result<()> {
    match msg.message_type() {
        MSG_TYPE_SET_CHUNK_SIZE => {
            if msg.payload.len() < 4 {
                return Err(Error::protocol("Invalid chunk size message"));
            }

            let size = u32::from_be_bytes([
                msg.payload[0],
                msg.payload[1],
                msg.payload[2],
                msg.payload[3],
            ]);

            if size < 128 || size > 0x7FFFFFFF {
                return Err(Error::protocol("Invalid chunk size"));
            }

            // Actual update would be done by caller
            Ok(())
        }
        MSG_TYPE_ABORT => {
            if msg.payload.len() < 4 {
                return Err(Error::protocol("Invalid abort message"));
            }

            let chunk_stream_id = u32::from_be_bytes([
                msg.payload[0],
                msg.payload[1],
                msg.payload[2],
                msg.payload[3],
            ]);

            // Actual abort would be done by caller
            Ok(())
        }
        MSG_TYPE_ACK => {
            // Process acknowledgement
            Ok(())
        }
        MSG_TYPE_WINDOW_ACK => {
            // Process window acknowledgement
            Ok(())
        }
        MSG_TYPE_SET_PEER_BW => {
            // Process peer bandwidth
            Ok(())
        }
        _ => Err(Error::protocol("Unknown control message type")),
    }
}