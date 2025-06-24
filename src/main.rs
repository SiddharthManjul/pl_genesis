use libp2p::{
    identity, mdns, noise, ping, swarm::NetworkBehaviour, tcp, yamux, Multiaddr, PeerId, SwarmBuilder,
};
use std::error::Error;
use tokio::io::{self, AsyncBufReadExt};
use tracing_subscriber;
use futures::StreamExt;

// Combine multiple behaviours
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    ping: ping::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {local_peer_id}");

    // Create combined behaviour
    let behaviour = MyBehaviour {
        ping: ping::Behaviour::new(ping::Config::new()),
        mdns: mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?,
    };

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_key| behaviour)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(std::time::Duration::from_secs(60)))
        .build();

    // Listen on all interfaces
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Read user input for dialing peers
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                if let Ok(Some(line)) = line {
                    if let Ok(addr) = line.parse::<Multiaddr>() {
                        match swarm.dial(addr.clone()) {
                            Ok(_) => println!("Dialing {addr}"),
                            Err(e) => println!("Failed to dial {addr}: {e}"),
                        }
                    }
                }
            }
            event = swarm.select_next_some() => {
                match event {
                    libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {address}");
                    }
                    libp2p::swarm::SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, multiaddr) in list {
                            println!("Discovered peer {peer_id} at {multiaddr}");
                            if let Err(e) = swarm.dial(multiaddr) {
                                println!("Failed to dial discovered peer: {e}");
                            }
                        }
                    }
                    libp2p::swarm::SwarmEvent::Behaviour(MyBehaviourEvent::Ping(ping::Event {
                        peer,
                        result: Ok(rtt),
                        ..
                    })) => {
                        println!("Ping to {peer} succeeded with RTT: {rtt:?}");
                    }
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("Connected to {peer_id}");
                    }
                    _ => {}
                }
            }
        }
    }
}