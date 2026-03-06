// crates/p2p-transport/src/lib.rs

use serde::{Serialize, Deserialize};
use wincode::{SchemaWrite, SchemaRead};
use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, Mutex};
use anyhow::{Result, anyhow};
use p2p_core::Core;

// ============================================================================
// PACKET STRUCTURES
// ============================================================================

const PACKET_TYPE_DISCOVERY: u8 = 0x01;
const PACKET_TYPE_TRANSPORT: u8 = 0x02;
const PACKET_TYPE_P2P_DATA: u8 = 0x03;

#[derive(SchemaWrite, SchemaRead, Clone, Debug)]
pub struct TransportPacket {
    pub packet_type: u8,      // 0x02 para Transport
    pub sequence: u64,
    pub flags: PacketFlags,
    pub payload: Vec<u8>,
}

#[derive(SchemaWrite, SchemaRead, Clone, Debug, Default)]
pub struct PacketFlags {
    pub reliable: bool,    // Requiere ACK
    pub is_ack: bool,      // Es un ACK
    pub ordered: bool,     // Mantener orden
}

impl TransportPacket {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        wincode::serialize(self).map_err(Into::into)
    }
    
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        wincode::deserialize(data).map_err(Into::into)
    }
}

// ============================================================================
// PENDING STRUCTURES
// ============================================================================

struct PendingPacket {
    packet: TransportPacket,
    destination: SocketAddr,
    sent_at: Instant,
    retries: u32,
}

struct PendingRequest {
    response_tx: oneshot::Sender<Vec<u8>>,
    created_at: Instant,
}

// ============================================================================
// TRANSPORT LAYER
// ============================================================================

pub struct Transport {
    /// Core para enviar/recibir
    core: Arc<Mutex<Core>>,
    
    /// HashMap para tracking de ACKs (sequence → PendingPacket)
    pending_acks: Arc<Mutex<HashMap<u64, PendingPacket>>>,
    
    /// HashMap para requests-response (sequence → oneshot)
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    
    /// Contador de secuencia
    next_sequence: Arc<Mutex<u64>>,
    
    /// Configuración
    config: TransportConfig,
}

#[derive(Clone)]
pub struct TransportConfig {
    pub max_retries: u32,
    pub retry_timeout: Duration,
    pub request_timeout: Duration,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            retry_timeout: Duration::from_millis(200),
            request_timeout: Duration::from_secs(5),
        }
    }
}

impl Transport {
    pub fn new(core: Core, config: TransportConfig) -> Arc<Self> {
        let core = Arc::new(Mutex::new(core));
        let pending_acks = Arc::new(Mutex::new(HashMap::new()));
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let next_sequence = Arc::new(Mutex::new(0));
        
        let transport = Arc::new(Self {
            core: core.clone(),
            pending_acks: pending_acks.clone(),
            pending_requests: pending_requests.clone(),
            next_sequence,
            config: config.clone(),
        });
        
        // Iniciar receive loop
        tokio::spawn(receive_loop(
            core.clone(),
            pending_acks.clone(),
            pending_requests.clone(),
        ));
        
        // Iniciar retransmit loop
        tokio::spawn(retransmit_loop(
            core.clone(),
            pending_acks.clone(),
            config,
        ));
        
        transport
    }
    
    /// Enviar paquete no confiable (fire-and-forget)
    pub async fn unreliable_send(
        &self,
        data: Vec<u8>,
        destination: SocketAddr,
    ) -> Result<()> {
        let sequence = self.next_sequence().await;
        
        let packet = TransportPacket {
            packet_type: PACKET_TYPE_P2P_DATA,
            sequence,
            flags: PacketFlags {
                reliable: false,
                is_ack: false,
                ordered: false,
            },
            payload: data,
        };
        
        let bytes = packet.to_bytes()?;
        
        let mut core = self.core.lock().await;
        core.send(bytes, destination).await?;
        
        Ok(())
    }
    
    /// Enviar paquete confiable (con ACK)
    pub async fn reliable_send(
        &self,
        data: Vec<u8>,
        destination: SocketAddr,
    ) -> Result<()> {
        let sequence = self.next_sequence().await;
        
        let packet = TransportPacket {
            packet_type: PACKET_TYPE_TRANSPORT,
            sequence,
            flags: PacketFlags {
                reliable: true,
                is_ack: false,
                ordered: true,
            },
            payload: data,
        };
        
        let bytes = packet.to_bytes()?;
        
        // Enviar
        {
            let mut core = self.core.lock().await;
            core.send(bytes, destination).await?;
        }
        
        // Registrar para reenvío si no llega ACK
        self.pending_acks.lock().await.insert(
            sequence,
            PendingPacket {
                packet: packet.clone(),
                destination,
                sent_at: Instant::now(),
                retries: 0,
            }
        );
        
        Ok(())
    }
    
    /// Enviar request y esperar response
    pub async fn send_request(
        &self,
        data: Vec<u8>,
        destination: SocketAddr,
    ) -> Result<Vec<u8>> {
        let sequence = self.next_sequence().await;
        
        // Crear oneshot channel para la respuesta
        let (response_tx, response_rx) = oneshot::channel();
        
        // Registrar request
        self.pending_requests.lock().await.insert(
            sequence,
            PendingRequest {
                response_tx,
                created_at: Instant::now(),
            }
        );
        
        // Crear paquete
        let packet = TransportPacket {
            packet_type: PACKET_TYPE_TRANSPORT,
            sequence,
            flags: PacketFlags {
                reliable: true,
                is_ack: false,
                ordered: true,
            },
            payload: data,
        };
        
        let bytes = packet.to_bytes()?;
        
        // Enviar
        {
            let mut core = self.core.lock().await;
            core.send(bytes, destination).await?;
        }
        
        // Registrar para ACK
        self.pending_acks.lock().await.insert(
            sequence,
            PendingPacket {
                packet: packet.clone(),
                destination,
                sent_at: Instant::now(),
                retries: 0,
            }
        );
        
        // Esperar respuesta con timeout
        match tokio::time::timeout(self.config.request_timeout, response_rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(anyhow!("Response channel closed")),
            Err(_) => {
                // Timeout - limpiar
                self.pending_requests.lock().await.remove(&sequence);
                self.pending_acks.lock().await.remove(&sequence);
                Err(anyhow!("Request timeout"))
            }
        }
    }
    
    /// Generar siguiente sequence number
    async fn next_sequence(&self) -> u64 {
        let mut seq = self.next_sequence.lock().await;
        *seq += 1;
        *seq
    }
    
    /// Enviar ACK
    async fn send_ack(&self, sequence: u64, destination: SocketAddr) -> Result<()> {
        let packet = TransportPacket {
            packet_type: PACKET_TYPE_TRANSPORT,
            sequence,
            flags: PacketFlags {
                reliable: false,
                is_ack: true,
                ordered: false,
            },
            payload: vec![],
        };
        
        let bytes = packet.to_bytes()?;
        
        let mut core = self.core.lock().await;
        core.send(bytes, destination).await?;
        
        Ok(())
    }
}

// ============================================================================
// RECEIVE LOOP - Procesa paquetes entrantes
// ============================================================================

async fn receive_loop(
    core: Arc<Mutex<Core>>,
    pending_acks: Arc<Mutex<HashMap<u64, PendingPacket>>>,
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
) {
    println!("📥 [Transport] Receive loop started");
    
    loop {
        // Recibir paquete del Core
        let event = {
            let mut core_guard = core.lock().await;
            core_guard.next_event().await
        };
        
        if let Some(p2p_core::CoreEvent::Message { from, data }) = event {
            // Verificar si es paquete de Transport (0x02)
            if let Some(&packet_type) = data.get(0) {
                if packet_type != PACKET_TYPE_TRANSPORT {
                    // No es para Transport, ignorar
                    continue;
                }
            }
            
            // Parsear paquete
            let packet = match TransportPacket::from_bytes(&data) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("[Transport] Failed to parse packet: {}", e);
                    continue;
                }
            };
            
            println!("📦 [Transport] Received packet seq={} from {}", packet.sequence, from);
            
            // Procesar según tipo
            if packet.flags.is_ack {
                // Es un ACK - remover de pending
                pending_acks.lock().await.remove(&packet.sequence);
                println!("✅ [Transport] ACK received for seq={}", packet.sequence);
            } else if packet.flags.reliable {
                // Es DATA confiable - enviar ACK
                let transport_temp = Transport {
                    core: core.clone(),
                    pending_acks: pending_acks.clone(),
                    pending_requests: pending_requests.clone(),
                    next_sequence: Arc::new(Mutex::new(0)),
                    config: TransportConfig::default(),
                };
                
                if let Err(e) = transport_temp.send_ack(packet.sequence, from).await {
                    eprintln!("[Transport] Failed to send ACK: {}", e);
                }
                
                // Si hay request pendiente, responder
                let mut requests = pending_requests.lock().await;
                if let Some(request) = requests.remove(&packet.sequence) {
                    let _ = request.response_tx.send(packet.payload);
                    println!("📬 [Transport] Response sent for seq={}", packet.sequence);
                }
            }
        }
    }
}

// ============================================================================
// RETRANSMIT LOOP - Reenvía paquetes sin ACK
// ============================================================================

async fn retransmit_loop(
    core: Arc<Mutex<Core>>,
    pending_acks: Arc<Mutex<HashMap<u64, PendingPacket>>>,
    config: TransportConfig,
) {
    println!("🔄 [Transport] Retransmit loop started");
    
    let mut interval = tokio::time::interval(config.retry_timeout / 2);
    
    loop {
        interval.tick().await;
        
        let mut pending = pending_acks.lock().await;
        let now = Instant::now();
        let mut to_remove = Vec::new();
        
        for (seq, packet_info) in pending.iter_mut() {
            if now.duration_since(packet_info.sent_at) > config.retry_timeout {
                if packet_info.retries >= config.max_retries {
                    // Excedió reintentos - remover
                    println!("❌ [Transport] Packet seq={} failed after {} retries", seq, packet_info.retries);
                    to_remove.push(*seq);
                } else {
                    // Reenviar
                    if let Ok(bytes) = packet_info.packet.to_bytes() {
                        let mut core_guard = core.lock().await;
                        if let Err(e) = core_guard.send(bytes, packet_info.destination).await {
                            eprintln!("[Transport] Retransmit failed: {}", e);
                        } else {
                            packet_info.retries += 1;
                            packet_info.sent_at = now;
                            println!("🔁 [Transport] Retransmit seq={} (retry {})", seq, packet_info.retries);
                        }
                    }
                }
            }
        }
        
        // Limpiar fallidos
        for seq in to_remove {
            pending.remove(&seq);
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
    async fn test_unreliable_send() -> Result<()> {
        let core = Core::bind("127.0.0.1:0".parse().unwrap()).await?;
        let transport = Transport::new(core, TransportConfig::default());
        
        let data = b"Hello, unreliable!".to_vec();
        transport.unreliable_send(data, "127.0.0.1:9000".parse()?).await?;
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_reliable_send() -> Result<()> {
        let core = Core::bind("127.0.0.1:0".parse().unwrap()).await?;
        let transport = Transport::new(core, TransportConfig::default());
        
        let data = b"Hello, reliable!".to_vec();
        transport.reliable_send(data, "127.0.0.1:9000".parse()?).await?;
        
        // El paquete quedará en pending_acks esperando ACK
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let pending = transport.pending_acks.lock().await;
        assert!(!pending.is_empty(), "Should have pending ACK");
        
        Ok(())
    }
}