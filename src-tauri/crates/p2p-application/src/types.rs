use libp2p::{
    gossipsub, mdns,
    swarm::NetworkBehaviour,
};

use serde::{Serialize, Deserialize};

#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

// Estructuras para eventos del frontend
#[derive(Serialize, Clone, Debug)]
pub struct PeerDiscovered {
    pub peer_id: String,
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub peer_id: String,
    pub msg: String,
    pub topic: String,
    pub uuid: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendMessagePayload {
    pub peer_id: String,
    pub content: String,
    pub topic: String,
    pub uuid: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct MyAddressInfo {
    pub peer_id: String,
    pub addresses: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerInfoToSave {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub failed_attempts: u8, // establecer intento maximo de conexion
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PeerIdToSave {
    // peer_id: String,
    pub peer_id_private: String,
    pub peer_id_public: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChannelInfoToSave {
    pub topic: String,
    pub uuid: String,
    pub last_message_uuid: Option<String>,
}