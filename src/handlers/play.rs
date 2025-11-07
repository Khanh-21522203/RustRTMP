use std::sync::Arc;
use crate::{Amf0Value, ConnectionContext, Error, RtmpCommand, RtmpData, RtmpHeader, RtmpPacket, Result, HandlerContext, PublisherInfo};
use crate::handlers::CommandHandler;
use crate::handlers::publish::create_stream_begin_packet;

pub struct PlayHandler;

impl PlayHandler {
    pub fn new() -> Self {
        PlayHandler
    }

    async fn find_publisher(
        &self,
        stream_name: &str,
        context: Arc<ConnectionContext>,
    ) -> Result<PublisherInfo> {
        let registry = context.get_publisher_registry()
            .ok_or_else(|| Error::stream("No publisher registry"))?;

        registry.get(stream_name).await
            .ok_or_else(|| Error::stream(format!("Stream '{}' not found", stream_name)))
    }

    fn create_play_status_messages(&self, stream_name: &str, stream_id: u32) -> Vec<RtmpPacket> {
        let mut packets = Vec::new();

        // Stream Begin
        packets.push(create_stream_begin_packet(stream_id));

        // Play.Reset
        let reset = RtmpCommand::on_status(
            "status",
            "NetStream.Play.Reset",
            &format!("Playing and resetting {}", stream_name),
        );
        let bytes = reset.encode().unwrap();
        let header = RtmpHeader::command(0, bytes.len() as u32, stream_id);
        packets.push(RtmpPacket::new(header, bytes));

        // Play.Start
        let start = RtmpCommand::on_status(
            "status",
            "NetStream.Play.Start",
            &format!("Started playing {}", stream_name),
        );
        let bytes = start.encode().unwrap();
        let header = RtmpHeader::command(0, bytes.len() as u32, stream_id);
        packets.push(RtmpPacket::new(header, bytes));

        // RtmpSampleAccess
        packets.push(create_sample_access_packet(stream_id));

        // Data.Start
        let data_start = RtmpCommand::on_status(
            "status",
            "NetStream.Data.Start",
            "Data start",
        );
        let bytes = data_start.encode().unwrap();
        let header = RtmpHeader::data(0, bytes.len() as u32, stream_id);
        packets.push(RtmpPacket::new(header, bytes));

        packets
    }
}

#[async_trait::async_trait]
impl CommandHandler for PlayHandler {
    fn command_name(&self) -> &str {
        "play"
    }

    async fn handle(
        &self,
        command: RtmpCommand,
        context: Arc<ConnectionContext>,
    ) -> Result<Option<RtmpPacket>> {
        // Extract parameters
        let stream_name = command.arguments.get(0)
            .and_then(|v| v.as_string())
            .ok_or_else(|| Error::protocol("Missing stream name"))?
            .to_string();

        let start = command.arguments.get(1)
            .and_then(|v| v.as_number())
            .unwrap_or(0.0);

        let duration = command.arguments.get(2)
            .and_then(|v| v.as_number())
            .unwrap_or(-1.0);

        let reset = command.arguments.get(3)
            .and_then(|v| v.as_boolean())
            .unwrap_or(true);

        // Get stream ID
        let stream_id = context.get_property("stream_id").await
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| Error::protocol("No stream ID"))?;

        // Find publisher
        let publisher = self.find_publisher(&stream_name, context.clone()).await?;

        // Subscribe to publisher
        if let Some(registry) = context.get_publisher_registry() {
            registry.increment_subscribers(&stream_name).await?;
        }

        // Update context
        context.set_property("playing".to_string(), "true".to_string()).await;
        context.set_property("stream_name".to_string(), stream_name.clone()).await;
        context.set_property("play_start".to_string(), start.to_string()).await;
        context.set_property("play_duration".to_string(), duration.to_string()).await;

        // Send status messages
        let messages = self.create_play_status_messages(&stream_name, stream_id);
        for msg in messages {
            context.send_packet(msg).await?;
        }

        // Note: Actual data delivery would be handled by stream management (Phase 10)

        Ok(None) // All responses sent directly
    }
}

fn create_sample_access_packet(stream_id: u32) -> RtmpPacket {
    let mut data = RtmpData::new("|RtmpSampleAccess".to_string());
    data.values.push(Amf0Value::Boolean(true)); // Audio
    data.values.push(Amf0Value::Boolean(true)); // Video

    let bytes = data.encode().unwrap();
    let header = RtmpHeader::data(0, bytes.len() as u32, stream_id);

    RtmpPacket::new(header, bytes)
}