import { useState, useRef, useEffect } from 'react'
import { useP2P } from './useP2P'

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
    <div className="min-h-screen bg-neutral-950">
      <div className="container mx-auto p-4 h-screen flex flex-col">
        {/* Header */}
        <div className="bg-black/30 backdrop-blur-lg rounded-2xl border border-white/10 p-4 mb-2">
          <div className="flex items-center justify-between mb-2">
            <div>
              <h1 className="text-3xl font-bold text-white">P2P Chat</h1>
              <p className="text-purple-300 mt-1">
                {peers.length} peer{peers.length !== 1 ? 's' : ''} • {messages.length} messages
              </p>
            </div>
            
            <div className="flex gap-2">
              <button
                onClick={refreshPeers}
                className="px-4 py-2 bg-indigo-600 hover:bg-indigo-700 text-white rounded-lg transition"
                title="Refresh peers list"
              >
                🔄 Refresh
              </button>
              <button
                onClick={() => setShowManualConnect(!showManualConnect)}
                className="px-4 py-2 bg-purple-600 hover:bg-purple-700 text-white rounded-lg transition"
              >
                {showManualConnect ? '✕ Cancel' : '🔗 Connect Manually'}
              </button>
            </div>
          </div>

          {/* Connection Status */}
          {connectionStatus && (
            <div className="bg-blue-500/20 border border-blue-500/50 rounded-lg p-3 mb-2">
              <p className="text-blue-300 text-sm">{connectionStatus}</p>
            </div>
          )}

          {/* Manual Connect Form */}
          {showManualConnect && (
            <div className="bg-white/5 rounded-lg p-4 mb-2">
              <h3 className="text-white font-semibold mb-2">Connect to Peer</h3>
              <p className="text-purple-300 text-xs mb-3">
                Enter the full multiaddr (e.g., /ip4/192.168.1.100/tcp/54321/p2p/12D3KooW...)
              </p>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={manualAddress}
                  onChange={(e) => setManualAddress(e.target.value)}
                  placeholder="/ip4/192.168.1.100/tcp/54321/p2p/12D3KooW..."
                  className="flex-1 px-3 py-2 bg-white/10 text-white placeholder-purple-300 rounded border border-white/20 focus:outline-none focus:border-purple-400 font-mono text-sm"
                />
                <button
                  onClick={handleConnectManually}
                  className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded transition font-semibold"
                >
                  Connect
                </button>
              </div>
            </div>
          )}

          {/* My Info */}
          {myInfo && (
            <div className="bg-white/5 rounded-lg p-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-purple-300 text-sm font-semibold">Your Peer ID:</span>
                <button
                  onClick={() => copyToClipboard(myInfo.peer_id)}
                  className="text-xs px-2 py-1 bg-purple-600 hover:bg-purple-700 text-white rounded"
                >
                  📋 Copy
                </button>
              </div>
              <p className="text-white font-mono text-xs break-all mb-3">
                {myInfo.peer_id}
              </p>
              
              {myInfo.addresses && myInfo.addresses.length > 0 && (
                <>
                  <span className="text-purple-300 text-sm font-semibold">Your Addresses:</span>
                  <div className="space-y-2 mt-2">
                    {myInfo.addresses.map((addr, idx) => (
                      <div key={idx} className="bg-white/5 rounded p-2">
                        <div className="flex items-center justify-between">
                          <p className="text-white font-mono text-xs break-all flex-1">
                            {addr}
                          </p>
                          <button
                            onClick={() => copyToClipboard(`${addr}/p2p/${myInfo.peer_id}`)}
                            className="text-xs px-2 py-1 bg-green-600 hover:bg-green-700 text-white rounded ml-2 whitespace-nowrap"
                          >
                            📋 Full
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                </>
              )}
            </div>
          )}
        </div>

        <div className="flex-1 flex gap-2 min-h-0">
          {/* Sidebar - Peers */}
          <div className="w-80 bg-black/30 backdrop-blur-lg rounded-2xl border border-white/10 flex flex-col">
            <div className="p-4 border-b border-white/10">
              <h2 className="text-lg font-semibold text-white">Peers</h2>
            </div>
            
            <div className="flex-1 overflow-y-auto p-3 space-y-2">
              {peers.length === 0 ? (
                <div className="text-center text-purple-300 text-sm mt-8">
                  <div className="text-4xl mb-3">👥</div>
                  <p>No peers yet</p>
                  <p className="mt-2 text-xs">Waiting for discovery...</p>
                  <p className="mt-4 text-xs">Or connect manually</p>
                </div>
              ) : (
                peers.map((peer) => (
                  <div
                    key={peer.peer_id}
                    className="bg-white/5 hover:bg-white/10 rounded-lg p-3 transition"
                  >
                    <div className="flex items-start gap-2">
                      <div className={`w-2 h-2 rounded-full mt-1 flex-shrink-0 ${
                        peer.status === 'connected' ? 'bg-green-400' : 'bg-yellow-400'
                      }`}></div>
                      <div className="flex-1 min-w-0">
                        <div className="text-white text-sm font-medium mb-1">
                          {peer.status === 'connected' ? '🔗 Connected' : '👋 Discovered'}
                        </div>
                        <div className="text-purple-300 text-xs font-mono break-all">
                          {formatPeerId(peer.peer_id)}
                        </div>
                        <div className="text-purple-400 text-xs mt-1 break-all">
                          {peer.address}
                        </div>
                      </div>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>

          {/* Main Chat */}
          <div className="flex-1 bg-black/30 backdrop-blur-lg rounded-2xl border border-white/10 flex flex-col">
            {/* Messages */}
            <div className="flex-1 overflow-y-auto p-4 space-y-3">
              {messages.length === 0 ? (
                <div className="h-full flex items-center justify-center">
                  <div className="text-center text-purple-300">
                    <div className="text-6xl mb-4">💬</div>
                    <p className="text-xl">No messages yet</p>
                    <p className="text-sm mt-2">Start chatting!</p>
                  </div>
                </div>
              ) : (
                <>
                  {messages.map((msg, idx) => (
                    <div
                      key={idx}
                      className={`flex ${msg.own ? 'justify-end' : 'justify-start'}`}
                    >
                      <div
                        className={`max-w-md px-4 py-3 rounded-2xl ${
                          msg.own
                            ? 'bg-purple-600 text-white'
                            : 'bg-white/10 text-white backdrop-blur-md'
                        }`}
                      >
                        {!msg.own && (
                          <div className="text-xs font-semibold mb-1 text-purple-300 font-mono">
                            {formatPeerId(msg.from)}
                          </div>
                        )}
                        <div className="break-words">{msg.content}</div>
                        <div className="text-xs mt-1 opacity-70">
                          {formatTime(msg.timestamp)}
                        </div>
                      </div>
                    </div>
                  ))}
                  <div ref={messagesEndRef} />
                </>
              )}
            </div>

            {/* Input */}
            <form onSubmit={handleSendMessage} className="p-4 border-t border-white/10">
              <div className="flex gap-3">
                <input
                  type="text"
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  placeholder="Type a message..."
                  className="flex-1 px-4 py-3 bg-white/10 text-white placeholder-purple-300 rounded-lg border border-white/20 focus:outline-none focus:border-purple-400 focus:ring-2 focus:ring-purple-400/50"
                />
                <button
                  type="submit"
                  disabled={!input.trim()}
                  className="px-6 py-3 bg-purple-600 hover:bg-purple-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg font-semibold transition"
                >
                  Send
                </button>
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  )
}

export default App