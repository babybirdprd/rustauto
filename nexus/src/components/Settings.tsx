import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Config {
    provider: string;
    api_key: string;
    model: string;
    base_url: string | null;
}

interface SettingsProps {
    onClose: () => void;
}

const PROVIDERS = [
    { id: 'anthropic', name: 'Anthropic' },
    { id: 'openai', name: 'OpenAI' },
    { id: 'openrouter', name: 'OpenRouter' },
    { id: 'gemini', name: 'Gemini' },
    { id: 'grok', name: 'Grok' },
    { id: 'deepseek', name: 'DeepSeek' },
];

export function Settings({ onClose }: SettingsProps) {
    const [config, setConfig] = useState<Config>({
        provider: 'anthropic',
        api_key: '',
        model: 'claude-3-sonnet-20240229',
        base_url: null,
    });
    const [showBaseUrl, setShowBaseUrl] = useState(false);
    const [saving, setSaving] = useState(false);

    useEffect(() => {
        const loadConfig = async () => {
            try {
                const currentConfig = await invoke<Config>('get_config');
                setConfig(currentConfig);
                setShowBaseUrl(!!currentConfig.base_url);
            } catch (err) {
                console.error('Failed to load config:', err);
            }
        };
        loadConfig();
    }, []);

    const handleSave = async () => {
        setSaving(true);
        try {
            const finalConfig = {
                ...config,
                base_url: showBaseUrl ? config.base_url : null,
            };
            await invoke('save_config', { config: finalConfig });
            onClose();
        } catch (err) {
            console.error('Failed to save config:', err);
            alert('Error saving config: ' + err);
        } finally {
            setSaving(false);
        }
    };

    return (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
            <div className="bg-gray-900 border border-gray-800 rounded-xl shadow-2xl w-full max-w-md overflow-hidden flex flex-col">
                <div className="p-6 border-b border-gray-800 flex justify-between items-center">
                    <h2 className="text-xl font-bold text-white">Settings</h2>
                    <button onClick={onClose} className="text-gray-500 hover:text-white transition-colors">
                        <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <div className="p-6 space-y-6 flex-1 overflow-y-auto">
                    {/* Provider */}
                    <div className="space-y-2">
                        <div className="flex justify-between items-center">
                            <label className="text-sm font-medium text-gray-300">API Provider</label>
                            <a href="#" className="text-xs text-blue-400 hover:underline">Documentation</a>
                        </div>
                        <select
                            value={config.provider}
                            onChange={(e) => setConfig({ ...config, provider: e.target.value })}
                            className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                        >
                            {PROVIDERS.map((p) => (
                                <option key={p.id} value={p.id}>{p.name}</option>
                            ))}
                        </select>
                    </div>

                    {/* API Key */}
                    <div className="space-y-2">
                        <label className="text-sm font-medium text-gray-300">API Key</label>
                        <input
                            type="password"
                            value={config.api_key}
                            placeholder="Enter your API key"
                            onChange={(e) => setConfig({ ...config, api_key: e.target.value })}
                            className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                        />
                        <p className="text-[10px] text-gray-500">API keys are stored locally in your app config directory.</p>
                    </div>

                    {/* Model */}
                    <div className="space-y-2">
                        <label className="text-sm font-medium text-gray-300">Model</label>
                        <input
                            type="text"
                            value={config.model}
                            placeholder="e.g. gpt-4o, claude-3-opus"
                            onChange={(e) => setConfig({ ...config, model: e.target.value })}
                            className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all"
                        />
                    </div>

                    {/* Custom Base URL (OpenAI only) */}
                    {config.provider === 'openai' && (
                        <div className="space-y-3 pt-2">
                            <div className="flex items-center gap-3">
                                <input
                                    type="checkbox"
                                    id="use-base-url"
                                    checked={showBaseUrl}
                                    onChange={(e) => setShowBaseUrl(e.target.checked)}
                                    className="w-4 h-4 rounded border-gray-700 bg-gray-800 text-blue-600 focus:ring-blue-500"
                                />
                                <label htmlFor="use-base-url" className="text-sm text-gray-300">Use custom base URL</label>
                            </div>

                            {showBaseUrl && (
                                <input
                                    type="text"
                                    value={config.base_url || ''}
                                    placeholder="https://api.openai.com/v1"
                                    onChange={(e) => setConfig({ ...config, base_url: e.target.value })}
                                    className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all animate-in slide-in-from-top-2 duration-200"
                                />
                            )}
                        </div>
                    )}
                </div>

                <div className="p-6 bg-gray-900/50 border-t border-gray-800 flex gap-3">
                    <button
                        onClick={onClose}
                        className="flex-1 px-4 py-2.5 rounded-lg border border-gray-700 text-gray-300 hover:bg-gray-800 transition-colors"
                    >
                        Cancel
                    </button>
                    <button
                        onClick={handleSave}
                        disabled={saving}
                        className="flex-1 px-4 py-2.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-medium transition-colors disabled:opacity-50"
                    >
                        {saving ? 'Saving...' : 'Save Settings'}
                    </button>
                </div>
            </div>
        </div>
    );
}
