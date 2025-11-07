use crate::protocol::RtmpPacket;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{RtmpData, Result};
use crate::stream::gop_cache::GopCache;
use crate::stream::stream::{Stream, StreamMetadata};

pub struct Publisher {
    /// Base stream
    stream: Arc<Stream>,

    /// GOP cache
    gop_cache: Arc<RwLock<GopCache>>,

    /// Subscribers
    subscribers: Arc<RwLock<Vec<SubscriberHandle>>>,

    /// Audio codec config
    audio_codec_config: Arc<RwLock<Option<Vec<u8>>>>,

    /// Video codec config
    video_codec_config: Arc<RwLock<Option<Vec<u8>>>>,

    /// Metadata packet
    metadata_packet: Arc<RwLock<Option<RtmpPacket>>>,
}

pub struct SubscriberHandle {
    /// Subscriber ID
    id: String,

    /// Packet sender
    sender: mpsc::Sender<RtmpPacket>,

    /// Stream ID for subscriber
    stream_id: u32,
}

impl Publisher {
    /// Create new publisher
    pub fn new(stream: Arc<Stream>, gop_cache_size: usize) -> Self {
        Publisher {
            stream,
            gop_cache: Arc::new(RwLock::new(GopCache::new(gop_cache_size))),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            audio_codec_config: Arc::new(RwLock::new(None)),
            video_codec_config: Arc::new(RwLock::new(None)),
            metadata_packet: Arc::new(RwLock::new(None)),
        }
    }

    /// Process audio packet
    pub async fn process_audio(&self, packet: RtmpPacket) -> Result<()> {
        // Check for AAC sequence header
        if is_aac_sequence_header(&packet.payload) {
            let mut config = self.audio_codec_config.write().await;
            *config = Some(packet.payload.clone());
        }

        // Update stats
        self.stream.update_stats(|stats| {
            stats.audio_packets += 1;
            stats.bytes_in += packet.payload.len() as u64;
            stats.last_audio_timestamp = packet.timestamp();
        }).await;

        // Distribute to subscribers
        self.distribute_packet(packet).await?;

        Ok(())
    }

    /// Process video packet
    pub async fn process_video(&self, mut packet: RtmpPacket) -> Result<()> {
        // Check for AVC sequence header
        if is_avc_sequence_header(&packet.payload) {
            let mut config = self.video_codec_config.write().await;
            *config = Some(packet.payload.clone());
        }

        // Add to GOP cache if keyframe
        if is_keyframe(&packet.payload) {
            let mut cache = self.gop_cache.write().await;
            cache.add_keyframe(packet.clone());
        } else {
            let mut cache = self.gop_cache.write().await;
            cache.add_frame(packet.clone());
        }

        // Update stats
        self.stream.update_stats(|stats| {
            stats.video_packets += 1;
            stats.bytes_in += packet.payload.len() as u64;
            stats.last_video_timestamp = packet.timestamp();
        }).await;

        // Distribute to subscribers
        self.distribute_packet(packet).await?;

        Ok(())
    }

    /// Process metadata
    pub async fn process_metadata(&self, packet: RtmpPacket) -> Result<()> {
        // Parse metadata
        let data = RtmpData::decode(&packet.payload)?;
        if let Some(metadata_obj) = data.values.first().and_then(|v| v.as_object()) {
            let metadata = StreamMetadata::from_amf(metadata_obj);
            self.stream.set_metadata(metadata).await;
        }

        // Store metadata packet
        let mut stored = self.metadata_packet.write().await;
        *stored = Some(packet.clone());

        // Update stats
        self.stream.update_stats(|stats| {
            stats.data_packets += 1;
            stats.bytes_in += packet.payload.len() as u64;
        }).await;

        // Distribute to subscribers
        self.distribute_packet(packet).await?;

        Ok(())
    }

    /// Add subscriber
    pub async fn add_subscriber(
        &self,
        id: String,
        stream_id: u32,
    ) -> mpsc::Receiver<RtmpPacket> {
        let (tx, rx) = mpsc::channel(100);

        // Send initial packets
        self.send_initial_packets(&tx, stream_id).await;

        // Add to subscribers
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(SubscriberHandle {
            id,
            sender: tx,
            stream_id,
        });

        rx
    }

    /// Remove subscriber
    pub async fn remove_subscriber(&self, id: &str) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.retain(|s| s.id != id);
    }

    /// Send initial packets to new subscriber
    async fn send_initial_packets(&self, sender: &mpsc::Sender<RtmpPacket>, stream_id: u32) {
        // Send metadata
        if let Some(metadata) = self.metadata_packet.read().await.as_ref() {
            let mut packet = metadata.clone();
            packet.header.message_stream_id = stream_id;
            let _ = sender.send(packet).await;
        }

        // Send audio codec config
        if let Some(config) = self.audio_codec_config.read().await.as_ref() {
            let packet = crate::protocol::make_audio_packet(
                config.clone(),
                0,
                stream_id,
            );
            let _ = sender.send(packet).await;
        }

        // Send video codec config
        if let Some(config) = self.video_codec_config.read().await.as_ref() {
            let packet = crate::protocol::make_video_packet(
                config.clone(),
                0,
                stream_id,
            );
            let _ = sender.send(packet).await;
        }

        // Send GOP cache
        let cache = self.gop_cache.read().await;
        for packet in cache.get_gop() {
            let mut p = packet.clone();
            p.header.message_stream_id = stream_id;
            let _ = sender.send(p).await;
        }
    }

    /// Distribute packet to all subscribers
    async fn distribute_packet(&self, packet: RtmpPacket) -> Result<()> {
        let mut failed = Vec::new();
        let subscribers = self.subscribers.read().await;

        for subscriber in subscribers.iter() {
            let mut p = packet.clone();
            p.header.message_stream_id = subscriber.stream_id;

            if subscriber.sender.send(p).await.is_err() {
                failed.push(subscriber.id.clone());
            }
        }

        // Remove failed subscribers
        if !failed.is_empty() {
            drop(subscribers);
            let mut subscribers = self.subscribers.write().await;
            for id in failed {
                subscribers.retain(|s| s.id != id);
            }
        }

        Ok(())
    }

    /// Get subscriber count
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }
}

// Helper functions
fn is_keyframe(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    let video_tag_header = data[0];
    let frame_type = (video_tag_header >> 4) & 0x0F;
    frame_type == 1 // Keyframe
}

fn is_aac_sequence_header(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    let sound_format = (data[0] >> 4) & 0x0F;
    let aac_packet_type = data[1];
    sound_format == 10 && aac_packet_type == 0
}

fn is_avc_sequence_header(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    let video_tag_header = data[0];
    let codec_id = video_tag_header & 0x0F;
    let avc_packet_type = data[1];
    codec_id == 7 && avc_packet_type == 0
}