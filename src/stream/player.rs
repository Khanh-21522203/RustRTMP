use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use crate::{RtmpPacket, Result};
use crate::stream::publisher::Publisher;
use crate::stream::stream::Stream;

pub struct Player {
    /// Base stream
    stream: Arc<Stream>,

    /// Stream name playing
    stream_name: String,

    /// Packet receiver
    receiver: Option<mpsc::Receiver<RtmpPacket>>,

    /// Playback state
    state: Arc<RwLock<PlaybackState>>,

    /// Buffer time in ms
    buffer_time: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackState {
    Idle,
    Buffering,
    Playing,
    Paused,
    Stopped,
}

impl Player {
    /// Create new player
    pub fn new(stream: Arc<Stream>, stream_name: String, buffer_time: u32) -> Self {
        Player {
            stream,
            stream_name,
            receiver: None,
            state: Arc::new(RwLock::new(PlaybackState::Idle)),
            buffer_time,
        }
    }

    /// Start playing
    pub async fn start(&mut self, publisher: Arc<Publisher>) -> Result<()> {
        // Get stream info
        let info = self.stream.info().await;

        // Subscribe to publisher
        let receiver = publisher.add_subscriber(
            format!("player-{}", info.id),
            info.id,
        ).await;

        self.receiver = Some(receiver);

        // Update state
        let mut state = self.state.write().await;
        *state = PlaybackState::Buffering;

        Ok(())
    }

    /// Stop playing
    pub async fn stop(&mut self, publisher: Arc<Publisher>) -> Result<()> {
        let info = self.stream.info().await;
        publisher.remove_subscriber(&format!("player-{}", info.id)).await;

        self.receiver = None;

        let mut state = self.state.write().await;
        *state = PlaybackState::Stopped;

        Ok(())
    }

    /// Pause playback
    pub async fn pause(&mut self) -> Result<()> {
        let mut state = self.state.write().await;
        if *state == PlaybackState::Playing {
            *state = PlaybackState::Paused;
        }
        Ok(())
    }

    /// Resume playback
    pub async fn resume(&mut self) -> Result<()> {
        let mut state = self.state.write().await;
        if *state == PlaybackState::Paused {
            *state = PlaybackState::Playing;
        }
        Ok(())
    }

    /// Get next packet
    pub async fn next_packet(&mut self) -> Option<RtmpPacket> {
        let state = *self.state.read().await;

        if state != PlaybackState::Playing && state != PlaybackState::Buffering {
            return None;
        }

        if let Some(ref mut receiver) = self.receiver {
            receiver.recv().await
        } else {
            None
        }
    }

    /// Get playback state
    pub async fn state(&self) -> PlaybackState {
        *self.state.read().await
    }
}