import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { ThoughtStream } from "./components/ThoughtStream";
import { MemoryView } from "./components/MemoryView";
import { BrowserPreview } from "./components/BrowserPreview";
import { Settings } from "./components/Settings";

function App() {
  const [prompt, setPrompt] = useState("");
  const [loading, setLoading] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!prompt.trim() || loading) return;

    setLoading(true);
    try {
      await invoke("run_agent", { prompt });
    } catch (err) {
      console.error("Agent error:", err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="flex h-screen bg-gray-950 text-white font-mono overflow-hidden">
      {/* Sidebar / Left Column: Agent Activity */}
      <div className="w-1/3 border-r border-gray-800 flex flex-col p-4 space-y-4">
        <div className="h-1/2 min-h-0">
          <ThoughtStream />
        </div>
        <div className="h-1/2 min-h-0">
          <MemoryView />
        </div>
      </div>

      {/* Main Content: Input and Browser Preview */}
      <div className="flex-1 flex flex-col min-w-0 bg-gray-900/50">
        {/* Header */}
        <header className="h-16 border-b border-gray-800 flex items-center justify-between px-6 bg-gray-900/80 backdrop-blur-md z-10">
          <h1 className="text-2xl font-bold text-transparent bg-clip-text bg-linear-to-r from-blue-400 to-purple-500">
            Nexus
          </h1>
          <button
            onClick={() => setShowSettings(true)}
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-all"
            title="Settings"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        </header>

        {/* Browser Preview Area */}
        <div className="flex-1 p-6 overflow-hidden">
          <BrowserPreview />
        </div>

        {/* Input Bar */}
        <div className="p-6 bg-linear-to-t from-gray-950 to-transparent">
          <form onSubmit={handleSubmit} className="max-w-3xl mx-auto relative group">
            <input
              type="text"
              value={prompt}
              onChange={(e) => setPrompt(e.target.value)}
              placeholder="Assign a task to Nexus..."
              disabled={loading}
              className="w-full bg-gray-800/50 backdrop-blur-xl border border-gray-700/50 rounded-2xl px-6 py-5 text-lg focus:outline-none focus:ring-2 focus:ring-blue-500 shadow-2xl disabled:opacity-50 transition-all group-hover:border-gray-600 pr-16"
            />
            <button
              type="submit"
              disabled={loading || !prompt.trim()}
              className="absolute right-4 top-1/2 -translate-y-1/2 p-2 bg-blue-600 hover:bg-blue-500 text-white rounded-xl transition-all disabled:opacity-0 disabled:scale-90"
            >
              {loading ? (
                <div className="w-6 h-6 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
              ) : (
                <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 5l7 7-7 7M5 5l7 7-7 7" />
                </svg>
              )}
            </button>
          </form>
        </div>
      </div>

      {/* Settings Modal */}
      {showSettings && <Settings onClose={() => setShowSettings(false)} />}
    </div>
  );
}

export default App;
