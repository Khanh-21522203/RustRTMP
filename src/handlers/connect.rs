use crate::{ConnectionContext, Error, HandlerContext, Result};
use crate::protocol::{RtmpCommand, RtmpPacket, RtmpHeader};
use crate::protocol::constants::*;
use crate::amf::Amf0Value;
use std::collections::HashMap;
use std::sync::Arc;
use crate::handlers::CommandHandler;

pub struct ConnectHandler {
    /// Supported encoding
    object_encoding: f64,
}

impl ConnectHandler {
    pub fn new() -> Self {
        ConnectHandler {
            object_encoding: 0.0, // AMF0
        }
    }

    fn validate_connect_params(&self, command: &RtmpCommand) -> Result<ConnectParams> {
        let params = command.command_object.as_ref()
            .and_then(|v| v.as_object())
            .ok_or_else(|| Error::protocol("Missing connect parameters"))?;

        let app = params.get("app")
            .and_then(|v| v.as_string())
            .ok_or_else(|| Error::protocol("Missing app parameter"))?
            .to_string();

        let tc_url = params.get("tcUrl")
            .and_then(|v| v.as_string())
            .ok_or_else(|| Error::protocol("Missing tcUrl parameter"))?
            .to_string();

        let flash_ver = params.get("flashVer")
            .and_then(|v| v.as_string())
            .unwrap_or("FMLE/3.0")
            .to_string();

        let object_encoding = params.get("objectEncoding")
            .and_then(|v| v.as_number())
            .unwrap_or(0.0);

        Ok(ConnectParams {
            app,
            tc_url,
            flash_ver,
            object_encoding,
        })
    }

    fn create_connect_result(&self, transaction_id: f64) -> RtmpCommand {
        // Create result properties
        let mut props = HashMap::new();
        props.insert("fmsVer".to_string(), Amf0Value::String("FMS/3,5,5,2004".to_string()));
        props.insert("capabilities".to_string(), Amf0Value::Number(31.0));
        props.insert("mode".to_string(), Amf0Value::Number(1.0));

        // Create info object
        let mut info = HashMap::new();
        info.insert("level".to_string(), Amf0Value::String("status".to_string()));
        info.insert("code".to_string(), Amf0Value::String("NetConnection.Connect.Success".to_string()));
        info.insert("description".to_string(), Amf0Value::String("Connection succeeded".to_string()));
        info.insert("data".to_string(), Amf0Value::Object(HashMap::new()));
        info.insert("objectEncoding".to_string(), Amf0Value::Number(self.object_encoding));

        let mut result = RtmpCommand::result(transaction_id, Amf0Value::Object(props));
        result.arguments.push(Amf0Value::Object(info));

        result
    }

    async fn send_server_bandwidth(&self, context: Arc<ConnectionContext>) -> Result<()> {
        // Send Window Acknowledgement Size
        let window_ack = create_window_ack_packet(2500000);
        context.send_packet(window_ack).await?;

        // Send Set Peer Bandwidth
        let peer_bw = create_peer_bandwidth_packet(2500000, 2);
        context.send_packet(peer_bw).await?;

        // Send Set Chunk Size
        let chunk_size = create_chunk_size_packet(4096);
        context.send_packet(chunk_size).await?;
        context.set_chunk_size_out(4096).await;

        Ok(())
    }
}

#[async_trait::async_trait]
impl CommandHandler for ConnectHandler {
    fn command_name(&self) -> &str {
        "connect"
    }

    async fn handle(
        &self,
        command: RtmpCommand,
        context: Arc<ConnectionContext>,
    ) -> Result<Option<RtmpPacket>> {
        // Validate parameters
        let params = self.validate_connect_params(&command)?;

        // Store connection info in context
        context.set_property("app".to_string(), params.app.clone()).await;
        context.set_property("tc_url".to_string(), params.tc_url.clone()).await;
        context.set_property("flash_ver".to_string(), params.flash_ver.clone()).await;

        // Send server bandwidth settings
        self.send_server_bandwidth(context.clone()).await?;

        // Create success response
        let response = self.create_connect_result(command.transaction_id);
        let bytes = response.encode()?;
        let header = RtmpHeader::command(0, bytes.len() as u32, 0);

        Ok(Some(RtmpPacket::new(header, bytes)))
    }
}

struct ConnectParams {
    app: String,
    tc_url: String,
    flash_ver: String,
    object_encoding: f64,
}

// Helper functions for control messages
fn create_window_ack_packet(size: u32) -> RtmpPacket {
    let mut payload = Vec::new();
    payload.extend_from_slice(&size.to_be_bytes());

    let header = RtmpHeader::new(
        0,
        payload.len() as u32,
        MSG_TYPE_WINDOW_ACK,
        0,
        CHUNK_STREAM_PROTOCOL,
    );

    RtmpPacket::new(header, payload)
}

fn create_peer_bandwidth_packet(size: u32, limit_type: u8) -> RtmpPacket {
    let mut payload = Vec::new();
    payload.extend_from_slice(&size.to_be_bytes());
    payload.push(limit_type);

    let header = RtmpHeader::new(
        0,
        payload.len() as u32,
        MSG_TYPE_SET_PEER_BW,
        0,
        CHUNK_STREAM_PROTOCOL,
    );

    RtmpPacket::new(header, payload)
}

fn create_chunk_size_packet(size: u32) -> RtmpPacket {
    let mut payload = Vec::new();
    payload.extend_from_slice(&size.to_be_bytes());

    let header = RtmpHeader::new(
        0,
        payload.len() as u32,
        MSG_TYPE_SET_CHUNK_SIZE,
        0,
        CHUNK_STREAM_PROTOCOL,
    );

    RtmpPacket::new(header, payload)
}