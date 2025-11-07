use std::sync::Arc;
use crate::{ConnectionContext, Error, RtmpCommand, RtmpHeader, RtmpPacket, Result, HandlerContext, MSG_TYPE_USER_CONTROL, CHUNK_STREAM_PROTOCOL};
use crate::handlers::CommandHandler;

pub struct PublishHandler;

impl PublishHandler {
    pub fn new() -> Self {
        PublishHandler
    }

    async fn validate_publish(
        &self,
        stream_name: &str,
        context: Arc<ConnectionContext>,
    ) -> Result<()> {
        // Check if stream already exists
        if let Some(registry) = context.get_publisher_registry() {
            if registry.is_publishing(stream_name).await {
                return Err(Error::stream(format!(
                    "Stream '{}' is already being published",
                    stream_name
                )));
            }
        }

        Ok(())
    }

    fn create_publish_status(&self, stream_name: &str, stream_id: u32) -> RtmpPacket {
        let status = RtmpCommand::on_status(
            "status",
            "NetStream.Publish.Start",
            &format!("{} is now published", stream_name),
        );

        let bytes = status.encode().unwrap();
        let header = RtmpHeader::command(0, bytes.len() as u32, stream_id);

        RtmpPacket::new(header, bytes)
    }
}

#[async_trait::async_trait]
impl CommandHandler for PublishHandler {
    fn command_name(&self) -> &str {
        "publish"
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

        let publish_type = command.arguments.get(1)
            .and_then(|v| v.as_string())
            .unwrap_or("live")
            .to_string();

        // Get stream ID
        let stream_id = context.get_property("stream_id").await
            .and_then(|s| s.parse::<u32>().ok())
            .ok_or_else(|| Error::protocol("No stream ID"))?;

        // Validate
        self.validate_publish(&stream_name, context.clone()).await?;

        // Register publisher
        if let Some(registry) = context.get_publisher_registry() {
            registry.register(
                stream_name.clone(),
                context.connection_id().to_string(),
                stream_id,
            ).await?;
        }

        // Update context state
        context.set_property("publishing".to_string(), "true".to_string()).await;
        context.set_property("stream_name".to_string(), stream_name.clone()).await;
        context.set_property("publish_type".to_string(), publish_type).await;

        // Send Stream Begin
        let stream_begin = create_stream_begin_packet(stream_id);
        context.send_packet(stream_begin).await?;

        // Create status response
        let response = self.create_publish_status(&stream_name, stream_id);

        Ok(Some(response))
    }
}

pub fn create_stream_begin_packet(stream_id: u32) -> RtmpPacket {
    let mut payload = Vec::new();
    payload.extend_from_slice(&[0x00, 0x00]); // Event type: Stream Begin
    payload.extend_from_slice(&stream_id.to_be_bytes());

    let header = RtmpHeader::new(
        0,
        payload.len() as u32,
        MSG_TYPE_USER_CONTROL,
        0,
        CHUNK_STREAM_PROTOCOL,
    );

    RtmpPacket::new(header, payload)
}