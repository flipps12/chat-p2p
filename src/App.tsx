import { useState, useRef, useEffect } from 'react'
import { useP2P } from './useP2P'
import UserList from './components/UserList'

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
  } = useP2P()

  const [input, setInput] = useState('')
  const [manualAddress, setManualAddress] = useState('')
  const [showManualConnect, setShowManualConnect] = useState(false)
  const [channel, setChannel] = useState('general')
  
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
      await sendMessage(input)
      setInput('')
    } catch (error) {
      alert(`Error: ${error}`)
    }
  }

  const handleConnectManually = async () => {
    if (!manualAddress.trim()) {
      alert('Please enter an address')
      return
    }

    try {
      await connectToPeer(manualAddress)
      setShowManualConnect(false)
      setManualAddress('')
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
      <div className="container mx-auto h-screen flex flex-row">
        <div className='flex flex-col flex-1 border-r border-neutral-800 bg-[#070709] min-w-0 overflow-hidden'>
          <div className='w-full p-4 border-b border-neutral-700 flex flex-row'><h3 className='text-xl flex-1'>Channels</h3><button className='rounded-3xl p-1 hover:bg-neutral-900'><svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#eee" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"></path><path d="M12 5v14"></path></svg></button></div>
          <div className='w-full p-2'>
            <ul>
              <li onClick={ () => { setChannel('general') }} className='text-2xl flex flex-row px-3 py-2 text-neutral-400 hover:bg-neutral-800 focus:bg-neutral-800 rounded-xl' style={channel == "general" ? { backgroundColor: "#1a1a1a" } : {}}>#<div className='w-full min-w-0 overflow-hidden mx-2 text-xl'>{"general"}</div> <div className='text-xl'>{ 3 }</div></li>
              <li onClick={ () => { setChannel('pruebas') }} className='text-2xl flex flex-row px-3 py-2 text-neutral-400 hover:bg-neutral-800 focus:bg-neutral-800 rounded-xl' style={channel == "pruebas" ? { backgroundColor: "#1a1a1a" } : {}}>#<div className='w-full min-w-0 overflow-hidden mx-2 text-xl'>{"pruebas"}</div> <div className='text-xl'>{ 3 }</div></li>
              <li onClick={ () => { setChannel('test') }} className='text-2xl flex flex-row px-3 py-2 text-neutral-400 hover:bg-neutral-800 focus:bg-neutral-800 rounded-xl' style={channel == "test" ? { backgroundColor: "#1a1a1a" } : {}}>#<div className='w-full min-w-0 overflow-hidden mx-2 text-xl'>{"test"}</div> <div className='text-xl'>{ 3 }</div></li>
            </ul>
          </div>
        </div>
        <div className='flex flex-col flex-3 border-r border-neutral-800'>
          <div className='w-full p-4 border-b border-neutral-700'><h3 className='text-xl '>Messages</h3></div>
          <div className='flex-1 overflow-y-auto px-2 pt-2'>
            {messages.map((message, index) => (
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