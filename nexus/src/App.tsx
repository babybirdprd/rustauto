import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";
import { Sidebar } from "./components/Sidebar";
import { BrowserPreview } from "./components/BrowserPreview";
import { Settings } from "./components/Settings";
import { MemoryView } from "./components/MemoryView";
import { ExecutionFlow } from "./components/ExecutionFlow";
import { ResultPanel } from "./components/ResultPanel";
import { TraceViewer } from "./components/TraceViewer";

type ActiveView = "main" | "traces";

function App() {
  const [prompt, setPrompt] = useState("");
  const [loading, setLoading] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showLeftPanel, setShowLeftPanel] = useState(true);
  const [activeView, setActiveView] = useState<ActiveView>("main");

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

  const handleReset = useCallback(async () => {
    try {
      await invoke("reset_session");
      setPrompt("");
      // Logic to clear components is handled via internal event listeners 
      // but we could also force a re-render if needed.
      window.location.reload(); // Hard reset for the best experience on v2.1
    } catch (err) {
      console.error("Reset error:", err);
    }
  }, []);

  return (
    <div className="flex h-screen bg-gray-950 text-white font-mono overflow-hidden selection:bg-blue-500/30">
      {/* 1. Slim Navigation Sidebar */}
      <Sidebar
        onNewTask={handleReset}
        onOpenSettings={() => setShowSettings(true)}
        onOpenTraces={() => setActiveView("traces")}
        onOpenMain={() => setActiveView("main")}
        activeView={activeView}
      />

      {/* 2. Main Interaction Stage (Center) */}
      <div className="flex-1 flex flex-col min-w-0 bg-gray-900/20 relative">
        <header className="h-14 border-b border-gray-800/50 flex items-center px-6 justify-between bg-black/20 backdrop-blur-xl z-10">
          <div className="flex items-center gap-4">
            <h1 className="text-lg font-bold tracking-tight text-white/90">
              Nexus <span className="text-xs font-normal text-gray-500 ml-2">v2.1</span>
            </h1>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={() => setShowLeftPanel(!showLeftPanel)}
              className="text-[10px] uppercase font-bold text-gray-500 hover:text-white transition-colors"
              title="Toggle Status Panel"
            >
              {showLeftPanel ? 'Hide panels' : 'Show panels'}
            </button>
          </div>
        </header>

        {/* Browser Feed or Trace Viewer */}
        <div className="flex-1 p-4 overflow-hidden flex flex-col space-y-4">
          {activeView === "main" ? (
            <BrowserPreview />
          ) : (
            <TraceViewer />
          )}
        </div>

        {/* Omnibox / Input */}
        <div className="p-6 bg-linear-to-t from-gray-950 to-transparent">
          <form onSubmit={handleSubmit} className="max-w-4xl mx-auto relative group">
            <div className="absolute -inset-0.5 bg-linear-to-r from-blue-500 to-purple-600 rounded-3xl blur opacity-20 group-hover:opacity-40 transition duration-1000 group-focus-within:opacity-50"></div>
            <div className="relative">
              <input
                type="text"
                value={prompt}
                onChange={(e) => setPrompt(e.target.value)}
                placeholder="What can I research for you today?"
                disabled={loading}
                className="w-full bg-gray-900 border border-gray-800 rounded-2xl px-8 py-5 text-lg focus:outline-none focus:ring-1 focus:ring-blue-500/50 shadow-3xl disabled:opacity-50 transition-all pr-20 placeholder:text-gray-600"
              />
              <button
                type="submit"
                disabled={loading || !prompt.trim()}
                className="absolute right-4 top-1/2 -translate-y-1/2 w-12 h-12 flex items-center justify-center bg-blue-600 hover:bg-blue-500 text-white rounded-xl transition-all disabled:opacity-0 disabled:scale-95 shadow-xl shadow-blue-600/20 active:scale-95"
              >
                {loading ? (
                  <div className="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                ) : (
                  <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M13 5l7 7-7 7M5 5l7 7-7 7" />
                  </svg>
                )}
              </button>
            </div>
          </form>
        </div>
      </div>

      {/* 3. Status & Results Panel (Right) */}
      {showLeftPanel && (
        <div className="w-[450px] border-l border-gray-800 flex flex-col p-4 bg-gray-950 space-y-4 animate-in slide-in-from-right-full duration-500">
          <div className="h-2/5 min-h-0">
            <ExecutionFlow />
          </div>
          <div className="h-2/5 min-h-0">
            <ResultPanel />
          </div>
          <div className="h-1/5 min-h-0">
            <MemoryView />
          </div>
        </div>
      )}

      {/* Overlays */}
      {showSettings && <Settings onClose={() => setShowSettings(false)} />}
    </div>
  );
}

export default App;
