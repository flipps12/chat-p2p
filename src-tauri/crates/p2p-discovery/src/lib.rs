use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use reqwest::Client;
use anyhow::{Result, anyhow};

use p2p_core::Core;

struct Discovery {
    discovered_peers: Vec<String>,
    client: Client,
}

#[derive(Serialize, Deserialize)]
struct PublicPeerInfo {
    nametag: String,
    peer_id: String,
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
    async fn discover_peers_by_nametag(&mut self, nametag: &str) -> Result<()> {
        // send nametag to the network and discover peers
        let response = self.client
            .get("http://localhost:3000/discover")
            .query(&[
                ("nametag", nametag),
            ])
            .send()
            .await?
            .text()
            .await?;

        println!("{}", response);
        self.discover(&response);
        
        Ok(())
    }

    // mudar a udp socket
    async fn public_peer_id_nametag(&self, nametag: &str, peer_id: &str) -> Result<()> {
        let response = self.client
            .post("http://localhost:3000/public")
            .json(&PublicPeerInfo {
                nametag: nametag.to_string(),
                peer_id: peer_id.to_string(),
            })
            .send()
            .await?
            .text()
            .await?;

        println!("{}", response);
        Ok(())
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
            .public_peer_id_nametag("test_nametag", "test_peer_id")
            .await?;

        // send nametag and get peerid - address
        discovery
            .discover_peers_by_nametag("test_nametag")
            .await?;

        // discover ADDR:PORT of nat
        discovery
            .get_my_address(&mut core, "127.0.0.1:5005".parse().unwrap())
            .await?;

        Ok(())
    }
}