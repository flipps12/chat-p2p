import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function App() {
  const [responses, setResponses] = useState([]);
  const [command, setCommand] = useState("status");
  const [arg, setArg] = useState("");
  const [rtt, setRtt] = useState(null);

  // Configuramos los listeners al montar el componente
  useEffect(() => {
    let unlistenResponse;
    let unlistenRtt;

    async function setupListeners() {
      // Escucha respuestas de la red Knot
      unlistenResponse = await listen("knot-response", (event) => {
        setResponses((prev) => [JSON.stringify(event.payload), ...prev].slice(0, 10));
      });

      // Escucha el RTT de los pings
      unlistenRtt = await listen("knot-rtt", (event) => {
        setRtt(event.payload);
      });
    }

    setupListeners();

    // Limpieza al desmontar
    return () => {
      if (unlistenResponse) unlistenResponse();
      if (unlistenRtt) unlistenRtt();
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

  return (
    <main className="container">
      <h1>Knot Chat Terminal</h1>

      <div className="status-bar">
        <span>RTT: {rtt !== null ? `${rtt}ms` : "---"}</span>
      </div>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          sendCommand();
        }}
      >
        <select value={command} onChange={(e) => setCommand(e.target.value)}>
          <option value="status">Status</option>
          <option value="version">Version</option>
          <option value="connect">Connect (Addr)</option>
          <option value="ping">Ping (PeerId)</option>
        </select>

        <input
          onChange={(e) => setArg(e.currentTarget.value)}
          placeholder="Argumento (Address/ID)..."
          value={arg}
        />
        <button type="submit">Enviar</button>
      </form>

      <div className="response-log">
        <h3>Historial de red:</h3>
        <ul>
          {responses.map((res, i) => (
            <li key={i}><code>{res}</code></li>
          ))}
        </ul>
      </div>
    </main>
  );
}

export default App;