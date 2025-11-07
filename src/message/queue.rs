use crate::{Error, Result};
use crate::protocol::{RtmpPacket, *};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use tokio::sync::{mpsc, RwLock};
use std::sync::Arc;

/// Priority wrapper for packets
#[derive(Clone)]
struct PriorityPacket {
    packet: RtmpPacket,
    priority: u8,
}

impl Eq for PriorityPacket {}

impl PartialEq for PriorityPacket {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Ord for PriorityPacket {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for PriorityPacket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct MessageQueue {
    /// Sender for queue
    sender: mpsc::Sender<PriorityPacket>,

    /// Receiver for queue
    receiver: Arc<RwLock<mpsc::Receiver<PriorityPacket>>>,

    /// Priority queue for ordering
    priority_queue: Arc<RwLock<BinaryHeap<PriorityPacket>>>,

    /// Queue size limit
    max_size: usize,

    /// Current queue size
    current_size: Arc<RwLock<usize>>,
}

impl MessageQueue {
    /// Create new message queue
    pub fn new(max_size: usize) -> Self {
        let (sender, receiver) = mpsc::channel(max_size);

        MessageQueue {
            sender,
            receiver: Arc::new(RwLock::new(receiver)),
            priority_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            max_size,
            current_size: Arc::new(RwLock::new(0)),
        }
    }

    /// Push message to queue
    pub async fn push(&self, packet: RtmpPacket) -> Result<()> {
        // Check queue size
        let mut size = self.current_size.write().await;
        if *size >= self.max_size {
            return Err(Error::protocol("Message queue full"));
        }

        // Determine priority based on message type
        let priority = self.get_priority(&packet);

        // Create priority packet
        let priority_packet = PriorityPacket { packet, priority };

        // Send to channel
        self.sender.send(priority_packet).await
            .map_err(|_| Error::protocol("Failed to send to queue"))?;

        *size += 1;
        Ok(())
    }

    /// Pop message from queue
    pub async fn pop(&self) -> Result<Option<RtmpPacket>> {
        let mut receiver = self.receiver.write().await;

        // Try to receive from channel
        match receiver.try_recv() {
            Ok(priority_packet) => {
                let mut size = self.current_size.write().await;
                *size = size.saturating_sub(1);
                Ok(Some(priority_packet.packet))
            }
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err(Error::protocol("Queue disconnected"))
            }
        }
    }

    /// Pop with timeout
    pub async fn pop_timeout(&self, timeout: std::time::Duration) -> Result<Option<RtmpPacket>> {
        let mut receiver = self.receiver.write().await;

        match tokio::time::timeout(timeout, receiver.recv()).await {
            Ok(Some(priority_packet)) => {
                let mut size = self.current_size.write().await;
                *size = size.saturating_sub(1);
                Ok(Some(priority_packet.packet))
            }
            Ok(None) => Ok(None),
            Err(_) => Ok(None), // Timeout
        }
    }

    /// Get queue size
    pub async fn size(&self) -> usize {
        *self.current_size.read().await
    }

    /// Check if queue is empty
    pub async fn is_empty(&self) -> bool {
        self.size().await == 0
    }

    /// Clear queue
    pub async fn clear(&self) {
        let mut receiver = self.receiver.write().await;
        while receiver.try_recv().is_ok() {}

        let mut size = self.current_size.write().await;
        *size = 0;
    }

    /// Get priority for packet
    fn get_priority(&self, packet: &RtmpPacket) -> u8 {
        let msg_type = packet.message_type();
        
        // Control messages have highest priority
        if msg_type == MSG_TYPE_SET_CHUNK_SIZE
            || msg_type == MSG_TYPE_ABORT
            || msg_type == MSG_TYPE_ACK
            || msg_type == MSG_TYPE_WINDOW_ACK
            || msg_type == MSG_TYPE_SET_PEER_BW
        {
            return 10;
        }

        // Commands have high priority
        if msg_type == MSG_TYPE_COMMAND_AMF0 || msg_type == MSG_TYPE_COMMAND_AMF3 {
            return 8;
        }

        // Data messages have medium priority
        if msg_type == MSG_TYPE_DATA_AMF0 || msg_type == MSG_TYPE_DATA_AMF3 {
            return 5;
        }

        // Audio has lower priority than video
        if msg_type == MSG_TYPE_AUDIO {
            return 3;
        }

        // Video has lowest priority (largest messages)
        if msg_type == MSG_TYPE_VIDEO {
            return 2;
        }

        // Unknown
        1
    }
}

#[cfg(test)]
mod tests {
    use crate::MSG_TYPE_AUDIO;
    use super::*;
    use crate::protocol::{make_audio_packet, make_video_packet};

    #[tokio::test]
    async fn test_queue_priority() {
        let queue = MessageQueue::new(10);

        // Add packets with different priorities
        let video = make_video_packet(vec![1, 2, 3], 1000, 1);
        let audio = make_audio_packet(vec![4, 5, 6], 2000, 1);

        queue.push(video.clone()).await.unwrap();
        queue.push(audio.clone()).await.unwrap();

        // Audio should come out first (higher priority)
        let first = queue.pop().await.unwrap().unwrap();
        assert_eq!(first.message_type(), MSG_TYPE_AUDIO);
    }

    #[tokio::test]
    async fn test_queue_size_limit() {
        let queue = MessageQueue::new(2);

        let packet1 = make_audio_packet(vec![1], 1000, 1);
        let packet2 = make_audio_packet(vec![2], 2000, 1);
        let packet3 = make_audio_packet(vec![3], 3000, 1);

        assert!(queue.push(packet1).await.is_ok());
        assert!(queue.push(packet2).await.is_ok());
        assert!(queue.push(packet3).await.is_err()); // Should fail - queue full
    }
}