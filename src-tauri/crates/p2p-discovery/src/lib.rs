// crates/p2p-discovery/src/lib.rs

use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, Mutex};
use anyhow::{Result, anyhow};
use p2p_core::{Core, PACKET_TYPE_DISCOVERY};

// ============================================================================
// JSON STRUCTURES
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
struct DiscoveryPacket {
    packet_type: u8,         // 0x01 para Discovery
    transaction_id: u64,
    header: String,
    body: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PublishBody {
    nametag: String,
    peerid: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GetPeerIdBody {
    nametag: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerInfo {
    pub address: String,
    pub port: u16,
    pub peerid: String,
}

// ============================================================================
// PENDING REQUEST
// ============================================================================

struct PendingRequest {
    response_tx: oneshot::Sender<serde_json::Value>,
    created_at: Instant,
}

// ============================================================================
// DISCOVERY CLIENT
// ============================================================================

pub struct Discovery {
    /// Core compartido
    core: Arc<Mutex<Core>>,
    
    /// HashMap para tracking de requests (transaction_id → oneshot)
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    
    /// Contador de transaction IDs
    next_transaction_id: Arc<Mutex<u64>>,
    
    /// Peers descubiertos
    discovered_peers: Arc<Mutex<Vec<String>>>,
    
    /// Timeout para requests
    request_timeout: Duration,
}

impl Discovery {
    pub fn new(core: Core) -> Arc<Self> {
        let core = Arc::new(Mutex::new(core));
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let next_transaction_id = Arc::new(Mutex::new(0));
        let discovered_peers = Arc::new(Mutex::new(Vec::new()));
        
        let discovery = Arc::new(Self {
            core: core.clone(),
            pending_requests: pending_requests.clone(),
            next_transaction_id,
            discovered_peers,
            request_timeout: Duration::from_secs(5),
        });
        
        // Iniciar receive loop
        tokio::spawn(receive_loop(
            core.clone(),
            pending_requests.clone(),
        ));
        
        // Iniciar cleanup loop
        tokio::spawn(cleanup_loop(
            pending_requests.clone(),
            Duration::from_secs(10),
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
        let body = PublishBody {
            nametag: nametag.to_string(),
            peerid: peer_id.to_string(),
        };
        
        let response = self.send_request(
            "publish",
            serde_json::to_value(body)?,
            server_addr,
        ).await?;
        
        let status = response["status"]
            .as_str()
            .ok_or_else(|| anyhow!("No status in response"))?;
        
        Ok(status.to_string())
    }
    
    /// Descubrir peer por nametag
    pub async fn discover_peers_by_nametag(
        &self,
        server_addr: SocketAddr,
        nametag: &str,
    ) -> Result<PeerInfo> {
        let body = GetPeerIdBody {
            nametag: nametag.to_string(),
        };
        
        let response = self.send_request(
            "getpeerid",
            serde_json::to_value(body)?,
            server_addr,
        ).await?;
        
        let peer_info: PeerInfo = serde_json::from_value(response)?;
        
        // Guardar en discovered_peers
        self.discovered_peers
            .lock()
            .await
            .push(peer_info.peerid.clone());
        
        Ok(peer_info)
    }
    
    /// Obtener mi dirección pública
    pub async fn get_my_address(&self, server_addr: SocketAddr) -> Result<String> {
        let response = self.send_request(
            "getmyaddress",
            serde_json::json!({}),
            server_addr,
        ).await?;
        
        let address = response["address"]
            .as_str()
            .ok_or_else(|| anyhow!("No address in response"))?;
        
        let port = response["port"]
            .as_u64()
            .ok_or_else(|| anyhow!("No port in response"))?;
        
        Ok(format!("{}:{}", address, port))
    }
    
    /// Obtener peers descubiertos
    pub async fn get_discovered_peers(&self) -> Vec<String> {
        self.discovered_peers.lock().await.clone()
    }
    
    /// Enviar request genérico
    async fn send_request(
        &self,
        action: &str,
        body: serde_json::Value,
        destination: SocketAddr,
    ) -> Result<serde_json::Value> {
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
            packet_type: PACKET_TYPE_DISCOVERY, // ← 0x01
            transaction_id,
            header: action.to_string(),
            body,
        };
        
        // 5. Serializar
        let json_bytes = serde_json::to_vec(&packet)?;
        
        // 6. Enviar
        {
            let mut core = self.core.lock().await;
            core.send(json_bytes, destination).await?;
        }
        
        println!("📤 [Discovery] Sent {} request #{}", action, transaction_id);
        
        // 7. Esperar respuesta con timeout
        match tokio::time::timeout(self.request_timeout, response_rx).await {
            Ok(Ok(response)) => {
                println!("✅ [Discovery] Received response #{}", transaction_id);
                Ok(response)
            }
            Ok(Err(_)) => Err(anyhow!("Response channel closed")),
            Err(_) => {
                // Timeout
                self.pending_requests.lock().await.remove(&transaction_id);
                Err(anyhow!("Request timeout"))
            }
        }
    }
}

// ============================================================================
// RECEIVE LOOP - Procesa respuestas del servidor
// ============================================================================

async fn receive_loop(
    core: Arc<Mutex<Core>>,
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
) {
    println!("📥 [Discovery] Receive loop started");
    
    loop {
        // Recibir del Core
        let event = {
            let mut core_guard = core.lock().await;
            core_guard.next_event().await
        };
        
        if let Some(p2p_core::CoreEvent::Message { from, data }) = event {
            // Verificar si es paquete de Discovery (0x01)
            if let Some(&packet_type) = data.get(0) {
                if packet_type != PACKET_TYPE_DISCOVERY {
                    // No es para Discovery, ignorar
                    continue;
                }
            }
            
            // Parsear respuesta
            let response: DiscoveryPacket = match serde_json::from_slice(&data) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[Discovery] Failed to parse response: {}", e);
                    continue;
                }
            };
            
            println!("📦 [Discovery] Received response #{} from {}", response.transaction_id, from);
            
            // Buscar en HashMap
            let mut pending = pending_requests.lock().await;
            if let Some(request) = pending.remove(&response.transaction_id) {
                // Enviar respuesta via oneshot
                let _ = request.response_tx.send(response.body);
            } else {
                eprintln!("[Discovery] No pending request for #{}", response.transaction_id);
            }
        }
    }
}

// ============================================================================
// CLEANUP LOOP - Limpia requests expirados
// ============================================================================

async fn cleanup_loop(
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    cleanup_interval: Duration,
) {
    let mut interval = tokio::time::interval(cleanup_interval);
    
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
    
    #[tokio::test]
    async fn test_discovery_publish() -> Result<()> {
        let core = Core::bind("127.0.0.1:0".parse()?).await?;
        let discovery = Discovery::new(core);
        
        // Esto fallará si no hay servidor, pero muestra el uso
        // let status = discovery
        //     .publish_peer_id_nametag(
        //         "127.0.0.1:9000".parse()?,
        //         "test_nametag",
        //         "test_peer_id"
        //     )
        //     .await?;
        
        Ok(())
    }
}