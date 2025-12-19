import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';

export function ResultPanel() {
    const [result, setResult] = useState<string>('');
    const [loading, setLoading] = useState(false);
    const scrollRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const unlisten = listen('agent-event', (event: any) => {
            if (event.payload.type === 'success') {
                const message = event.payload.message;
                const report = message.replace('Agent finished: ', '');
                setResult(report);
            }
        });

        return () => {
            unlisten.then(f => f());
        };
    }, []);

    return (
        <div className="flex flex-col h-full bg-gray-900/30 border border-gray-800 rounded-2xl overflow-hidden backdrop-blur-sm">
            <div className="px-6 py-4 border-b border-gray-800 flex justify-between items-center bg-gray-900/50">
                <h2 className="text-sm font-bold text-gray-400 uppercase tracking-widest">Final Report</h2>
                {result && (
                    <button
                        onClick={() => {
                            navigator.clipboard.writeText(result);
                        }}
                        className="text-[10px] text-blue-400 hover:text-blue-300 transition-colors"
                    >
                        Copy Markdown
                    </button>
                )}
            </div>
            <div className="flex-1 overflow-y-auto p-6 font-sans text-gray-200 prose prose-invert max-w-none">
                {result ? (
                    <div className="animate-in fade-in slide-in-from-bottom-4 duration-700">
                        {/* Simple markdown placeholder - in a real app we'd use react-markdown */}
                        <div className="whitespace-pre-wrap leading-relaxed">
                            {result}
                        </div>
                    </div>
                ) : (
                    <div className="h-full flex flex-col items-center justify-center text-gray-600 italic space-y-4">
                        <svg className="w-12 h-12 opacity-20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                        </svg>
                        <p className="text-sm">The agent's findings will appear here.</p>
                    </div>
                )}
            </div>
        </div>
    );
}
