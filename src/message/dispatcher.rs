use crate::{Error, PublisherRegistry, Result, MSG_TYPE_COMMAND_AMF0, MSG_TYPE_COMMAND_AMF3};
use crate::protocol::{RtmpPacket, RtmpCommand, RtmpData};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Message handler trait
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&self, packet: RtmpPacket, context: Arc<dyn HandlerContext>) -> Result<()>;
}

/// Handler context provided to message handlers
#[async_trait::async_trait]
pub trait HandlerContext: Send + Sync {
    async fn send_packet(&self, packet: RtmpPacket) -> Result<()>;
    async fn get_property(&self, key: &str) -> Option<String>;
    async fn set_property(&self, key: String, value: String);
    async fn remove_property(&self, key: &str);
    fn get_publisher_registry(&self) -> Option<Arc<PublisherRegistry>>;
}

/// Type-erased handler
type Handler = Arc<dyn MessageHandler>;

pub struct MessageDispatcher {
    /// Handlers by message type
    handlers: Arc<RwLock<HashMap<u8, Vec<Handler>>>>,

    /// Command handlers by name
    command_handlers: Arc<RwLock<HashMap<String, Handler>>>,

    /// Default handler for unhandled messages
    default_handler: Option<Handler>,
}

impl MessageDispatcher {
    /// Create new dispatcher
    pub fn new() -> Self {
        MessageDispatcher {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            command_handlers: Arc::new(RwLock::new(HashMap::new())),
            default_handler: None,
        }
    }

    /// Register handler for message type
    pub async fn register_handler(&self, message_type: u8, handler: Handler) {
        let mut handlers = self.handlers.write().await;
        handlers.entry(message_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    /// Register command handler
    pub async fn register_command(&self, command: String, handler: Handler) {
        let mut handlers = self.command_handlers.write().await;
        handlers.insert(command, handler);
    }

    /// Set default handler
    pub fn set_default_handler(&mut self, handler: Handler) {
        self.default_handler = Some(handler);
    }

    /// Dispatch message to handlers
    pub async fn dispatch(
        &self,
        packet: RtmpPacket,
        context: Arc<dyn HandlerContext>
    ) -> Result<()> {
        let message_type = packet.message_type();

        // Special handling for command messages
        if message_type == MSG_TYPE_COMMAND_AMF0 || message_type == MSG_TYPE_COMMAND_AMF3 {
            return self.dispatch_command(packet, context).await;
        }

        // Get handlers for message type
        let handlers = self.handlers.read().await;
        if let Some(type_handlers) = handlers.get(&message_type) {
            for handler in type_handlers {
                handler.handle(packet.clone(), context.clone()).await?;
            }
            return Ok(());
        }

        // Use default handler if available
        if let Some(ref handler) = self.default_handler {
            return handler.handle(packet, context).await;
        }

        // No handler found
        Err(Error::protocol(format!(
            "No handler for message type: {}",
            message_type
        )))
    }

    /// Dispatch command message
    async fn dispatch_command(
        &self,
        packet: RtmpPacket,
        context: Arc<dyn HandlerContext>
    ) -> Result<()> {
        // Decode command
        let command = RtmpCommand::decode(&packet.payload)?;

        // Find handler for command
        let handlers = self.command_handlers.read().await;
        if let Some(handler) = handlers.get(&command.name) {
            return handler.handle(packet, context).await;
        }

        // Check for generic command handler
        let type_handlers = self.handlers.read().await;
        if let Some(handlers) = type_handlers.get(&packet.message_type()) {
            for handler in handlers {
                handler.handle(packet.clone(), context.clone()).await?;
            }
            return Ok(());
        }

        // No handler found
        Err(Error::protocol(format!(
            "No handler for command: {}",
            command.name
        )))
    }
}

/// Example handler implementation
pub struct LoggingHandler;

#[async_trait::async_trait]
impl MessageHandler for LoggingHandler {
    async fn handle(&self, packet: RtmpPacket, _context: Arc<dyn HandlerContext>) -> Result<()> {
        println!(
            "Received packet: type={}, stream_id={}, timestamp={}, size={}",
            packet.message_type(),
            packet.message_stream_id(),
            packet.timestamp(),
            packet.payload.len()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::MSG_TYPE_AUDIO;
    use super::*;

    struct MockContext;

    #[async_trait::async_trait]
    impl HandlerContext for MockContext {
        async fn send_packet(&self, _packet: RtmpPacket) -> Result<()> {
            Ok(())
        }
        async fn get_property(&self, _key: &str) -> Option<String> {
            None
        }
        async fn set_property(&self, _key: String, _value: String) {}
        async fn remove_property(&self, _key: &str) {}
        fn get_publisher_registry(&self) -> Option<Arc<PublisherRegistry>> {
            None
        }
    }

    #[tokio::test]
    async fn test_dispatcher() {
        let dispatcher = MessageDispatcher::new();
        let handler = Arc::new(LoggingHandler);

        dispatcher.register_handler(MSG_TYPE_AUDIO, handler.clone()).await;
        dispatcher.register_command("connect".to_string(), handler).await;

        let context = Arc::new(MockContext);

        // Test audio packet dispatch
        let audio_packet = crate::protocol::make_audio_packet(vec![1, 2, 3], 1000, 1);
        assert!(dispatcher.dispatch(audio_packet, context.clone()).await.is_ok());
    }
}