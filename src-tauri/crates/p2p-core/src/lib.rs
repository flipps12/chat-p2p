use anyhow::Result;
use tokio::{
    net::UdpSocket,
    sync::mpsc,
};
use std::{net::SocketAddr, sync::Arc};

#[derive(Debug, Clone)]
pub enum CoreEvent {
    Message {
        data: Vec<u8>,
        from: SocketAddr,
    }
}

pub struct Core {
    tx: mpsc::Sender<(Vec<u8>, SocketAddr)>,
    event_rx: mpsc::Receiver<CoreEvent>,
}

impl Core {
    pub async fn bind(address: SocketAddr) -> Result<Self> {
        let socket = Arc::new(UdpSocket::bind(address).await?);

        let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(100);
        let (event_tx, event_rx) = mpsc::channel::<CoreEvent>(100);

        let sock_clone = socket.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 2048];

            loop {
                tokio::select! {
                    Some((data, addr)) = rx.recv() => {
                        if let Err(e) = sock_clone.send_to(&data, addr).await {
                            eprintln!("send error: {:?}", e);
                        }
                    }

                    Ok((len, addr)) = sock_clone.recv_from(&mut buf) => {
                        let data = buf[..len].to_vec();

                        let event = CoreEvent::Message {
                            data,
                            from: addr,
                        };

                        let _ = event_tx.send(event).await;
                    }
                }
            }
        });

        Ok(Self {
            tx,
            event_rx,
        })
    }

    pub async fn send(&self, data: Vec<u8>, addr: SocketAddr) -> Result<()> {
        self.tx.send((data, addr)).await?;
        Ok(())
    }

    pub async fn next_event(&mut self) -> Option<CoreEvent> {
        self.event_rx.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_two_nodes_udp_event() -> Result<()> {
        let addr1: SocketAddr = "127.0.0.1:9001".parse()?;
        let addr2: SocketAddr = "127.0.0.1:9002".parse()?;

        let node1 = Core::bind(addr1).await?;
        let mut node2 = Core::bind(addr2).await?;

        sleep(Duration::from_millis(100)).await;

        node1.send(b"hola evento".to_vec(), addr2).await?;

        if let Some(CoreEvent::Message { data, from }) = node2.next_event().await {
            assert_eq!(data, b"hola evento".to_vec());
            assert_eq!(from, addr1);
        } else {
            panic!("No event received");
        }

        Ok(())
    }
}