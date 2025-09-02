use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use webrtc_signal_server::{ClientMsg, ServerMsg};
use webrtc_signal_server::session_manager::{
    SessionManager, SessionStorage, InMemorySessionStorage, StoredSession
};

type DeviceSender = mpsc::UnboundedSender<Message>;
type DeviceMap = Arc<Mutex<HashMap<String, DeviceSender>>>;
type SessionStore = Arc<Mutex<InMemorySessionStorage>>;

#[tokio::main]
async fn main() {
    let devices: DeviceMap = Arc::new(Mutex::new(HashMap::new()));
    let session_store: SessionStore = Arc::new(Mutex::new(InMemorySessionStorage::new()));
    let listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();
    println!("Signal server listening on 0.0.0.0:9000");

    let shutdown_signal = async {
        signal::ctrl_c()
            .await
            .expect("Failed to listen for shutdown signal");
        println!("Shutdown signal received. Terminating...");
    };

    let server = async {
        while let Ok((stream, _)) = listener.accept().await {
            let devices = devices.clone();
            let session_store = session_store.clone();

            tokio::spawn(async move {
                let ws_stream = accept_async(stream).await.unwrap();
                let (mut ws_sink, mut ws_stream) = ws_stream.split();
                let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
                let mut device_id: Option<String> = None;

                // Task to forward messages from rx to ws_sink
                let ws_sink_task = tokio::spawn(async move {
                    while let Some(msg) = rx.recv().await {
                        if ws_sink.send(msg).await.is_err() {
                            break;
                        }
                    }
                });

                loop {
                    tokio::select! {
                        Some(msg) = ws_stream.next() => {
                            let msg = match msg {
                                Ok(m) if m.is_text() => m.into_text().unwrap(),
                                Ok(m) if m.is_close() => break,
                                _ => continue,
                            };

                            let parsed: Result<ClientMsg, _> = serde_json::from_str(&msg);

                            match parsed {
                                Ok(ClientMsg::Register { device_id: reg_id }) => {
                                    let mut devices_guard = devices.lock().unwrap();
                                    if devices_guard.contains_key(&reg_id) {
                                        let err = ServerMsg::Error { error: "device_id already registered".to_string() };
                                        let _ = tx.send(Message::Text(serde_json::to_string(&err).unwrap().into()));
                                        break;
                                    }
                                    device_id = Some(reg_id.clone());
                                    devices_guard.insert(reg_id.clone(), tx.clone());
                                    println!("Registered device: {}", reg_id);

                                    // Broadcast updated device list
                                    let device_list: Vec<String> = devices_guard.keys().cloned().collect();
                                    let msg = ServerMsg::Devices { devices: device_list.clone() };
                                    let msg_txt = serde_json::to_string(&msg).unwrap();
                                    for (_id, ptx) in devices_guard.iter() {
                                        let _ = ptx.send(Message::Text(msg_txt.clone().into()));
                                    }
                                }
                                Ok(ClientMsg::ListDevices) => {
                                    let devices_guard = devices.lock().unwrap();
                                    let device_list: Vec<String> = devices_guard.keys().cloned().collect();
                                    let msg = ServerMsg::Devices { devices: device_list };
                                    let _ = tx.send(Message::Text(serde_json::to_string(&msg).unwrap().into()));
                                }
                                Ok(ClientMsg::Relay { to, data }) => {
                                    // Check if this is a SessionUpdate using shared logic
                                    let devices_guard = devices.lock().unwrap();
                                    let connected_devices: Vec<String> = devices_guard.keys().cloned().collect();
                                    drop(devices_guard);
                                    
                                    let mut store_guard = session_store.lock().unwrap();
                                    SessionManager::process_session_update(
                                        &data, 
                                        &mut *store_guard,
                                        &connected_devices
                                    );
                                    drop(store_guard);
                                    
                                    // Relay the message
                                    let devices_guard = devices.lock().unwrap();
                                    if to == "*" {
                                        let relay = ServerMsg::Relay {
                                            from: device_id.as_deref().unwrap_or_default().to_string(),
                                            data: data.clone(),
                                        };
                                        let relay_text = serde_json::to_string(&relay).unwrap();
                                        
                                        for (id, device_tx) in devices_guard.iter() {
                                            if Some(id) != device_id.as_ref() {
                                                let _ = device_tx.send(Message::Text(relay_text.clone().into()));
                                            }
                                        }
                                    } else {
                                        if let Some(device_tx) = devices_guard.get(&to) {
                                            let relay = ServerMsg::Relay {
                                                from: device_id.as_deref().unwrap_or_default().to_string(),
                                                data: data.clone(),
                                            };
                                            let _ = device_tx.send(Message::Text(serde_json::to_string(&relay).unwrap().into()));
                                        } else {
                                            let err = ServerMsg::Error { error: format!("unknown device: {}", to) };
                                            let _ = tx.send(Message::Text(serde_json::to_string(&err).unwrap().into()));
                                        }
                                    }
                                }
                                Ok(ClientMsg::AnnounceSession { session_info }) => {
                                    if let Some(ref device) = device_id {
                                        let session_key = SessionManager::extract_session_key(&session_info);
                                        
                                        // Store session using shared logic
                                        let stored_session = StoredSession {
                                            session_info: session_info.clone(),
                                            active_participants: vec![device.clone()],
                                        };
                                        
                                        let mut store_guard = session_store.lock().unwrap();
                                        store_guard.store_session(session_key.clone(), stored_session);
                                        store_guard.add_device_session(device.clone(), session_key.clone());
                                        drop(store_guard);
                                        
                                        println!("Stored session '{}' from device '{}'", session_key, device);
                                    }
                                    
                                    // Broadcast session announcement
                                    let devices_guard = devices.lock().unwrap();
                                    let msg = ServerMsg::SessionAvailable { session_info };
                                    let msg_txt = serde_json::to_string(&msg).unwrap();
                                    for (id, device_tx) in devices_guard.iter() {
                                        if Some(id) != device_id.as_ref() {
                                            let _ = device_tx.send(Message::Text(msg_txt.clone().into()));
                                        }
                                    }
                                }
                                Ok(ClientMsg::RequestActiveSessions) => {
                                    println!("Session list request from {}", device_id.as_deref().unwrap_or("unknown"));
                                    
                                    // Send all stored sessions
                                    let store_guard = session_store.lock().unwrap();
                                    for (_key, session) in store_guard.get_all_sessions() {
                                        let msg = ServerMsg::SessionAvailable { 
                                            session_info: session.session_info.clone() 
                                        };
                                        let msg_txt = serde_json::to_string(&msg).unwrap();
                                        let _ = tx.send(Message::Text(msg_txt.into()));
                                    }
                                    drop(store_guard);
                                    
                                    // Also broadcast request
                                    let devices_guard = devices.lock().unwrap();
                                    let msg = ServerMsg::SessionListRequest {
                                        from: device_id.as_deref().unwrap_or_default().to_string(),
                                    };
                                    let msg_txt = serde_json::to_string(&msg).unwrap();
                                    for (id, device_tx) in devices_guard.iter() {
                                        if Some(id) != device_id.as_ref() {
                                            let _ = device_tx.send(Message::Text(msg_txt.clone().into()));
                                        }
                                    }
                                }
                                Ok(ClientMsg::QueryMyActiveSessions) => {
                                    if let Some(ref dev_id) = device_id {
                                        println!("Device '{}' querying for active sessions", dev_id);
                                        
                                        // Use shared logic for rejoin
                                        let mut store_guard = session_store.lock().unwrap();
                                        let my_sessions = SessionManager::handle_device_rejoin(
                                            dev_id,
                                            &mut *store_guard
                                        );
                                        drop(store_guard);
                                        
                                        let response = ServerMsg::SessionsForDevice {
                                            sessions: my_sessions.clone(),
                                        };
                                        let msg_txt = serde_json::to_string(&response).unwrap();
                                        println!("Found {} sessions for device '{}'", my_sessions.len(), dev_id);
                                        let _ = tx.send(Message::Text(msg_txt.into()));
                                    }
                                }
                                Ok(ClientMsg::SessionStatusUpdate { session_info }) => {
                                    println!("Session status update from {}: {:?}", 
                                        device_id.as_deref().unwrap_or("unknown"), session_info);
                                }
                                Err(_) => {
                                    let err = ServerMsg::Error { error: "invalid message".to_string() };
                                    let _ = tx.send(Message::Text(serde_json::to_string(&err).unwrap().into()));
                                }
                            }
                        }
                        else => break,
                    }
                }

                // Cleanup on disconnect using shared logic
                if let Some(my_id) = device_id {
                    let mut store_guard = session_store.lock().unwrap();
                    let removed_sessions = SessionManager::handle_device_disconnect(
                        &my_id,
                        &mut *store_guard
                    );
                    drop(store_guard);
                    
                    // Notify about removed sessions
                    if !removed_sessions.is_empty() {
                        let devices_guard = devices.lock().unwrap();
                        for removed_session_id in removed_sessions {
                            let msg = ServerMsg::SessionRemoved {
                                session_id: removed_session_id.clone(),
                                reason: "All participants disconnected".to_string(),
                            };
                            let msg_txt = serde_json::to_string(&msg).unwrap();
                            for (_id, device_tx) in devices_guard.iter() {
                                let _ = device_tx.send(Message::Text(msg_txt.clone().into()));
                            }
                        }
                        drop(devices_guard);
                        
                        println!("Removed {} sessions after '{}' disconnected", 
                            removed_sessions.len(), my_id);
                    }
                    
                    // Remove device from active list
                    let mut devices_guard = devices.lock().unwrap();
                    devices_guard.remove(&my_id);
                    println!("Device {} disconnected", my_id);

                    // Broadcast updated device list
                    let device_list: Vec<String> = devices_guard.keys().cloned().collect();
                    let msg = ServerMsg::Devices {
                        devices: device_list.clone(),
                    };
                    let msg_txt = serde_json::to_string(&msg).unwrap();
                    for (_id, ptx) in devices_guard.iter() {
                        let _ = ptx.send(Message::Text(msg_txt.clone().into()));
                    }
                }
                ws_sink_task.abort();
            });
        }
    };

    tokio::select! {
        _ = server => {},
        _ = shutdown_signal => {},
    }

    println!("Server has shut down.");
}