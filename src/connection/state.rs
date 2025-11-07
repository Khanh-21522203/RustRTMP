#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionState {
    /// Not initialized
    Uninitialized,

    /// Performing handshake
    Handshaking,

    /// Handshake complete, connected
    Connected,

    /// Publishing stream
    Publishing,

    /// Playing stream
    Playing,

    /// Connection closing
    Closing,

    /// Connection closed
    Closed,
}

impl ConnectionState {
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        matches!(self, 
            ConnectionState::Connected | 
            ConnectionState::Publishing | 
            ConnectionState::Playing)
    }

    /// Check if can publish
    pub fn can_publish(&self) -> bool {
        *self == ConnectionState::Connected
    }

    /// Check if can play
    pub fn can_play(&self) -> bool {
        matches!(self, ConnectionState::Connected | ConnectionState::Playing)
    }

    /// Validate transition
    pub fn can_transition_to(&self, next: ConnectionState) -> bool {
        match (*self, next) {
            (ConnectionState::Uninitialized, ConnectionState::Handshaking) => true,
            (ConnectionState::Handshaking, ConnectionState::Connected) => true,
            (ConnectionState::Connected, ConnectionState::Publishing) => true,
            (ConnectionState::Connected, ConnectionState::Playing) => true,
            (ConnectionState::Publishing, ConnectionState::Connected) => true,
            (ConnectionState::Playing, ConnectionState::Connected) => true,
            (_, ConnectionState::Closing) => true,
            (ConnectionState::Closing, ConnectionState::Closed) => true,
            _ => false,
        }
    }
}