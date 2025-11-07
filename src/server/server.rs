use crate::{Error, Result};
use crate::connection::Connection;
use crate::message::MessageDispatcher;
use tokio::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::server::config::ServerConfig;
use crate::server::context::ServerContext;

pub struct RtmpServer {
    /// Server configuration
    config: Arc<ServerConfig>,

    /// Server context
    context: Arc<ServerContext>,

    /// Active connections
    connections: Arc<RwLock<HashMap<String, Arc<Connection>>>>,

    /// Message dispatcher template
    dispatcher: Arc<MessageDispatcher>,

    /// Shutdown flag
    shutdown: Arc<RwLock<bool>>,
}

impl RtmpServer {
    /// Create new server
    pub fn new(config: ServerConfig) -> Self {
        let config = Arc::new(config);
        let context = Arc::new(ServerContext::new(config.clone()));
        let dispatcher = Arc::new(MessageDispatcher::new());

        RtmpServer {
            config,
            context,
            connections: Arc::new(RwLock::new(HashMap::new())),
            dispatcher,
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// Get server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Get server context
    pub fn context(&self) -> Arc<ServerContext> {
        self.context.clone()
    }

    /// Listen and accept connections
    pub async fn listen(&self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await
            .map_err(|e| Error::connection(format!("Failed to bind {}: {}", addr, e)))?;

        println!("RTMP Server listening on {}", addr);

        // Accept loop
        loop {
            // Check shutdown
            if *self.shutdown.read().await {
                break;
            }

            // Accept connection
            let (stream, peer_addr) = match listener.accept().await {
                Ok((s, a)) => (s, a),
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                    continue;
                }
            };

            println!("New connection from {}", peer_addr);

            // Check connection limit
            if self.connections.read().await.len() >= self.config.max_connections {
                eprintln!("Connection limit reached, rejecting {}", peer_addr);
                drop(stream);
                continue;
            }

            // Check IP limits
            let ip = peer_addr.ip();
            if !self.context.can_accept_from_ip(ip).await {
                eprintln!("IP limit reached for {}, rejecting", ip);
                drop(stream);
                continue;
            }

            // Handle connection
            self.handle_connection(stream, peer_addr.to_string()).await;
        }

        println!("Server stopped");
        Ok(())
    }

    /// Handle new connection
    async fn handle_connection(&self, stream: TcpStream, peer_addr: String) {
        // Configure TCP
        if let Err(e) = stream.set_nodelay(true) {
            eprintln!("Failed to set TCP_NODELAY: {}", e);
        }

        // Generate connection ID
        let conn_id = self.context.generate_connection_id();

        // Create connection context
        let (packet_tx, packet_rx) = tokio::sync::mpsc::channel(100);
        let conn_context = Arc::new(crate::connection::ConnectionContext::new(
            conn_id.clone(),
            packet_tx,
        ));

        // Create connection
        let connection = Arc::new(Connection::new(
            conn_id.clone(),
            conn_context,
            self.dispatcher.clone(),
        ));

        // Store connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(conn_id.clone(), connection.clone());
        }

        // Increment IP counter
        let ip = peer_addr.parse::<std::net::SocketAddr>()
            .map(|a| a.ip())
            .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
        self.context.increment_ip_count(ip).await;

        // Process connection
        let connections = self.connections.clone();
        let context = self.context.clone();
        let conn_id_clone = conn_id.clone();

        tokio::spawn(async move {
            // Process connection
            if let Err(e) = connection.process_server(stream).await {
                eprintln!("Connection {} error: {}", conn_id_clone, e);
            }

            // Remove connection
            connections.write().await.remove(&conn_id_clone);

            // Decrement IP counter
            context.decrement_ip_count(ip).await;

            println!("Connection {} closed", conn_id_clone);
        });
    }

    /// Shutdown server
    pub async fn shutdown(&self) {
        println!("Shutting down server...");

        // Set shutdown flag
        *self.shutdown.write().await = true;

        // Close all connections
        let connections = self.connections.read().await;
        for (id, conn) in connections.iter() {
            println!("Closing connection {}", id);
            if let Err(e) = conn.close().await {
                eprintln!("Error closing connection {}: {}", id, e);
            }
        }
    }

    /// Get active connections count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }
}