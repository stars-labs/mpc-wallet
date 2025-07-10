use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Register { device_id: String },
    ListDevices,
    Relay { to: String, data: serde_json::Value },
}
