use crate::{Error, Result};
use crate::amf::{Amf0Value, Amf0Encoder, Amf0Decoder};
use crate::ByteBuffer;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RtmpCommand {
    pub name: String,
    pub transaction_id: f64,
    pub command_object: Option<Amf0Value>,
    pub arguments: Vec<Amf0Value>,
}

impl RtmpCommand {
    /// Create new command
    pub fn new(name: String, transaction_id: f64) -> Self {
        RtmpCommand {
            name,
            transaction_id,
            command_object: None,
            arguments: Vec::new(),
        }
    }

    /// Create connect command
    pub fn connect(app: &str, tc_url: &str) -> Self {
        let mut cmd = RtmpCommand::new("connect".to_string(), 1.0);

        let mut obj = HashMap::new();
        obj.insert("app".to_string(), Amf0Value::String(app.to_string()));
        obj.insert("type".to_string(), Amf0Value::String("nonprivate".to_string()));
        obj.insert("flashVer".to_string(), Amf0Value::String("FMLE/3.0".to_string()));
        obj.insert("tcUrl".to_string(), Amf0Value::String(tc_url.to_string()));

        cmd.command_object = Some(Amf0Value::Object(obj));
        cmd
    }

    /// Create createStream command
    pub fn create_stream(transaction_id: f64) -> Self {
        let mut cmd = RtmpCommand::new("createStream".to_string(), transaction_id);
        cmd.command_object = Some(Amf0Value::Null);
        cmd
    }

    /// Create publish command
    pub fn publish(stream_name: &str, publish_type: &str) -> Self {
        let mut cmd = RtmpCommand::new("publish".to_string(), 0.0);
        cmd.command_object = Some(Amf0Value::Null);
        cmd.arguments.push(Amf0Value::String(stream_name.to_string()));
        cmd.arguments.push(Amf0Value::String(publish_type.to_string()));
        cmd
    }

    /// Create play command
    pub fn play(stream_name: &str, start: f64, duration: f64, reset: bool) -> Self {
        let mut cmd = RtmpCommand::new("play".to_string(), 0.0);
        cmd.command_object = Some(Amf0Value::Null);
        cmd.arguments.push(Amf0Value::String(stream_name.to_string()));
        cmd.arguments.push(Amf0Value::Number(start));
        cmd.arguments.push(Amf0Value::Number(duration));
        cmd.arguments.push(Amf0Value::Boolean(reset));
        cmd
    }

    /// Create result response
    pub fn result(transaction_id: f64, result: Amf0Value) -> Self {
        let mut cmd = RtmpCommand::new("_result".to_string(), transaction_id);
        cmd.command_object = Some(Amf0Value::Null);
        cmd.arguments.push(result);
        cmd
    }

    /// Create error response
    pub fn error(transaction_id: f64, error_obj: Amf0Value) -> Self {
        let mut cmd = RtmpCommand::new("_error".to_string(), transaction_id);
        cmd.command_object = Some(Amf0Value::Null);
        cmd.arguments.push(error_obj);
        cmd
    }

    /// Create onStatus response
    pub fn on_status(level: &str, code: &str, description: &str) -> Self {
        let mut cmd = RtmpCommand::new("onStatus".to_string(), 0.0);
        cmd.command_object = Some(Amf0Value::Null);

        let mut info = HashMap::new();
        info.insert("level".to_string(), Amf0Value::String(level.to_string()));
        info.insert("code".to_string(), Amf0Value::String(code.to_string()));
        info.insert("description".to_string(), Amf0Value::String(description.to_string()));

        cmd.arguments.push(Amf0Value::Object(info));
        cmd
    }

    /// Encode command to bytes
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut encoder = Amf0Encoder::new();

        // Encode command name
        encoder.encode(&Amf0Value::String(self.name.clone()))?;

        // Encode transaction ID
        encoder.encode(&Amf0Value::Number(self.transaction_id))?;

        // Encode command object
        if let Some(ref obj) = self.command_object {
            encoder.encode(obj)?;
        } else {
            encoder.encode(&Amf0Value::Null)?;
        }

        // Encode arguments
        for arg in &self.arguments {
            encoder.encode(arg)?;
        }

        Ok(encoder.get_bytes())
    }

    /// Decode command from bytes
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut buffer = ByteBuffer::new(data.to_vec());
        let mut decoder = Amf0Decoder::new(&mut buffer);

        // Decode command name
        let name_val = decoder.decode()?;
        let name = name_val.as_string()
            .ok_or_else(|| Error::amf_decode("Command name must be string"))?
            .to_string();

        // Decode transaction ID
        let tid_val = decoder.decode()?;
        let transaction_id = tid_val.as_number()
            .ok_or_else(|| Error::amf_decode("Transaction ID must be number"))?;

        // Decode command object
        let command_object = if decoder.has_remaining() {
            Some(decoder.decode()?)
        } else {
            None
        };

        // Decode remaining arguments
        let mut arguments = Vec::new();
        while decoder.has_remaining() {
            arguments.push(decoder.decode()?);
        }

        Ok(RtmpCommand {
            name,
            transaction_id,
            command_object,
            arguments,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_command() {
        let cmd = RtmpCommand::connect("live", "rtmp://localhost/live");
        assert_eq!(cmd.name, "connect");
        assert_eq!(cmd.transaction_id, 1.0);

        let obj = cmd.command_object.unwrap();
        assert_eq!(obj.get_property("app").and_then(|v| v.as_string()), Some("live"));
    }

    #[test]
    fn test_command_round_trip() {
        let original = RtmpCommand::create_stream(2.0);
        let bytes = original.encode().unwrap();
        let decoded = RtmpCommand::decode(&bytes).unwrap();

        assert_eq!(original.name, decoded.name);
        assert_eq!(original.transaction_id, decoded.transaction_id);
    }
}