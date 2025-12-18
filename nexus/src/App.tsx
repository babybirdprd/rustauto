import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { ThoughtStream } from "./components/ThoughtStream";

function App() {
  const [prompt, setPrompt] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!prompt.trim()) return;

    setLoading(true);
    try {
      await invoke("run_agent", { prompt });
    } catch (err) {
      console.error("Agent error:", err);
      // Maybe add error to ThoughtStream manually if needed, but backend should emit error event too
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white p-4 font-mono">
      <div className="flex flex-col items-center justify-center space-y-8 mt-20">
        <h1 className="text-4xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-600">
          Nexus
        </h1>

        {/* Omnibox */}
        <form onSubmit={handleSubmit} className="w-full max-w-2xl">
          <input
            type="text"
            value={prompt}
            onChange={(e) => setPrompt(e.target.value)}
            placeholder="What is your goal?"
            disabled={loading}
            className="w-full bg-gray-800/50 backdrop-blur-md border border-gray-700 rounded-lg px-6 py-4 text-xl focus:outline-none focus:ring-2 focus:ring-blue-500 shadow-2xl disabled:opacity-50"
          />
        </form>

        {/* Thought Stream */}
        <ThoughtStream />
      </div>
    </div>
  );
}

export default App;
