import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface TraceEvent {
    id: string;
    session_id: string;
    timestamp: number;
    level: string;
    target: string;
    span_name: string | null;
    message: string;
    fields: string;
}

const levelColors: Record<string, string> = {
    ERROR: "text-red-400 bg-red-500/10",
    WARN: "text-yellow-400 bg-yellow-500/10",
    INFO: "text-blue-400 bg-blue-500/10",
    DEBUG: "text-gray-400 bg-gray-500/10",
    TRACE: "text-gray-500 bg-gray-600/10",
};

const levelBadgeColors: Record<string, string> = {
    ERROR: "bg-red-500",
    WARN: "bg-yellow-500",
    INFO: "bg-blue-500",
    DEBUG: "bg-gray-500",
    TRACE: "bg-gray-600",
};

export function TraceViewer() {
    const [traces, setTraces] = useState<TraceEvent[]>([]);
    const [filter, setFilter] = useState<string>("");
    const [levelFilter, setLevelFilter] = useState<string>("ALL");
    const [autoScroll, setAutoScroll] = useState(true);
    const [expandedId, setExpandedId] = useState<string | null>(null);

    // Load initial traces
    useEffect(() => {
        const loadTraces = async () => {
            try {
                const level = levelFilter === "ALL" ? null : levelFilter;
                const result = await invoke<TraceEvent[]>("get_traces", { level });
                setTraces(result);
            } catch (err) {
                console.error("Failed to load traces:", err);
            }
        };

        loadTraces();
        const interval = setInterval(loadTraces, 1000);
        return () => clearInterval(interval);
    }, [levelFilter]);

    // Listen for real-time trace events
    useEffect(() => {
        const unlisten = listen<TraceEvent>("trace-event", (event) => {
            setTraces((prev) => [...prev, event.payload]);
        });

        return () => {
            unlisten.then((fn) => fn());
        };
    }, []);

    // Auto-scroll to bottom
    useEffect(() => {
        if (autoScroll) {
            const container = document.getElementById("trace-container");
            if (container) {
                container.scrollTop = container.scrollHeight;
            }
        }
    }, [traces, autoScroll]);

    const handleClear = async () => {
        try {
            await invoke("clear_traces");
            setTraces([]);
        } catch (err) {
            console.error("Failed to clear traces:", err);
        }
    };

    const filteredTraces = traces.filter((trace) => {
        if (filter && !trace.message.toLowerCase().includes(filter.toLowerCase()) &&
            !trace.target.toLowerCase().includes(filter.toLowerCase())) {
            return false;
        }
        return true;
    });

    const formatTimestamp = (ts: number) => {
        const date = new Date(ts);
        return date.toLocaleTimeString("en-US", {
            hour12: false,
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
        }) + "." + String(date.getMilliseconds()).padStart(3, "0");
    };

    const parseFields = (fields: string): Record<string, unknown> => {
        try {
            return JSON.parse(fields);
        } catch {
            return {};
        }
    };

    return (
        <div className="flex flex-col h-full bg-gray-950 rounded-xl border border-gray-800 overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between p-3 border-b border-gray-800 bg-black/30">
                <div className="flex items-center gap-2">
                    <div className="w-2 h-2 rounded-full bg-green-500 animate-pulse" />
                    <h2 className="text-sm font-bold text-white/90 uppercase tracking-wider">
                        Flight Recorder
                    </h2>
                    <span className="text-xs text-gray-500">
                        {filteredTraces.length} events
                    </span>
                </div>
                <div className="flex items-center gap-2">
                    <button
                        onClick={handleClear}
                        className="px-2 py-1 text-xs bg-red-500/20 text-red-400 rounded hover:bg-red-500/30 transition-colors"
                    >
                        Clear
                    </button>
                </div>
            </div>

            {/* Filters */}
            <div className="flex items-center gap-2 p-2 border-b border-gray-800/50 bg-gray-900/30">
                <input
                    type="text"
                    value={filter}
                    onChange={(e) => setFilter(e.target.value)}
                    placeholder="Filter traces..."
                    className="flex-1 bg-gray-800 border border-gray-700 rounded px-2 py-1 text-xs text-white placeholder:text-gray-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
                />
                <select
                    value={levelFilter}
                    onChange={(e) => setLevelFilter(e.target.value)}
                    className="bg-gray-800 border border-gray-700 rounded px-2 py-1 text-xs text-white focus:outline-none focus:ring-1 focus:ring-blue-500"
                >
                    <option value="ALL">All Levels</option>
                    <option value="ERROR">ERROR</option>
                    <option value="WARN">WARN</option>
                    <option value="INFO">INFO</option>
                    <option value="DEBUG">DEBUG</option>
                </select>
                <label className="flex items-center gap-1 text-xs text-gray-400 cursor-pointer">
                    <input
                        type="checkbox"
                        checked={autoScroll}
                        onChange={(e) => setAutoScroll(e.target.checked)}
                        className="w-3 h-3"
                    />
                    Auto-scroll
                </label>
            </div>

            {/* Trace List */}
            <div
                id="trace-container"
                className="flex-1 overflow-y-auto overflow-x-hidden"
            >
                {filteredTraces.length === 0 ? (
                    <div className="flex items-center justify-center h-full text-gray-500 text-sm">
                        No traces recorded yet. Run the agent to see traces.
                    </div>
                ) : (
                    <div className="divide-y divide-gray-800/50">
                        {filteredTraces.map((trace) => {
                            const fields = parseFields(trace.fields);
                            const hasFields = Object.keys(fields).length > 0;
                            const isExpanded = expandedId === trace.id;

                            return (
                                <div
                                    key={trace.id}
                                    className={`px-3 py-2 hover:bg-gray-800/30 cursor-pointer transition-colors ${levelColors[trace.level] || ""}`}
                                    onClick={() => hasFields && setExpandedId(isExpanded ? null : trace.id)}
                                >
                                    <div className="flex items-start gap-2">
                                        {/* Timestamp */}
                                        <span className="text-[10px] text-gray-500 font-mono shrink-0 w-20">
                                            {formatTimestamp(trace.timestamp)}
                                        </span>

                                        {/* Level Badge */}
                                        <span
                                            className={`text-[9px] font-bold px-1.5 py-0.5 rounded shrink-0 text-white ${levelBadgeColors[trace.level] || "bg-gray-600"}`}
                                        >
                                            {trace.level}
                                        </span>

                                        {/* Target */}
                                        <span className="text-[10px] text-gray-400 font-mono shrink-0 truncate max-w-32">
                                            {trace.target}
                                        </span>

                                        {/* Message */}
                                        <span className="text-xs text-white/80 flex-1 truncate">
                                            {trace.message}
                                        </span>

                                        {/* Expand indicator */}
                                        {hasFields && (
                                            <span className="text-gray-500 text-xs">
                                                {isExpanded ? "▼" : "▶"}
                                            </span>
                                        )}
                                    </div>

                                    {/* Expanded Fields */}
                                    {isExpanded && hasFields && (
                                        <div className="mt-2 ml-24 p-2 bg-gray-900 rounded text-xs font-mono">
                                            <pre className="text-gray-300 whitespace-pre-wrap wrap-break-word">
                                                {JSON.stringify(fields, null, 2)}
                                            </pre>
                                        </div>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                )}
            </div>
        </div>
    );
}
