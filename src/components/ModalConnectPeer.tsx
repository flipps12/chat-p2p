function ModalConnectPeer() {
  return (
    <div className='fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50'>
        <div className='bg-[#202020] p-6 rounded-lg w-96'>
            <h2 className='text-2xl mb-4'>Connect to Peer</h2>
            <input 
                type="text"
                placeholder="Multiaddress or Peer ID"
                className='w-full p-2 mb-4 rounded bg-[#303030] focus:outline-none'
            />
            <div className='flex justify-end gap-2'>
                <button className='px-4 py-2 rounded bg-[#1a1a1a] hover:bg-[#333] text-gray-100'>Connect</button>
                <button className='px-4 py-2 rounded bg-[#1a1a1a] hover:bg-[#333] text-gray-100'>Cancel</button>
            </div>
        </div>
    </div>
  )
}

export default ModalConnectPeer

