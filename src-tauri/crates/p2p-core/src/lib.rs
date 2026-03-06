// crates/p2p-core/src/lib.rs

use tokio::net::UdpSocket;
use std::net::SocketAddr;
use anyhow::Result;

// ============================================================================
// PACKET TYPE CONSTANTS
// ============================================================================

pub const PACKET_TYPE_DISCOVERY: u8 = 0x01;
pub const PACKET_TYPE_TRANSPORT: u8 = 0x02;
pub const PACKET_TYPE_P2P_DATA: u8 = 0x03;

// ============================================================================
// CORE EVENT
// ============================================================================

#[derive(Debug, Clone)]
pub enum CoreEvent {
    Message {
        from: SocketAddr,
        data: Vec<u8>,
    },
}

// ============================================================================
// CORE SOCKET
// ============================================================================

pub struct Core {
    socket: UdpSocket,
    buffer: Vec<u8>,
}

impl Core {
    /// Bind a una dirección local
    pub async fn bind(addr: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        println!("✅ [Core] Bound to {}", socket.local_addr()?);
        
        Ok(Self {
            socket,
            buffer: vec![0u8; 65535],
        })
    }
    
    /// Obtener dirección local
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.local_addr()?)
    }
    
    /// Enviar datos crudos
    pub async fn send(&mut self, mut data: Vec<u8>, destination: SocketAddr) -> Result<()> {
        // IMPORTANTE: Si el primer byte no está seteado, setearlo a P2P_DATA
        if data.is_empty() || data[0] == 0 {
            // Prepend packet type
            let mut new_data = vec![PACKET_TYPE_P2P_DATA];
            new_data.extend_from_slice(&data);
            data = new_data;
        }
        
        let sent = self.socket.send_to(&data, destination).await?;
        tracing::debug!("[Core] Sent {} bytes to {}", sent, destination);
        
        Ok(())
    }
    
    /// Recibir siguiente evento
    pub async fn next_event(&mut self) -> Option<CoreEvent> {
        match self.socket.recv_from(&mut self.buffer).await {
            Ok((len, from)) => {
                let data = self.buffer[..len].to_vec();
                
                Some(CoreEvent::Message { from, data })
            }
            Err(e) => {
                tracing::error!("[Core] Receive error: {}", e);
                None
            }
        }
    }
    
    /// Peek del tipo de paquete sin consumir
    pub fn peek_packet_type(data: &[u8]) -> u8 {
        data.get(0).copied().unwrap_or(PACKET_TYPE_P2P_DATA)
    }
}

// ============================================================================
// CORE CON ROUTING (AVANZADO)
// ============================================================================

use tokio::sync::mpsc;

pub struct CoreWithRouting {
    socket: UdpSocket,
    
    /// Canal para Discovery
    discovery_tx: Option<mpsc::Sender<CoreEvent>>,
    
    /// Canal para Transport
    transport_tx: Option<mpsc::Sender<CoreEvent>>,
    
    /// Canal para datos P2P
    p2p_data_tx: Option<mpsc::Sender<CoreEvent>>,
}

impl CoreWithRouting {
    pub async fn bind(
        addr: SocketAddr,
        discovery_tx: Option<mpsc::Sender<CoreEvent>>,
        transport_tx: Option<mpsc::Sender<CoreEvent>>,
        p2p_data_tx: Option<mpsc::Sender<CoreEvent>>,
    ) -> Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        println!("✅ [Core] Bound to {} with routing", socket.local_addr()?);
        
        let core = Self {
            socket,
            discovery_tx,
            transport_tx,
            p2p_data_tx,
        };
        
        Ok(core)
    }
    
    /// Enviar datos
    pub async fn send(&self, data: Vec<u8>, destination: SocketAddr) -> Result<()> {
        let sent = self.socket.send_to(&data, destination).await?;
        tracing::debug!("[Core] Sent {} bytes to {}", sent, destination);
        Ok(())
    }
    
    /// Iniciar receive loop con routing automático
    pub async fn start_routing(mut self) {
        println!("🔀 [Core] Starting routing loop");
        
        let mut buffer = vec![0u8; 65535];
        
        loop {
            match self.socket.recv_from(&mut buffer).await {
                Ok((len, from)) => {
                    let data = buffer[..len].to_vec();
                    
                    // Peek primer byte
                    let packet_type = data.get(0).copied().unwrap_or(PACKET_TYPE_P2P_DATA);
                    
                    let event = CoreEvent::Message {
                        from,
                        data,
                    };
                    
                    // Rutear según tipo
                    match packet_type {
                        PACKET_TYPE_DISCOVERY => {
                            if let Some(ref tx) = self.discovery_tx {
                                let _ = tx.send(event).await;
                            }
                        }
                        PACKET_TYPE_TRANSPORT => {
                            if let Some(ref tx) = self.transport_tx {
                                let _ = tx.send(event).await;
                            }
                        }
                        PACKET_TYPE_P2P_DATA => {
                            if let Some(ref tx) = self.p2p_data_tx {
                                let _ = tx.send(event).await;
                            }
                        }
                        _ => {
                            tracing::warn!("[Core] Unknown packet type: 0x{:02x}", packet_type);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("[Core] Receive error: {}", e);
                }
            }
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_core_bind() -> Result<()> {
        let core = Core::bind("127.0.0.1:0".parse()?).await?;
        assert!(core.local_addr()?.port() > 0);
        Ok(())
    }
    
    #[tokio::test]
    async fn test_peek_packet_type() {
        let discovery_packet = vec![PACKET_TYPE_DISCOVERY, 1, 2, 3];
        assert_eq!(Core::peek_packet_type(&discovery_packet), PACKET_TYPE_DISCOVERY);
        
        let transport_packet = vec![PACKET_TYPE_TRANSPORT, 1, 2, 3];
        assert_eq!(Core::peek_packet_type(&transport_packet), PACKET_TYPE_TRANSPORT);
    }
}