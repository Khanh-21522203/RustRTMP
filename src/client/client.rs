use std::collections::HashMap;
use crate::{Error, Result};
use crate::connection::{Connection, ConnectionContext};
use crate::handshake::{C0C1, S0S1S2, C2};
use crate::protocol::{RtmpCommand, RtmpPacket, RtmpData};
use crate::message::MessageDispatcher;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use url::Url;
use crate::client::config::ClientConfig;
use crate::client::state::ClientState;

pub struct RtmpClient {
    /// Client configuration
    config: Arc<ClientConfig>,

    /// Client state
    state: Arc<RwLock<ClientState>>,

    /// Connection
    connection: Option<Arc<Connection>>,

    /// Server URL
    url: Option<Url>,

    /// App name
    app: Option<String>,

    /// Stream name
    stream_name: Option<String>,

    /// Stream ID
    stream_id: Arc<RwLock<Option<u32>>>,

    /// Transaction ID counter
    transaction_id: Arc<RwLock<f64>>,
}

impl RtmpClient {
    /// Create new client
    pub fn new() -> Self {
        RtmpClient::with_config(ClientConfig::default())
    }

    /// Create client with config
    pub fn with_config(config: ClientConfig) -> Self {
        RtmpClient {
            config: Arc::new(config),
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            connection: None,
            url: None,
            app: None,
            stream_name: None,
            stream_id: Arc::new(RwLock::new(None)),
            transaction_id: Arc::new(RwLock::new(1.0)),
        }
    }

    /// Connect to RTMP server
    pub async fn connect(&mut self, url: &str) -> Result<()> {
        // Parse URL
        let parsed_url = Url::parse(url)
            .map_err(|e| Error::config(format!("Invalid URL: {}", e)))?;

        // Validate scheme
        match parsed_url.scheme() {
            "rtmp" | "rtmps" => {},
            scheme => return Err(Error::config(format!("Unsupported scheme: {}", scheme))),
        }

        // Extract components
        let host = parsed_url.host_str()
            .ok_or_else(|| Error::config("Missing host in URL"))?;
        let port = parsed_url.port().unwrap_or(1935);
        let path = parsed_url.path().trim_start_matches('/');

        // Parse app and stream name
        let parts: Vec<&str> = path.split('/').collect();
        let app = parts.get(0).map(|s| s.to_string())
            .unwrap_or_else(|| "live".to_string());

        // Store URL and app
        self.url = Some(parsed_url.clone());
        self.app = Some(app.clone());

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ClientState::Connecting;
        }

        // Connect TCP
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr).await
            .map_err(|e| Error::connection(format!("Failed to connect to {}: {}", addr, e)))?;

        // Set TCP options
        stream.set_nodelay(true)?;

        // Perform client handshake
        let stream = self.client_handshake(stream).await?;

        // Create connection
        let (packet_tx, packet_rx) = mpsc::channel(100);
        let conn_context = Arc::new(ConnectionContext::new(
            "client".to_string(),
            packet_tx,
        ));

        let dispatcher = Arc::new(MessageDispatcher::new());
        // Register client handlers...

        let connection = Arc::new(Connection::new(
            "client".to_string(),
            conn_context,
            dispatcher,
        ));

        self.connection = Some(connection.clone());

        // Start connection processing
        let connection_clone = connection.clone();
        tokio::spawn(async move {
            if let Err(e) = connection_clone.process_client(stream).await {
                eprintln!("Client connection error: {}", e);
            }
        });

        // Send connect command
        self.send_connect(&app, url).await?;

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ClientState::Connected;
        }

        Ok(())
    }

    /// Perform client handshake
    async fn client_handshake(&self, stream: TcpStream) -> Result<TcpStream> {
        let (mut reader, mut writer) = stream.into_split();
        
        // Send C0+C1
        let c0c1 = C0C1::create_client();
        writer.write_all(&c0c1.encode()).await?;
        writer.flush().await?;

        // Read S0+S1+S2
        let mut s0s1s2_buf = vec![0u8; 3073];
        reader.read_exact(&mut s0s1s2_buf).await?;
        let s0s1s2 = S0S1S2::parse(&s0s1s2_buf)?;

        // Send C2
        let c2 = C2::create_from_s1(&s0s1s2);
        writer.write_all(&c2.encode()).await?;
        writer.flush().await?;

        // Reunite stream
        let stream = reader.reunite(writer)
            .map_err(|e| Error::io(format!("Failed to reunite stream: {}", e)))?;
        Ok(stream)
    }

    /// Send connect command
    async fn send_connect(&self, app: &str, tc_url: &str) -> Result<()> {
        let mut tid = self.transaction_id.write().await;
        let connect_cmd = RtmpCommand::connect(app, tc_url);
        *tid += 1.0;

        let connection = self.connection.as_ref()
            .ok_or_else(|| Error::invalid_state("Not connected"))?;

        let bytes = connect_cmd.encode()?;
        let header = crate::protocol::RtmpHeader::command(0, bytes.len() as u32, 0);
        let packet = RtmpPacket::new(header, bytes);

        connection.send_packet(packet).await?;
        Ok(())
    }

    /// Create stream for publishing/playing
    pub async fn create_stream(&self) -> Result<u32> {
        let connection = self.connection.as_ref()
            .ok_or_else(|| Error::invalid_state("Not connected"))?;

        let mut tid = self.transaction_id.write().await;
        let cmd = RtmpCommand::create_stream(*tid);
        *tid += 1.0;

        let bytes = cmd.encode()?;
        let header = crate::protocol::RtmpHeader::command(0, bytes.len() as u32, 0);
        let packet = RtmpPacket::new(header, bytes);

        connection.send_packet(packet).await?;

        // Wait for response (simplified - real impl would use futures)
        // For now, return placeholder
        let stream_id = 1;

        let mut sid = self.stream_id.write().await;
        *sid = Some(stream_id);

        Ok(stream_id)
    }

    /// Publish stream
    pub async fn publish(&mut self, stream_name: &str, publish_type: &str) -> Result<()> {
        // Ensure connected
        let state = *self.state.read().await;
        if state != ClientState::Connected {
            return Err(Error::invalid_state("Must be connected to publish"));
        }

        // Create stream if needed
        if self.stream_id.read().await.is_none() {
            self.create_stream().await?;
        }

        let stream_id = self.stream_id.read().await
            .ok_or_else(|| Error::invalid_state("No stream ID"))?;

        // Send publish command
        let publish_cmd = RtmpCommand::publish(stream_name, publish_type);
        let bytes = publish_cmd.encode()?;
        let header = crate::protocol::RtmpHeader::command(0, bytes.len() as u32, stream_id);
        let packet = RtmpPacket::new(header, bytes);

        let connection = self.connection.as_ref()
            .ok_or_else(|| Error::invalid_state("Not connected"))?;

        connection.send_packet(packet).await?;

        // Update state
        self.stream_name = Some(stream_name.to_string());
        let mut state = self.state.write().await;
        *state = ClientState::Publishing;

        Ok(())
    }

    /// Play stream
    pub async fn play(&mut self, stream_name: &str, start: f64, duration: f64, reset: bool) -> Result<()> {
        // Ensure connected
        let state = *self.state.read().await;
        if state != ClientState::Connected {
            return Err(Error::invalid_state("Must be connected to play"));
        }

        // Create stream if needed
        if self.stream_id.read().await.is_none() {
            self.create_stream().await?;
        }

        let stream_id = self.stream_id.read().await
            .ok_or_else(|| Error::invalid_state("No stream ID"))?;

        // Send play command
        let play_cmd = RtmpCommand::play(stream_name, start, duration, reset);
        let bytes = play_cmd.encode()?;
        let header = crate::protocol::RtmpHeader::command(0, bytes.len() as u32, stream_id);
        let packet = RtmpPacket::new(header, bytes);

        let connection = self.connection.as_ref()
            .ok_or_else(|| Error::invalid_state("Not connected"))?;

        connection.send_packet(packet).await?;

        // Update state
        self.stream_name = Some(stream_name.to_string());
        let mut state = self.state.write().await;
        *state = ClientState::Playing;

        Ok(())
    }

    /// Send audio data
    pub async fn send_audio(&self, data: Vec<u8>, timestamp: u32) -> Result<()> {
        let state = *self.state.read().await;
        if state != ClientState::Publishing {
            return Err(Error::invalid_state("Not publishing"));
        }

        let stream_id = self.stream_id.read().await
            .ok_or_else(|| Error::invalid_state("No stream ID"))?;

        let packet = crate::protocol::make_audio_packet(data, timestamp, stream_id);

        let connection = self.connection.as_ref()
            .ok_or_else(|| Error::invalid_state("Not connected"))?;

        connection.send_packet(packet).await
    }

    /// Send video data
    pub async fn send_video(&self, data: Vec<u8>, timestamp: u32) -> Result<()> {
        let state = *self.state.read().await;
        if state != ClientState::Publishing {
            return Err(Error::invalid_state("Not publishing"));
        }

        let stream_id = self.stream_id.read().await
            .ok_or_else(|| Error::invalid_state("No stream ID"))?;

        let packet = crate::protocol::make_video_packet(data, timestamp, stream_id);

        let connection = self.connection.as_ref()
            .ok_or_else(|| Error::invalid_state("Not connected"))?;

        connection.send_packet(packet).await
    }

    /// Send metadata
    pub async fn send_metadata(&self, metadata: HashMap<String, crate::amf::Amf0Value>) -> Result<()> {
        let state = *self.state.read().await;
        if state != ClientState::Publishing {
            return Err(Error::invalid_state("Not publishing"));
        }

        let stream_id = self.stream_id.read().await
            .ok_or_else(|| Error::invalid_state("No stream ID"))?;

        let data_msg = RtmpData::on_metadata(metadata);
        let bytes = data_msg.encode()?;
        let header = crate::protocol::RtmpHeader::data(0, bytes.len() as u32, stream_id);
        let packet = RtmpPacket::new(header, bytes);

        let connection = self.connection.as_ref()
            .ok_or_else(|| Error::invalid_state("Not connected"))?;

        connection.send_packet(packet).await
    }

    /// Disconnect from server
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(connection) = &self.connection {
            connection.close().await?;
        }

        self.connection = None;
        self.stream_id = Arc::new(RwLock::new(None));
        self.stream_name = None;

        let mut state = self.state.write().await;
        *state = ClientState::Disconnected;

        Ok(())
    }

    /// Get current state
    pub async fn state(&self) -> ClientState {
        *self.state.read().await
    }
}