//! Flight Recorder: Comprehensive tracing with SQLite persistence
//!
//! Provides operation-level tracing that records every significant action
//! the agent performs, stored in SQLite for debugging and observability.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use tauri_plugin_sql::{Migration, MigrationKind};
use tokio::sync::Mutex;
use uuid::Uuid;

/// Global trace store instance
pub static TRACE_STORE: OnceLock<Arc<Mutex<TraceStore>>> = OnceLock::new();

/// Represents a single trace event in the flight recorder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub id: String,
    pub session_id: String,
    pub timestamp: i64,
    pub level: String,
    pub target: String,
    pub span_name: Option<String>,
    pub message: String,
    pub fields: String, // JSON-encoded additional data
}

/// Manages trace storage and retrieval
pub struct TraceStore {
    session_id: String,
    events: Vec<TraceEvent>, // In-memory buffer, synced to SQLite
}

impl TraceStore {
    pub fn new() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            events: Vec::new(),
        }
    }

    /// Get the current session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Record a trace event
    pub fn record(
        &mut self,
        level: &str,
        target: &str,
        span_name: Option<&str>,
        message: &str,
        fields: serde_json::Value,
    ) {
        let event = TraceEvent {
            id: Uuid::new_v4().to_string(),
            session_id: self.session_id.clone(),
            timestamp: Utc::now().timestamp_millis(),
            level: level.to_string(),
            target: target.to_string(),
            span_name: span_name.map(|s| s.to_string()),
            message: message.to_string(),
            fields: fields.to_string(),
        };

        // Emit to frontend immediately
        if let Some(app) = crate::GLOBAL_APP.get() {
            let _ = tauri::Emitter::emit(app, "trace-event", &event);
        }

        self.events.push(event);
    }

    /// Get all events for the current session
    pub fn get_events(&self) -> Vec<TraceEvent> {
        self.events.clone()
    }

    /// Get events filtered by level
    pub fn get_events_by_level(&self, level: &str) -> Vec<TraceEvent> {
        self.events
            .iter()
            .filter(|e| e.level == level)
            .cloned()
            .collect()
    }

    /// Clear all events and start a new session
    pub fn reset(&mut self) {
        self.events.clear();
        self.session_id = Uuid::new_v4().to_string();
    }

    /// Get event count
    pub fn count(&self) -> usize {
        self.events.len()
    }
}

/// Initialize the trace store
pub fn init_tracing() {
    let store = Arc::new(Mutex::new(TraceStore::new()));
    let _ = TRACE_STORE.set(store);
}

/// Get SQLite migrations for trace table
pub fn get_migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "Create traces table",
        sql: r#"
            CREATE TABLE IF NOT EXISTS traces (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                level TEXT NOT NULL,
                target TEXT NOT NULL,
                span_name TEXT,
                message TEXT NOT NULL,
                fields TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_traces_session ON traces(session_id);
            CREATE INDEX IF NOT EXISTS idx_traces_timestamp ON traces(timestamp);
            CREATE INDEX IF NOT EXISTS idx_traces_level ON traces(level);
        "#,
        kind: MigrationKind::Up,
    }]
}

// ============================================================================
// Tracing Macros - Operation-level logging helpers
// ============================================================================

/// Record an INFO level trace
#[macro_export]
macro_rules! trace_info {
    ($target:expr, $msg:expr) => {
        $crate::tracing::record_trace("INFO", $target, None, $msg, serde_json::json!({}))
    };
    ($target:expr, $msg:expr, $($key:ident = $value:expr),+) => {
        $crate::tracing::record_trace("INFO", $target, None, $msg, serde_json::json!({
            $(stringify!($key): $value),+
        }))
    };
}

/// Record a DEBUG level trace
#[macro_export]
macro_rules! trace_debug {
    ($target:expr, $msg:expr) => {
        $crate::tracing::record_trace("DEBUG", $target, None, $msg, serde_json::json!({}))
    };
    ($target:expr, $msg:expr, $($key:ident = $value:expr),+) => {
        $crate::tracing::record_trace("DEBUG", $target, None, $msg, serde_json::json!({
            $(stringify!($key): $value),+
        }))
    };
}

/// Record an ERROR level trace
#[macro_export]
macro_rules! trace_error {
    ($target:expr, $msg:expr) => {
        $crate::tracing::record_trace("ERROR", $target, None, $msg, serde_json::json!({}))
    };
    ($target:expr, $msg:expr, $($key:ident = $value:expr),+) => {
        $crate::tracing::record_trace("ERROR", $target, None, $msg, serde_json::json!({
            $(stringify!($key): $value),+
        }))
    };
}

/// Record a WARN level trace
#[macro_export]
macro_rules! trace_warn {
    ($target:expr, $msg:expr) => {
        $crate::tracing::record_trace("WARN", $target, None, $msg, serde_json::json!({}))
    };
    ($target:expr, $msg:expr, $($key:ident = $value:expr),+) => {
        $crate::tracing::record_trace("WARN", $target, None, $msg, serde_json::json!({
            $(stringify!($key): $value),+
        }))
    };
}

/// Helper function called by macros
pub fn record_trace(
    level: &str,
    target: &str,
    span_name: Option<&str>,
    message: &str,
    fields: serde_json::Value,
) {
    if let Some(store) = TRACE_STORE.get() {
        // Use try_lock to avoid blocking - traces are best-effort
        if let Ok(mut guard) = store.try_lock() {
            guard.record(level, target, span_name, message, fields);
        }
    }
}
