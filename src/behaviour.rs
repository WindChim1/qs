use std::error::Error;

use libp2p::{
    identity::Keypair,
    mdns, ping,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    Multiaddr, StreamProtocol,
};
use serde::{Deserialize, Serialize};

#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub mdns: libp2p::mdns::tokio::Behaviour,
    pub request_reponse: request_response::cbor::Behaviour<FileRequest, FileResponse>,
    pub ping: ping::Behaviour,
}

impl ChatBehaviour {
    pub fn new(local_key: Keypair) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            mdns: mdns::tokio::Behaviour::new(Default::default(), local_key.public().to_peer_id())?,
            request_reponse: request_response::cbor::Behaviour::new(
                [(
                    StreamProtocol::new("/file-exchange/1"),
                    ProtocolSupport::Full,
                )],
                Default::default(),
            ),
            ping: ping::Behaviour::new(Default::default()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResponse {
    pub content: Vec<u8>,
    pub addrs: Vec<Multiaddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRequest {
    pub password: String,
}

impl FileRequest {
    pub fn new(password: String) -> Self {
        Self { password }
    }
}
