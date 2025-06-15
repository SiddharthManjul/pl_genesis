use libp2p::{
    identity, noise, ping, tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
    core::Transport
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

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise::Config::new(&local_key)?)
        .multiplex(yamux::Config::default())
        .boxed();

    let behaviour = ping::Behaviour::new(ping::Config::new().with_interval(std::time::Duration::from_secs(15)));

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_behaviour(|_| behaviour)?
        .build();
    Ok(())
}
