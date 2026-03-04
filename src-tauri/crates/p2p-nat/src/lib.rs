use std::net::SocketAddr;
use tokio::time;
use anyhow::{Result, Error};

use p2p_core::Core;

struct HolePuncher;

impl HolePuncher {
    pub async fn punch(address: SocketAddr, socket: Core) {
        for _ in 0..10 {
            time::sleep(std::time::Duration::from_millis(100)).await;
            let _ = socket.send(vec![0], address);
        }
    }
}

#[cfg(test)]
mod test_hole_punching {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_punching() -> Result<(), Error> {
        let addr1: SocketAddr = "127.0.0.1:9001".parse()?;
        
        let node1 = p2p_core::Core::bind(addr1).await?;
        sleep(Duration::from_millis(100)).await;
        HolePuncher::punch(addr1, node1).await;

        Ok(())
    }
}