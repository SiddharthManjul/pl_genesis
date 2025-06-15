use libp2p::{
    identity, noise, ping, tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder
};
use std::error::Error;
use tokio::io;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);
    Ok(())
}
