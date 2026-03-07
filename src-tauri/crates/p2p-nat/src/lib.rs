// crates/p2p-nat/src/lib.rs

use p2p_transport::{Transport, PacketListener, ReceivedPacket};
use p2p_transport::{PACKET_TYPE_STUN, PACKET_TYPE_NAT_PUNCH};
use wincode::{SchemaWrite, SchemaRead};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

// ============================================================================
// STUN PROTOCOL
// ============================================================================

#[derive(SchemaWrite, SchemaRead, Debug)]
struct StunBindingRequest {
    packet_type: u8,  // 0x05
    transaction_id: u64,
}

#[derive(SchemaWrite, SchemaRead, Debug)]
struct StunBindingResponse {
    packet_type: u8,  // 0x05
    transaction_id: u64,
    mapped_address: String,
    mapped_port: u16,
}

// ============================================================================
// NAT PUNCH PROTOCOL
// ============================================================================

#[derive(SchemaWrite, SchemaRead, Debug)]
struct PunchPacket {
    packet_type: u8,  // 0x06
    peer_id: String,
    timestamp: u64,
}

// ============================================================================
// STUN CLIENT - Usa Transport via PacketListener
// ============================================================================

pub struct StunClient {
    /// Transport compartido
    transport: Arc<Transport>,
    
    /// Canal para recibir respuestas STUN
    stun_rx: Arc<Mutex<mpsc::Receiver<ReceivedPacket>>>,
    
    /// Servidores STUN
    stun_servers: Vec<String>,
    
    /// Timeout para STUN
    timeout: Duration,
}

/// Listener para paquetes STUN
struct StunListener {
    tx: mpsc::Sender<ReceivedPacket>,
}

#[async_trait::async_trait]
impl PacketListener for StunListener {
    fn packet_types(&self) -> Vec<u8> {
        vec![PACKET_TYPE_STUN]
    }
    
    async fn on_packet(&self, packet: ReceivedPacket) {
        println!("🔵 [STUN Listener] Received STUN packet from {}", packet.from);
        let _ = self.tx.send(packet).await;
    }
}

impl StunClient {
    pub async fn new(transport: Arc<Transport>) -> Arc<Self> {
        // Crear canal para recibir paquetes STUN
        let (stun_tx, stun_rx) = mpsc::channel(100);
        
        // Crear listener y registrarlo en Transport
        let listener = Arc::new(StunListener { tx: stun_tx });
        transport.register_listener(listener).await;
        
        Arc::new(Self {
            transport,
            stun_rx: Arc::new(Mutex::new(stun_rx)),
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
            ],
            timeout: Duration::from_secs(3),
        })
    }
    
    /// Detectar IP pública usando STUN
    pub async fn get_public_address(&self) -> Result<SocketAddr> {
        let transaction_id = rand::random::<u64>();
        
        // Crear STUN Binding Request
        let request = StunBindingRequest {
            packet_type: PACKET_TYPE_STUN,
            transaction_id,
        };
        
        let request_bytes = wincode::serialize(&request)?;
        
        // Enviar a servidor STUN via Transport (unreliable es suficiente)
        let stun_addr: SocketAddr = self.stun_servers[0].parse()?;
        
        println!("📤 [STUN] Sending Binding Request to {}", stun_addr);
        
        self.transport.unreliable_send(request_bytes, stun_addr).await?;
        
        // Esperar respuesta en canal STUN
        let response = self.wait_for_stun_response(transaction_id).await?;
        
        let mapped_addr = format!("{}:{}", response.mapped_address, response.mapped_port);
        
        println!("✅ [STUN] Public address: {}", mapped_addr);
        
        Ok(mapped_addr.parse()?)
    }
    
    /// Detectar tipo de NAT
    pub async fn detect_nat_type(&self) -> Result<NatType> {
        // TODO: Implementar detección de tipo de NAT
        // Requiere múltiples pruebas STUN
        
        println!("🔍 [STUN] Detecting NAT type...");
        
        Ok(NatType::Unknown)
    }
    
    /// Esperar respuesta STUN con transaction_id específico
    async fn wait_for_stun_response(
        &self,
        transaction_id: u64,
    ) -> Result<StunBindingResponse> {
        let mut rx = self.stun_rx.lock().await;
        
        let start = std::time::Instant::now();
        
        loop {
            if start.elapsed() > self.timeout {
                return Err(anyhow!("STUN timeout"));
            }
            
            match tokio::time::timeout(
                self.timeout - start.elapsed(),
                rx.recv()
            ).await {
                Ok(Some(packet)) => {
                    // Parsear respuesta
                    let response: StunBindingResponse = wincode::deserialize(&packet.data)?;
                    
                    if response.transaction_id == transaction_id {
                        return Ok(response);
                    } else {
                        println!("⚠️ [STUN] Mismatched transaction_id, waiting for correct one");
                    }
                }
                Ok(None) => {
                    return Err(anyhow!("STUN channel closed"));
                }
                Err(_) => {
                    return Err(anyhow!("STUN timeout"));
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    OpenInternet,
    FullCone,
    RestrictedCone,
    PortRestrictedCone,
    Symmetric,
    Unknown,
}

// ============================================================================
// HOLE PUNCHER - Usa Transport via PacketListener
// ============================================================================

pub struct HolePuncher {
    /// Transport compartido
    transport: Arc<Transport>,
    
    /// Canal para recibir paquetes PUNCH
    punch_rx: Arc<Mutex<mpsc::Receiver<ReceivedPacket>>>,
    
    /// Mi peer ID
    my_peer_id: String,
    
    /// Timeout para hole punching
    timeout: Duration,
}

/// Listener para paquetes PUNCH
struct PunchListener {
    tx: mpsc::Sender<ReceivedPacket>,
}

#[async_trait::async_trait]
impl PacketListener for PunchListener {
    fn packet_types(&self) -> Vec<u8> {
        vec![PACKET_TYPE_NAT_PUNCH]
    }
    
    async fn on_packet(&self, packet: ReceivedPacket) {
        println!("👊 [PUNCH Listener] Received PUNCH packet from {}", packet.from);
        let _ = self.tx.send(packet).await;
    }
}

impl HolePuncher {
    pub async fn new(transport: Arc<Transport>, my_peer_id: String) -> Arc<Self> {
        // Crear canal para recibir paquetes PUNCH
        let (punch_tx, punch_rx) = mpsc::channel(100);
        
        // Crear listener y registrarlo en Transport
        let listener = Arc::new(PunchListener { tx: punch_tx });
        transport.register_listener(listener).await;
        
        Arc::new(Self {
            transport,
            punch_rx: Arc::new(Mutex::new(punch_rx)),
            my_peer_id,
            timeout: Duration::from_secs(5),
        })
    }
    
    /// Realizar hole punching con peer
    pub async fn punch(
        &self,
        peer_local: SocketAddr,
        peer_public: SocketAddr,
    ) -> Result<SocketAddr> {
        println!("🔓 [PUNCH] Starting hole punching...");
        println!("   Peer local:  {}", peer_local);
        println!("   Peer public: {}", peer_public);
        
        // Crear paquete PUNCH
        let punch = PunchPacket {
            packet_type: PACKET_TYPE_NAT_PUNCH,
            peer_id: self.my_peer_id.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        let punch_bytes = wincode::serialize(&punch)?;
        
        // Enviar PUNCH a ambas direcciones simultáneamente
        let transport_local = self.transport.clone();
        let transport_public = self.transport.clone();
        let punch_local = punch_bytes.clone();
        let punch_public = punch_bytes.clone();
        
        // Task para enviar a dirección local
        let local_handle = tokio::spawn(async move {
            for i in 0..10 {
                if let Err(e) = transport_local.unreliable_send(
                    punch_local.clone(),
                    peer_local
                ).await {
                    eprintln!("[PUNCH] Local send failed: {}", e);
                }
                
                println!("👊 [PUNCH] Sent to local (attempt {})", i + 1);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        
        // Task para enviar a dirección pública
        let public_handle = tokio::spawn(async move {
            for i in 0..10 {
                if let Err(e) = transport_public.unreliable_send(
                    punch_public.clone(),
                    peer_public
                ).await {
                    eprintln!("[PUNCH] Public send failed: {}", e);
                }
                
                println!("👊 [PUNCH] Sent to public (attempt {})", i + 1);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        
        // Esperar respuesta PUNCH
        let result = self.wait_for_punch_response().await;
        
        // Cancelar tasks de envío
        local_handle.abort();
        public_handle.abort();
        
        match result {
            Ok(peer_addr) => {
                println!("✅ [PUNCH] Hole punching succeeded! Connected to: {}", peer_addr);
                Ok(peer_addr)
            }
            Err(e) => {
                println!("❌ [PUNCH] Hole punching failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Esperar respuesta PUNCH
    async fn wait_for_punch_response(&self) -> Result<SocketAddr> {
        let mut rx = self.punch_rx.lock().await;
        
        match tokio::time::timeout(self.timeout, rx.recv()).await {
            Ok(Some(packet)) => {
                // Parsear paquete PUNCH
                let punch: PunchPacket = wincode::deserialize(&packet.data)?;
                
                println!("✅ [PUNCH] Received PUNCH from peer: {}", punch.peer_id);
                
                Ok(packet.from)
            }
            Ok(None) => {
                Err(anyhow!("PUNCH channel closed"))
            }
            Err(_) => {
                Err(anyhow!("PUNCH timeout - no response from peer"))
            }
        }
    }
}

// ============================================================================
// NAT TRAVERSAL (Módulo completo)
// ============================================================================

pub struct NatTraversal {
    stun_client: Arc<StunClient>,
    hole_puncher: Arc<HolePuncher>,
}

impl NatTraversal {
    pub async fn new(transport: Arc<Transport>, my_peer_id: String) -> Arc<Self> {
        let stun = StunClient::new(transport.clone()).await;
        let puncher = HolePuncher::new(transport.clone(), my_peer_id).await;
        
        Arc::new(Self {
            stun_client: stun,
            hole_puncher: puncher,
        })
    }
    
    /// Detectar IP pública
    pub async fn get_public_address(&self) -> Result<SocketAddr> {
        self.stun_client.get_public_address().await
    }
    
    /// Detectar tipo de NAT
    pub async fn detect_nat_type(&self) -> Result<NatType> {
        self.stun_client.detect_nat_type().await
    }
    
    /// Conectar con peer via hole punching
    pub async fn connect_to_peer(
        &self,
        peer_local: SocketAddr,
        peer_public: SocketAddr,
    ) -> Result<SocketAddr> {
        self.hole_puncher.punch(peer_local, peer_public).await
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
    async fn test_stun_listener() -> Result<()> {
        let core = Core::bind("127.0.0.1:0".parse()?).await?;
        let transport = Transport::new(core, TransportConfig::default());
        
        let stun = StunClient::new(transport.clone()).await;
        
        // El listener está registrado
        // Cuando llegue un paquete 0x05, se enrutará al canal de STUN
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_hole_puncher_listener() -> Result<()> {
        let core = Core::bind("127.0.0.1:0".parse()?).await?;
        let transport = Transport::new(core, TransportConfig::default());
        
        let puncher = HolePuncher::new(transport.clone(), "test_peer".to_string()).await;
        
        // El listener está registrado
        // Cuando llegue un paquete 0x06, se enrutará al canal de PUNCH
        
        Ok(())
    }
}