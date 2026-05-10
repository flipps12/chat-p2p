function SideBar({ localPeerid, peers, sendCommand, setPeerid, peerid }) {
  return (
    <aside className="flex-2 flex bg-mist-900 rounded-r-2xl flex-col overflow-hidden">
      <div className="flex flex-row p-2 border-b border-mist-600">
        <div className="flex-1 py-2">Peers</div>
        <button
          onClick={() => {
            sendCommand("getpeers", []);
            sendCommand("getpeerid", []);
          }}
          className="bg-mist-900 px-3.5 py-2  rounded-full hover:bg-mist-700 transition-colors"
        >
          R
        </button>
      </div>
      <div className="flex-1 overflow-hidden mt-2">
        <ul className="overflow-hidden">
          <li
            className="m-1 p-2 overflow-x-hidden hover:bg-mist-700 rounded-md"
            onClick={() => {
              navigator.clipboard.writeText(localPeerid);
            }}
          >
            {localPeerid}
          </li>
          <hr className="my-3 text-mist-600"></hr>
          {peers.map((peer, i) => (
            <>
              <li
                onClick={() => {
                  setPeerid(peer);
                  navigator.clipboard.writeText(peer);
                }}
                className={`${peerid == peer ? "bg-mist-700" : "bg-mist-900"} m-1 p-2 hover:bg-mist-700 rounded-md overflow-x-hidden text-mist-200`}
                key={i}
              >
                {peer}
              </li>
            </>
          ))}
        </ul>
      </div>
    </aside>
  );
}

export default SideBar;
