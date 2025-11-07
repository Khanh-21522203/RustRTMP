mod connect;
mod create_stream;
mod publish;
mod play;
mod delete_stream;

use std::collections::HashMap;
use crate::{Amf0Value, Error, Result};
use crate::protocol::{RtmpCommand, RtmpPacket};
use crate::connection::ConnectionContext;
use std::sync::Arc;
use crate::handlers::connect::ConnectHandler;
use crate::handlers::create_stream::CreateStreamHandler;
use crate::handlers::delete_stream::DeleteStreamHandler;
use crate::handlers::play::PlayHandler;
use crate::handlers::publish::PublishHandler;

#[async_trait::async_trait]
pub trait CommandHandler: Send + Sync {
    /// Get command name this handler processes
    fn command_name(&self) -> &str;

    /// Handle the command
    async fn handle(
        &self,
        command: RtmpCommand,
        context: Arc<ConnectionContext>,
    ) -> Result<Option<RtmpPacket>>;

    /// Check if can handle command
    fn can_handle(&self, command_name: &str) -> bool {
        self.command_name() == command_name
    }
}

/// Command handler registry
pub struct CommandHandlerRegistry {
    handlers: HashMap<String, Arc<dyn CommandHandler>>,
}

impl CommandHandlerRegistry {
    pub fn new() -> Self {
        let mut registry = CommandHandlerRegistry {
            handlers: HashMap::new(),
        };

        // Register default handlers
        registry.register(Arc::new(ConnectHandler::new()));
        registry.register(Arc::new(CreateStreamHandler::new()));
        registry.register(Arc::new(PublishHandler::new()));
        registry.register(Arc::new(PlayHandler::new()));
        registry.register(Arc::new(DeleteStreamHandler::new()));

        registry
    }

    pub fn register(&mut self, handler: Arc<dyn CommandHandler>) {
        self.handlers.insert(
            handler.command_name().to_string(),
            handler,
        );
    }

    pub async fn handle(
        &self,
        command: RtmpCommand,
        context: Arc<ConnectionContext>,
    ) -> Result<Option<RtmpPacket>> {
        if let Some(handler) = self.handlers.get(&command.name) {
            handler.handle(command, context).await
        } else {
            Err(Error::protocol(format!("Unknown command: {}", command.name)))
        }
    }
}

pub fn validate_connect_params(params: &Amf0Value) -> Result<()> {
    let obj = params.as_object()
        .ok_or_else(|| Error::protocol("Connect params must be object"))?;

    // Required fields
    if !obj.contains_key("app") {
        return Err(Error::protocol("Missing 'app' parameter"));
    }

    if !obj.contains_key("tcUrl") {
        return Err(Error::protocol("Missing 'tcUrl' parameter"));
    }

    Ok(())
}

pub fn generate_connect_response(success: bool, transaction_id: f64) -> RtmpCommand {
    if success {
        let mut props = HashMap::new();
        props.insert("fmsVer".to_string(), Amf0Value::String("FMS/3,5,5,2004".to_string()));
        props.insert("capabilities".to_string(), Amf0Value::Number(31.0));

        let mut info = HashMap::new();
        info.insert("level".to_string(), Amf0Value::String("status".to_string()));
        info.insert("code".to_string(), Amf0Value::String("NetConnection.Connect.Success".to_string()));
        info.insert("description".to_string(), Amf0Value::String("Connection succeeded".to_string()));

        let mut cmd = RtmpCommand::result(transaction_id, Amf0Value::Object(props));
        cmd.arguments.push(Amf0Value::Object(info));
        cmd
    } else {
        let mut error = HashMap::new();
        error.insert("level".to_string(), Amf0Value::String("error".to_string()));
        error.insert("code".to_string(), Amf0Value::String("NetConnection.Connect.Rejected".to_string()));
        error.insert("description".to_string(), Amf0Value::String("Connection rejected".to_string()));

        RtmpCommand::error(transaction_id, Amf0Value::Object(error))
    }
}