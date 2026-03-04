use libp2p::{
    gossipsub, mdns, noise,
    swarm::SwarmEvent,
    tcp, yamux, PeerId, Multiaddr,
    multiaddr::Protocol,
};
use std::{
    sync::Arc,
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
    time::Duration,
};
use tokio::{select, sync::{mpsc, Mutex}};
use futures::StreamExt;
use tauri::{AppHandle, Emitter};

use crate::fs::FileManager;
use crate::types::{
    Message, 
    MyBehaviour,
    MyBehaviourEvent,
    MyAddressInfo, 
    PeerDiscovered,
    // Info to save
    PeerInfoToSave,
    ChannelInfoToSave,
    PeerIdToSave,
};

async fn connect_to_peer(addr: Multiaddr, swarm: &mut libp2p::Swarm<MyBehaviour>) -> Result<(), Box<dyn Error>> {
    match swarm.dial(addr.clone()) {
        Ok(_) => {
            println!("✅ Dialing {}", addr);
            // save peer and ipv6 address to local state if needed
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to dial: {}", e);
            Err(Box::new(e))
        }
    }
}


pub async fn start_p2p(
    mut rx: mpsc::Receiver<String>,
    app: AppHandle,
    file_manager: Arc<Mutex<FileManager>>,
    known_peers: Vec<PeerInfoToSave>,
    channels: Vec<ChannelInfoToSave>,
) -> Result<(), Box<dyn Error>> {
    // reading saved peer_id
    let saved_peer_id = file_manager.lock().await.load_peer_identity()?;

    for peer in &known_peers {
        println!("📋 Known peer: {} at {:?}", peer.peer_id, peer.addresses);
    }

    // reading saved channels    
    for channel in &channels {
        println!("📋 Saved channel: {} with last message {}", channel.topic, channel.last_message_uuid.as_deref().unwrap_or("None"));
    }

    println!("🚀 Starting P2P network...");

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            let config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(message_id_fn)
                .build()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                config,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            let peer_id = PeerId::from(key.public());
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                peer_id,
            )?;

            Ok(MyBehaviour { gossipsub, mdns })
        })?
        .build();

    // let topic = gossipsub::IdentTopic::new("test-net");
    // swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Guardar el peer_id local
    let local_peer_id = *swarm.local_peer_id();
    println!("🆔 Local Peer ID: {}", local_peer_id);

    swarm.listen_on("/ip6/::/tcp/0".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("✅ P2P network started");

    // Almacenar direcciones locales
    let mut local_addresses: Vec<String> = Vec::new();

    loop {
        select! {
            // 📤 Comandos desde frontend
            Some(msg) = rx.recv() => {
                // Parsear comandos especiales
                if msg.starts_with("CMD:CONNECT:") {
                    // Comando: Conectar a un peer manualmente
                    let addr_str = msg.strip_prefix("CMD:CONNECT:").unwrap();
                    println!("🔗 Attempting manual connection to: {}", addr_str);
                    
                    match addr_str.parse::<Multiaddr>() {
                        Ok(addr) => {
                            match connect_to_peer(addr.clone(), &mut swarm).await {
                                Ok(()) => {
                                    println!("✅ Dialing {}", addr);
                                    let _ = app.emit("connection-status", "Connecting...");
                                }
                                Err(e) => {
                                    let _ = app.emit("connection-error", format!("Failed to dial: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Invalid multiaddr: {}", e);
                            let _ = app.emit("connection-error", format!("Invalid address: {}", e));
                        }
                    }
                } else if msg.starts_with("CMD:GET_PEERS") {
                    // Comando: Obtener lista de peers conectados
                    let connected_peers: Vec<String> = swarm.connected_peers()
                        .map(|p| p.to_string())
                        .collect();
                    
                    println!("📋 Connected peers: {:?}", connected_peers);
                    let _ = app.emit("peers-list", connected_peers);
                    
                } else if msg.starts_with("CMD:GET_INFO") {
                    // Comando: Obtener información del nodo local
                    let info = MyAddressInfo {
                        peer_id: local_peer_id.to_string(),
                        addresses: local_addresses.clone(),
                    };
                    // println!("📋 My info: {:?}", info);
                    let _ = app.emit("my-info", info);
                    
                } else if msg.starts_with("CMD:ADD_TOPIC:") {
                    let topic = msg.strip_prefix("CMD:ADD_TOPIC:").unwrap();
                    
                    let topic_parsed = gossipsub::IdentTopic::new(topic);
                    swarm.behaviour_mut().gossipsub.subscribe(&topic_parsed)?;
                    
                } else if msg.starts_with("CMD:SEND_MESSAGE:") {
                    // Mensaje normal de chat
                    let msg_content = msg.strip_prefix("CMD:SEND_MESSAGE:").unwrap();
                    let msg: Message = serde_json::from_str(msg_content).unwrap();
                    let topic = gossipsub::IdentTopic::new(msg.topic.clone());
                    println!("📤 Sending message: {} at {}", msg.msg, topic);

                    if let Err(e) = swarm.behaviour_mut()
                        .gossipsub
                        .publish(topic, msg_content.as_bytes())
                    {
                        eprintln!("❌ Failed to publish: {}", e);
                    }
                }
            }

            // 📥 Eventos de red
            event = swarm.select_next_some() => match event {
                // Nueva dirección de escucha
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("🎧 Listening on: {}", address);
                    
                    let addr_str = address.to_string();
                    local_addresses.push(addr_str);
                    
                    // Emitir información al frontend
                    let info = MyAddressInfo {
                        peer_id: local_peer_id.to_string(),
                        addresses: local_addresses.clone(),
                    };
                    let _ = app.emit("my-address", info);
                }

                // Conexión establecida
                SwarmEvent::ConnectionEstablished { 
                    peer_id, 
                    endpoint, 
                    .. 
                } => {
                    println!("🔗 Connection established with: {} at {}", 
                        peer_id, endpoint.get_remote_address());
                    
                    let mut clean_addr = endpoint.get_remote_address().clone();
                    
                    clean_addr = clean_addr
                        .into_iter()
                        .filter(|p| !matches!(p, Protocol::P2p(_)))
                        .collect();

                    let peer_info = PeerDiscovered {
                        peer_id: peer_id.to_string(),
                        address: clean_addr.to_string(),
                    };
                    let _ = app.emit("peer-connected", peer_info);

                    // ✅ Guardar peer
                    let peer_info = PeerInfoToSave {
                        peer_id: peer_id.to_string(),
                        addresses: vec![endpoint.get_remote_address().to_string()],
                        failed_attempts: 0,
                    };
                    
                    let fm = file_manager.lock().await;
                    if let Err(e) = fm.add_or_update_peer(peer_info) {
                        eprintln!("Failed to save peer: {}", e);
                    }
                    
                    // ✅ Resetear intentos fallidos
                    if let Err(e) = fm.reset_peer_failed_attempts(&peer_id.to_string()) {
                        eprintln!("Failed to reset attempts: {}", e);
                    }
                }

                // Cuando falla una conexión
                SwarmEvent::OutgoingConnectionError { peer_id: Some(peer_id), .. } => {
                    println!("❌ Connection failed to: {}", peer_id);
                    
                    // ✅ Incrementar intentos fallidos
                    let fm = file_manager.lock().await;
                    if let Err(e) = fm.increment_peer_failed_attempts(&peer_id.to_string()) {
                        eprintln!("Failed to increment attempts: {}", e);
                    }
                }

                // Conexión cerrada
                SwarmEvent::ConnectionClosed { 
                    peer_id, 
                    cause, 
                    .. 
                } => {
                    println!("🔌 Connection closed with: {} (cause: {:?})", peer_id, cause);
                    let _ = app.emit("peer-disconnected", peer_id.to_string());
                }

                // Peer descubierto via mDNS
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(
                    mdns::Event::Discovered(peers)
                )) => {
                    for (peer_id, multiaddr) in peers {
                        println!("👋 Discovered peer: {} at {}", peer_id, multiaddr);

                        // Emitir al frontend
                        let peer_info = PeerDiscovered {
                            peer_id: peer_id.to_string(),
                            address: multiaddr.to_string(),
                        };

                        let _ = app.emit("peer-discovered", &peer_info);
                        println!("✅ Dialing {}", multiaddr);
                        
                        // if let Err(e) = swarm.dial(multiaddr.clone()) {
                        //     println!("❌ Dial error: {:?}", e);
                        // }  
                        match connect_to_peer(multiaddr.clone(), &mut swarm).await {
                            Ok(()) => {
                                println!("✅ Connected to {}", multiaddr);
                                swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);

                                // let _ = app.emit("peer-connected", &peer_info); already emitted in the listeners
                            }
                            Err(e) => {
                                let _ = app.emit("connection-error", format!("Failed to dial: {}", e));
                            }
                        }
                    }
                }

                // Peer expirado via mDNS
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(
                    mdns::Event::Expired(peers)
                )) => {
                    for (peer_id, _) in peers {
                        println!("👋 Peer expired: {}", peer_id);
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        let _ = app.emit("peer-expired", peer_id.to_string());
                    }
                }

                // Mensaje recibido via Gossipsub
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message {
                        propagation_source,
                        message,
                        ..
                    }
                )) => {
                    let topic_hash = message.topic.clone();
                    let topic_id = topic_hash.to_string();

                    // Intentar parsear como UTF-8
                    let text = match String::from_utf8(message.data.clone()) {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("❌ Invalid UTF-8 message: {}", e);
                            return Ok(());
                        }
                    };

                    // Intentar deserializar
                    let message_json: Message = match serde_json::from_str(&text) {
                        Ok(m) => m,
                        Err(e) => {
                            eprintln!("❌ Invalid JSON message: {}", e);
                            return Ok(());
                        }
                    };

                    println!(
                        "📥 Received from {} on topic {}: {}",
                        propagation_source,
                        topic_id,
                        message_json.msg
                    );

                    // Construir payload para frontend
                    let msg_data = serde_json::json!({
                        "from": propagation_source.to_string(),
                        "content": message_json.msg,
                        "name": message_json.name,
                        "topic": topic_id,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "uuid": message_json.uuid,
                    });

                    if let Err(e) = app.emit("p2p-message", msg_data) {
                        eprintln!("❌ Failed to emit to frontend: {}", e);
                    }

                    // ✅ Usar el UUID REAL del mensaje, no generar uno nuevo
                    let fm = file_manager.lock().await;
                    if let Err(e) = fm.update_channel_last_message(&topic_id, message_json.uuid) {
                        eprintln!("❌ Failed to update channel: {}", e);
                    }
                }

                // Peer suscrito al topic
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(
                    gossipsub::Event::Subscribed { peer_id, topic }
                )) => {
                    println!("📢 Peer {} subscribed to topic: {}", peer_id, topic);
                    let _ = app.emit("peer-subscribed", serde_json::json!({
                        "peer_id": peer_id.to_string(),
                        "topic": topic.to_string(),
                    }));
                }

                // Otros eventos (opcional: loggear para debug)
                _ => {}
            }
        }
    }
}