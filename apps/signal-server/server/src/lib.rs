use serde::{Deserialize, Serialize};

pub mod session_manager;
pub mod cloudflare_storage;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub total: usize,
    pub threshold: usize,
    pub participants: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMsg {
    Devices {
        devices: Vec<String>,
    },
    Relay {
        from: String,
        data: serde_json::Value,
    },
    Error {
        error: String,
    },
    // Session discovery messages
    SessionAvailable {
        session_info: serde_json::Value,
    },
    SessionListRequest {
        from: String,
    },
    // Simple session query response - just return what device was in
    SessionsForDevice {
        sessions: Vec<serde_json::Value>,  // List of session_info objects
    },
    // Notify when session is removed (creator disconnected)
    SessionRemoved {
        session_id: String,
        reason: String,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Register { device_id: String },
    ListDevices,
    Relay { to: String, data: serde_json::Value },
    // Session discovery messages
    AnnounceSession { session_info: serde_json::Value },
    RequestActiveSessions,
    SessionStatusUpdate { session_info: serde_json::Value },
    // Simple stateless rejoin support
    QueryMyActiveSessions,  // Device asks: "What sessions am I in?"
}
