use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use reqwest::Client;
use anyhow::{Result, anyhow};

use p2p_core::Core;

// json
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Publish {
    header: String,
    body: PublishBody
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PublishBody {
    nametag: String,
    peerid: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GetPeerId {
    header: String,
    body: GetPeerIdBody
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GetPeerIdBody {
    nametag: String
}

// Object

struct Discovery {
    discovered_peers: Vec<String>,
    client: Client,
}

impl Discovery {
    fn new() -> Self {
        Discovery {
            discovered_peers: Vec::new(),
            client: Client::new(),
        }
    }

    fn discover(&mut self, peer_id: &str) {
        // Simulate peer discovery
        self.discovered_peers.push(peer_id.to_string());
    }

    fn get_discovered_peers(&self) -> &Vec<String> {
        &self.discovered_peers
    }

    // usar udp Socket y devolver address - peerid
    async fn discover_peers_by_nametag(&mut self, core: &mut Core, addr: SocketAddr, nametag: &str) -> Result<()> {
        let message = GetPeerId {
            header: "getpeerid".to_string(),
            body: GetPeerIdBody {
                nametag: nametag.to_string(),
            },
        };
        
        let json = serde_json::to_vec(&message)?;

        core.send(json, addr).await?;

        if let Some(p2p_core::CoreEvent::Message { from, .. }) = core.next_event().await {
            self.discover(&from.to_string());
            Ok(())
        } else {
            Err(anyhow!("Failed to get address"))
        }
    }

    // mudar a udp socket
    async fn publish_peer_id_nametag(&self, core: &mut Core, addr: SocketAddr, nametag: &str, peer_id: &str) -> Result<()> {
        let message = Publish {
            header: "publish".to_string(),
            body: PublishBody {
                nametag: nametag.to_string(),
                peerid: peer_id.to_string(),
            },
        };
        
        let json = serde_json::to_vec(&message)?;

        core.send(json, addr).await?;

        if let Some(p2p_core::CoreEvent::Message { from, .. }) = core.next_event().await {
            // resolve status
            Ok(())
        } else {
            Err(anyhow!("Failed to publish peerid"))
        }
    }

    async fn get_my_address(&self, core: &mut Core, addr: SocketAddr) -> Result<String> {
        core.send(vec![0], addr).await?;
        // Get the local address of the core
        if let Some(p2p_core::CoreEvent::Message { from, .. }) = core.next_event().await {
            Ok(from.to_string())
        } else {
            Err(anyhow!("Failed to get address"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_public_peer() -> Result<()> {
        let mut discovery = Discovery::new();
        let mut core = Core::bind("127.0.0.1:5006".parse().unwrap()).await?;


        // register nametag with peerid
        discovery
            .publish_peer_id_nametag(&mut core, "127.0.0.1:5005".parse().unwrap(), "test_nametag", "test_peer_id")
            .await?;

        // send nametag and get peerid - address
        discovery
            .discover_peers_by_nametag(&mut core, "127.0.0.1:5005".parse().unwrap(), "test_nametag")
            .await?;

        // discover ADDR:PORT of nat
        discovery
            .get_my_address(&mut core, "127.0.0.1:5005".parse().unwrap())
            .await?;

        Ok(())
    }
}