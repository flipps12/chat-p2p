// Types para la aplicación P2P Chat

export interface Message {
  from: string
  content: string
  timestamp: string
  own: boolean
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
}

export interface PeerSubscribedPayload {
  peer_id: string
  topic: string
}

// Tauri Commands
export type TauriCommand = 
  | 'send_message'
  | 'connect_to_peer'
  | 'get_connected_peers'
  | 'get_my_info'

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