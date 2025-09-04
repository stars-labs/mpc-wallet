use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use worker::*;

// Global config: if true, newer registration overrides older for same device_id
const OVERRIDE_EXISTING_DEVICE: bool = true;

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
    // Simple session query response
    SessionsForDevice {
        sessions: Vec<serde_json::Value>,
    },
    // Notify when session is removed
    SessionRemoved {
        session_id: String,
        reason: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
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
    QueryMyActiveSessions,
}

// Durable Object for managing devices
#[durable_object]
pub struct Devices {
    devices: Rc<RefCell<HashMap<String, WebSocket>>>,
    state: Rc<State>,
}

#[durable_object]
impl DurableObject for Devices {
    fn new(state: State, _env: Env) -> Self {
        Self {
            devices: Rc::new(RefCell::new(HashMap::new())),
            state: Rc::new(state),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let upgrade_header = match req.headers().get("Upgrade") {
            Ok(Some(h)) => h,
            Ok(None) => "".to_string(),
            Err(_) => "".to_string(),
        };
        if upgrade_header != "websocket" {
            return Response::error("Expected Upgrade: websocket", 426);
        }

        let ws_pair = WebSocketPair::new()?;
        let client = ws_pair.client;
        let server = ws_pair.server;
        server.accept()?;

        let devices = self.devices.clone();
        let state = self.state.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let server = server.clone();
            let devices = devices.clone();
            let state = state.clone();
            {
                let mut device_id: Option<String> = None;
                let mut event_stream = server.events().expect("could not open stream");

                while let Some(event) = event_stream.next().await {
                    match event.expect("received error in websocket") {
                        WebsocketEvent::Message(msg) => {
                            if let Some(text) = msg.text() {
                                let parsed = serde_json::from_str::<ClientMsg>(&text);
                                match parsed {
                                    Ok(ClientMsg::Register { device_id: reg_id }) => {
                                        // Load device list from storage
                                        let mut device_list: Vec<String> = state
                                            .storage()
                                            .get("device_list")
                                            .await
                                            .unwrap_or_else(|_| Some(vec![]))
                                            .unwrap_or(vec![]);
                                        let already_registered = device_list.contains(&reg_id);
                                        if already_registered && !OVERRIDE_EXISTING_DEVICE {
                                            let err = ServerMsg::Error {
                                                error: "device_id already registered".to_string(),
                                            };
                                            let _ = server.send_with_str(
                                                &serde_json::to_string(&err).unwrap(),
                                            );
                                            break;
                                        }
                                        // If override is enabled, remove the old connection if present
                                        if already_registered && OVERRIDE_EXISTING_DEVICE {
                                            devices.borrow_mut().remove(&reg_id);
                                        }
                                        device_id = Some(reg_id.clone());
                                        devices.borrow_mut().insert(reg_id.clone(), server.clone());
                                        if !already_registered {
                                            device_list.push(reg_id.clone());
                                        }
                                        // Save updated device list to storage
                                        let _ = state.storage().put("device_list", &device_list).await;

                                        // Broadcast updated device list to all *other* devices
                                        let msg = ServerMsg::Devices {
                                            devices: device_list.clone(),
                                        };
                                        let msg_txt = serde_json::to_string(&msg).unwrap();
                                        for (id, ws) in devices.borrow().iter() {
                                            if id != &reg_id {
                                                let _ = ws.send_with_str(&msg_txt);
                                            }
                                        }
                                        // Optionally, send the device list to the newly registered node as well
                                        let _ = server.send_with_str(&msg_txt);
                                    }
                                    Ok(ClientMsg::ListDevices) => {
                                        // Load device list from storage
                                        let device_list: Vec<String> = state
                                            .storage()
                                            .get("device_list")
                                            .await
                                            .unwrap_or_else(|_| Some(vec![]))
                                            .unwrap_or(vec![]);
                                        let msg = ServerMsg::Devices { devices: device_list };
                                        let _ = server
                                            .send_with_str(&serde_json::to_string(&msg).unwrap());
                                    }
                                    Ok(ClientMsg::Relay { to, data }) => {
                                        // Check if this is a SessionUpdate to track active participants
                                        if let Ok(relay_msg) = serde_json::from_value::<serde_json::Value>(data.clone()) {
                                            if relay_msg.get("type").and_then(|v| v.as_str()) == Some("SessionUpdate") {
                                                if let (Some(session_code), Some(participants)) = (
                                                    relay_msg.get("session_code").and_then(|v| v.as_str()),
                                                    relay_msg.get("participants").and_then(|v| v.as_array())
                                                ) {
                                                    // Update session's active participants
                                                    let session_key = format!("session:{}", session_code);
                                                    if let Ok(mut session_data) = state.storage().get::<serde_json::Value>(&session_key).await {
                                                        // Update active participants based on who's connected
                                                        let mut active_participants = Vec::new();
                                                        for p in participants {
                                                            if let Some(participant_id) = p.as_str() {
                                                                if devices.borrow().contains_key(participant_id) {
                                                                    active_participants.push(participant_id.to_string());
                                                                }
                                                            }
                                                        }
                                                        session_data["active_participants"] = serde_json::json!(active_participants);
                                                        session_data["session_info"] = relay_msg.clone();
                                                        let _ = state.storage().put(&session_key, &session_data).await;
                                                        
                                                        // Update device sessions for new participants
                                                        for p in participants {
                                                            if let Some(participant_id) = p.as_str() {
                                                                let device_sessions_key = format!("device_sessions:{}", participant_id);
                                                                let mut device_sessions: Vec<String> = state.storage()
                                                                    .get(&device_sessions_key)
                                                                    .await
                                                                    .unwrap_or_else(|_| Some(vec![]))
                                                                    .unwrap_or(vec![]);
                                                                if !device_sessions.contains(&session_code.to_string()) {
                                                                    device_sessions.push(session_code.to_string());
                                                                    let _ = state.storage().put(&device_sessions_key, &device_sessions).await;
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // Relay the message
                                        let from = device_id.clone().unwrap_or_default();
                                        let relay = ServerMsg::Relay { from, data };
                                        let found = devices.borrow().get(&to).cloned();
                                        if let Some(ws) = found {
                                            let _ = ws.send_with_str(
                                                &serde_json::to_string(&relay).unwrap(),
                                            );
                                        } else {
                                            let err = ServerMsg::Error {
                                                error: format!("unknown device: {}", to),
                                            };
                                            let _ = server.send_with_str(
                                                &serde_json::to_string(&err).unwrap(),
                                            );
                                        }
                                    }
                                    Ok(ClientMsg::AnnounceSession { session_info }) => {
                                        // Store session bound to creator device
                                        if let Some(ref device) = device_id {
                                            let session_key = session_info.get("session_code")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("unknown")
                                                .to_string();
                                            
                                            // Store session with active participants
                                            let session_data = serde_json::json!({
                                                "session_info": session_info,
                                                "active_participants": vec![device.clone()]
                                            });
                                            let _ = state.storage().put(&format!("session:{}", session_key), &session_data).await;
                                            
                                            // Track session for this device
                                            let device_sessions_key = format!("device_sessions:{}", device);
                                            let mut device_sessions: Vec<String> = state.storage()
                                                .get(&device_sessions_key)
                                                .await
                                                .unwrap_or_else(|_| Some(vec![]))
                                                .unwrap_or(vec![]);
                                            if !device_sessions.contains(&session_key) {
                                                device_sessions.push(session_key.clone());
                                                let _ = state.storage().put(&device_sessions_key, &device_sessions).await;
                                            }
                                            
                                            // Broadcast to all connected devices
                                            let msg = ServerMsg::SessionAvailable { session_info };
                                            let msg_str = serde_json::to_string(&msg).unwrap();
                                            for (id, ws) in devices.borrow().iter() {
                                                if id != device {
                                                    let _ = ws.send_with_str(&msg_str);
                                                }
                                            }
                                        }
                                    }
                                    Ok(ClientMsg::RequestActiveSessions) => {
                                        // Broadcast the request to all devices
                                        // They will respond with their active sessions
                                        if let Some(from_id) = &device_id {
                                            let msg = ServerMsg::SessionListRequest {
                                                from: from_id.clone(),
                                            };
                                            let msg_str = serde_json::to_string(&msg).unwrap();
                                            for (id, ws) in devices.borrow().iter() {
                                                if from_id != id {
                                                    let _ = ws.send_with_str(&msg_str);
                                                }
                                            }
                                        }
                                    }
                                    Ok(ClientMsg::QueryMyActiveSessions) => {
                                        // Return all sessions where this device is a participant  
                                        if let Some(ref dev_id) = device_id {
                                            let mut my_sessions = Vec::new();
                                            let mut tracked_sessions = Vec::new();
                                            
                                            // Scan ALL sessions to find where device is participant
                                            let list_result = state.storage().list().await;
                                            if let Ok(keys) = list_result {
                                                for key_result in keys.keys() {
                                                    if let Ok(key_value) = key_result {
                                                        if let Some(key_str) = key_value.as_string() {
                                                            if key_str.starts_with("session:") {
                                                                if let Ok(mut session_data) = state.storage().get::<serde_json::Value>(&key_str).await {
                                                                    if let Some(info) = session_data.get("session_info").cloned() {
                                                                        // Check if device is in participants
                                                                        if let Some(participants) = info.get("participants").and_then(|v| v.as_array()) {
                                                                            let is_participant = participants.iter()
                                                                                .any(|p| p.as_str() == Some(dev_id.as_str()));
                                                                            if is_participant {
                                                                                // Add to active participants if rejoining
                                                                                if let Some(active) = session_data.get_mut("active_participants").and_then(|v| v.as_array_mut()) {
                                                                                    let dev_value = serde_json::Value::String(dev_id.clone());
                                                                                    if !active.contains(&dev_value) {
                                                                                        active.push(dev_value);
                                                                                        let _ = state.storage().put(&key_str, &session_data).await;
                                                                                    }
                                                                                }
                                                                                my_sessions.push(info);
                                                                                tracked_sessions.push(key_str.replace("session:", ""));
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            // Update device sessions tracking
                                            let device_sessions_key = format!("device_sessions:{}", dev_id);
                                            let _ = state.storage().put(&device_sessions_key, &tracked_sessions).await;
                                            
                                            // Send response
                                            let response = ServerMsg::SessionsForDevice {
                                                sessions: my_sessions,
                                            };
                                            let _ = server.send_with_str(&serde_json::to_string(&response).unwrap());
                                        }
                                    }
                                    Ok(ClientMsg::SessionStatusUpdate { session_info }) => {
                                        // Legacy - kept for compatibility
                                        let session_key = session_info.get("session_code")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown")
                                            .to_string();
                                        let _ = state.storage().put(&format!("session:{}", session_key), &session_info).await;
                                    }
                                    Err(_) => {
                                        let err = ServerMsg::Error {
                                            error: "invalid message".to_string(),
                                        };
                                        let _ = server
                                            .send_with_str(&serde_json::to_string(&err).unwrap());
                                    }
                                }
                            }
                        }
                        WebsocketEvent::Close(_event) => {
                            // Cleanup on disconnect
                            if let Some(my_id) = device_id.clone() {
                                // Remove device from active participants in sessions
                                let device_sessions_key = format!("device_sessions:{}", my_id);
                                if let Ok(session_ids) = state.storage().get::<Vec<String>>(&device_sessions_key).await {
                                    let mut sessions_to_remove = Vec::new();
                                    
                                    for session_id in &session_ids {
                                        let session_key = format!("session:{}", session_id);
                                        if let Ok(mut session_data) = state.storage().get::<serde_json::Value>(&session_key).await {
                                            // Remove from active participants
                                            if let Some(active) = session_data.get_mut("active_participants").and_then(|v| v.as_array_mut()) {
                                                active.retain(|p| p.as_str() != Some(&my_id));
                                                
                                                // Only remove session if NO active participants remain
                                                if active.is_empty() {
                                                    sessions_to_remove.push(session_id.clone());
                                                    let _ = state.storage().delete(&session_key).await;
                                                } else {
                                                    // Session continues with remaining participants
                                                    let _ = state.storage().put(&session_key, &session_data).await;
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Notify about removed sessions only
                                    for session_id in sessions_to_remove {
                                        let msg = ServerMsg::SessionRemoved {
                                            session_id: session_id.clone(),
                                            reason: "All participants disconnected".to_string(),
                                        };
                                        let msg_str = serde_json::to_string(&msg).unwrap();
                                        for (_id, ws) in devices.borrow().iter() {
                                            let _ = ws.send_with_str(&msg_str);
                                        }
                                    }
                                    
                                    // Delete the device's session list
                                    let _ = state.storage().delete(&device_sessions_key).await;
                                }
                                
                                // Now remove device from active list
                                devices.borrow_mut().remove(&my_id);
                                let mut device_list: Vec<String> = state
                                    .storage()
                                    .get("device_list")
                                    .await
                                    .unwrap_or_else(|_| Some(vec![]))
                                    .unwrap_or(vec![]);
                                device_list.retain(|id| id != &my_id);
                                let _ = state.storage().put("device_list", &device_list).await;
                                
                                // Broadcast updated device list
                                let msg = ServerMsg::Devices {
                                    devices: device_list.clone(),
                                };
                                for (_id, ws) in devices.borrow().iter() {
                                    let _ = ws.send_with_str(&serde_json::to_string(&msg).unwrap());
                                }
                            }
                        }
                    }
                }
            }  // End of while loop
        });  // End of spawn_local (async move)

        Response::from_websocket(client)
    }
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Route all websocket requests to the Devices Durable Object
    let devices_ns = env.durable_object("Devices")?;
    let id = devices_ns.id_from_name("global")?;
    let stub = id.get_stub()?;
    stub.fetch_with_request(req).await
}
