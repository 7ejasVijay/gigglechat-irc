use anyhow::Result;
use iroh::{Endpoint, SecretKey, protocol::Router};
use iroh_gossip::{net::Gossip, proto::TopicId};

#[tokio::main]
async fn main() -> Result<()> {
    let secret_key = SecretKey::generate(rand::rngs::OsRng);
    let endpoint = Endpoint::builder().secret_key(secret_key).discovery_n0().bind().await?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = Router::builder(endpoint.clone()).accept(iroh_gossip::ALPN, gossip.clone()).spawn();

    // Create a new topic
    let id = TopicId::from_bytes(rand::random());
    let node_ids = vec![];

    // Subscribe to the topic
    let topic = gossip.subscribe(id, node_ids).await?;
    
    // Get the GossipSender and the GossipReceiver
    let (mut sender, _receiver) = topic.split();

    // Broadcast a message to the topic
    sender.broadcast("sup".into()).await?;

    // Cleanly shutdown the router
    router.shutdown().await?;

    Ok(())
}
