import { useState, useRef, useEffect } from 'react'
import { useP2P } from './useP2P'
import UserList from './components/UserList'
// import ModalConnectPeer from './components/ModalConnectPeer'
import type { Channel } from './types'
import ChannelsList from './components/ChannelsList'
import ModalAddTopic from './components/ModalAddTopic'
import ModalAddPeer from './components/ModalAddPeer'

function App() {
  const {
    messages,
    peers,
    myInfo,
    channels,
    // connectionStatus,
    sendMessage,
    connectToPeer,
    // refreshPeers,
    // setStatus,
    loadChannels,
    add_topic,
    saveChannel,
    setChannels,
  } = useP2P()

  // load 
  useEffect(() => {
    loadChannels()
  }, [])

  const [input, setInput] = useState('')
  const [channel, setChannel] = useState('')
  // const [showConnectModal, setShowConnectModal] = useState(false)
  const [showAddTopic, setShowAddTopic] = useState(false)
  const [showAddPeer, setShowAddPeer] = useState(false)
  const [filteredMessages, setFilteredMessages] = useState(messages)
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

  // const copyToClipboard = (text: string) => {
  //   navigator.clipboard.writeText(text)
  //   setStatus('✅ Copied to clipboard!', 2000)
  // }

  // const formatPeerId = (peerId: string): string => {
  //   if (!peerId) return ''
  //   return `${peerId.substring(0, 8)}...${peerId.substring(peerId.length - 6)}`
  // }

  const formatTime = (timestamp: string): string => {
    const date = new Date(timestamp)
    return date.toLocaleTimeString('en-US', { 
      hour: '2-digit', 
      minute: '2-digit' 
    })
  }

  useEffect(() => {
    const filteredMessages = messages.filter(
      (message) => message.topic === channel
    )
    setFilteredMessages(filteredMessages)
  }, [messages, channel])

  return (
    <div className="min-h-screen bg-[#030303] text-gray-100">
      < ModalAddPeer
        add_peer={connectToPeer}
        setShowAddPeer={setShowAddPeer}
        showAddPeer={showAddPeer}
      />
      < ModalAddTopic
        add_topic={add_topic}
        setChannels={setChannels}
        saveChannel={saveChannel}
        channels={channels}
        setShowAddTopic={setShowAddTopic}
        showAddTopic={showAddTopic}
      />
      <div className="container mx-auto h-screen flex flex-row">
        <div className='flex flex-col flex-1 border-r border-neutral-800 bg-[#070709] min-w-0 overflow-x-hidden custom-scroll'>
          <div className='w-full p-4 border-b border-neutral-700 flex flex-row'><h3 className='text-xl flex-1'>Channels</h3><button onClick={ () => { setShowAddTopic(true) }} className='rounded-3xl p-1 hover:bg-neutral-900'><svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#eee" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"></path><path d="M12 5v14"></path></svg></button></div>
          <div className='w-full p-2'>
            <ChannelsList channels={channels} setChannel={setChannel} channel={channel} />
          </div>
        </div>
        <div className='flex flex-col flex-3 border-r border-neutral-800 min-w-0 overflow-x-hidden custom-scroll'>
          <div className='w-full p-4 border-b border-neutral-700'><h3 className='text-xl '>Messages</h3></div>
          <div className='flex-1 overflow-y-auto px-2 pt-2'>
            <div className='text-gray-500 text-center'>{channel}</div>
            {filteredMessages.map((message, index) => {
              const previousMessage = filteredMessages[index - 1]
              const sameOwner = previousMessage?.from === message.from

              return (
                <div key={message.uuid} className="p-2 hover:bg-neutral-900 rounded-2xl">
                  {!sameOwner && (
                    <div className="flex flex-col text-sm">
                      <div className='flex flex-row items-center'>
                        <div className='text-gray-300 overflow-x-hidden mr-2 text-lg'>{message.from}</div>
                        <div className='text-gray-500 flex-1 text-right text-nowrap'>{formatTime(message.timestamp)}</div>
                      </div>
                      <div className="text-sm flex-1 text-gray-400">{message.content}</div>
                    </div>
                  )}

                  {sameOwner && (
                    <div className="flex flex-row text-sm">
                      <div className="text-sm flex-1 text-gray-400">{message.content}</div>
                      <div className='text-gray-500'>{formatTime(message.timestamp)}</div>
                    </div>
                  )}
                </div>
              )
            })}
            <div ref={messagesEndRef} />
          </div>
          <div className='w-full p-4'>
            <form onSubmit={handleSendMessage} className='flex flex-row'>
              <input
                type="text"
                value={input}
                onChange={(e) => setInput(e.target.value)}
                placeholder="Type your message..."
                className='flex-1 px-4 py-2 rounded-l-xl bg-[#050505] focus:outline-none'
              />
              <button type="submit" className='bg-[#1a1a1a] hover:bg-[#333] text-gray-100 px-4 rounded-r-xl'>Send</button>
            </form>
          </div>
        </div>
        <div className='flex flex-col flex-1 bg-[#070709] min-w-0 overflow-x-hidden custom-scroll'>
          <div className='w-full flex flex-row p-4 border-b border-neutral-700'><h3 className='text-xl flex-1'>Peers</h3><button onClick={ () => { setShowAddPeer(true) }} className='rounded-3xl p-1 hover:bg-neutral-900'><svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#eee" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"></path><path d="M12 5v14"></path></svg></button></div>
          <div className='w-full p-2'>
            <UserList myInfo={myInfo} peers={peers} />
          </div>
        </div>
      </div>
    </div>
  )
}

export default App