use std::sync::Arc;
use async_trait::async_trait;
use crate::handlers::CommandHandler;
use crate::{ConnectionContext, Error, HandlerContext, Result, RtmpCommand, RtmpPacket};

pub struct DeleteStreamHandler;

impl DeleteStreamHandler {
    pub fn new() -> Self {
        DeleteStreamHandler
    }
}

#[async_trait::async_trait]
impl CommandHandler for DeleteStreamHandler {
    fn command_name(&self) -> &str {
        "deleteStream"
    }

    async fn handle(
        &self,
        command: RtmpCommand,
        context: Arc<ConnectionContext>,
    ) -> Result<Option<RtmpPacket>> {
        // Get stream ID from first argument
        let stream_id = command.arguments.first()
            .and_then(|v| v.as_number())
            .ok_or_else(|| Error::protocol("Missing stream ID"))?;

        // Get stream name from context
        let stream_name = context.get_property("stream_name").await
            .ok_or_else(|| Error::protocol("No stream name"))?;

        // Check if publishing
        let is_publishing = context.get_property("publishing").await
            .map(|v| v == "true")
            .unwrap_or(false);

        // Check if playing
        let is_playing = context.get_property("playing").await
            .map(|v| v == "true")
            .unwrap_or(false);

        // Cleanup based on state
        if is_publishing {
            if let Some(registry) = context.get_publisher_registry() {
                registry.unregister(&stream_name).await?;
            }
            context.remove_property("publishing").await;
            context.remove_property("publish_type").await;
        }

        if is_playing {
            if let Some(registry) = context.get_publisher_registry() {
                registry.decrement_subscribers(&stream_name).await?;
            }
            context.remove_property("playing").await;
            context.remove_property("play_start").await;
            context.remove_property("play_duration").await;
        }

        // Remove stream context
        context.remove_property("stream_name").await;
        context.remove_property("stream_id").await;

        // Send deleteStream success (no response expected by spec)
        Ok(None)
    }
}