use crate::message::types::MessageType;
use crate::{Amf0Value, RtmpCommand, RtmpHeader, RtmpPacket};

mod dispatcher;
mod queue;
mod types;

pub use dispatcher::*;
pub use queue::*;
pub use types::*;

pub fn classify_message(packet: &RtmpPacket) -> MessageType {
    MessageType::from_id(packet.message_type())
}

pub fn create_response(request: &RtmpPacket, result: Amf0Value) -> RtmpPacket {
    // Decode request command to get transaction ID
    let command = match RtmpCommand::decode(&request.payload) {
        Ok(cmd) => cmd,
        Err(_) => {
            // Create error response
            let error_cmd = RtmpCommand::error(0.0, Amf0Value::Null);
            let bytes = error_cmd.encode().unwrap_or_default();
            let header = RtmpHeader::command(0, bytes.len() as u32, 0);
            return RtmpPacket::new(header, bytes);
        }
    };

    // Create result response
    let response_cmd = RtmpCommand::result(command.transaction_id, result);
    let bytes = response_cmd.encode().unwrap_or_default();

    // Create response header (same stream ID as request)
    let header = RtmpHeader::command(
        request.timestamp(),
        bytes.len() as u32,
        request.message_stream_id(),
    );

    RtmpPacket::new(header, bytes)
}