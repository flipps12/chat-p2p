use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, PeerId,
};
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
            // Message ID function
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            // ✅ Convertir errores de gossipsub a io::Error
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

            // ✅ Crear PeerId correctamente
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

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("✅ P2P network started");

    loop {
        select! {
            // 📤 Mensaje desde frontend
            Some(msg) = rx.recv() => {
                println!("📤 Sending message: {}", msg);

                if let Err(e) = swarm.behaviour_mut()
                    .gossipsub
                    .publish(topic.clone(), msg.as_bytes())
                {
                    eprintln!("❌ Failed to publish: {}", e);
                }
            }

            // 📥 Evento de red
            event = swarm.select_next_some() => match event {
                // Nueva dirección
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("🎧 Listening on: {}", address);
                }

                // Peer descubierto
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(
                    mdns::Event::Discovered(peers)
                )) => {
                    for (peer_id, multiaddr) in peers {
                        println!("👋 Discovered peer: {} at {}", peer_id, multiaddr);
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                }

                // Peer expirado
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(
                    mdns::Event::Expired(peers)
                )) => {
                    for (peer_id, _) in peers {
                        println!("👋 Peer expired: {}", peer_id);
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                }

                // Mensaje recibido
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message { message, .. }
                )) => {
                    let text = String::from_utf8_lossy(&message.data).to_string();
                    println!("📥 Received message: {}", text);

                    // Emitir al frontend
                    if let Err(e) = app.emit("p2p-message", text) {
                        eprintln!("❌ Failed to emit to frontend: {}", e);
                    }
                }

                _ => {}
            }
        }
    }
}