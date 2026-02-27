// Types para la aplicación P2P Chat

export interface Channel {
  topic: string
  unreadCount: number
  uuid: string
}

export interface CreateChannelPayload {
  name: string
  uuid: string
}

export interface Message {
  topic: string
  from: string
  content: string
  timestamp: string
  own: boolean
  uuid: string
}

export interface SendMessagePayload {
  msg: string,
  topic: string,
  peer_id: string,
  uuid?: string
}

export interface Peer {
  // name: string
  peer_id: string
  address: string
  status: 'discovered' | 'connected'
}

export interface MyInfo {
  peer_id: string
  addresses: string[]
}

// Event Payloads
export interface PeerDiscoveredPayload {
  peer_id: string
  address: string
}

export interface MessagePayload {
  from: string
  content: string
  timestamp: string
  topic: string
  uuid: string
}

export interface PeerSubscribedPayload {
  peer_id: string
  topic: string
}

// save data 

export interface PeerInfoToSave {
  peer_id: string
  addresses: string[]
  failed_attempts: number
}

export interface ChannelInfoToSave {
  topic: string
  uuid: string
  last_message_uuid: string | null
}

export interface PeerIdToSave {
  peer_id_private: string
  peer_id_public: string
}

// Tauri Commands
export type TauriCommand = 
  | 'send_message'
  | 'connect_to_peer'
  | 'get_connected_peers'
  | 'get_my_info'
  | 'add_channel'
  | 'get_channels'

// Tauri Events
export type TauriEvent =
  | 'my-address'
  | 'my-info'
  | 'peer-discovered'
  | 'peer-connected'
  | 'peer-disconnected'
  | 'peer-expired'
  | 'peers-list'
  | 'p2p-message'
  | 'connection-error'
  | 'connection-status'
  | 'peer-subscribed'