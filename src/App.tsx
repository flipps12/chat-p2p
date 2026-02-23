import { useState, useRef, useEffect } from 'react'
import { useP2P } from './useP2P'
import UserList from './components/UserList'
import ModalConnectPeer from './components/ModalConnectPeer'
import type { Message, Peer, MyInfo, Channel } from './types'
import ChannelsList from './components/ChannelsList'
import ModalAddTopic from './components/ModalAddTopic'

function App() {
  const {
    messages,
    peers,
    myInfo,
    connectionStatus,
    sendMessage,
    connectToPeer,
    refreshPeers,
    setStatus,
    add_topic,
  } = useP2P()

  const [input, setInput] = useState('')
  const [channel, setChannel] = useState('general')
  const [showConnectModal, setShowConnectModal] = useState(false)
  const [showAddTopic, setShowAddTopic] = useState(false)
  const [channels, setChannels] = useState<Channel[]>([])
  // encrypt
  // add topic/channel

  // debug 
  // setChannels([
  //   { name: 'general', unreadCount: 0, uuid: crypto.randomUUID() },
  //   { name: 'pruebas', unreadCount: 2, uuid: crypto.randomUUID() },
  //   { name: 'test-net', unreadCount: 0, uuid: crypto.randomUUID() },
  // ])


  const messagesEndRef = useRef<HTMLDivElement>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const handleSendMessage = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!input.trim()) return

    try {
      await sendMessage({
        msg: input,
        topic: channel,
        peer_id: myInfo?.peer_id || '',
        uuid: crypto.randomUUID(),
      })
      setInput('')
    } catch (error) {
      alert(`Error: ${error}`)
    }
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    setStatus('✅ Copied to clipboard!', 2000)
  }

  const formatPeerId = (peerId: string): string => {
    if (!peerId) return ''
    return `${peerId.substring(0, 8)}...${peerId.substring(peerId.length - 6)}`
  }

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp)
    return date.toLocaleTimeString('en-US', { 
      hour: '2-digit', 
      minute: '2-digit' 
    })
  }

  return (
    <div className="min-h-screen bg-[#030303] text-gray-100">
      < ModalAddTopic
        add_topic={add_topic}
        setChannels={setChannels}
        channels={channels}
        setShowAddTopic={setShowAddTopic}
        showAddTopic={showAddTopic}
      />
      < ModalConnectPeer 
        connectToPeer={connectToPeer}
        setShowConnectModal={setShowConnectModal}
        showConnectModal={showConnectModal}
      />
      <div className="container mx-auto h-screen flex flex-row">
        <div className='flex flex-col flex-1 border-r border-neutral-800 bg-[#070709] min-w-0 overflow-hidden'>
          <div className='w-full p-4 border-b border-neutral-700 flex flex-row'><h3 className='text-xl flex-1'>Channels</h3><button onClick={ () => { setShowAddTopic(true) }} className='rounded-3xl p-1 hover:bg-neutral-900'><svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#eee" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"></path><path d="M12 5v14"></path></svg></button></div>
          <div className='w-full p-2'>
            <ChannelsList channels={channels} setChannel={setChannel} channel={channel} />
          </div>
        </div>
        <div className='flex flex-col flex-3 border-r border-neutral-800'>
          <div className='w-full p-4 border-b border-neutral-700'><h3 className='text-xl '>Messages</h3></div>
          <div className='flex-1 overflow-y-auto px-2 pt-2'>
            <div className='text-gray-500 text-center'>{channel}</div>
            {messages
              .filter((message) => message.topic === channel)
              .map((message, index) => (
              <div key={index} className='p-2 hover:bg-neutral-900 rounded-2xl'>
                <div className='flex flex-row gap-3 items-center'>{ message.from }<div className='text-[10px] text-neutral-400'>{formatTime(message.timestamp)}</div></div>
                <div className='text-lg'>{message.content}</div>
              </div>
            ))}
            <div ref={messagesEndRef} />
          </div>
          <div className='w-full p-4 border-t border-neutral-700'>
            <form onSubmit={handleSendMessage} className='flex flex-row'>
              <input
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder="Type your message..."
                className='flex-1 p-2 rounded-l-xl bg-[#050505] focus:outline-none'
              />
              <button type="submit" className='bg-[#1a1a1a] hover:bg-[#333] text-gray-100 px-4 rounded-r-xl'>Send</button>
            </form>
          </div>
        </div>
        <div className='flex flex-col flex-1 bg-[#070709] min-w-0 overflow-hidden'>
          <div className='w-full p-4 border-b border-neutral-700'><h3 className='text-xl '>Peers</h3></div>
          <div className='w-full p-2'>
            <UserList myInfo={myInfo} peers={peers} />
          </div>
        </div>
      </div>
    </div>
  )
}

export default App