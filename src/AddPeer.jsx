function AddPeer({ setAddPeerOverlay }) {
  return (
    <>
      <div
        onClick={() => {
          setAddPeerOverlay(false);
        }}
        className="fixed inset-0 z-10 bg-black/70"
      ></div>
      <div className="fixed inset-0 z-50 flex items-center justify-center pointer-events-none">
        <div className="bg-mist-900 p-8 rounded-xl shadow-2xl w-full max-w-md pointer-events-auto">
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
            type="text"
            placeholder="12D3KooW..."
            className="w-full p-3 rounded bg-mist-800 text-white border border-mist-700 focus:outline-none focus:border-blue-500"
          />
          <button className="w-full mt-4 bg-blue-600 hover:bg-blue-700 text-white py-2 rounded transition-colors">
            Conectar
          </button>
        </div>
      </div>
    </>
  );
}

export default AddPeer;
