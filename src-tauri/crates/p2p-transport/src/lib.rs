// crates/p2p-transport/src/lib.rs

use serde::{Serialize, Deserialize};
use wincode::{SchemaWrite, SchemaRead};
use std::net::SocketAddr;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::cmp::Ordering;
use tokio::sync::{mpsc, oneshot, Mutex};
use anyhow::{Result, anyhow};
use p2p_core::Core;

// ============================================================================
// PACKET TYPE CONSTANTS
// ============================================================================

pub const PACKET_TYPE_DISCOVERY: u8 = 0x01;
pub const PACKET_TYPE_TRANSPORT: u8 = 0x02;
pub const PACKET_TYPE_P2P_DATA: u8 = 0x03;
pub const PACKET_TYPE_STUN: u8 = 0x05;
pub const PACKET_TYPE_NAT_PUNCH: u8 = 0x06;

// ============================================================================
// PACKET STRUCTURES
// ============================================================================

#[derive(SchemaWrite, SchemaRead, Clone, Debug)]
pub struct TransportPacket {
    pub packet_type: u8,
    pub sequence: u64,
    pub flags: PacketFlags,
    pub payload: Vec<u8>,
}

#[derive(SchemaWrite, SchemaRead, Clone, Debug, Default)]
pub struct PacketFlags {
    pub reliable: bool,
    pub is_ack: bool,
    pub ordered: bool,
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
// RECEIVED PACKET (para notificadores)
// ============================================================================

#[derive(Clone, Debug)]
pub struct ReceivedPacket {
    pub from: SocketAddr,
    pub data: Vec<u8>,
    pub packet_type: u8,
}

// ============================================================================
// PACKET LISTENER (Trait para extensibilidad)
// ============================================================================

/// Trait que implementan las capas superiores para recibir paquetes
#[async_trait::async_trait]
pub trait PacketListener: Send + Sync {
    /// Tipos de paquete que este listener quiere recibir
    fn packet_types(&self) -> Vec<u8>;
    
    /// Callback cuando llega un paquete del tipo registrado
    async fn on_packet(&self, packet: ReceivedPacket);
}

// ============================================================================
// PACKET ORDERING (para paquetes ordenados)
// ============================================================================

#[derive(Clone, Debug)]
struct OrderedPacket {
    sequence: u64,
    data: Vec<u8>,
    from: SocketAddr,
}

impl Eq for OrderedPacket {}

impl PartialEq for OrderedPacket {
    fn eq(&self, other: &Self) -> bool {
        self.sequence == other.sequence
    }
}

impl Ord for OrderedPacket {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order para BinaryHeap (min-heap)
        other.sequence.cmp(&self.sequence)
    }
}

impl PartialOrd for OrderedPacket {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Buffer para reordenar paquetes
struct ReorderBuffer {
    /// Heap de paquetes pendientes (min-heap)
    pending: BinaryHeap<OrderedPacket>,
    
    /// Próximo sequence esperado
    next_expected: u64,
    
    /// Canal para entregar paquetes ordenados
    output_tx: mpsc::Sender<(Vec<u8>, SocketAddr)>,
}

impl ReorderBuffer {
    fn new(output_tx: mpsc::Sender<(Vec<u8>, SocketAddr)>) -> Self {
        Self {
            pending: BinaryHeap::new(),
            next_expected: 0,
            output_tx,
        }
    }
    
    /// Insertar paquete en el buffer
    async fn insert(&mut self, packet: OrderedPacket) {
        if packet.sequence < self.next_expected {
            // Paquete duplicado o muy viejo, ignorar
            return;
        }
        
        if packet.sequence == self.next_expected {
            // Es el siguiente esperado, entregar inmediatamente
            let _ = self.output_tx.send((packet.data, packet.from)).await;
            self.next_expected += 1;
            
            // Verificar si hay más paquetes consecutivos en el heap
            self.flush_ready().await;
        } else {
            // Paquete futuro, guardar en heap
            self.pending.push(packet);
        }
    }
    
    /// Entregar todos los paquetes consecutivos listos
    async fn flush_ready(&mut self) {
        while let Some(packet) = self.pending.peek() {
            if packet.sequence == self.next_expected {
                let packet = self.pending.pop().unwrap();
                let _ = self.output_tx.send((packet.data, packet.from)).await;
                self.next_expected += 1;
            } else {
                break;
            }
        }
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
    
    /// HashMap para tracking de ACKs
    pending_acks: Arc<Mutex<HashMap<u64, PendingPacket>>>,
    
    /// HashMap para requests-response
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    
    /// Contador de secuencia
    next_sequence: Arc<Mutex<u64>>,
    
    /// Set de sequences ya recibidos (deduplicación)
    received_sequences: Arc<Mutex<HashSet<u64>>>,
    
    /// Listeners registrados (packet_type → Vec<listeners>)
    listeners: Arc<Mutex<HashMap<u8, Vec<Arc<dyn PacketListener>>>>>,
    
    /// Buffer de reordenamiento (por peer)
    reorder_buffers: Arc<Mutex<HashMap<SocketAddr, ReorderBuffer>>>,
    
    /// Configuración
    config: TransportConfig,
}

#[derive(Clone)]
pub struct TransportConfig {
    pub max_retries: u32,
    pub retry_timeout: Duration,
    pub request_timeout: Duration,
    pub dedup_window_size: usize,  // Máximo de sequences a recordar
    pub enable_ordering: bool,      // Activar reordenamiento
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            retry_timeout: Duration::from_millis(200),
            request_timeout: Duration::from_secs(5),
            dedup_window_size: 10000,
            enable_ordering: true,
        }
    }
}

impl Transport {
    pub fn new(core: Core, config: TransportConfig) -> Arc<Self> {
        let core = Arc::new(Mutex::new(core));
        let pending_acks = Arc::new(Mutex::new(HashMap::new()));
        let pending_requests = Arc::new(Mutex::new(HashMap::new()));
        let next_sequence = Arc::new(Mutex::new(0));
        let received_sequences = Arc::new(Mutex::new(HashSet::new()));
        let listeners = Arc::new(Mutex::new(HashMap::new()));
        let reorder_buffers = Arc::new(Mutex::new(HashMap::new()));
        
        let transport = Arc::new(Self {
            core: core.clone(),
            pending_acks: pending_acks.clone(),
            pending_requests: pending_requests.clone(),
            next_sequence,
            received_sequences: received_sequences.clone(),
            listeners: listeners.clone(),
            reorder_buffers: reorder_buffers.clone(),
            config: config.clone(),
        });
        
        // Iniciar receive loop
        tokio::spawn(receive_loop(
            core.clone(),
            pending_acks.clone(),
            pending_requests.clone(),
            received_sequences.clone(),
            listeners.clone(),
            reorder_buffers.clone(),
            config.clone(),
        ));
        
        // Iniciar retransmit loop
        tokio::spawn(retransmit_loop(
            core.clone(),
            pending_acks.clone(),
            config.clone(),
        ));
        
        // Iniciar cleanup loop
        tokio::spawn(cleanup_loop(
            received_sequences.clone(),
            config.dedup_window_size,
        ));
        
        transport
    }
    
    /// Registrar listener para tipos de paquete específicos
    pub async fn register_listener(&self, listener: Arc<dyn PacketListener>) {
        let mut listeners_map = self.listeners.lock().await;
        
        for packet_type in listener.packet_types() {
            listeners_map
                .entry(packet_type)
                .or_insert_with(Vec::new)
                .push(listener.clone());
        }
        
        println!("📡 [Transport] Registered listener for types: {:?}", listener.packet_types());
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
        
        let (response_tx, response_rx) = oneshot::channel();
        
        self.pending_requests.lock().await.insert(
            sequence,
            PendingRequest {
                response_tx,
                created_at: Instant::now(),
            }
        );
        
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
        
        {
            let mut core = self.core.lock().await;
            core.send(bytes, destination).await?;
        }
        
        self.pending_acks.lock().await.insert(
            sequence,
            PendingPacket {
                packet: packet.clone(),
                destination,
                sent_at: Instant::now(),
                retries: 0,
            }
        );
        
        match tokio::time::timeout(self.config.request_timeout, response_rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => Err(anyhow!("Response channel closed")),
            Err(_) => {
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
// RECEIVE LOOP
// ============================================================================

async fn receive_loop(
    core: Arc<Mutex<Core>>,
    pending_acks: Arc<Mutex<HashMap<u64, PendingPacket>>>,
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    received_sequences: Arc<Mutex<HashSet<u64>>>,
    listeners: Arc<Mutex<HashMap<u8, Vec<Arc<dyn PacketListener>>>>>,
    reorder_buffers: Arc<Mutex<HashMap<SocketAddr, ReorderBuffer>>>,
    config: TransportConfig,
) {
    println!("📥 [Transport] Receive loop started");
    
    loop {
        let event = {
            let mut core_guard = core.lock().await;
            core_guard.next_event().await
        };
        
        if let Some(p2p_core::CoreEvent::Message { from, data }) = event {
            let packet_type = data.get(0).copied().unwrap_or(PACKET_TYPE_P2P_DATA);
            
            // Notificar a listeners registrados para este tipo
            {
                let listeners_map = listeners.lock().await;
                if let Some(listener_list) = listeners_map.get(&packet_type) {
                    let received = ReceivedPacket {
                        from,
                        data: data.clone(),
                        packet_type,
                    };
                    
                    for listener in listener_list {
                        listener.on_packet(received.clone()).await;
                    }
                }
            }
            
            // Procesar paquetes Transport (0x02)
            if packet_type == PACKET_TYPE_TRANSPORT {
                let packet = match TransportPacket::from_bytes(&data) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("[Transport] Failed to parse packet: {}", e);
                        continue;
                    }
                };
                
                // DEDUPLICACIÓN: Verificar si ya recibimos este sequence
                {
                    let mut received = received_sequences.lock().await;
                    
                    if !packet.flags.is_ack && received.contains(&packet.sequence) {
                        println!("🔁 [Transport] Duplicate packet seq={}, ignoring", packet.sequence);
                        
                        // Pero reenviar ACK por si se perdió
                        if packet.flags.reliable {
                            let transport_temp = Transport {
                                core: core.clone(),
                                pending_acks: pending_acks.clone(),
                                pending_requests: pending_requests.clone(),
                                next_sequence: Arc::new(Mutex::new(0)),
                                received_sequences: received_sequences.clone(),
                                listeners: listeners.clone(),
                                reorder_buffers: reorder_buffers.clone(),
                                config: config.clone(),
                            };
                            
                            let _ = transport_temp.send_ack(packet.sequence, from).await;
                        }
                        
                        continue;
                    }
                    
                    // Marcar como recibido
                    if !packet.flags.is_ack {
                        received.insert(packet.sequence);
                    }
                }
                
                if packet.flags.is_ack {
                    // Es ACK
                    pending_acks.lock().await.remove(&packet.sequence);
                    println!("✅ [Transport] ACK received for seq={}", packet.sequence);
                } else if packet.flags.reliable {
                    // Es DATA confiable
                    println!("📦 [Transport] Received reliable packet seq={} from {}", packet.sequence, from);
                    
                    // Enviar ACK
                    let transport_temp = Transport {
                        core: core.clone(),
                        pending_acks: pending_acks.clone(),
                        pending_requests: pending_requests.clone(),
                        next_sequence: Arc::new(Mutex::new(0)),
                        received_sequences: received_sequences.clone(),
                        listeners: listeners.clone(),
                        reorder_buffers: reorder_buffers.clone(),
                        config: config.clone(),
                    };
                    
                    if let Err(e) = transport_temp.send_ack(packet.sequence, from).await {
                        eprintln!("[Transport] Failed to send ACK: {}", e);
                    }
                    
                    // Procesar payload
                    if packet.flags.ordered && config.enable_ordering {
                        // ORDENAMIENTO: Pasar al buffer de reordenamiento
                        let mut buffers = reorder_buffers.lock().await;
                        
                        if !buffers.contains_key(&from) {
                            let (output_tx, mut output_rx) = mpsc::channel(100);
                            
                            // Spawn task para procesar paquetes ordenados
                            let pending_reqs = pending_requests.clone();
                            tokio::spawn(async move {
                                while let Some((payload, source)) = output_rx.recv().await {
                                    // Verificar si hay request pendiente
                                    // (asumimos que el sequence viene en el payload o lo inferimos)
                                    // Por simplicidad, usamos el mismo sequence
                                    println!("📬 [Transport] Ordered packet delivered from {}", source);
                                }
                            });
                            
                            buffers.insert(from, ReorderBuffer::new(output_tx));
                        }
                        
                        let buffer = buffers.get_mut(&from).unwrap();
                        buffer.insert(OrderedPacket {
                            sequence: packet.sequence,
                            data: packet.payload.clone(),
                            from,
                        }).await;
                    } else {
                        // Sin ordenamiento, procesar inmediatamente
                        let mut requests = pending_requests.lock().await;
                        if let Some(request) = requests.remove(&packet.sequence) {
                            let _ = request.response_tx.send(packet.payload);
                            println!("📬 [Transport] Response sent for seq={}", packet.sequence);
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// RETRANSMIT LOOP
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
                    println!("❌ [Transport] Packet seq={} failed after {} retries", seq, packet_info.retries);
                    to_remove.push(*seq);
                } else {
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
        
        for seq in to_remove {
            pending.remove(&seq);
        }
    }
}

// ============================================================================
// CLEANUP LOOP
// ============================================================================

async fn cleanup_loop(
    received_sequences: Arc<Mutex<HashSet<u64>>>,
    max_size: usize,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        let mut received = received_sequences.lock().await;
        
        if received.len() > max_size {
            // Remover los más viejos (asumiendo que son los menores)
            let to_remove = received.len() - max_size;
            let min_sequences: Vec<u64> = received.iter()
                .copied()
                .take(to_remove)
                .collect();
            
            for seq in min_sequences {
                received.remove(&seq);
            }
            
            println!("🧹 [Transport] Cleaned {} old sequences", to_remove);
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
    async fn test_deduplication() -> Result<()> {
        let core = Core::bind("127.0.0.1:0".parse()?).await?;
        let transport = Transport::new(core, TransportConfig::default());
        
        // Simular recepción del mismo sequence dos veces
        // (esto requeriría acceso interno o un servidor de prueba)
        
        Ok(())
    }
}