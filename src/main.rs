use anyhow::Result;
use iroh::{Endpoint, SecretKey, protocol::Router, NodeId};
use iroh_gossip::{net::Gossip, proto::TopicId};
use iroh_gossip::api::{Event, GossipReceiver};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use futures_lite::StreamExt;

use iroh::NodeAddr;
use std::fmt;
use std::str::FromStr;
use iroh::Watcher;

#[tokio::main]
async fn main() -> Result<()> {
    let secret_key = SecretKey::generate(rand::rngs::OsRng);
    let endpoint = Endpoint::builder().secret_key(secret_key).discovery_n0().bind().await?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = Router::builder(endpoint.clone()).accept(iroh_gossip::ALPN, gossip.clone()).spawn();

    // Create a new topic
    let id = TopicId::from_bytes(rand::random());
    let node_ids = vec![];

    let ticket = {
        let me = endpoint.node_addr().initialized().await?;
        let nodes = vec![me];
        Ticket { topic: id, nodes }
    };
    println!("> Ticket to join us: {ticket}");

    // Subscribe to the topic
    let topic = gossip.subscribe(id, node_ids).await?;
    
    // Get the GossipSender and the GossipReceiver
    let (mut sender, receiver) = topic.split();

    let message = Message::new(MessageBody::AboutMe {
        from: endpoint.node_id(),
        name: String::from("alice")
    });

    // Broadcast a message to the topic
    sender.broadcast(message.to_vec().into()).await?;

    // Subscribe and print loop
    tokio::spawn(subscribe_loop(receiver));

    let (line_tx, mut line_rx) = tokio::sync::mpsc::channel(1);
    std::thread::spawn(move || input_loop(line_tx));

    // broadcast each line we type
    println!("> type a message and hit enter to broadcast...");

    while let Some(text) = line_rx.recv().await {

        let message = Message::new(MessageBody::Message {
            from: endpoint.node_id(),
            text: text.clone()
        });

        sender.broadcast(message.to_vec().into()).await?;
        println!("> sent: {text}");
    }

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

async fn subscribe_loop(mut receiver: GossipReceiver) -> Result<()> {
    let mut names: HashMap<NodeId, String> = HashMap::new();

    while let Some(event) = receiver.try_next().await? {
        if let Event::Received(msg) = event {
            match Message::from_bytes(&msg.content)?.body {
                MessageBody::AboutMe { from, name } => {
                    names.insert(from, name.clone());
                    println!("> {} is now known as {}", from.fmt_short(), name);
                }
                MessageBody::Message { from, text } => {
                    let name = names.get(&from).map_or_else(|| from.fmt_short(), String::to_string);
                    println!("{}: {}", name, text);
                }
            }
        }
    }

    Ok(())
}

fn input_loop(line_tx: tokio::sync::mpsc::Sender<String>) -> Result<()> {
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    loop {
        stdin.read_line(&mut buffer)?;
        line_tx.blocking_send(buffer.clone())?;
        buffer.clear();
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Ticket {
    topic: TopicId,
    nodes: Vec<NodeAddr>
}

impl Ticket {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).map_err(Into::into)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("serde_json::to_vec is infallible")
    }
}

impl fmt::Display for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut text = data_encoding::BASE32_NOPAD.encode(&self.to_bytes()[..]);
        text.make_ascii_lowercase();
        write!(f, "{}", text)
    }
}

impl FromStr for Ticket {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = data_encoding::BASE32_NOPAD.decode(s.to_ascii_uppercase().as_bytes())?;
        Self::from_bytes(&bytes)
    }
}
