use reqwest::{Client, Error};
use serde::{Serialize, Deserialize};

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

    async fn discover_peers_by_nametag(&mut self, nametag: &str) -> Result<(), Error> {
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

    async fn public_peer_id_nametag(&self, nametag: &str, peer_id: &str) -> Result<(), Error> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_public_peer() -> Result<(), Error> {
        let mut discovery = Discovery::new();

        discovery
            .public_peer_id_nametag("test_nametag", "test_peer_id")
            .await?;

        discovery
            .discover_peers_by_nametag("test_nametag")
            .await?;

        Ok(())
    }
}