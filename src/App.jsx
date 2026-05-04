import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function App() {
  const [responses, setResponses] = useState([]);
  const [command, setCommand] = useState("status");
  const [arg, setArg] = useState("");
  const [message, setMessage] = useState("");
  const [messageList, setMessageList] = useState([]);
  const [peerid, setPeerid] = useState("");
  const [rtt, setRtt] = useState(null);

  // Configuramos los listeners al montar el componente
  useEffect(() => {
    let unlistenResponse;
    let unlistenByte;

    async function setupListeners() {
      // Escucha respuestas de la red Knot
      unlistenResponse = await listen("knot-response", (event) => {
        setResponses((prev) => [JSON.stringify(event.payload), ...prev].slice(0, 10));
      });

      // Escucha el RTT de los pings
      unlistenByte = await listen("message", (event) => {
        setMessageList((prev) => [...prev, event.payload]);
      });
    }

    setupListeners();

    // Limpieza al desmontar
    return () => {
      if (unlistenResponse) unlistenResponse();
      if (unlistenByte) unlistenByte();
    };
  }, []);

  async function sendCommand() {
    try {
      // Llamamos al comando unificado en Rust
      await invoke("send_knot_command", {
        command: command,
        args: arg ? [arg] : []
      });
    } catch (error) {
      console.error("Error en Knot:", error);
      setResponses((prev) => [`Error: ${error}`, ...prev]);
    }
  }

  async function sendMessage() {
    try {
      await invoke("send_message_command", {
        message: message,
        peerid: peerid
      });
    } catch (error) {
      console.error("Error en Knot:", error);
    }
  }

  return (
    <main className="w-screen h-screen bg-black text-white flex flex-col">
	<h1 className="text-white font-bold text-3xl p-4 border-b border-white">Knot-chat</h1>
		<div className="flex items-center">
        	<input onChange={(e) => setPeerid(e.currentTarget.value)} placeholder="PeerId" type="text"
          	className="px-3 h-full outline-0 border-b flex-1 text-xl" />
      </div>
      <div className="flex-1 flex flex-row">
	  <div className="flex-2 flex border-r">
	  	<div className="flex-1">Peerid</div>
	  </div>
		<div className="flex-6 flex flex-col">
	  	<div className="flex-1 p-3">
        		<ul>
	  <li>Message</li>
          {messageList.map((mess, i) => (
            <li key={i}>{mess}</li>
          ))}
        		</ul>
	  	</div>
      
      		<form
        	onSubmit={(e) => {
          		e.preventDefault();
          		sendMessage();
        	}}
		className="p-3">
        <div className="p-2 bg-mist-900 rounded-2xl flex flex-row text-white">
          <input type="text" className="ml-2 w-full h-12 outline-0" placeholder="Message"
            onChange={(e) => setMessage(e.currentTarget.value)}
            value={message}
          />
          		<button type="submit" className="h-12 outline-0 bg-mist-950 px-6 rounded-2xl">Send</button>
          
    			</div>
      		</form>
	  	</div>
	  </div>
    </main>
    // <main className="container">
    //   <h1>Knot Chat Terminal</h1>

    //   <div className="status-bar">
    //     <span>RTT: {rtt !== null ? `${rtt}ms` : "---"}</span>
    //   </div>

    //   <form
    //     className="row"
    //     onSubmit={(e) => {
    //       e.preventDefault();
    //       sendCommand();
    //     }}
    //   >
    //     <select value={command} onChange={(e) => setCommand(e.target.value)}>
    //       <option value="status">Status</option>
    //       <option value="version">Version</option>
    //       <option value="connect">Connect (Addr)</option>
    //       <option value="ping">Ping (PeerId)</option>
    //     </select>

    //     <input
    //       onChange={(e) => setArg(e.currentTarget.value)}
    //       placeholder="Argumento (Address/ID)..."
    //       value={arg}
    //     />
    //     <button type="submit">Enviar</button>
    //   </form>

    //   <div className="response-log">
    //     <h3>Historial de red:</h3>
    //     <ul>
    //       {responses.map((res, i) => (
    //         <li key={i}><code>{res}</code></li>
    //       ))}
    //     </ul>
    //   </div>
    // </main>
  );
}

export default App;
