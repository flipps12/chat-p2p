import { useEffect, useState } from 'react'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import type { 
  Message, 
  Peer, 
  MyInfo, 
  PeerDiscoveredPayload, 
  MessagePayload, 
  SendMessagePayload,
  Channel
} from './types'

export function useP2P() {
  const [messages, setMessages] = useState<Message[]>([])
  const [peers, setPeers] = useState<Peer[]>([])
  const [myInfo, setMyInfo] = useState<MyInfo | null>(null)
  const [connectionStatus, setConnectionStatus] = useState('')
  const [channels, setChannels] = useState<Channel[]>([])

  useEffect(() => {
    console.log('🎧 Setting up P2P listeners...')

    let unlisteners: UnlistenFn[] = []

    const setupListeners = async () => {
      // Mi información
      const unlistenMyAddress = await listen<MyInfo>('my-address', (event) => {
        console.log('🆔 My info:', event.payload)
        setMyInfo(event.payload)
      })
      unlisteners.push(unlistenMyAddress)

      const unlistenMyInfo = await listen<MyInfo>('my-info', (event) => {
        console.log('📋 Full info:', event.payload)
        setMyInfo(event.payload)
      })
      unlisteners.push(unlistenMyInfo)

      // Peer descubierto
      const unlistenDiscovered = await listen<PeerDiscoveredPayload>(
        'peer-discovered',
        (event) => {
          console.log('👋 Peer discovered:', event.payload)
          setPeers((prev) => {
            if (prev.some(p => p.peer_id === event.payload.peer_id)) {
              return prev
            }
            return [...prev, { ...event.payload, status: 'discovered' }]
          })
        }
      )
      unlisteners.push(unlistenDiscovered)

      // Peer conectado
      const unlistenConnected = await listen<PeerDiscoveredPayload>(
        'peer-connected',
        (event) => {
          console.log('🔗 Peer connected:', event.payload)
          setPeers((prev) => {
            const exists = prev.some(p => p.peer_id === event.payload.peer_id)
            if (exists) {
              return prev.map(p =>
                p.peer_id === event.payload.peer_id
                  ? { ...p, status: 'connected' }
                  : p
              )
            }
            return [...prev, { ...event.payload, status: 'connected' }]
          })
          setConnectionStatus('✅ Peer connected!')
          setTimeout(() => setConnectionStatus(''), 3000)
        }
      )
      unlisteners.push(unlistenConnected)

      // Peer desconectado
      const unlistenDisconnected = await listen<string>(
        'peer-disconnected',
        (event) => {
          console.log('🔌 Peer disconnected:', event.payload)
          setPeers((prev) => prev.filter(p => p.peer_id !== event.payload))
        }
      )
      unlisteners.push(unlistenDisconnected)

      // Peer expirado
      const unlistenExpired = await listen<string>('peer-expired', (event) => {
        console.log('👋 Peer expired:', event.payload)
        setPeers((prev) => prev.filter(p => p.peer_id !== event.payload))
      })
      unlisteners.push(unlistenExpired)

      // Lista de peers
      const unlistenPeersList = await listen<string[]>('peers-list', (event) => {
        console.log('📋 Peers list:', event.payload)
      })
      unlisteners.push(unlistenPeersList)

      // Mensaje recibido
      const unlistenMessage = await listen<MessagePayload>(
        'p2p-message',
        (event) => {
          const incoming = event.payload

          setMessages((prev) => {
            const alreadyExists = prev.some(
              (m) => m.uuid === incoming.uuid
            )

            if (alreadyExists) {
              console.log('⚠️ Duplicate message ignored:', incoming.uuid)
              return prev
            }

            return [
              ...prev,
              {
                name: incoming.name,
                from: incoming.from,
                topic: incoming.topic,
                content: incoming.content,
                uuid: incoming.uuid,
                timestamp: incoming.timestamp,
                own: false,
              },
            ]
          })
        }
      )
      unlisteners.push(unlistenMessage)

      // Error de conexión
      const unlistenError = await listen<string>('connection-error', (event) => {
        console.error('❌ Connection error:', event.payload)
        alert(`Connection error: ${event.payload}`)
        setConnectionStatus('')
      })
      unlisteners.push(unlistenError)

      // Estado de conexión
      const unlistenStatus = await listen<string>('connection-status', (event) => {
        console.log('📊 Status:', event.payload)
        setConnectionStatus(event.payload)
      })
      unlisteners.push(unlistenStatus)

      // get my info

      try {
        await invoke('get_my_info')
      } catch (error) {
        console.error('Failed to get my info:', error)
      }
    }

    setupListeners()

    return () => {
      unlisteners.forEach(unlisten => unlisten())
    }
  }, [])

  const sendMessage = async (payload: SendMessagePayload) => {
    try {
      await invoke('send_message', { msg: payload })
      setMessages((prev) => [
        ...prev,
        {
          from: 'You',
          name: payload.name,
          content: payload.msg,
          topic: payload.topic,
          uuid: payload.uuid || crypto.randomUUID(),
          timestamp: new Date().toISOString(),
          own: true,
        },
      ])
    } catch (error) {
      console.error('❌ Send failed:', error)
      throw error
    }
  }

  const connectToPeer = async (address: string) => {
    try {
      await invoke('connect_to_peer', { address })
    } catch (error) {
      console.error('❌ Connect failed:', error)
      throw error
    }
  }

  const refreshPeers = async () => {
    try {
      await invoke('get_connected_peers')
    } catch (error) {
      console.error('❌ Failed to get peers:', error)
      throw error
    }
  }

  const add_topic = async (topic: string) => {
    try {
      await invoke('add_topic', { topic })
    } catch (error) {
      console.error('❌ Failed to add topic:', error)
      throw error
    }
  }

  const setStatus = (status: string, duration = 3000) => {
    setConnectionStatus(status)
    if (duration > 0) {
      setTimeout(() => setConnectionStatus(''), duration)
    }
  }

  // save data
  const saveChannel = async (topic: string, uuid: string) => {
    try {
      console.log('💾 Saving channel:', {
        topic: topic,
        uuid: uuid
      })
      await invoke('add_channel', { topic, uuid }) // CON await
      console.log('✅ Channel saved successfully')
    } catch (error) {
      console.error('❌ Failed to save channel:', error)
      throw error
    }
  }

  // load data
  const loadChannels = async () => {
    try {
      const savedChannels: Channel[] = await invoke('get_channels')
      console.log('📂 Loaded channels:', savedChannels)
      setChannels(savedChannels)
      for (const ch of savedChannels) {
        await add_topic(ch.uuid)
      }
    } catch (error) {
      console.error('❌ Failed to load channels:', error)
    }
  }

  return {
    messages,
    peers,
    myInfo,
    connectionStatus,
    // load data
    channels,
    sendMessage,
    connectToPeer,
    refreshPeers,
    setStatus,
    add_topic,
    // save data
    saveChannel,
    // load data
    loadChannels,
    // support function to add channels
    setChannels,
  }
}