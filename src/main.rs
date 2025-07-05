use anyhow::Result;
use iroh::{Endpoint, SecretKey, protocol::Router};
use iroh_gossip::net::Gossip;

#[tokio::main]
async fn main() -> Result<()> {
    let secret_key = SecretKey::generate(rand::rngs::OsRng);
    let endpoint = Endpoint::builder().secret_key(secret_key).discovery_n0().bind().await?;
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let router = Router::builder(endpoint.clone()).accept(iroh_gossip::ALPN, gossip.clone()).spawn();
    // Cleanly shutdown the router
    router.shutdown().await?;

    Ok(())
}
