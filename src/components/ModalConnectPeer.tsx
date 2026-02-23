import { useState } from "react";

interface ModalConnectPeerProps {
    connectToPeer: (address: string) => Promise<void>;
    setShowConnectModal: (show: boolean) => void;
    showConnectModal: boolean;
}

function ModalConnectPeer(functions: ModalConnectPeerProps) {
  const [manualAddress, setManualAddress] = useState('')
//   const [showManualConnect, setShowManualConnect] = useState(false)

  const handleConnectManually = async (connectToPeer: (address: string) => Promise<void>) => {
    if (!manualAddress.trim()) {
      alert('Please enter an address')
      return
    }

    try {
      await connectToPeer(manualAddress)
    //   setShowManualConnect(false)
      setManualAddress('')
    } catch (error) {
      alert(`Error: ${error}`)
    }
}

  return (
    <div className={functions.showConnectModal ? 'fixed inset-0 transition-opacity bg-black bg-opacity-50 flex items-center justify-center z-50' : 'hidden'}>
        <div className='bg-[#202020] p-6 rounded-lg w-96'>
            <h2 className='text-2xl mb-4'>Connect to Peer</h2>
            <input 
                type="text"
                placeholder="Multiaddress or Peer ID"
                className='w-full p-2 mb-4 rounded bg-[#303030] focus:outline-none'
                value={manualAddress}
                onChange={(e) => setManualAddress(e.target.value)}
            />
            <div className='flex justify-end gap-2'>
                <button onClick={ () => {handleConnectManually(functions.connectToPeer)} } className='px-4 py-2 rounded bg-[#1a1a1a] hover:bg-[#333] text-gray-100'>Connect</button>
                <button onClick={ ()=> {functions.setShowConnectModal(false)} } className='px-4 py-2 rounded bg-[#1a1a1a] hover:bg-[#333] text-gray-100'>Cancel</button>
            </div>
        </div>
    </div>
  )
}

export default ModalConnectPeer

