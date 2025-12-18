import "./App.css";

function App() {
  return (
    <div className="min-h-screen bg-gray-900 text-white p-4 font-mono">
      <div className="flex flex-col items-center justify-center space-y-8 mt-20">
        <h1 className="text-4xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-purple-600">
          Nexus
        </h1>

        {/* Omnibox Placeholder */}
        <div className="w-full max-w-2xl">
          <input
            type="text"
            placeholder="What is your goal?"
            className="w-full bg-gray-800/50 backdrop-blur-md border border-gray-700 rounded-lg px-6 py-4 text-xl focus:outline-none focus:ring-2 focus:ring-blue-500 shadow-2xl"
          />
        </div>

        {/* Thought Stream Placeholder */}
        <div className="w-full max-w-2xl bg-black/30 rounded-lg p-4 h-48 overflow-y-auto border border-gray-800">
          <p className="text-gray-400 text-sm">[System] Nexus initialized.</p>
          <p className="text-blue-400 text-sm">[Sifter] Ready to scan.</p>
        </div>
      </div>
    </div>
  );
}

export default App;
