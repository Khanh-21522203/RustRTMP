use crate::protocol::RtmpPacket;
use std::collections::VecDeque;

pub struct GopCache {
    /// Maximum GOPs to cache
    max_gops: usize,

    /// Current GOP being built
    current_gop: Vec<RtmpPacket>,

    /// Completed GOPs
    cached_gops: VecDeque<Vec<RtmpPacket>>,

    /// Total cached packets
    total_packets: usize,
}

impl GopCache {
    /// Create new GOP cache
    pub fn new(max_gops: usize) -> Self {
        GopCache {
            max_gops,
            current_gop: Vec::new(),
            cached_gops: VecDeque::new(),
            total_packets: 0,
        }
    }

    /// Add keyframe (starts new GOP)
    pub fn add_keyframe(&mut self, packet: RtmpPacket) {
        // Save current GOP if not empty
        if !self.current_gop.is_empty() {
            self.finish_current_gop();
        }

        // Start new GOP with keyframe
        self.current_gop.push(packet);
        self.total_packets += 1;
    }

    /// Add regular frame to current GOP
    pub fn add_frame(&mut self, packet: RtmpPacket) {
        if !self.current_gop.is_empty() {
            self.current_gop.push(packet);
            self.total_packets += 1;
        }
        // Ignore frames without keyframe
    }

    /// Finish current GOP and cache it
    fn finish_current_gop(&mut self) {
        if self.current_gop.is_empty() {
            return;
        }

        let gop = std::mem::take(&mut self.current_gop);
        self.cached_gops.push_back(gop);

        // Limit cache size
        while self.cached_gops.len() > self.max_gops {
            if let Some(removed) = self.cached_gops.pop_front() {
                self.total_packets -= removed.len();
            }
        }
    }

    /// Get all cached packets for new subscriber
    pub fn get_gop(&self) -> Vec<RtmpPacket> {
        let mut packets = Vec::with_capacity(self.total_packets);

        // Add all cached GOPs
        for gop in &self.cached_gops {
            packets.extend_from_slice(gop);
        }

        // Add current GOP if has keyframe
        if !self.current_gop.is_empty() {
            packets.extend_from_slice(&self.current_gop);
        }

        packets
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.current_gop.clear();
        self.cached_gops.clear();
        self.total_packets = 0;
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.total_packets
    }

    /// Get GOP count
    pub fn gop_count(&self) -> usize {
        self.cached_gops.len() + if self.current_gop.is_empty() { 0 } else { 1 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gop_cache() {
        let mut cache = GopCache::new(2);

        // Create test packets
        let keyframe1 = create_test_keyframe(1000);
        let frame1 = create_test_frame(1033);
        let frame2 = create_test_frame(1066);
        let keyframe2 = create_test_keyframe(2000);

        // Add first GOP
        cache.add_keyframe(keyframe1);
        cache.add_frame(frame1);
        cache.add_frame(frame2);

        // Add second GOP
        cache.add_keyframe(keyframe2);

        // Check cache
        assert_eq!(cache.gop_count(), 2);
        assert_eq!(cache.size(), 4);

        // Get cached packets
        let packets = cache.get_gop();
        assert_eq!(packets.len(), 4);
    }

    fn create_test_keyframe(timestamp: u32) -> RtmpPacket {
        let data = vec![0x17, 0x00]; // Keyframe marker
        crate::protocol::make_video_packet(data, timestamp, 1)
    }

    fn create_test_frame(timestamp: u32) -> RtmpPacket {
        let data = vec![0x27, 0x01]; // Inter-frame marker
        crate::protocol::make_video_packet(data, timestamp, 1)
    }
}