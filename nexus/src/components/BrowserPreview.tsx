import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export function BrowserPreview() {
    const [screenshot, setScreenshot] = useState<string | null>(null);
    const [url, setUrl] = useState<string>('');
    const [loading, setLoading] = useState(false);

    const updatePreview = async () => {
        try {
            setLoading(true);
            const [currentUrl, currentScreenshot] = await Promise.all([
                invoke<string>('get_current_url'),
                invoke<string>('take_screenshot'),
            ]);
            setUrl(currentUrl);
            setScreenshot(currentScreenshot);
        } catch (err) {
            console.error('Failed to update browser preview:', err);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        updatePreview();

        // Update on navigation events from backend
        const unlisten = listen('browser-update', (event: any) => {
            setUrl(event.payload.url);
            updatePreview();
        });

        // Also poll occasionally just in case
        const interval = setInterval(updatePreview, 10000);

        return () => {
            unlisten.then(f => f());
            clearInterval(interval);
        };
    }, []);

    return (
        <div className="bg-gray-800/30 rounded-lg p-4 border border-gray-700 h-full flex flex-col">
            <div className="flex justify-between items-center mb-4">
                <h2 className="text-lg font-bold text-purple-400">Browser Live</h2>
                <button
                    onClick={updatePreview}
                    disabled={loading}
                    className="text-[10px] bg-blue-500/20 hover:bg-blue-500/40 text-blue-300 px-2 py-1 rounded transition-colors disabled:opacity-50"
                >
                    {loading ? 'Refreshing...' : 'Refresh'}
                </button>
            </div>

            <div className="flex-1 flex flex-col min-h-0">
                <div className="bg-black/40 px-3 py-1.5 rounded-t border-t border-x border-gray-800 text-[10px] text-gray-400 truncate">
                    {url || 'No active session'}
                </div>
                <div className="flex-1 bg-gray-900 rounded-b border border-gray-800 overflow-hidden relative group">
                    {screenshot ? (
                        <img
                            src={screenshot}
                            alt="Browser Preview"
                            className="w-full h-full object-contain"
                        />
                    ) : (
                        <div className="absolute inset-0 flex items-center justify-center text-gray-600 text-xs italic">
                            Wait for navigation...
                        </div>
                    )}
                    {loading && (
                        <div className="absolute inset-0 bg-black/20 backdrop-blur-[1px] flex items-center justify-center">
                            <div className="w-5 h-5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
