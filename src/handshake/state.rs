use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandshakeState {
    /// Waiting for C0+C1 from client
    Uninitialized,

    /// Sent S0+S1+S2, waiting for C2
    SentS0S1S2,

    /// Received C2, handshake complete
    Done,

    /// Error occurred
    Failed,
}

impl HandshakeState {
    /// Initial state
    pub fn new() -> Self {
        HandshakeState::Uninitialized
    }

    /// Check if handshake is complete
    pub fn is_done(&self) -> bool {
        *self == HandshakeState::Done
    }

    /// Check if handshake failed
    pub fn is_failed(&self) -> bool {
        *self == HandshakeState::Failed
    }

    /// Transition to next state
    pub fn transition(&mut self, event: HandshakeEvent) -> Result<()> {
        match (*self, event) {
            (HandshakeState::Uninitialized, HandshakeEvent::ReceivedC0C1) => {
                *self = HandshakeState::SentS0S1S2;
                Ok(())
            }
            (HandshakeState::SentS0S1S2, HandshakeEvent::ReceivedC2) => {
                *self = HandshakeState::Done;
                Ok(())
            }
            (_, HandshakeEvent::Error) => {
                *self = HandshakeState::Failed;
                Err(Error::handshake("Handshake failed"))
            }
            _ => {
                Err(Error::handshake(format!(
                    "Invalid transition from {:?} with event {:?}",
                    self, event
                )))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HandshakeEvent {
    ReceivedC0C1,
    ReceivedC2,
    Error,
}

/// Handshake format type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HandshakeFormat {
    /// Simple handshake (format 0) - random data
    Simple,

    /// Format 1 - with digest
    Format1,

    /// Format 2 - with digest at different position
    Format2,
}