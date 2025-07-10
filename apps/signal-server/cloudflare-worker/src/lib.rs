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
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMsg {
    Register { device_id: String },
    ListDevices,
    Relay { to: String, data: serde_json::Value },
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
        wasm_bindgen_futures::spawn_local({
            let server = server.clone();
            let devices = devices.clone();
            let state = state.clone();
            async move {
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
                                devices.borrow_mut().remove(&my_id);
                                // Remove from storage
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
            }
        });

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
