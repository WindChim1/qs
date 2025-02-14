use std::error::Error;

use libp2p::{identity::Keypair, noise, yamux, Swarm, SwarmBuilder};

use crate::behaviour::ChatBehaviour;

pub fn build_swarm(local_key: Keypair) -> Result<Swarm<ChatBehaviour>, Box<dyn Error>> {
    let swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            Default::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|b| ChatBehaviour::new(local_key).unwrap())?
        .build();

    Ok(swarm)
}
