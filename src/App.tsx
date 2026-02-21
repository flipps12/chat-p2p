import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

import Channel from "./components/Channel";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="w-screen h-screen font-medium text-white bg-stone-950 flex flex-col items-center justify-center gap-4">
      <h1 className="text-2xl font-bold text-stone-300">
        Welcome to Tauri + React
      </h1>
      <div className="w-full h-full flex flex-row">
        <div className="flex-1 flex flex-col overflow-y-auto overflow-x-hidden border-r border-stone-700">
          <h3 className="pl-2">Name group</h3>
          <hr className="my-2"/>
          <Channel text="General" />
          <Channel text="General" />
          <Channel text="General" />
          <Channel text="General" />
          <Channel text="General" />
          <Channel text="General" />
        </div>
        <div className="flex-2 flex flex-col">
          <div className="flex-1"></div>
          <form
            className="flex flex-row items-center gap-1 w-full"
            onSubmit={(e) => {
              e.preventDefault();
              greet();
            }}
          >
            <input
              className="p-2 bg-stone-800 rounded-xl text-white outline-0 w-full"
              id="greet-input"
              onChange={(e) => setName(e.currentTarget.value)}
              placeholder="Enter message"
            />
            <button
              type="submit"
              className="bg-stone-700 hover:bg-stone-600 text-white p-2 rounded-xl"
            >
              Submit
            </button>
          </form>
        </div>
        <div className="flex-1">asdasd</div>
      </div>

      <p>{greetMsg}</p>
    </main>
  );
}

export default App;
