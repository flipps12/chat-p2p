function SideBar({ localPeerid, peers, sendCommand, setPeerid, peerid }) {
  return (
    <aside className="w-[20vw] flex bg-olive-900 rounded-3xl flex-col overflow-hidden m-2">
      <div className="flex flex-row p-2 border-b border-olive-600">
        <div className="flex-1 py-2">Peers</div>
        <button
          onClick={() => {
            sendCommand("getpeers", []);
            sendCommand("getpeerid", []);
          }}
          className="bg-olive-700 px-3.5 py-2  rounded-full hover:bg-olive-700 transition-colors"
        >
          R
        </button>
      </div>
      <div className="flex-1 overflow-hidden mt-2">
        <ul className="overflow-hidden">
          <li
            className="m-1 p-2 overflow-x-hidden hover:bg-olive-700 rounded-md"
            onClick={() => {
              navigator.clipboard.writeText(localPeerid);
            }}
          >
            {localPeerid}
          </li>
          <hr className="my-3 text-olive-600"></hr>
          {peers.map((peer, i) => (
            <>
              <li
                onClick={() => {
                  setPeerid(peer);
                  navigator.clipboard.writeText(peer);
                }}
                className={`${peerid == peer ? "bg-olive-700" : "bg-olive-900"} m-1 p-2 hover:bg-olive-700 rounded-md overflow-x-hidden text-mist-200`}
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
