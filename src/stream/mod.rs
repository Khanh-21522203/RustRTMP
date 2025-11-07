use crate::{PublisherInfo, PublisherRegistry};

mod stream;
mod publisher;
mod player;
mod gop_cache;


pub async fn find_publisher(name: &str, registry: &PublisherRegistry) -> Option<PublisherInfo> {
    registry.get(name).await
}