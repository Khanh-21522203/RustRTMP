mod utils;
mod amf;
mod protocol;
mod handshake;
mod chunk;
mod message;
mod connection;
mod server;
mod client;
mod handlers;
mod stream;
mod processing;

// Re-export commonly used types at crate root
pub use utils::*;
pub use amf::*;
pub use protocol::*;
pub use message::*;
pub use connection::*;
pub use chunk::*;
pub use handshake::*;

// Server exports
pub use server::{RtmpServer, ServerConfig, ServerContext, PublisherRegistry, PublisherInfo};

// Client exports
pub use client::{RtmpClient, ClientConfig};

// Stream exports
pub use stream::*;

// Processing exports
pub use processing::*;
