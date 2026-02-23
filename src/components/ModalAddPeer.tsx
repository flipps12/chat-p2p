import { useState } from "react";

interface ModalAddPeerProps {
    add_peer: (payload: string) => Promise<void>;
    setShowAddPeer: (show: boolean) => void;
    showAddPeer: boolean;
}

function ModalAddPeer(props: ModalAddPeerProps) {
    const [peerId, setPeerId] = useState("");

    return (
        <div 
        className={
        props.showAddPeer
          ? "fixed inset-0 transition-opacity bg-black bg-opacity-50 flex items-center justify-center z-50"
          : "hidden"
      }
        >
        <div className="flex flex-col bg-[#070709] p-6 rounded-lg w-80 gap-4">
            <h2>Add Peer</h2>
            <input className="outline-0" type="text" placeholder="Peer ID" value={peerId} onChange={(e) => setPeerId(e.target.value)} />
            <div className="flex flex-row gap-3">
                <button className="p-2 bg-neutral-800 hover:bg-neutral-600 rounded-2xl" onClick={() => {
                    props.add_peer(peerId);
                    props.setShowAddPeer(false);
                }}>Add</button>
                <button className="p-2 bg-neutral-800 hover:bg-neutral-600 rounded-2xl" onClick={() => props.setShowAddPeer(false)}>Cancel</button>
            </div>
        </div>
        </div>
    );
}

export default ModalAddPeer;