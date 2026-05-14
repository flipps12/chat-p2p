import { useState } from "react";

function AddPeer({ setAddPeerOverlay, sendCommand }) {
  const [multiaddr, setMultiaddr] = useState("");

  const handleSubmit = () => {
    if (multiaddr.length <= 0) return;
    sendCommand("connect", [multiaddr]);
  };

  return (
    <>
      <div
        onClick={() => {
          setAddPeerOverlay(false);
        }}
        className="fixed inset-0 z-10 bg-black/70"
      ></div>
      <div className="fixed inset-0 z-50 flex items-center justify-center pointer-events-none">
        <div className="bg-olive-900 p-8 rounded-xl shadow-2xl w-full max-w-md pointer-events-auto">
          <h2 className="text-white mb-4 text-lg font-semibold">
            Conect Peer{" "}
            <button
              onClick={() => {
                setAddPeerOverlay(false);
              }}
              className="float-end mr-2 text-2xl"
            >
              x
            </button>
          </h2>
          <input
            onChange={(e) => {
              setMultiaddr(e.currentTarget.value);
              // close
            }}
            type="text"
            placeholder="/ip4/127.0.0.1/udp/1234/quic-v1/p2p/12D3KooW..."
            className="w-full p-3 rounded bg-olive-800 text-white border border-olive-700 focus:outline-none focus:border-emerald-500"
          />
          <button
            onSubmit={handleSubmit}
            className="w-full mt-4 bg-emerald-600 hover:bg-emerald-700 text-white py-2 rounded transition-colors"
          >
            Conectar
          </button>
        </div>
      </div>
    </>
  );
}

export default AddPeer;
