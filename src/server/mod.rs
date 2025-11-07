use tokio::net::TcpListener;
use crate::{Error, Result};

mod server;
mod config;
mod context;
mod registry;

pub use server::RtmpServer;
pub use config::{ServerConfig, ServerConfigBuilder};
pub use context::ServerContext;
pub use registry::*;


pub async fn bind_server(config: &config::ServerConfig) -> Result<TcpListener> {
    let addr = format!("{}:{}", config.host, config.port);

    // Try binding with SO_REUSEADDR
    let socket = match &addr.parse::<std::net::SocketAddr>() {
        Ok(addr) => {
            let socket = if addr.is_ipv4() {
                tokio::net::TcpSocket::new_v4()?
            } else {
                tokio::net::TcpSocket::new_v6()?
            };

            socket.set_reuseaddr(true)?;
            socket.bind(*addr)?;
            socket
        }
        Err(e) => {
            return Err(Error::config(format!("Invalid address {}: {}", addr, e)));
        }
    };

    let listener = socket.listen(1024)?;
    Ok(listener)
}