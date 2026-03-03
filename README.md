# 🏗️ Arquitectura P2P - Plan de Desarrollo por Capas

## 🎯 Objetivo del Proyecto

Crear una aplicación P2P robusta que funcione en internet real usando:
- **Hole Punching** para atravesar NATs
- **UDP** para transmisión rápida (con verificación opcional)
- **Arquitectura en capas** modular y escalable
- **Audio en tiempo real** sin latencia
- **Mensajería confiable** con confirmación de entrega

---

## 📊 Arquitectura de Capas

```
┌─────────────────────────────────────────────────────────────┐
│                      Layer 0: UI/Frontend                    │
│                   (React/Tauri Interface)                    │
└─────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────┐
│                  Layer 1: Application Layer                  │
│           (Mensajería, Archivos, Audio, Video)              │
└─────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────┐
│                 Layer 2: Protocol Layer                      │
│         (Serialización, Compresión, Encriptación)           │
└─────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────┐
│               Layer 3: Connection Layer                      │
│         (Gestión de conexiones, Keep-alive, QoS)            │
└─────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────┐
│                Layer 4: Transport Layer                      │
│        (UDP con verificación opcional, Reordenamiento)      │
└─────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────┐
│                 Layer 5: NAT Traversal                       │
│           (Hole Punching, STUN, TURN, ICE)                  │
└─────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────┐
│                Layer 6: Discovery Layer                      │
│         (Servidor de registro, DHT, mDNS local)             │
└─────────────────────────────────────────────────────────────┘
                              ↕
┌─────────────────────────────────────────────────────────────┐
│                 Layer 7: Network Layer                       │
│                  (UDP/TCP Raw Sockets)                       │
└─────────────────────────────────────────────────────────────┘
```

---

## 📦 Módulos del Proyecto (Cargo Workspace)

### Estructura de Carpetas

```
chat-p2p/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── p2p-core/                 # Layer 7: Red cruda
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── udp.rs
│   │       └── tcp.rs
│   │
│   ├── p2p-discovery/            # Layer 6: Descubrimiento
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── mdns.rs
│   │       ├── dht.rs
│   │       └── registry.rs
│   │
│   ├── p2p-nat/                  # Layer 5: NAT Traversal
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── stun.rs
│   │       ├── turn.rs
│   │       ├── ice.rs
│   │       └── hole_punching.rs
│   │
│   ├── p2p-transport/            # Layer 4: Transporte
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── reliable_udp.rs
│   │       ├── unreliable_udp.rs
│   │       ├── packet.rs
│   │       └── congestion.rs
│   │
│   ├── p2p-connection/           # Layer 3: Conexiones
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── manager.rs
│   │       ├── peer.rs
│   │       ├── keepalive.rs
│   │       └── qos.rs
│   │
│   ├── p2p-protocol/             # Layer 2: Protocolo
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── codec.rs
│   │       ├── compression.rs
│   │       ├── encryption.rs
│   │       └── types.rs
│   │
│   ├── p2p-application/          # Layer 1: Aplicación
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── messaging.rs
│   │       ├── audio.rs
│   │       ├── video.rs
│   │       └── files.rs
│   │
│   └── p2p-interface/            # Layer 0: Interfaz
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── tauri_bridge.rs
│           └── events.rs
│
└── src-tauri/                    # Aplicación Tauri
    ├── Cargo.toml
    ├── src/
    │   └── main.rs
    └── tauri.conf.json
```

---

## 🔧 Detalle de Cada Módulo

### **Layer 7: p2p-core** (Network Layer)
**Responsabilidad:** Abstracción sobre sockets UDP/TCP

```toml
[package]
name = "p2p-core"
version = "0.1.0"

[dependencies]
tokio = { version = "1.35", features = ["net", "sync"] }
anyhow = "1.0"
```

**Funciones principales:**
- `bind_udp(addr) -> UdpSocket`
- `send_to(socket, data, addr) -> Result<()>`
- `recv_from(socket) -> Result<(Vec<u8>, SocketAddr)>`
- `bind_tcp(addr) -> TcpListener`

**Comunicación:**
- → **p2p-transport**: Proporciona sockets crudos
- → **p2p-nat**: Usado para STUN/TURN

---

### **Layer 6: p2p-discovery** (Discovery Layer)
**Responsabilidad:** Descubrir peers en red local e internet

```toml
[package]
name = "p2p-discovery"
version = "0.1.0"

[dependencies]
p2p-core = { path = "../p2p-core" }
tokio = { version = "1.35", features = ["full"] }
mdns-sd = "0.10"
serde = { version = "1.0", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
```

**Módulos:**

#### `mdns.rs` - Descubrimiento local
```rust
pub struct MdnsDiscovery {
    service_name: String,
}

impl MdnsDiscovery {
    pub async fn advertise(&self, port: u16) -> Result<()>;
    pub async fn discover(&self) -> Result<Vec<PeerInfo>>;
}
```

#### `registry.rs` - Servidor de registro
```rust
pub struct RegistryClient {
    server_url: String,
}

impl RegistryClient {
    pub async fn register(&self, my_info: PeerInfo) -> Result<()>;
    pub async fn get_peers(&self) -> Result<Vec<PeerInfo>>;
    pub async fn heartbeat(&self) -> Result<()>;
}
```

#### `dht.rs` - Distributed Hash Table (opcional)
```rust
pub struct Dht {
    // Kademlia DHT para descubrimiento sin servidor central
}
```

**Comunicación:**
- ← **p2p-core**: Usa sockets para mDNS
- → **p2p-nat**: Envía lista de peers descubiertos
- → **p2p-interface**: Notifica nuevos peers

---

### **Layer 5: p2p-nat** (NAT Traversal)
**Responsabilidad:** Atravesar NATs y firewalls

```toml
[package]
name = "p2p-nat"
version = "0.1.0"

[dependencies]
p2p-core = { path = "../p2p-core" }
p2p-discovery = { path = "../p2p-discovery" }
tokio = { version = "1.35", features = ["full"] }
stun_codec = "0.3"
webrtc-ice = "0.10"
```

**Módulos:**

#### `stun.rs` - Detectar IP pública y tipo de NAT
```rust
pub struct StunClient {
    stun_servers: Vec<String>,
}

impl StunClient {
    pub async fn get_public_address(&self) -> Result<SocketAddr>;
    pub async fn detect_nat_type(&self) -> Result<NatType>;
}

pub enum NatType {
    Open,              // Sin NAT
    FullCone,          // Mejor caso
    RestrictedCone,    // Bueno
    PortRestricted,    // Complicado
    Symmetric,         // Peor caso - necesita TURN
}
```

#### `hole_punching.rs` - UDP Hole Punching
```rust
pub struct HolePuncher {
    local_socket: UdpSocket,
}

impl HolePuncher {
    // Coordinar con servidor de señalización
    pub async fn initiate_punch(
        &self, 
        peer_public_addr: SocketAddr
    ) -> Result<Connection>;
    
    // Enviar paquetes simultáneos
    pub async fn simultaneous_open(
        &self,
        peer_addr: SocketAddr
    ) -> Result<()>;
}
```

#### `ice.rs` - Interactive Connectivity Establishment
```rust
pub struct IceAgent {
    // Implementar protocolo ICE completo
}

impl IceAgent {
    pub async fn gather_candidates(&self) -> Vec<IceCandidate>;
    pub async fn connect(&self, remote_candidates: Vec<IceCandidate>) -> Result<Connection>;
}
```

#### `turn.rs` - Relay fallback
```rust
pub struct TurnClient {
    turn_servers: Vec<String>,
}

impl TurnClient {
    pub async fn allocate_relay(&self) -> Result<SocketAddr>;
    pub async fn send_via_relay(&self, data: &[u8], peer: PeerId) -> Result<()>;
}
```

**Comunicación:**
- ← **p2p-discovery**: Recibe lista de peers
- ← **p2p-core**: Usa sockets crudos
- → **p2p-transport**: Proporciona conexiones establecidas
- → **p2p-interface**: Notifica estado de NAT

---

### **Layer 4: p2p-transport** (Transport Layer)
**Responsabilidad:** Transporte UDP confiable y no confiable

```toml
[package]
name = "p2p-transport"
version = "0.1.0"

[dependencies]
p2p-core = { path = "../p2p-core" }
p2p-nat = { path = "../p2p-nat" }
tokio = { version = "1.35", features = ["full"] }
bytes = "1.5"
```

**Módulos:**

#### `packet.rs` - Estructura de paquetes
```rust
#[derive(Serialize, Deserialize)]
pub struct Packet {
    pub sequence: u64,
    pub ack: Option<u64>,
    pub timestamp: u64,
    pub flags: PacketFlags,
    pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct PacketFlags {
    pub reliable: bool,      // Requiere ACK
    pub ordered: bool,       // Mantener orden
    pub fragmented: bool,    // Es fragmento de mensaje grande
    pub priority: Priority,  // Alta/Media/Baja
}

pub enum Priority {
    High,    // Audio/Video en tiempo real
    Medium,  // Mensajes de chat
    Low,     // Transferencia de archivos
}
```

#### `reliable_udp.rs` - UDP con garantías
```rust
pub struct ReliableChannel {
    socket: UdpSocket,
    pending_acks: HashMap<u64, PendingPacket>,
    received: HashSet<u64>,
}

impl ReliableChannel {
    pub async fn send_reliable(&mut self, data: &[u8]) -> Result<()>;
    pub async fn recv_reliable(&mut self) -> Result<Vec<u8>>;
    
    // Reenviar paquetes sin ACK
    async fn retransmit(&mut self);
    
    // Procesar ACKs recibidos
    async fn handle_ack(&mut self, ack: u64);
}
```

#### `unreliable_udp.rs` - UDP sin garantías (audio/video)
```rust
pub struct UnreliableChannel {
    socket: UdpSocket,
}

impl UnreliableChannel {
    pub async fn send_unreliable(&self, data: &[u8]) -> Result<()>;
    pub async fn recv_unreliable(&self) -> Result<Vec<u8>>;
}
```

#### `congestion.rs` - Control de congestión
```rust
pub struct CongestionController {
    rtt: Duration,
    bandwidth: u64,
    loss_rate: f32,
}

impl CongestionController {
    pub fn update_rtt(&mut self, rtt: Duration);
    pub fn calculate_send_rate(&self) -> u64;
    pub fn should_throttle(&self) -> bool;
}
```

**Comunicación:**
- ← **p2p-nat**: Recibe conexiones establecidas
- → **p2p-connection**: Proporciona canales de transporte
- → **p2p-interface**: Métricas de red (RTT, pérdida)

---

### **Layer 3: p2p-connection** (Connection Layer)
**Responsabilidad:** Gestionar múltiples conexiones simultáneas

```toml
[package]
name = "p2p-connection"
version = "0.1.0"

[dependencies]
p2p-transport = { path = "../p2p-transport" }
tokio = { version = "1.35", features = ["full"] }
dashmap = "5.5"
```

**Módulos:**

#### `manager.rs` - Gestor de conexiones
```rust
pub struct ConnectionManager {
    connections: DashMap<PeerId, Connection>,
}

impl ConnectionManager {
    pub async fn add_connection(&self, peer: PeerId, conn: Connection);
    pub async fn remove_connection(&self, peer: PeerId);
    pub async fn get_connection(&self, peer: PeerId) -> Option<Connection>;
    pub async fn broadcast(&self, data: &[u8]);
    pub async fn send_to(&self, peer: PeerId, data: &[u8]) -> Result<()>;
}
```

#### `peer.rs` - Información del peer
```rust
pub struct Connection {
    pub peer_id: PeerId,
    pub address: SocketAddr,
    pub reliable_channel: ReliableChannel,
    pub unreliable_channel: UnreliableChannel,
    pub state: ConnectionState,
    pub stats: ConnectionStats,
}

pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnecting,
    Disconnected,
}

pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub rtt: Duration,
    pub packet_loss: f32,
    pub last_seen: Instant,
}
```

#### `keepalive.rs` - Mantener conexiones vivas
```rust
pub struct KeepAlive {
    interval: Duration,
    timeout: Duration,
}

impl KeepAlive {
    pub async fn start(&self, connection: &Connection);
    pub async fn send_ping(&self) -> Result<()>;
    pub async fn handle_pong(&self);
    pub fn is_alive(&self) -> bool;
}
```

#### `qos.rs` - Quality of Service
```rust
pub struct QoS {
    priorities: HashMap<DataType, Priority>,
}

impl QoS {
    pub fn prioritize(&self, packets: &mut Vec<Packet>);
    pub fn should_drop(&self, packet: &Packet) -> bool;
}
```

**Comunicación:**
- ← **p2p-transport**: Usa canales de transporte
- → **p2p-protocol**: Proporciona conexiones activas
- → **p2p-interface**: Estado de conexiones

---

### **Layer 2: p2p-protocol** (Protocol Layer)
**Responsabilidad:** Serialización, compresión y encriptación

```toml
[package]
name = "p2p-protocol"
version = "0.1.0"

[dependencies]
p2p-connection = { path = "../p2p-connection" }
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
zstd = "0.13"
chacha20poly1305 = "0.10"
```

**Módulos:**

#### `codec.rs` - Serialización
```rust
pub trait Codec {
    fn encode(&self, data: &impl Serialize) -> Result<Vec<u8>>;
    fn decode<T: DeserializeOwned>(&self, data: &[u8]) -> Result<T>;
}

pub struct BincodeCodec;
pub struct JsonCodec;
pub struct MessagePackCodec;
```

#### `compression.rs` - Compresión
```rust
pub trait Compressor {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>>;
}

pub struct ZstdCompressor {
    level: i32,
}

pub struct Lz4Compressor;
```

#### `encryption.rs` - Encriptación E2E
```rust
pub struct CryptoSession {
    local_keypair: KeyPair,
    remote_public_key: PublicKey,
    shared_secret: SharedSecret,
}

impl CryptoSession {
    pub fn new(local_keypair: KeyPair) -> Self;
    pub fn establish(&mut self, remote_public: PublicKey) -> Result<()>;
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>>;
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>>;
}
```

#### `types.rs` - Tipos de mensajes
```rust
#[derive(Serialize, Deserialize)]
pub enum Message {
    Text { content: String, topic: String },
    Audio { data: Vec<u8>, codec: AudioCodec },
    Video { data: Vec<u8>, codec: VideoCodec },
    File { name: String, chunk: Vec<u8>, chunk_id: u64 },
    Typing { is_typing: bool },
    Reaction { message_id: String, emoji: String },
}
```

**Comunicación:**
- ← **p2p-connection**: Recibe conexiones
- → **p2p-application**: Proporciona mensajes decodificados
- ← **p2p-application**: Recibe mensajes para enviar

---

### **Layer 1: p2p-application** (Application Layer)
**Responsabilidad:** Lógica de aplicación (chat, audio, archivos)

```toml
[package]
name = "p2p-application"
version = "0.1.0"

[dependencies]
p2p-protocol = { path = "../p2p-protocol" }
tokio = { version = "1.35", features = ["full"] }
opus = "0.3"
```

**Módulos:**

#### `messaging.rs` - Chat de texto
```rust
pub struct MessagingService {
    connection_manager: Arc<ConnectionManager>,
}

impl MessagingService {
    pub async fn send_message(&self, topic: &str, content: &str) -> Result<()>;
    pub async fn on_message_received(&self) -> Receiver<TextMessage>;
    pub async fn send_typing_indicator(&self, is_typing: bool);
}
```

#### `audio.rs` - Audio en tiempo real
```rust
pub struct AudioService {
    codec: OpusCodec,
    unreliable_channel: UnreliableChannel,
}

impl AudioService {
    pub async fn start_call(&mut self, peer: PeerId) -> Result<()>;
    pub async fn send_audio_frame(&self, frame: &[f32]) -> Result<()>;
    pub async fn recv_audio_frame(&self) -> Result<Vec<f32>>;
    pub async fn end_call(&mut self);
}
```

#### `video.rs` - Video en tiempo real
```rust
pub struct VideoService {
    codec: VideoCodec,
    unreliable_channel: UnreliableChannel,
}

impl VideoService {
    pub async fn start_video(&mut self, peer: PeerId) -> Result<()>;
    pub async fn send_video_frame(&self, frame: &[u8]) -> Result<()>;
    pub async fn recv_video_frame(&self) -> Result<Vec<u8>>;
}
```

#### `files.rs` - Transferencia de archivos
```rust
pub struct FileTransferService {
    reliable_channel: ReliableChannel,
}

impl FileTransferService {
    pub async fn send_file(&self, path: &Path, peer: PeerId) -> Result<()>;
    pub async fn recv_file(&self) -> Result<(String, Vec<u8>)>;
    pub fn progress(&self) -> f32;
}
```

**Comunicación:**
- ← **p2p-protocol**: Recibe mensajes decodificados
- → **p2p-interface**: Notifica eventos de aplicación

---

### **Layer 0: p2p-interface** (Interface Layer)
**Responsabilidad:** Puente entre Rust y frontend

```toml
[package]
name = "p2p-interface"
version = "0.1.0"

[dependencies]
p2p-application = { path = "../p2p-application" }
tauri = "1.5"
tokio = { version = "1.35", features = ["sync"] }
```

**Módulos:**

#### `tauri_bridge.rs` - Comandos Tauri
```rust
#[tauri::command]
pub async fn send_message(
    content: String,
    topic: String,
    state: State<AppState>,
) -> Result<(), String>;

#[tauri::command]
pub async fn start_audio_call(
    peer_id: String,
    state: State<AppState>,
) -> Result<(), String>;

#[tauri::command]
pub async fn send_file(
    path: String,
    peer_id: String,
    state: State<AppState>,
) -> Result<(), String>;
```

#### `events.rs` - Eventos al frontend
```rust
pub fn emit_message_received(
    app: &AppHandle,
    message: TextMessage,
) -> Result<()>;

pub fn emit_peer_connected(
    app: &AppHandle,
    peer: PeerInfo,
) -> Result<()>;

pub fn emit_audio_frame(
    app: &AppHandle,
    frame: Vec<f32>,
) -> Result<()>;
```

**Comunicación:**
- ← **p2p-application**: Recibe eventos de aplicación
- → **Frontend (React)**: Emite eventos Tauri
- ← **Frontend (React)**: Recibe comandos Tauri

---

## 🔄 Flujo de Comunicación Completo

### Ejemplo: Enviar Mensaje de Texto

```
1. Frontend (React)
   └→ invoke('send_message', { content, topic })
   
2. p2p-interface
   └→ MessagingService::send_message()
   
3. p2p-application (messaging.rs)
   └→ Crea Message::Text { content, topic }
   └→ CryptoSession::encrypt()
   
4. p2p-protocol (encryption.rs)
   └→ Compressor::compress()
   └→ Codec::encode()
   
5. p2p-connection (manager.rs)
   └→ ConnectionManager::send_to(peer)
   └→ Connection::reliable_channel
   
6. p2p-transport (reliable_udp.rs)
   └→ ReliableChannel::send_reliable()
   └→ Crea Packet { reliable: true, ... }
   
7. p2p-core (udp.rs)
   └→ UdpSocket::send_to()
   
8. INTERNET (atraviesa NAT via hole punching)
   
9. Peer recibe en orden inverso:
   p2p-core → p2p-transport → p2p-connection
   → p2p-protocol → p2p-application → p2p-interface
   
10. p2p-interface
    └→ app.emit('message-received', message)
    
11. Frontend (React)
    └→ listen('message-received', handler)
    └→ Actualiza UI
```

### Ejemplo: Audio en Tiempo Real

```
1. Frontend
   └→ invoke('start_audio_call', { peer_id })
   └→ Captura audio del micrófono

2. p2p-interface
   └→ AudioService::start_call()
   └→ loop: send_audio_frame()

3. p2p-application (audio.rs)
   └→ OpusCodec::encode(frame)
   └→ Message::Audio { data, codec }

4. p2p-protocol
   └→ NO comprime (ya comprimido por Opus)
   └→ NO encripta (opcional para audio)
   └→ Codec::encode()

5. p2p-connection
   └→ Connection::unreliable_channel
   
6. p2p-transport (unreliable_udp.rs)
   └→ UnreliableChannel::send_unreliable()
   └→ Crea Packet { reliable: false, priority: High }

7. p2p-core
   └→ UdpSocket::send_to()
   
8. INTERNET (sin retransmisiones)

9. Peer recibe:
   └→ p2p-application::AudioService
   └→ OpusCodec::decode(frame)
   
10. p2p-interface
    └→ app.emit('audio-frame', decoded_frame)
    
11. Frontend
    └→ Reproduce en speakers
```

---

## 📋 Checklist de Implementación

### Fase 1: Fundamentos (Semanas 1-2)
- [ ] Crear workspace de Cargo
- [ ] Implementar `p2p-core` (sockets UDP/TCP)
- [ ] Implementar `p2p-transport` básico (paquetes)
- [ ] Implementar `p2p-protocol` (serialización)
- [ ] Tests unitarios de cada capa

### Fase 2: Discovery (Semana 3)
- [ ] Implementar `p2p-discovery` (mDNS)
- [ ] Crear servidor de registro simple
- [ ] Integrar con `p2p-interface`
- [ ] Probar descubrimiento local

### Fase 3: NAT Traversal (Semanas 4-5)
- [ ] Implementar `p2p-nat` (STUN)
- [ ] Implementar hole punching básico
- [ ] Implementar ICE (sin TURN)
- [ ] Probar entre redes diferentes

### Fase 4: Transporte Confiable (Semana 6)
- [ ] Implementar ACKs y retransmisiones
- [ ] Implementar reordenamiento
- [ ] Control de congestión básico
- [ ] Tests de pérdida de paquetes

### Fase 5: Aplicación - Mensajería (Semana 7)
- [ ] Implementar `p2p-application` (messaging)
- [ ] Encriptación E2E
- [ ] Integrar con UI existente
- [ ] Persistencia de mensajes

### Fase 6: Audio (Semanas 8-9)
- [ ] Implementar `p2p-application` (audio)
- [ ] Integrar codec Opus
- [ ] Buffer de jitter
- [ ] Cancelación de eco
- [ ] UI de llamadas

### Fase 7: Optimizaciones (Semana 10)
- [ ] Implementar TURN fallback
- [ ] Mejorar control de congestión
- [ ] QoS y priorización
- [ ] Métricas y monitoreo

### Fase 8: Extras (Semanas 11-12)
- [ ] Transferencia de archivos
- [ ] Video (opcional)
- [ ] Llamadas grupales
- [ ] Compartir pantalla

---

## 🎯 Decisiones Técnicas Clave

### Por qué UDP sobre TCP
- ✅ Menor latencia (crítico para audio/video)
- ✅ Mejor control sobre retransmisiones
- ✅ Más fácil hole punching
- ❌ Más complejo de implementar

### Por qué Módulos Separados
- ✅ Facilita testing
- ✅ Reutilizable en otros proyectos
- ✅ Compilación paralela más rápida
- ✅ Mantenibilidad

### Stack de Seguridad
- **Encriptación**: ChaCha20-Poly1305
- **Key Exchange**: X25519 (Curve25519)
- **Autenticación**: Ed25519
- **Perfect Forward Secrecy**: Sí

---

¿Quieres que profundice en alguna capa específica o empecemos a implementar algún módulo? 🚀