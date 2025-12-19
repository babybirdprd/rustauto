import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface MemoryEntry {
  content: string;
  tags: string[];
  timestamp: number;
}

export function MemoryView() {
  const [memories, setMemories] = useState<MemoryEntry[]>([]);

  const fetchMemories = async () => {
    try {
      const res = await invoke<MemoryEntry[]>('get_memories');
      setMemories(res);
    } catch (err) {
      console.error('Failed to fetch memories:', err);
    }
  };

  const clearMemories = async () => {
    try {
      await invoke('clear_memories');
      setMemories([]);
    } catch (err) {
      console.error('Failed to clear memories:', err);
    }
  };

  useEffect(() => {
    fetchMemories();
    const interval = setInterval(fetchMemories, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="bg-gray-800/30 rounded-lg p-4 border border-gray-700 h-full flex flex-col">
      <div className="flex justify-between items-center mb-4">
        <h2 className="text-lg font-bold text-blue-400">Agent Memory</h2>
        <button
          onClick={clearMemories}
          className="text-xs bg-red-500/20 hover:bg-red-500/40 text-red-300 px-2 py-1 rounded transition-colors"
        >
          Clear
        </button>
      </div>
      <div className="flex-1 overflow-y-auto space-y-3 pr-2">
        {memories.length === 0 ? (
          <p className="text-gray-500 text-sm italic">No memories yet...</p>
        ) : (
          memories.map((mem, i) => (
            <div key={i} className="bg-gray-900/50 p-3 rounded border border-gray-800 text-sm">
              <p className="text-gray-300 whitespace-pre-wrap">{mem.content}</p>
              {mem.tags.length > 0 && (
                <div className="mt-2 flex flex-wrap gap-1">
                  {mem.tags.map((tag, j) => (
                    <span key={j} className="text-[10px] bg-blue-500/20 text-blue-300 px-1.5 py-0.5 rounded">
                      #{tag}
                    </span>
                  ))}
                </div>
              )}
              <p className="text-[10px] text-gray-600 mt-2">
                {new Date(mem.timestamp * 1000).toLocaleString()}
              </p>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
