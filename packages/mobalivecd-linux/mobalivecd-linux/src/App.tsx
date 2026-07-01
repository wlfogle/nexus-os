import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function App() {
  const [status, setStatus] = useState<string>("Initializing MobaLiveCD...");

  useEffect(() => {
    async function execute() {
      try {
        const result = await invoke<string>("run_mobalivecd");
        setStatus(result);
      } catch (e) {
        setStatus(`Error: ${e}`);
      }
    }
    execute();
  }, []);

  return (
    <div className="container">
    <h1>MobaLiveCD</h1>
    <pre style={{ background: "#222", color: "#0f0", padding: "10px" }}>
    {status}
    </pre>
    </div>
  );
}

export default App;
