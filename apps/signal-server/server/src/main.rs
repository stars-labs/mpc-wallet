use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::mpsc; // Add this import for signal handling

use tokio_tungstenite::{accept_async, tungstenite::Message};

// Import shared types from the library crate

use webrtc_signal_server::{ClientMsg, ServerMsg};

type DeviceSender = mpsc::UnboundedSender<Message>;
type DeviceMap = Arc<Mutex<HashMap<String, DeviceSender>>>;

#[tokio::main]
async fn main() {
    let devices: DeviceMap = Arc::new(Mutex::new(HashMap::new()));
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

                                    // Broadcast updated device list to all devices (owned Vec)
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
                                    let devices_guard = devices.lock().unwrap();
                                    if let Some(device_tx) = devices_guard.get(&to) {
                                        let relay = ServerMsg::Relay {
                                            from: device_id.as_deref().unwrap_or_default().to_string(),
                                            data: data.clone(), // Clone data for the message
                                        };
                                        // Log the relay action
                                        println!("Relaying message from {} to {}: {:?}", device_id.as_deref().unwrap_or("unknown"), to, data);
                                        let _ = device_tx.send(Message::Text(serde_json::to_string(&relay).unwrap().into()));
                                    } else {
                                        println!("Relay failed: unknown device {}", to);
                                        let err = ServerMsg::Error { error: format!("unknown device: {}", to) };
                                        let _ = tx.send(Message::Text(serde_json::to_string(&err).unwrap().into()));
                                    }
                                    // Explicitly drop the lock
                                    drop(devices_guard);
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

                // Cleanup on disconnect
                if let Some(my_id) = device_id {
                    let mut devices_guard = devices.lock().unwrap();
                    devices_guard.remove(&my_id);
                    println!("Device {} disconnected", my_id);

                    // Broadcast updated device list to all devices (owned Vec)
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
