use anyhow::Result;
use tokio::{
    net::UdpSocket,
    sync::mpsc,
};
use std::{net::SocketAddr, sync::Arc};

pub struct Core {
    incoming: mpsc::Receiver<(Vec<u8>, SocketAddr)>,
    tx: mpsc::Sender<(Vec<u8>, SocketAddr)>,
}

impl Core {
    pub async fn bind(address: SocketAddr) -> Result<Self> {
        let socket = Arc::new(UdpSocket::bind(address).await?);

        let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(100);
        let (incoming_tx, incoming_rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(100);

        let sock_clone = socket.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 2048];

            loop {
                tokio::select! {
                    Some((data, addr)) = rx.recv() => {
                        let _ = sock_clone.send_to(&data, addr).await;
                    }

                    Ok((len, addr)) = sock_clone.recv_from(&mut buf) => {
                        let data = buf[..len].to_vec();
                        let _ = incoming_tx.send((data, addr)).await;
                    }
                }
            }
        });

        Ok(Self {
            tx,
            incoming: incoming_rx,
        })
    }

    pub async fn send(&self, data: Vec<u8>, addr: SocketAddr) -> Result<()> {
        self.tx.send((data, addr)).await?;
        Ok(())
    }

    pub async fn recv(&mut self) -> Option<(Vec<u8>, SocketAddr)> {
        self.incoming.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_two_nodes_udp() -> Result<()> {
        let addr1: SocketAddr = "127.0.0.1:9001".parse()?;
        let addr2: SocketAddr = "127.0.0.1:9002".parse()?;

        let node1 = Core::bind(addr1).await?;
        let mut node2 = Core::bind(addr2).await?;

        // pequeña espera para asegurar que ambos sockets están listos
        sleep(Duration::from_millis(100)).await;

        node1.send(b"hola nodo2".to_vec(), addr2).await?;

        if let Some((data, from)) = node2.recv().await {
            assert_eq!(data, b"hola nodo2".to_vec());
            assert_eq!(from, addr1);
        } else {
            panic!("node2 did not receive message");
        }

        Ok(())
    }
}