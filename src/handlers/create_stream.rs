use std::sync::Arc;
use crate::amf::Amf0Value;
use std::sync::atomic::{AtomicU32, Ordering};
use crate::handlers::CommandHandler;
use crate::{ConnectionContext, RtmpCommand, RtmpHeader, RtmpPacket, Result, HandlerContext};

pub struct CreateStreamHandler {
    next_stream_id: AtomicU32,
}

impl CreateStreamHandler {
    pub fn new() -> Self {
        CreateStreamHandler {
            next_stream_id: AtomicU32::new(1), // Start from 1, 0 is reserved
        }
    }

    fn allocate_stream_id(&self) -> u32 {
        self.next_stream_id.fetch_add(1, Ordering::SeqCst)
    }
}

#[async_trait::async_trait]
impl CommandHandler for CreateStreamHandler {
    fn command_name(&self) -> &str {
        "createStream"
    }

    async fn handle(
        &self,
        command: RtmpCommand,
        context: Arc<ConnectionContext>,
    ) -> Result<Option<RtmpPacket>> {
        // Allocate new stream ID
        let stream_id = self.allocate_stream_id();

        // Store stream ID in context
        context.set_property("stream_id".to_string(), stream_id.to_string()).await;

        // Create response
        let response = RtmpCommand::result(
            command.transaction_id,
            Amf0Value::Number(stream_id as f64),
        );

        let bytes = response.encode()?;
        let header = RtmpHeader::command(0, bytes.len() as u32, 0);

        Ok(Some(RtmpPacket::new(header, bytes)))
    }
}