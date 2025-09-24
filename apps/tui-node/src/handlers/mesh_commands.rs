use crate::utils::appstate_compat::AppState;
use crate::utils::state::{InternalCommand, MeshStatus};
use crate::utils::device::{send_webrtc_message, check_and_send_mesh_ready};
use crate::protocal::signal::WebRTCMessage;
use frost_core::Ciphersuite;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;

/// Handles reporting that a channel is open
pub async fn handle_report_channel_open<C>(
    device_id: String,
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
    self_device_id: String,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    // Mark device as connected
    let session_exists = {
        let mut guard = state.lock().await;
        guard.device_statuses.insert(device_id.clone(), RTCPeerConnectionState::Connected);
        guard.session.is_some()
    };

    if session_exists {
        // Send channel open message
        let msg = WebRTCMessage::ChannelOpen { device_id: self_device_id };
        let _ = send_webrtc_message(&device_id, &msg, state.clone()).await;
        
        // Check if mesh is ready
        check_and_send_mesh_ready(state, internal_cmd_tx).await;
    }
}

/// Handles sending own mesh ready signal
pub async fn handle_send_own_mesh_ready_signal<C>(
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    let internal_cmd_tx_clone = internal_cmd_tx.clone();
    
    tokio::spawn(async move { 
        let self_device_id_local;
        let session_id_local;
        let participants_local;
        let mut mesh_became_ready = false;
        
        { 
            let mut state_guard = state_clone.lock().await;
            self_device_id_local = state_guard.device_id.clone();

            if let Some(session) = &state_guard.session {
                // FIX: Check participants instead of participants
                let accepted_count = session.participants.len();
                let total_needed = session.total as usize;
                
                if accepted_count < total_needed {
                    return; // Wait for more acceptances
                }
                
                session_id_local = session.session_id.clone();
                participants_local = session.participants.clone();
                let participants = session.participants.clone();
                let total_needed = session.total as usize;
                
                let mut ready_devices = match &state_guard.mesh_status {
                    MeshStatus::PartiallyReady { ready_devices, .. } => ready_devices.clone(),
                    MeshStatus::Ready => return, // Already ready, nothing to do
                    _ => HashSet::new(),
                };
                
                ready_devices.insert(self_device_id_local.clone());

                if ready_devices.len() == total_needed {
                    state_guard.mesh_status = MeshStatus::Ready;
                    mesh_became_ready = true;
                    
                    // Create identifier map when mesh is ready
                    let mut sorted_participants = participants.clone();
                    sorted_participants.sort();
                    
                    let mut identifier_map = std::collections::HashMap::new();
                    for (index, device_id) in sorted_participants.iter().enumerate() {
                        let identifier_value = (index + 1) as u16;
                        let mut padded_bytes = [0u8; 32];
                        let bytes = identifier_value.to_be_bytes();
                        padded_bytes[30..32].copy_from_slice(&bytes);
                        
                        let identifier = frost_core::Identifier::<C>::deserialize(&padded_bytes)
                            .expect("Failed to create FROST identifier");
                        
                        identifier_map.insert(device_id.clone(), identifier);
                    }
                    
                    state_guard.identifier_map = Some(identifier_map);
                } else {
                    state_guard.mesh_status = MeshStatus::PartiallyReady {
                        ready_devices: ready_devices.clone(),
                        total_devices: total_needed,
                    };
                    // Mesh partially ready
                }
            } else {
                return;
            }
        } 
        
        if mesh_became_ready {
            let _ = internal_cmd_tx_clone.send(InternalCommand::CheckAndTriggerDkg);
        }

        // Set the flag FIRST to prevent race conditions
        state_clone.lock().await.own_mesh_ready_sent = true;
        
        // Then create and send mesh ready message to all devices
        let mesh_ready_msg = WebRTCMessage::MeshReady {
            session_id: session_id_local.clone(), 
            device_id: self_device_id_local.clone(), 
        };
        
        // Mesh ready message prepared

        let devices_to_notify: Vec<String> = participants_local
            .iter() 
            .filter(|p| **p != self_device_id_local)
            .cloned() 
            .collect();

        for device in devices_to_notify { 
            let _ = send_webrtc_message(&device, &mesh_ready_msg, state_clone.clone()).await;       
        }
    });
}

/// Handles processing a mesh ready message from another device
pub async fn handle_process_mesh_ready<C>(
    device_id: String,
    state: Arc<Mutex<AppState<C>>>,
    internal_cmd_tx: mpsc::UnboundedSender<InternalCommand<C>>,
) where
    C: Ciphersuite + Send + Sync + 'static,
    <<C as Ciphersuite>::Group as frost_core::Group>::Element: Send + Sync,
    <<<C as Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: Send + Sync,
{
    let state_clone = state.clone();
    let internal_cmd_tx_clone = internal_cmd_tx.clone();
    
    tokio::spawn(async move { 
        let mut mesh_became_ready = false;

        { 
            let mut state_guard = state_clone.lock().await;
            
            if let Some(session) = &state_guard.session {
                let _current_count = session.participants.len();
                let total_needed = session.total as usize;
                let accepted_count = session.participants.len();
                let participants = session.participants.clone();
                
                // FIX: Only check participants count
                if accepted_count < total_needed {
                    state_guard.pending_mesh_ready_signals.insert(device_id.clone());
                    return; // Wait for all acceptances
                }
                
                let mut ready_devices = match &state_guard.mesh_status {
                    MeshStatus::PartiallyReady { ready_devices, .. } => ready_devices.clone(),
                    MeshStatus::Ready => return, // Already ready
                    _ => HashSet::new(),
                };

                if ready_devices.contains(&device_id) {
                    return; // Already processed
                }
                
                ready_devices.insert(device_id.clone());
                
                if ready_devices.len() == total_needed {
                    state_guard.mesh_status = MeshStatus::Ready;
                    mesh_became_ready = true;
                    
                    // Create identifier map if needed
                    if state_guard.identifier_map.is_none() {
                        let mut sorted_participants = participants.clone();
                        sorted_participants.sort();
                        
                        let mut identifier_map = std::collections::HashMap::new();
                        for (index, device_id) in sorted_participants.iter().enumerate() {
                            let identifier_value = (index + 1) as u16;
                            let mut padded_bytes = [0u8; 32];
                            let bytes = identifier_value.to_be_bytes();
                            padded_bytes[30..32].copy_from_slice(&bytes);
                            
                            let identifier = frost_core::Identifier::<C>::deserialize(&padded_bytes)
                                .expect("Failed to create FROST identifier");
                            
                            identifier_map.insert(device_id.clone(), identifier);
                        }
                        
                        state_guard.identifier_map = Some(identifier_map);
                    }
                } else {
                    state_guard.mesh_status = MeshStatus::PartiallyReady {
                        ready_devices: ready_devices.clone(),
                        total_devices: total_needed,
                    };
                    // Mesh partially ready
                }
            } else {
                // No session yet, buffer for later
                state_guard.pending_mesh_ready_signals.insert(device_id.clone());
            }
        } 
        
        if mesh_became_ready {
            let _ = internal_cmd_tx_clone.send(InternalCommand::CheckAndTriggerDkg);
        }
    });
}
