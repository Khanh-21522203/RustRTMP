use crate::{Error, Result};
use crate::handshake::{HandshakeState, C0C1, S0S1S2, validate_c0c1, generate_s0s1s2, validate_c2};
use crate::chunk::{ChunkReader, ChunkWriter};
use crate::message::{MessageDispatcher, MessageQueue};
use crate::protocol::RtmpPacket;
use tokio::net::TcpStream;
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use crate::connection::context::ConnectionContext;
use crate::connection::state::ConnectionState;
use crate::connection::stream_manager::StreamManager;

pub struct Connection {
    /// Connection ID
    id: String,

    /// Connection state
    state: Arc<RwLock<ConnectionState>>,

    /// Connection context
    context: Arc<ConnectionContext>,

    /// Chunk reader
    chunk_reader: Arc<RwLock<ChunkReader>>,

    /// Chunk writer
    chunk_writer: Arc<RwLock<ChunkWriter>>,

    /// Message dispatcher
    dispatcher: Arc<MessageDispatcher>,

    /// Message queue
    message_queue: Arc<MessageQueue>,

    /// Stream manager
    stream_manager: Arc<RwLock<StreamManager>>,

    /// Shutdown signal
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: Arc<RwLock<mpsc::Receiver<()>>>,
}

impl Connection {
    /// Create new connection
    pub fn new(
        id: String,
        context: Arc<ConnectionContext>,
        dispatcher: Arc<MessageDispatcher>,
    ) -> Self {
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        Connection {
            id,
            state: Arc::new(RwLock::new(ConnectionState::Uninitialized)),
            context,
            chunk_reader: Arc::new(RwLock::new(ChunkReader::new())),
            chunk_writer: Arc::new(RwLock::new(ChunkWriter::new())),
            dispatcher,
            message_queue: Arc::new(MessageQueue::new(1000)),
            stream_manager: Arc::new(RwLock::new(StreamManager::new())),
            shutdown_tx,
            shutdown_rx: Arc::new(RwLock::new(shutdown_rx)),
        }
    }

    /// Get connection ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get current state
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Process server connection
    pub async fn process_server<S>(&self, stream: S) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let (read_half, write_half) = tokio::io::split(stream);

        // Perform handshake
        let (read_half, write_half) = self.server_handshake(read_half, write_half).await?;

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connected;
        }

        // Start processing loops
        let read_handle = self.start_read_loop(read_half);
        let write_handle = self.start_write_loop(write_half);
        let process_handle = self.start_process_loop();

        // Wait for shutdown or error
        tokio::select! {
            result = read_handle => {
                if let Err(e) = result {
                    eprintln!("Read loop error: {}", e);
                }
            }
            result = write_handle => {
                if let Err(e) = result {
                    eprintln!("Write loop error: {}", e);
                }
            }
            result = process_handle => {
                if let Err(e) = result {
                    eprintln!("Process loop error: {}", e);
                }
            }
            _ = self.wait_shutdown() => {
                println!("Connection {} shutting down", self.id);
            }
        }

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Closed;
        }

        Ok(())
    }

    /// Process client connection (no handshake needed - done by RtmpClient)
    pub async fn process_client<S>(&self, stream: S) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let (read_half, write_half) = tokio::io::split(stream);

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Connected;
        }

        // Start processing loops (no handshake - client already did it)
        let read_handle = self.start_read_loop(read_half);
        let write_handle = self.start_write_loop(write_half);
        let process_handle = self.start_process_loop();

        // Wait for shutdown or error
        tokio::select! {
            result = read_handle => {
                if let Err(e) = result {
                    eprintln!("Read loop error: {}", e);
                }
            }
            result = write_handle => {
                if let Err(e) = result {
                    eprintln!("Write loop error: {}", e);
                }
            }
            result = process_handle => {
                if let Err(e) = result {
                    eprintln!("Process loop error: {}", e);
                }
            }
            _ = self.wait_shutdown() => {
                println!("Connection {} shutting down", self.id);
            }
        }

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ConnectionState::Closed;
        }

        Ok(())
    }

    /// Perform server handshake
    async fn server_handshake<R, W>(&self, mut reader: R, mut writer: W) -> Result<(R, W)>
    where
        R: AsyncRead + Unpin,
        W: AsyncWrite + Unpin,
    {
        let mut handshake_state = HandshakeState::new();

        // Read C0+C1
        let mut c0c1_buf = vec![0u8; 1537];
        reader.read_exact(&mut c0c1_buf).await
            .map_err(|e| Error::handshake(format!("Failed to read C0+C1: {}", e)))?;

        let c0c1 = validate_c0c1(&c0c1_buf)?;

        // Generate and send S0+S1+S2
        let s0s1s2_bytes = generate_s0s1s2(&c0c1)?;
        writer.write_all(&s0s1s2_bytes).await
            .map_err(|e| Error::handshake(format!("Failed to write S0+S1+S2: {}", e)))?;
        writer.flush().await
            .map_err(|e| Error::handshake(format!("Failed to flush: {}", e)))?;

        // For validation
        let s0s1s2 = S0S1S2::generate(&c0c1)?;

        handshake_state.transition(crate::handshake::HandshakeEvent::ReceivedC0C1)?;

        // Read C2
        let mut c2_buf = vec![0u8; 1536];
        reader.read_exact(&mut c2_buf).await
            .map_err(|e| Error::handshake(format!("Failed to read C2: {}", e)))?;

        validate_c2(&c2_buf, &s0s1s2)?;

        handshake_state.transition(crate::handshake::HandshakeEvent::ReceivedC2)?;

        Ok((reader, writer))
    }

    /// Start read loop
    fn start_read_loop<R>(&self, mut reader: R) -> tokio::task::JoinHandle<Result<()>>
    where
        R: AsyncRead + Unpin + Send + 'static,
    {
        let chunk_reader = self.chunk_reader.clone();
        let message_queue = self.message_queue.clone();
        let shutdown_rx = self.shutdown_rx.clone();

        tokio::spawn(async move {
            loop {
                // Check shutdown
                {
                    let mut rx = shutdown_rx.write().await;
                    if rx.try_recv().is_ok() {
                        break;
                    }
                }

                // Read chunk
                let packet = {
                    let mut reader_lock = chunk_reader.write().await;
                    reader_lock.read_chunk(&mut reader).await?
                };

                // Queue message if complete
                if let Some(packet) = packet {
                    message_queue.push(packet).await?;
                }
            }

            Ok(())
        })
    }

    /// Start write loop
    fn start_write_loop<W>(&self, mut writer: W) -> tokio::task::JoinHandle<Result<()>>
    where
        W: AsyncWrite + Unpin + Send + 'static,
    {
        // Simplified - real implementation would have outgoing queue
        tokio::spawn(async move {
            // Write loop implementation
            Ok(())
        })
    }

    /// Start process loop
    fn start_process_loop(&self) -> tokio::task::JoinHandle<Result<()>> {
        let dispatcher = self.dispatcher.clone();
        let message_queue = self.message_queue.clone();
        let context = self.context.clone();
        let shutdown_rx = self.shutdown_rx.clone();

        tokio::spawn(async move {
            loop {
                // Check shutdown
                {
                    let mut rx = shutdown_rx.write().await;
                    if rx.try_recv().is_ok() {
                        break;
                    }
                }

                // Process queued messages
                if let Some(packet) = message_queue.pop_timeout(
                    std::time::Duration::from_millis(100)
                ).await? {
                    dispatcher.dispatch(packet, context.clone()).await?;
                }
            }

            Ok(())
        })
    }

    /// Wait for shutdown signal
    async fn wait_shutdown(&self) {
        let mut rx = self.shutdown_rx.write().await;
        let _ = rx.recv().await;
    }

    /// Send packet
    pub async fn send_packet(&self, packet: RtmpPacket) -> Result<()> {
        // Implementation would write to outgoing queue
        Ok(())
    }

    /// Close connection
    pub async fn close(&self) -> Result<()> {
        // Send shutdown signal
        let _ = self.shutdown_tx.send(()).await;

        // Update state
        let mut state = self.state.write().await;
        *state = ConnectionState::Closed;

        Ok(())
    }
}