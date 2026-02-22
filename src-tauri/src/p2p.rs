use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId, Multiaddr,
    multiaddr::Protocol,
};
use serde::{Serialize, Deserialize};
use std::{
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
    time::Duration,
};
use tokio::{select, sync::mpsc};
use futures::StreamExt;
use tauri::{AppHandle, Emitter};

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

// Estructuras para eventos del frontend
#[derive(Serialize, Clone, Debug)]
struct PeerDiscovered {
    peer_id: String,
    address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    peer_id: String,
    msg: String,
    topic: String,
    uuid: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendMessagePayload {
    peer_id: String,
    content: String,
    topic: String,
    uuid: String,
}

#[derive(Serialize, Clone, Debug)]
struct MyAddressInfo {
    peer_id: String,
    addresses: Vec<String>,
}

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
) -> Result<(), Box<dyn Error>> {
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

    let topic = gossipsub::IdentTopic::new("test-net");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

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
                        propagation_source: peer_id,
                        message, 
                        .. 
                    }
                )) => {
                    let text = String::from_utf8_lossy(&message.data).to_string();
                    let message_json = serde_json::from_str::<Message>(&text).unwrap_or(Message {
                        peer_id: peer_id.to_string(),
                        msg: text.clone(),
                        topic: "unknown".to_string(),
                        uuid: "".to_string(),
                    });

                    println!("📥 Received from {}: {}", peer_id, text);

                    // Emitir al frontend con información del remitente
                    let msg_data = serde_json::json!({
                        "from": peer_id.to_string(),
                        "content": message_json.msg.to_string(),
                        "topic": message_json.topic.to_string(),
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                        "uuid": message_json.uuid.to_string(),
                    });
                    
                    if let Err(e) = app.emit("p2p-message", msg_data) {
                        eprintln!("❌ Failed to emit to frontend: {}", e);
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