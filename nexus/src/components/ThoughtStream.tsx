import { useEffect, useState, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';

interface AgentEvent {
  type: 'system' | 'tool_call' | 'tool_result' | 'error' | 'success';
  message: string;
  timestamp: number;
}

export function ThoughtStream() {
  const [events, setEvents] = useState<AgentEvent[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Listen for agent-event
    const unlisten = listen<AgentEvent>('agent-event', (event) => {
      setEvents((prev) => [...prev, event.payload]);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [events]);

  return (
    <div
        ref={scrollRef}
        className="w-full max-w-2xl bg-black/30 rounded-lg p-4 h-64 overflow-y-auto border border-gray-800 font-mono text-sm space-y-1"
    >
      <p className="text-gray-400">[System] Thought Stream Connected.</p>
      {events.map((evt, i) => (
        <div key={i} className="flex gap-2">
            <span className="text-gray-600 shrink-0">
                [{new Date(evt.timestamp).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit', second:'2-digit'})}]
            </span>
            <span className={
                evt.type === 'error' ? 'text-red-400' :
                evt.type === 'tool_call' ? 'text-yellow-400' :
                evt.type === 'tool_result' ? 'text-green-400' :
                evt.type === 'success' ? 'text-blue-400 font-bold' :
                'text-gray-300'
            }>
                [{evt.type.toUpperCase()}] {evt.message}
            </span>
        </div>
      ))}
    </div>
  );
}
