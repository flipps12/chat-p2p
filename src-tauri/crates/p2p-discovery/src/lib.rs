// crates/p2p-discovery/src/lib.rs

use wincode::{SchemaWrite, SchemaRead};
use p2p_transport::{Transport, PacketListener, ReceivedPacket, PACKET_TYPE_DISCOVERY};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, Mutex};
use anyhow::{Result, anyhow};

// ============================================================================
// DISCOVERY PROTOCOL STRUCTURES
// ============================================================================

#[derive(SchemaWrite, SchemaRead, Clone, Debug)]
pub struct DiscoveryPacket {
    pub packet_type: u8,         // 0x01 para Discovery
    pub transaction_id: u64,
    pub action: DiscoveryAction,
}

#[derive(SchemaWrite, SchemaRead, Clone, Debug)]
pub enum DiscoveryAction {
    Publish {
        nametag: String,
        peerid: String,
    },
    GetPeerId {
        nametag: String,
    },
    GetMyAddress,
}

#[derive(SchemaWrite, SchemaRead, Clone, Debug)]
pub struct DiscoveryResponse {
    pub packet_type: u8,         // 0x01
    pub transaction_id: u64,
    pub result: DiscoveryResult,
}

#[derive(SchemaWrite, SchemaRead, Clone, Debug)]
pub enum DiscoveryResult {
    PublishOk {
        status: String,
    },
    PeerInfo {
        address: String,
        port: u16,
        peerid: String,
    },
    MyAddress {
        address: String,
        port: u16,
    },
    Error {
        message: String,
    },
}

// Para compatibilidad con código existente
#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub address: String,
    pub port: u16,
    pub peerid: String,
}

// ============================================================================
// PENDING REQUEST
// ============================================================================

struct PendingRequest {
    response_tx: oneshot::Sender<DiscoveryResult>,
    created_at: Instant,
}

// ============================================================================
// DISCOVERY CLIENT - Usa Transport via PacketListener
// ============================================================================

pub struct Discovery {
    /// Transport compartido
    transport: Arc<Transport>,
    
    /// Canal para recibir respuestas Discovery
    discovery_rx: Arc<Mutex<mpsc::Receiver<ReceivedPacket>>>,
    
    /// HashMap para tracking de requests
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    
    /// Contador de transaction IDs
    next_transaction_id: Arc<Mutex<u64>>,
    
    /// Peers descubiertos
    discovered_peers: Arc<Mutex<Vec<String>>>,
    
    /// Timeout para requests
    request_timeout: Duration,
}

/// Listener para paquetes Discovery
struct DiscoveryListener {
    tx: mpsc::Sender<ReceivedPacket>,
}

#[async_trait::async_trait]
impl PacketListener for DiscoveryListener {
    fn packet_types(&self) -> Vec<u8> {
        vec![PACKET_TYPE_DISCOVERY]
    }
    
    async fn on_packet(&self, packet: ReceivedPacket) {
        println!("📡 [Discovery Listener] Received packet from {}", packet.from);
        let _ = self.tx.send(packet).await;
    }
}

impl Discovery {
    pub async fn new(transport: Arc<Transport>) -> Arc<Self> {
        // Crear canal para recibir paquetes Discovery
        let (discovery_tx, discovery_rx) = mpsc::channel(100);
        
        // Crear listener y registrarlo en Transport
        let listener = Arc::new(DiscoveryListener { tx: discovery_tx });
        transport.register_listener(listener).await;
        
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let next_transaction_id = Arc::new(Mutex::new(0));
        let discovered_peers = Arc::new(Mutex::new(Vec::new()));
        
        let discovery = Arc::new(Self {
            transport,
            discovery_rx: Arc::new(Mutex::new(discovery_rx)),
            pending_requests: pending_requests.clone(),
            next_transaction_id,
            discovered_peers,
            request_timeout: Duration::from_secs(5),
        });
        
        // Iniciar receive loop
        tokio::spawn(receive_loop(
            discovery.discovery_rx.clone(),
            discovery.pending_requests.clone(),
        ));
        
        // Iniciar cleanup loop
        tokio::spawn(cleanup_loop(
            pending_requests.clone(),
        ));
        
        discovery
    }
    
    /// Publicar peer en servidor
    pub async fn publish_peer_id_nametag(
        &self,
        server_addr: SocketAddr,
        nametag: &str,
        peer_id: &str,
    ) -> Result<String> {
        let action = DiscoveryAction::Publish {
            nametag: nametag.to_string(),
            peerid: peer_id.to_string(),
        };
        
        let result = self.send_request(action, server_addr).await?;
        
        match result {
            DiscoveryResult::PublishOk { status } => Ok(status),
            DiscoveryResult::Error { message } => Err(anyhow!("Publish failed: {}", message)),
            _ => Err(anyhow!("Unexpected response type")),
        }
    }
    
    /// Descubrir peer por nametag
    pub async fn discover_peers_by_nametag(
        &self,
        server_addr: SocketAddr,
        nametag: &str,
    ) -> Result<PeerInfo> {
        let action = DiscoveryAction::GetPeerId {
            nametag: nametag.to_string(),
        };
        
        let result = self.send_request(action, server_addr).await?;
        
        match result {
            DiscoveryResult::PeerInfo { address, port, peerid } => {
                let peer_info = PeerInfo {
                    address: address.clone(),
                    port,
                    peerid: peerid.clone(),
                };
                
                // Guardar en discovered_peers
                self.discovered_peers.lock().await.push(peerid);
                
                Ok(peer_info)
            }
            DiscoveryResult::Error { message } => {
                Err(anyhow!("GetPeerId failed: {}", message))
            }
            _ => Err(anyhow!("Unexpected response type")),
        }
    }
    
    /// Obtener mi dirección pública
    pub async fn get_my_address(&self, server_addr: SocketAddr) -> Result<String> {
        let action = DiscoveryAction::GetMyAddress;
        
        let result = self.send_request(action, server_addr).await?;
        
        match result {
            DiscoveryResult::MyAddress { address, port } => {
                Ok(format!("{}:{}", address, port))
            }
            DiscoveryResult::Error { message } => {
                Err(anyhow!("GetMyAddress failed: {}", message))
            }
            _ => Err(anyhow!("Unexpected response type")),
        }
    }
    
    /// Obtener peers descubiertos
    pub async fn get_discovered_peers(&self) -> Vec<String> {
        self.discovered_peers.lock().await.clone()
    }
    
    /// Enviar request genérico
    async fn send_request(
        &self,
        action: DiscoveryAction,
        destination: SocketAddr,
    ) -> Result<DiscoveryResult> {
        // 1. Generar transaction_id
        let transaction_id = {
            let mut id = self.next_transaction_id.lock().await;
            *id += 1;
            *id
        };
        
        // 2. Crear oneshot channel
        let (response_tx, response_rx) = oneshot::channel();
        
        // 3. Registrar en HashMap
        self.pending_requests.lock().await.insert(
            transaction_id,
            PendingRequest {
                response_tx,
                created_at: Instant::now(),
            }
        );
        
        // 4. Crear paquete
        let packet = DiscoveryPacket {
            packet_type: PACKET_TYPE_DISCOVERY,
            transaction_id,
            action,
        };
        
        // 5. Serializar con wincode
        let packet_bytes = wincode::serialize(&packet)?;
        
        // 6. Enviar via Transport (unreliable es suficiente)
        self.transport.unreliable_send(packet_bytes, destination).await?;
        
        println!("📤 [Discovery] Sent request #{}", transaction_id);
        
        // 7. Esperar respuesta con timeout
        match tokio::time::timeout(self.request_timeout, response_rx).await {
            Ok(Ok(result)) => {
                println!("✅ [Discovery] Received response #{}", transaction_id);
                Ok(result)
            }
            Ok(Err(_)) => Err(anyhow!("Response channel closed")),
            Err(_) => {
                self.pending_requests.lock().await.remove(&transaction_id);
                Err(anyhow!("Request timeout"))
            }
        }
    }
}

// ============================================================================
// RECEIVE LOOP
// ============================================================================

async fn receive_loop(
    discovery_rx: Arc<Mutex<mpsc::Receiver<ReceivedPacket>>>,
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
) {
    println!("📥 [Discovery] Receive loop started");
    
    let mut rx = discovery_rx.lock().await;
    
    while let Some(packet) = rx.recv().await {
        // Parsear respuesta con wincode
        let response: DiscoveryResponse = match wincode::deserialize::<DiscoveryResponse>(
            &packet.data
        ) {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("[Discovery] Failed to parse response: {}", e);
                continue;
            }
        };
        
        println!("📦 [Discovery] Received response #{} from {}", 
                 response.transaction_id, packet.from);
        
        // Buscar en HashMap
        let mut pending = pending_requests.lock().await;
        if let Some(request) = pending.remove(&response.transaction_id) {
            let _ = request.response_tx.send(response.result);
        } else {
            eprintln!("[Discovery] No pending request for #{}", response.transaction_id);
        }
    }
}

// ============================================================================
// CLEANUP LOOP
// ============================================================================

async fn cleanup_loop(
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    
    loop {
        interval.tick().await;
        
        let mut pending = pending_requests.lock().await;
        let now = Instant::now();
        
        pending.retain(|id, request| {
            let expired = now.duration_since(request.created_at) > Duration::from_secs(30);
            if expired {
                println!("🧹 [Discovery] Cleaned up expired request #{}", id);
            }
            !expired
        });
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use p2p_core::Core;
    use p2p_transport::TransportConfig;
    
    #[tokio::test]
    async fn test_discovery_with_transport() -> Result<()> {
        let core = Core::bind("127.0.0.1:0".parse()?).await?;
        let transport = Transport::new(core, TransportConfig::default());
        
        let discovery = Discovery::new(transport).await;
        
        // El listener está registrado
        // Cuando llegue un paquete 0x01, se enrutará al canal de Discovery
        
        Ok(())
    }
}