mod client;
mod config;
mod state;

pub use client::RtmpClient;
pub use config::{ClientConfig, ClientConfigBuilder};

use tokio::net::TcpStream;
use tokio::time::timeout;
use std::time::Duration;
use crate::{Error, Result};

pub async fn connect_to_server(url: &str, connect_timeout: Duration) -> Result<TcpStream> {
    // Parse URL
    let url = url::Url::parse(url)
        .map_err(|e| Error::config(format!("Invalid URL: {}", e)))?;

    let host = url.host_str()
        .ok_or_else(|| Error::config("Missing host"))?;
    let port = url.port().unwrap_or(1935);

    let addr = format!("{}:{}", host, port);

    // Connect with timeout
    match timeout(connect_timeout, TcpStream::connect(&addr)).await {
        Ok(Ok(stream)) => Ok(stream),
        Ok(Err(e)) => Err(Error::connection(format!("Connection failed: {}", e))),
        Err(_) => Err(Error::timeout("Connection timeout")),
    }
}