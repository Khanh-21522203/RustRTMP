#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientState {
    /// Not connected
    Disconnected,

    /// Connecting to server
    Connecting,

    /// Connected and ready
    Connected,

    /// Publishing stream
    Publishing,

    /// Playing stream
    Playing,

    /// Connection error
    Error,
}

impl ClientState {
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        matches!(self,
            ClientState::Connected |
            ClientState::Publishing |
            ClientState::Playing)
    }

    /// Check if can publish
    pub fn can_publish(&self) -> bool {
        *self == ClientState::Connected
    }

    /// Check if can play
    pub fn can_play(&self) -> bool {
        *self == ClientState::Connected
    }

    /// Check if disconnected
    pub fn is_disconnected(&self) -> bool {
        matches!(self,
            ClientState::Disconnected |
            ClientState::Error)
    }
}