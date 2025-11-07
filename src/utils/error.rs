use std::fmt;
use std::error::Error as StdError;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] IoError),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Handshake error: {0}")]
    Handshake(String),

    #[error("AMF decode error: {0}")]
    AmfDecode(String),

    #[error("AMF encode error: {0}")]
    AmfEncode(String),

    #[error("Chunk error: {0}")]
    Chunk(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl Error {
    /// Create an IO error from message
    pub fn io(msg: impl Into<String>) -> Self {
        Error::Io(std::io::Error::new(std::io::ErrorKind::Other, msg.into()))
    }
    /// Create a protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        Error::Protocol(msg.into())
    }

    /// Create a handshake error
    pub fn handshake(msg: impl Into<String>) -> Self {
        Error::Handshake(msg.into())
    }

    /// Create an AMF decode error
    pub fn amf_decode(msg: impl Into<String>) -> Self {
        Error::AmfDecode(msg.into())
    }

    /// Create an AMF encode error
    pub fn amf_encode(msg: impl Into<String>) -> Self {
        Error::AmfEncode(msg.into())
    }

    /// Create a chunk error
    pub fn chunk(msg: impl Into<String>) -> Self {
        Error::Chunk(msg.into())
    }

    /// Create a connection error
    pub fn connection(msg: impl Into<String>) -> Self {
        Error::Connection(msg.into())
    }

    /// Create a stream error
    pub fn stream(msg: impl Into<String>) -> Self {
        Error::Stream(msg.into())
    }

    /// Create an invalid state error
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Error::InvalidState(msg.into())
    }

    /// Create a not implemented error
    pub fn not_implemented(msg: impl Into<String>) -> Self {
        Error::NotImplemented(msg.into())
    }

    /// Create a timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        Error::Timeout(msg.into())
    }

    /// Create an authentication error
    pub fn auth_failed(msg: impl Into<String>) -> Self {
        Error::AuthenticationFailed(msg.into())
    }

    /// Create a configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Error::Configuration(msg.into())
    }

    /// Create an unknown error
    pub fn unknown(msg: impl Into<String>) -> Self {
        Error::Unknown(msg.into())
    }
}

/// Result type alias for the library
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::protocol("Invalid message type");
        assert_eq!(format!("{}", err), "Protocol error: Invalid message type");

        let err = Error::handshake("Version mismatch");
        assert_eq!(format!("{}", err), "Handshake error: Version mismatch");
    }

    #[test]
    fn test_error_conversion() {
        use std::io::{Error as IoError, ErrorKind};

        let io_err = IoError::new(ErrorKind::UnexpectedEof, "EOF");
        let err: Error = io_err.into();

        match err {
            Error::Io(_) => assert!(true),
            _ => assert!(false, "Expected IO error variant"),
        }
    }
}