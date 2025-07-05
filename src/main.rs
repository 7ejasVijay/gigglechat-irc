use anyhow::Result;
use iroh::{Endpoint, SecretKey, protocol::Router, NodeId};
use iroh_gossip::{net::Gossip, proto::TopicId};
use serde::{Serialize, Deserialize};

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

    let message = Message::new(MessageBody::AboutMe {
        from: endpoint.node_id(),
        name: String::from("alice")
    });

    // Broadcast a message to the topic
    sender.broadcast(message.to_vec().into()).await?;

    // Cleanly shutdown the router
    router.shutdown().await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    body: MessageBody,
    nonce: [u8; 16],
}

#[derive(Debug, Serialize, Deserialize)]
enum MessageBody {
    AboutMe { from: NodeId, name: String },
    Message { from: NodeId, text: String },
}

impl Message {
    pub fn new(body: MessageBody) -> Self {
        Self {
            body,
            nonce: rand::random(),
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("serde_json::to_vec is infallible")
    }
}
