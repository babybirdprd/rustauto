import { useEffect, useState, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';

interface AgentEvent {
    type: 'system' | 'tool_call' | 'tool_result' | 'error' | 'success';
    message: string;
    timestamp: number;
}

export function ExecutionFlow() {
    const [events, setEvents] = useState<AgentEvent[]>([]);
    const [currentStep, setCurrentStep] = useState<string>('Initializing...');
    const scrollRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const unlisten = listen<AgentEvent>('agent-event', (event) => {
            const { type, message } = event.payload;
            setEvents((prev) => [...prev, event.payload]);

            if (type === 'tool_call') {
                setCurrentStep(message);
            } else if (type === 'success') {
                setCurrentStep('Task Completed');
            } else if (type === 'error') {
                setCurrentStep('Error encountered');
            }
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
        <div className="flex flex-col h-full bg-gray-950/50 border border-gray-800 rounded-2xl overflow-hidden backdrop-blur-sm">
            <div className="px-6 py-4 border-b border-gray-800 bg-gray-900/50 flex flex-col space-y-2">
                <h2 className="text-sm font-bold text-gray-400 uppercase tracking-widest">Execution Flow</h2>
                <div className="flex items-center gap-3">
                    <div className={`w-2 h-2 rounded-full animate-pulse ${currentStep.includes('Error') ? 'bg-red-500' :
                        currentStep.includes('Completed') ? 'bg-green-500' :
                            'bg-blue-500'
                        }`} />
                    <span className="text-sm font-semibold text-blue-300 truncate">{currentStep}</span>
                </div>
            </div>

            <div ref={scrollRef} className="flex-1 overflow-y-auto p-4 space-y-3 font-mono text-[11px]">
                {events.length === 0 && (
                    <p className="text-gray-600 italic">Waiting for agent activity...</p>
                )}
                {events.map((evt, i) => (
                    <div key={i} className={`flex flex-col p-2 rounded-lg border ${evt.type === 'error' ? 'bg-red-500/10 border-red-900/50' :
                        evt.type === 'tool_call' ? 'bg-yellow-500/5 border-yellow-900/30' :
                            evt.type === 'tool_result' ? 'bg-green-500/5 border-green-900/30' :
                                evt.type === 'success' ? 'bg-blue-500/10 border-blue-900/50' :
                                    'bg-gray-800/10 border-gray-800/50'
                        }`}>
                        <div className="flex justify-between items-center mb-1">
                            <span className={`uppercase font-bold tracking-tighter ${evt.type === 'error' ? 'text-red-400' :
                                evt.type === 'tool_call' ? 'text-yellow-400' :
                                    evt.type === 'tool_result' ? 'text-green-400' :
                                        evt.type === 'success' ? 'text-blue-400' :
                                            'text-gray-500'
                                }`}>{evt.type}</span>
                            <span className="text-gray-600">{new Date(evt.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}</span>
                        </div>
                        <p className="text-gray-300 wrap-break-word line-clamp-3 hover:line-clamp-none transition-all cursor-default">
                            {evt.message}
                        </p>
                    </div>
                ))}
            </div>
        </div>
    );
}
