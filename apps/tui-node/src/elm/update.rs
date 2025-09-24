//! Update - The state transition function
//!
//! The update function is the heart of the Elm Architecture. It takes the current
//! model and a message, and returns an updated model along with optional commands
//! to execute side effects.

use crate::elm::model::{Model, Screen, Modal, Notification, NotificationKind, ConnectionStatus, Operation, ProgressInfo, WalletConfig, CurveType, WalletMode, CreateWalletState};
use crate::elm::message::{Message, DKGRound};
use crate::elm::command::Command;
use crate::protocal::signal::{SessionInfo, SessionType};
use chrono::Utc;
use crossterm::event::{KeyCode, KeyModifiers};
use tracing::{info, debug, warn, error};
use uuid::Uuid;

/// The main update function that handles all state transitions
pub fn update(model: &mut Model, msg: Message) -> Option<Command> {
    debug!("Processing message: {:?}", msg);
    
    match msg {
        // ============= Navigation Messages =============
        Message::Navigate(screen) => {
            info!("Navigating to screen: {:?}", screen);
            model.push_screen(screen.clone());
            
            // Update focus based on screen
            match screen {
                Screen::CreateWallet(_) => {
                    model.ui_state.focus = crate::elm::model::ComponentId::CreateWallet;
                    // Initialize selected index for CreateWallet if not exists
                    model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::CreateWallet).or_insert(0);
                    debug!("🎯 CreateWallet focus set, selected index: {}", 
                           model.ui_state.selected_indices[&crate::elm::model::ComponentId::CreateWallet]);
                }
                Screen::ModeSelection => {
                    model.ui_state.focus = crate::elm::model::ComponentId::ModeSelection;
                    model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::ModeSelection).or_insert(0);
                    debug!("🎯 ModeSelection focus set");
                }
                Screen::CurveSelection => {
                    model.ui_state.focus = crate::elm::model::ComponentId::CurveSelection;
                    model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::CurveSelection).or_insert(0);
                    debug!("🎯 CurveSelection focus set");
                }
                Screen::ManageWallets => {
                    model.ui_state.focus = crate::elm::model::ComponentId::WalletList;
                }
                Screen::JoinSession => {
                    model.ui_state.focus = crate::elm::model::ComponentId::JoinSession;
                    model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::JoinSession).or_insert(0);
                    debug!("🎯 JoinSession focus set");
                }
                Screen::MainMenu | Screen::Welcome => {
                    model.ui_state.focus = crate::elm::model::ComponentId::MainMenu;
                }
                _ => {}
            }
            
            // Load data for the new screen if needed
            match screen {
                Screen::ManageWallets => Some(Command::LoadWallets),
                Screen::JoinSession => Some(Command::LoadSessions),
                _ => None,
            }
        }
        
        Message::NavigateBack => {
            debug!("🔙 NavigateBack message received!");
            debug!("Current screen: {:?}", model.current_screen);
            debug!("Navigation stack length: {}", model.navigation_stack.len());
            
            // Check if we're at the root screen (main menu with empty stack)
            if model.navigation_stack.is_empty() && matches!(model.current_screen, Screen::MainMenu | Screen::Welcome) {
                // At root level - Esc should quit the app
                debug!("🚪 At root screen, Esc should quit");
                return Some(Command::SendMessage(Message::Quit));
            }
            
            // Otherwise, navigate back normally
            if !model.pop_screen() {
                // Fallback - shouldn't happen after the check above
                debug!("🚨 Already at root screen, staying put");
            } else {
                debug!("✅ Successfully popped screen, new current screen: {:?}", model.current_screen);
                // Update focus based on new current screen
                match model.current_screen {
                    Screen::MainMenu | Screen::Welcome => {
                        model.ui_state.focus = crate::elm::model::ComponentId::MainMenu;
                        debug!("🎯 Focus set to MainMenu");
                    }
                    Screen::ManageWallets => {
                        model.ui_state.focus = crate::elm::model::ComponentId::WalletList;
                        debug!("🎯 Focus set to WalletList");
                    }
                    Screen::CreateWallet(_) => {
                        model.ui_state.focus = crate::elm::model::ComponentId::CreateWallet;
                        debug!("🎯 Focus set to CreateWallet");
                    }
                    _ => {
                        debug!("🎯 No specific focus set for screen: {:?}", model.current_screen);
                    }
                }
            }
            None
        }
        
        Message::NavigateHome => {
            model.go_home();
            None
        }
        
        Message::PushScreen(screen) => {
            model.push_screen(screen);
            None
        }
        
        Message::PopScreen => {
            model.pop_screen();
            None
        }
        
        Message::ForceRemount => {
            // This message forces a remount of the current screen's components
            // Used when returning from sub-screens to ensure UI updates
            info!("ForceRemount triggered for screen: {:?}", model.current_screen);
            None
        }
        
        // ============= Wallet Management Messages =============
        Message::CreateWallet { config } => {
            info!("Creating wallet with config: {:?}", config);
            
            // Don't generate session ID here - wait for StartDKG command to generate the real DKG session ID
            // Use a placeholder for now that will be updated by UpdateDKGSessionId message
            let temp_session_id = "pending".to_string();
            info!("Starting DKG process - session ID will be generated by StartDKG command");
            
            // Initialize session state with current device as first participant
            let participants = vec![model.device_id.clone()];
            info!("Added current device as participant: {}", model.device_id);
            
            // Create active session with placeholder session ID
            model.active_session = Some(SessionInfo {
                session_id: temp_session_id.clone(),
                proposer_id: model.device_id.clone(),
                total: config.total_participants,
                threshold: config.threshold,
                participants: participants.clone(),
                session_type: SessionType::DKG,
                curve_type: format!("{:?}", config.curve),
                coordination_type: "online".to_string(),
            });
            
            // Navigate to DKG Progress screen with placeholder
            model.push_screen(Screen::DKGProgress { session_id: temp_session_id.clone() });
            
            // Show progress modal
            model.ui_state.modal = Some(Modal::Progress {
                title: "Creating Wallet".to_string(),
                message: "Initializing DKG protocol...".to_string(),
                progress: 0.0,
            });
            
            // Add to pending operations
            model.pending_operations.push(Operation::CreateWallet(config.clone()));
            
            // Start DKG process - this will generate the real session ID
            Some(Command::StartDKG { config })
        }
        
        Message::SelectWallet { wallet_id } => {
            info!("Selected wallet: {}", wallet_id);
            model.selected_wallet = Some(wallet_id.clone());
            
            // Navigate to wallet detail
            model.push_screen(Screen::WalletDetail { wallet_id: wallet_id.clone() });
            
            // Load wallet details
            Some(Command::LoadWalletDetails { wallet_id })
        }
        
        Message::ListWallets => {
            Some(Command::LoadWallets)
        }
        
        Message::WalletsLoaded { wallets } => {
            info!("Loaded {} wallets", wallets.len());
            let old_count = model.wallet_state.wallets.len();
            model.wallet_state.wallets = wallets;
            
            // If on main menu and wallet count changed, force remount to update menu
            if matches!(model.current_screen, Screen::MainMenu | Screen::Welcome) && 
               old_count != model.wallet_state.wallets.len() {
                info!("Wallet count changed from {} to {}, forcing menu update", 
                      old_count, model.wallet_state.wallets.len());
                Some(Command::SendMessage(Message::Refresh))
            } else {
                None
            }
        }
        
        Message::DeleteWallet { wallet_id } => {
            // Show confirmation modal
            model.ui_state.modal = Some(Modal::Confirm {
                title: "Delete Wallet".to_string(),
                message: format!("Are you sure you want to delete wallet '{}'? This action cannot be undone.", wallet_id),
                on_confirm: Box::new(Message::WalletDeleted { wallet_id: wallet_id.clone() }),
                on_cancel: Box::new(Message::CloseModal),
            });
            None
        }
        
        Message::WalletDeleted { wallet_id } => {
            info!("Deleting wallet: {}", wallet_id);
            model.ui_state.modal = None;
            Some(Command::DeleteWallet { wallet_id })
        }
        
        // ============= Wallet Creation Flow =============
        Message::SelectMode(mode) => {
            if let Screen::CreateWallet(ref mut state) = model.current_screen {
                state.mode = Some(mode);
                // Auto-navigate to next step
                model.push_screen(Screen::CurveSelection);
            }
            None
        }
        
        Message::SelectCurve(curve) => {
            if let Screen::CreateWallet(ref mut state) = model.current_screen {
                state.curve = Some(curve);
                // Auto-navigate to next step
                model.push_screen(Screen::TemplateSelection);
            }
            None
        }
        
        Message::SelectTemplate(template) => {
            if let Screen::CreateWallet(ref mut state) = model.current_screen {
                let is_custom = template.name == "Custom";
                state.template = Some(template);
                // Auto-navigate to configuration if custom, otherwise start DKG
                if is_custom {
                    model.push_screen(Screen::WalletConfiguration(Default::default()));
                } else {
                    // Start DKG with template configuration
                    return Some(Command::SendMessage(Message::ConfirmWalletCreation));
                }
            }
            None
        }
        
        // ============= DKG Operations =============
        Message::UpdateDKGSessionId { real_session_id } => {
            info!("Updating DKG session ID to real ID: {}", real_session_id);
            
            // Update the active session with the real DKG session ID
            if let Some(ref mut session) = model.active_session {
                session.session_id = real_session_id.clone();
            }
            
            // Update the screen to show the real session ID
            if let Screen::DKGProgress { ref mut session_id } = model.current_screen {
                *session_id = real_session_id.clone();
            }
            
            // Force a remount to update the display
            Some(Command::SendMessage(Message::ForceRemount))
        }
        
        Message::UpdateParticipants { participants } => {
            info!("Updating participants list: {:?}", participants);
            
            // Update the active session with the current participants
            if let Some(ref mut session) = model.active_session {
                session.participants = participants.clone();
                info!("Updated session participants to: {:?}", session.participants);
            }
            
            // Force a remount to update the display with new participants
            Some(Command::SendMessage(Message::ForceRemount))
        }
        
        Message::UpdateParticipantWebRTCStatus { device_id, webrtc_connected, data_channel_open } => {
            info!("Updating WebRTC status for {}: WebRTC={}, DataChannel={}",
                 device_id, webrtc_connected, data_channel_open);

            // Store the WebRTC status in the model's network state
            model.network_state.participant_webrtc_status
                .entry(device_id.clone())
                .and_modify(|status| {
                    status.0 = webrtc_connected;
                    status.1 = data_channel_open;
                })
                .or_insert((webrtc_connected, data_channel_open));

            // Force a remount to update the display with new WebRTC status
            if matches!(model.current_screen, Screen::DKGProgress { .. }) {
                Some(Command::SendMessage(Message::ForceRemount))
            } else {
                None
            }
        }

        Message::UpdateMeshStatus { ready_count, total_count, all_connected } => {
            info!("Mesh status update: {}/{} ready, all_connected={}",
                 ready_count, total_count, all_connected);

            // Force a remount to update the display
            if matches!(model.current_screen, Screen::DKGProgress { .. }) {
                Some(Command::SendMessage(Message::ForceRemount))
            } else {
                None
            }
        }

        Message::UpdateDKGProgress { round, progress } => {
            let message = match round {
                DKGRound::Initialization => "Initializing DKG protocol...",
                DKGRound::WaitingForParticipants => "Waiting for participants to join...",
                DKGRound::Round1 => "Round 1: Generating commitments...",
                DKGRound::Round2 => "Round 2: Distributing shares...",
                DKGRound::Finalization => "Finalizing wallet creation...",
            };
            
            model.ui_state.modal = Some(Modal::Progress {
                title: "DKG in Progress".to_string(),
                message: message.to_string(),
                progress,
            });
            
            None
        }
        
        Message::DKGComplete { result } => {
            info!("DKG completed successfully: {:?}", result);
            
            // Clear modal
            model.ui_state.modal = None;
            
            // Show success notification
            let notification = Notification {
                id: Uuid::new_v4().to_string(),
                text: format!("Wallet '{}' created successfully!", result.wallet_id),
                kind: NotificationKind::Success,
                timestamp: Utc::now(),
                dismissible: true,
            };
            model.ui_state.notifications.push(notification);
            
            // Navigate back to main menu to show updated menu with Sign Transaction
            model.go_home();
            
            // Reload wallet list which will trigger menu update
            Some(Command::LoadWallets)
        }
        
        Message::DKGFailed { error } => {
            error!("DKG failed: {}", error);
            
            // Show error modal but stay on DKG screen
            model.ui_state.modal = Some(Modal::Error {
                title: "DKG Failed".to_string(),
                message: error.clone(),
            });
            
            // Only navigate back if it's a critical failure (not just waiting for participants)
            if !error.contains("Need") && !error.contains("participants") {
                // Critical error - go back to main menu
                model.wallet_state.creating_wallet = None;
                model.navigation_stack.clear();
                model.current_screen = Screen::MainMenu;
                model.ui_state.focus = crate::elm::model::ComponentId::MainMenu;
                model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::MainMenu).or_insert(0);
            }
            // Otherwise stay on DKG screen to let user wait or manually cancel
            
            None
        }

        Message::InitiateDKG { params } => {
            info!("Initiating DKG with params: {:?}", params);

            // This message is triggered when WebRTC mesh is ready and we want to start the actual DKG
            // Simply forward to the StartDKG command which has the full implementation
            Some(Command::StartDKG { config: params.wallet_config })
        }

        // ============= Network Events =============
        Message::InitiateWebRTCWithParticipants { participants } => {
            info!("Initiating WebRTC connections with {} participants", participants.len());
            Some(Command::InitiateWebRTCConnections { participants })
        }
        
        Message::CheckWebRTCConnections => {
            info!("Checking WebRTC connection status");
            Some(Command::VerifyWebRTCMesh)
        }
        
        Message::VerifyMeshConnectivity => {
            info!("Verifying full mesh connectivity");
            Some(Command::EnsureFullMesh)
        }
        
        Message::WebSocketConnected => {
            info!("WebSocket connected");
            model.network_state.connected = true;
            model.network_state.connection_status = ConnectionStatus::Connected;
            model.network_state.reconnect_attempts = 0;
            
            // Show success notification
            let notification = Notification {
                id: Uuid::new_v4().to_string(),
                text: "Connected to network".to_string(),
                kind: NotificationKind::Success,
                timestamp: Utc::now(),
                dismissible: true,
            };
            model.ui_state.notifications.push(notification);
            
            // Force remount if we're on DKGProgress screen to update WebSocket status
            if matches!(model.current_screen, Screen::DKGProgress { .. }) {
                Some(Command::SendMessage(Message::ForceRemount))
            } else {
                None
            }
        }
        
        Message::WebSocketDisconnected => {
            warn!("WebSocket disconnected");
            model.network_state.connected = false;
            model.network_state.connection_status = ConnectionStatus::Disconnected;
            
            // Show warning notification
            let notification = Notification {
                id: Uuid::new_v4().to_string(),
                text: "Disconnected from network".to_string(),
                kind: NotificationKind::Warning,
                timestamp: Utc::now(),
                dismissible: true,
            };
            model.ui_state.notifications.push(notification);
            
            // Attempt reconnection
            Some(Command::ReconnectWebSocket)
        }
        
        // ============= UI Events =============
        Message::KeyPressed(key) => {
            // Global key handling
            match key.code {
                KeyCode::Esc => {
                    // ALWAYS navigate back, NEVER exit
                    if model.ui_state.modal.is_some() {
                        // Close modal first
                        model.ui_state.modal = None;
                        None
                    } else {
                        // Navigate back
                        Some(Command::SendMessage(Message::NavigateBack))
                    }
                }
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Only Ctrl+Q exits the application
                    Some(Command::SendMessage(Message::Quit))
                }
                KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Ctrl+H goes home
                    Some(Command::SendMessage(Message::NavigateHome))
                }
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    // Ctrl+R refreshes
                    Some(Command::SendMessage(Message::Refresh))
                }
                _ => {
                    // Delegate to focused component
                    None
                }
            }
        }
        
        Message::FocusChanged { component } => {
            model.ui_state.focus = component;
            None
        }
        
        Message::InputChanged { value } => {
            model.ui_state.input_buffer = value;
            None
        }
        
        Message::ScrollUp => {
            info!("⬆️ ScrollUp: current screen = {:?}", model.current_screen);
            // Update selected index based on current screen
            match model.current_screen {
                Screen::MainMenu | Screen::Welcome => {
                    let current_idx = model.ui_state.selected_indices
                        .entry(model.ui_state.focus.clone())
                        .or_insert(0);
                    
                    // Get menu item count based on wallet state
                    let menu_item_count = if model.wallet_state.wallets.is_empty() {
                        4  // Create, Join, Settings, Exit
                    } else {
                        6  // Create, Join, Manage, Sign, Settings, Exit
                    };
                    
                    // Wrap around
                    if *current_idx == 0 {
                        *current_idx = menu_item_count - 1;  // Last item index
                    } else {
                        *current_idx = current_idx.saturating_sub(1);
                    }
                    info!("MainMenu selection moved up to: {}", current_idx);
                }
                Screen::CreateWallet(_) => {
                    // Handle CreateWallet navigation
                    debug!("🔼 ScrollUp on CreateWallet, focus: {:?}", model.ui_state.focus);
                    debug!("🔼 Before: selected_indices = {:?}", model.ui_state.selected_indices);
                    
                    let current_idx = model.ui_state.selected_indices
                        .entry(model.ui_state.focus.clone())
                        .or_insert(0);
                    
                    let old_idx = *current_idx;
                    if *current_idx == 0 {
                        *current_idx = 3;  // Wrap to bottom (4 items: 0-3)
                    } else {
                        *current_idx = current_idx.saturating_sub(1);
                    }
                    info!("🔼 CreateWallet selection moved up: {} -> {}", old_idx, current_idx);
                    debug!("🔼 After: selected_indices = {:?}", model.ui_state.selected_indices);
                }
                Screen::ModeSelection => {
                    // ModeSelection doesn't respond to Up - only Left/Right
                    debug!("ModeSelection: Ignoring Up arrow (use Left/Right to switch modes)");
                }
                Screen::ThresholdConfig => {
                    // Get which field is selected (0 = participants, 1 = threshold)
                    let selected_field = model.ui_state.selected_indices
                        .get(&crate::elm::model::ComponentId::ThresholdConfig)
                        .copied()
                        .unwrap_or(0);
                    
                    // Ensure we have the creating_wallet state with custom_config
                    if let Some(ref mut creating_wallet) = model.wallet_state.creating_wallet {
                        // Initialize custom_config if not present
                        if creating_wallet.custom_config.is_none() {
                            creating_wallet.custom_config = Some(WalletConfig {
                                name: "MPC Wallet".to_string(),
                                total_participants: 3,
                                threshold: 2,
                                curve: creating_wallet.curve.clone().unwrap_or(CurveType::Secp256k1),
                                mode: creating_wallet.mode.clone().unwrap_or_default(),
                            });
                        }
                        
                        if let Some(ref mut config) = creating_wallet.custom_config {
                            if selected_field == 0 {
                                // Increase participants (max 10)
                                if config.total_participants < 10 {
                                    config.total_participants += 1;
                                    // Ensure threshold doesn't exceed participants
                                    config.threshold = config.threshold.min(config.total_participants);
                                    info!("ThresholdConfig: Participants increased to {}", config.total_participants);
                                }
                            } else {
                                // Increase threshold (max = participants)
                                if config.threshold < config.total_participants {
                                    config.threshold += 1;
                                    info!("ThresholdConfig: Threshold increased to {}", config.threshold);
                                }
                            }
                        }
                    }
                }
                Screen::JoinSession => {
                    // Handle JoinSession navigation for arrow up
                    let current_idx = model.ui_state.selected_indices
                        .entry(crate::elm::model::ComponentId::JoinSession)
                        .or_insert(0);
                    
                    if *current_idx > 0 {
                        *current_idx = current_idx.saturating_sub(1);
                    }
                    info!("JoinSession selection moved up to: {}", current_idx);
                }
                _ => {
                    model.ui_state.scroll_position = model.ui_state.scroll_position.saturating_sub(1);
                }
            }
            None
        }
        
        Message::ScrollDown => {
            info!("⬇️ ScrollDown: current screen = {:?}", model.current_screen);
            // Update selected index based on current screen
            match model.current_screen {
                Screen::MainMenu | Screen::Welcome => {
                    let current_idx = model.ui_state.selected_indices
                        .entry(model.ui_state.focus.clone())
                        .or_insert(0);
                    
                    // Get menu item count based on wallet state
                    let menu_item_count = if model.wallet_state.wallets.is_empty() {
                        4  // Create, Join, Settings, Exit
                    } else {
                        6  // Create, Join, Manage, Sign, Settings, Exit
                    };
                    
                    // Wrap around
                    if *current_idx >= menu_item_count - 1 {
                        *current_idx = 0; // Back to top
                    } else {
                        *current_idx += 1;
                    }
                    info!("MainMenu selection moved down to: {}", current_idx);
                }
                Screen::CreateWallet(_) => {
                    // Handle CreateWallet navigation
                    debug!("🔽 ScrollDown on CreateWallet, focus: {:?}", model.ui_state.focus);
                    debug!("🔽 Before: selected_indices = {:?}", model.ui_state.selected_indices);
                    
                    let current_idx = model.ui_state.selected_indices
                        .entry(model.ui_state.focus.clone())
                        .or_insert(0);
                    
                    let old_idx = *current_idx;
                    if *current_idx >= 3 {
                        *current_idx = 0;  // Wrap to top
                    } else {
                        *current_idx += 1;
                    }
                    info!("🔽 CreateWallet selection moved down: {} -> {}", old_idx, current_idx);
                    debug!("🔽 After: selected_indices = {:?}", model.ui_state.selected_indices);
                }
                Screen::ModeSelection => {
                    // ModeSelection doesn't respond to Down - only Left/Right
                    debug!("ModeSelection: Ignoring Down arrow (use Left/Right to switch modes)");
                }
                Screen::ThresholdConfig => {
                    // Get which field is selected (0 = participants, 1 = threshold)
                    let selected_field = model.ui_state.selected_indices
                        .get(&crate::elm::model::ComponentId::ThresholdConfig)
                        .copied()
                        .unwrap_or(0);
                    
                    // Ensure we have the creating_wallet state with custom_config
                    if let Some(ref mut creating_wallet) = model.wallet_state.creating_wallet {
                        // Initialize custom_config if not present
                        if creating_wallet.custom_config.is_none() {
                            creating_wallet.custom_config = Some(WalletConfig {
                                name: "MPC Wallet".to_string(),
                                total_participants: 3,
                                threshold: 2,
                                curve: creating_wallet.curve.clone().unwrap_or(CurveType::Secp256k1),
                                mode: creating_wallet.mode.clone().unwrap_or_default(),
                            });
                        }
                        
                        if let Some(ref mut config) = creating_wallet.custom_config {
                            if selected_field == 0 {
                                // Decrease participants (min 2)
                                if config.total_participants > 2 {
                                    config.total_participants -= 1;
                                    // Ensure threshold doesn't exceed participants
                                    config.threshold = config.threshold.min(config.total_participants);
                                    info!("ThresholdConfig: Participants decreased to {}", config.total_participants);
                                }
                            } else {
                                // Decrease threshold (min 2)
                                if config.threshold > 2 {
                                    config.threshold -= 1;
                                    info!("ThresholdConfig: Threshold decreased to {}", config.threshold);
                                }
                            }
                        }
                    }
                }
                Screen::JoinSession => {
                    // Handle JoinSession navigation for arrow down
                    // Note: The actual session count will be handled by the component itself
                    let current_idx = model.ui_state.selected_indices
                        .entry(crate::elm::model::ComponentId::JoinSession)
                        .or_insert(0);
                    
                    // We don't know the actual count here, just increment
                    *current_idx += 1;
                    info!("JoinSession selection moved down to: {}", current_idx);
                }
                _ => {
                    model.ui_state.scroll_position = model.ui_state.scroll_position.saturating_add(1);
                }
            }
            None
        }
        
        Message::ScrollLeft => {
            info!("⬅️ ScrollLeft: current screen = {:?}", model.current_screen);
            match model.current_screen {
                Screen::ModeSelection => {
                    // Switch to Online mode (left side)
                    let current_idx = model.ui_state.selected_indices
                        .entry(model.ui_state.focus.clone())
                        .or_insert(0);
                    *current_idx = 0;
                    info!("ModeSelection switched to: Online");
                }
                Screen::CurveSelection => {
                    // Switch to Secp256k1 (left side)
                    let current_idx = model.ui_state.selected_indices
                        .entry(model.ui_state.focus.clone())
                        .or_insert(0);
                    *current_idx = 0;
                    info!("CurveSelection switched to: Secp256k1");
                }
                Screen::ThresholdConfig => {
                    // Switch to participants field (left side) only if we're not already there
                    let current_idx = model.ui_state.selected_indices
                        .entry(crate::elm::model::ComponentId::ThresholdConfig)
                        .or_insert(0);
                    if *current_idx != 0 {
                        *current_idx = 0;
                        info!("ThresholdConfig switched to: Participants field");
                    } else {
                        debug!("Already on Participants field");
                    }
                }
                Screen::DKGProgress { .. } => {
                    // Switch between Cancel DKG and Copy Session ID buttons
                    let current_idx = model.ui_state.selected_indices
                        .entry(crate::elm::model::ComponentId::DKGProgress)
                        .or_insert(0);
                    if *current_idx > 0 {
                        *current_idx = 0;  // Switch to Cancel DKG
                        info!("DKGProgress switched to: Cancel DKG button");
                    } else {
                        debug!("Already on Cancel DKG button");
                    }
                }
                Screen::JoinSession => {
                    // Switch to DKG tab (left)
                    model.ui_state.join_session_tab = 0;
                    // Reset session selection when switching tabs
                    model.ui_state.selected_indices.insert(crate::elm::model::ComponentId::JoinSession, 0);
                    info!("JoinSession switched to DKG tab");
                }
                _ => {
                    debug!("ScrollLeft not handled for this screen");
                }
            }
            None
        }
        
        Message::ScrollRight => {
            info!("➡️ ScrollRight: current screen = {:?}", model.current_screen);
            match model.current_screen {
                Screen::ModeSelection => {
                    // Switch to Offline mode (right side)
                    let current_idx = model.ui_state.selected_indices
                        .entry(model.ui_state.focus.clone())
                        .or_insert(0);
                    *current_idx = 1;
                    info!("ModeSelection switched to: Offline");
                }
                Screen::CurveSelection => {
                    // Switch to Ed25519 (right side)
                    let current_idx = model.ui_state.selected_indices
                        .entry(model.ui_state.focus.clone())
                        .or_insert(0);
                    *current_idx = 1;
                    info!("CurveSelection switched to: Ed25519");
                }
                Screen::ThresholdConfig => {
                    // Switch to threshold field (right side) only if we're not already there
                    let current_idx = model.ui_state.selected_indices
                        .entry(crate::elm::model::ComponentId::ThresholdConfig)
                        .or_insert(0);
                    if *current_idx != 1 {
                        *current_idx = 1;
                        info!("ThresholdConfig switched to: Threshold field");
                    } else {
                        debug!("Already on Threshold field");
                    }
                }
                Screen::DKGProgress { .. } => {
                    // Switch between Cancel DKG and Copy Session ID buttons
                    let current_idx = model.ui_state.selected_indices
                        .entry(crate::elm::model::ComponentId::DKGProgress)
                        .or_insert(0);
                    if *current_idx < 1 {
                        *current_idx = 1;  // Switch to Copy Session ID
                        info!("DKGProgress switched to: Copy Session ID button");
                    } else {
                        debug!("Already on Copy Session ID button");
                    }
                }
                Screen::JoinSession => {
                    // Switch to Signing tab (right)
                    model.ui_state.join_session_tab = 1;
                    // Reset session selection when switching tabs
                    model.ui_state.selected_indices.insert(crate::elm::model::ComponentId::JoinSession, 0);
                    info!("JoinSession switched to Signing tab");
                }
                _ => {
                    debug!("ScrollRight not handled for this screen");
                }
            }
            None
        }
        
        Message::SelectItem { index: _ } => {
            info!("SelectItem on screen: {:?}", model.current_screen);
            // Handle item selection based on current screen
            match model.current_screen {
                Screen::MainMenu | Screen::Welcome => {
                    // Get the current selected index
                    let selected_idx = model.ui_state.selected_indices
                        .get(&model.ui_state.focus)
                        .copied()
                        .unwrap_or(0);
                    
                    info!("MainMenu item selected: {}", selected_idx);
                    
                    // Check if we have wallets (affects menu structure)
                    let has_wallets = !model.wallet_state.wallets.is_empty();
                    
                    // Navigate based on menu selection
                    // Menu structure when no wallets: Create, Join, Settings, Exit (4 items)
                    // Menu structure with wallets: Create, Join, Manage, Sign, Settings, Exit (6 items)
                    match (selected_idx, has_wallets) {
                        (0, _) => {
                            // Create New Wallet - go directly to Mode Selection
                            info!("Navigating directly to Mode Selection");
                            // IMPORTANT: Reset the creating_wallet state to start fresh
                            model.wallet_state.creating_wallet = None;
                            info!("Reset creating_wallet state to None for fresh start");
                            model.push_screen(Screen::ModeSelection);
                            // Set focus to ModeSelection component
                            model.ui_state.focus = crate::elm::model::ComponentId::ModeSelection;
                            model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::ModeSelection).or_insert(0);
                            debug!("🎯 Focus set to ModeSelection");
                            None
                        }
                        (1, _) => {
                            // Join Session (always second)
                            info!("Navigating to Join Session");
                            model.push_screen(Screen::JoinSession);
                            // Set focus to JoinSession component
                            model.ui_state.focus = crate::elm::model::ComponentId::JoinSession;
                            model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::JoinSession).or_insert(0);
                            debug!("🎯 Focus set to JoinSession");
                            Some(Command::LoadSessions)
                        }
                        (2, false) => {
                            // Settings (when no wallets)
                            info!("Navigating to Settings");
                            model.push_screen(Screen::Settings);
                            None
                        }
                        (2, true) => {
                            // Manage Wallets (when wallets exist)
                            info!("Navigating to Manage Wallets");
                            model.push_screen(Screen::ManageWallets);
                            Some(Command::LoadWallets)
                        }
                        (3, false) => {
                            // Exit (when no wallets)
                            info!("Exiting application");
                            Some(Command::Quit)
                        }
                        (3, true) => {
                            // Sign Transaction (when wallets exist)
                            if let Some(ref wallet_id) = model.selected_wallet {
                                info!("Navigating to Sign Transaction");
                                model.push_screen(Screen::SignTransaction { wallet_id: wallet_id.clone() });
                                None
                            } else {
                                // Need to select a wallet first
                                info!("Navigating to Manage Wallets for wallet selection");
                                model.push_screen(Screen::ManageWallets);
                                Some(Command::LoadWallets)
                            }
                        }
                        (4, true) => {
                            // Settings (when wallets exist)
                            info!("Navigating to Settings");
                            model.push_screen(Screen::Settings);
                            None
                        }
                        (5, true) => {
                            // Exit (when wallets exist)
                            info!("Exiting application");
                            Some(Command::Quit)
                        }
                        _ => None,
                    }
                }
                Screen::CreateWallet(_) => {
                    debug!("✅ SelectItem on CreateWallet screen");
                    debug!("Current focus: {:?}", model.ui_state.focus);
                    debug!("Selected indices: {:?}", model.ui_state.selected_indices);
                    
                    // Get the current selected index
                    let selected_idx = model.ui_state.selected_indices
                        .get(&model.ui_state.focus)
                        .copied()
                        .unwrap_or(0);
                    
                    info!("✅ CreateWallet item selected: {} (focus: {:?})", selected_idx, model.ui_state.focus);
                    
                    // Handle selection based on current option
                    match selected_idx {
                        0 => {
                            // Option 1: Choose Mode (Online/Offline)
                            // Always allow navigation to change mode
                            info!("Selected: Choose Mode - navigating to mode selection");
                            model.push_screen(Screen::ModeSelection);
                            model.ui_state.focus = crate::elm::model::ComponentId::ModeSelection;
                            model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::ModeSelection).or_insert(0);
                            None
                        }
                        1 => {
                            // Option 2: Select Curve (Secp256k1/Ed25519)
                            // Always allow navigation to change curve
                            info!("Selected: Select Curve - navigating to curve selection");
                            model.push_screen(Screen::CurveSelection);
                            model.ui_state.focus = crate::elm::model::ComponentId::CurveSelection;
                            model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::CurveSelection).or_insert(0);
                            None
                        }
                        2 => {
                            // Option 3: Configure Threshold
                            info!("Selected: Configure Threshold - navigating to threshold configuration");
                            model.push_screen(Screen::ThresholdConfig);
                            model.ui_state.focus = crate::elm::model::ComponentId::ThresholdConfig;
                            model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::ThresholdConfig).or_insert(0);
                            None
                        }
                        3 => {
                            // Option 4: Start DKG Process
                            info!("Selected: Start DKG Process - initiating DKG");
                            
                            // Use the wallet state if available, otherwise use defaults
                            let wallet_state = model.wallet_state.creating_wallet.as_ref();
                            let config = WalletConfig {
                                name: wallet_state
                                    .and_then(|s| s.template.as_ref())
                                    .map(|t| t.name.clone())
                                    .unwrap_or_else(|| "MPC Wallet".to_string()),
                                threshold: wallet_state
                                    .and_then(|s| s.template.as_ref())
                                    .map(|t| t.threshold)
                                    .unwrap_or(2),
                                total_participants: wallet_state
                                    .and_then(|s| s.template.as_ref())
                                    .map(|t| t.total_participants)
                                    .unwrap_or(3),
                                curve: wallet_state
                                    .and_then(|s| s.curve.clone())
                                    .unwrap_or(CurveType::Secp256k1),
                                mode: wallet_state
                                    .and_then(|s| s.mode.clone())
                                    .unwrap_or(WalletMode::Online),
                            };
                            Some(Command::SendMessage(Message::CreateWallet { config }))
                        }
                        _ => {
                            debug!("Invalid selection index: {}", selected_idx);
                            None
                        }
                    }
                }
                Screen::ModeSelection => {
                    // Get the current selected mode (0 = Online, 1 = Offline)
                    let selected_mode = model.ui_state.selected_indices
                        .get(&model.ui_state.focus)
                        .copied()
                        .unwrap_or(0);
                    
                    info!("ModeSelection confirmed: {}", if selected_mode == 0 { "Online" } else { "Offline" });
                    
                    // Initialize creating_wallet if needed
                    if model.wallet_state.creating_wallet.is_none() {
                        model.wallet_state.creating_wallet = Some(CreateWalletState::default());
                    }
                    
                    // Update the create wallet state with the selected mode
                    if let Some(ref mut state) = model.wallet_state.creating_wallet {
                        state.mode = Some(if selected_mode == 0 {
                            WalletMode::Online
                        } else {
                            WalletMode::Offline
                        });
                    }
                    
                    // Navigate to Curve Selection screen
                    info!("Mode selected, navigating to Curve Selection");
                    model.push_screen(Screen::CurveSelection);
                    model.ui_state.focus = crate::elm::model::ComponentId::CurveSelection;
                    model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::CurveSelection).or_insert(0);
                    
                    None
                    
                }
                Screen::CurveSelection => {
                    // Get the current selected curve (0 = Secp256k1, 1 = Ed25519)
                    let selected_curve = model.ui_state.selected_indices
                        .get(&model.ui_state.focus)
                        .copied()
                        .unwrap_or(0);
                    
                    info!("CurveSelection confirmed: {}", if selected_curve == 0 { "Secp256k1" } else { "Ed25519" });
                    
                    // Initialize creating_wallet if needed
                    if model.wallet_state.creating_wallet.is_none() {
                        model.wallet_state.creating_wallet = Some(CreateWalletState::default());
                    }
                    
                    // Update the create wallet state with the selected curve
                    if let Some(ref mut state) = model.wallet_state.creating_wallet {
                        state.curve = Some(if selected_curve == 0 {
                            CurveType::Secp256k1
                        } else {
                            CurveType::Ed25519
                        });
                    }
                    
                    // Navigate to Threshold Configuration screen
                    info!("Curve selected, navigating to Threshold Configuration");
                    model.push_screen(Screen::ThresholdConfig);
                    model.ui_state.focus = crate::elm::model::ComponentId::ThresholdConfig;
                    model.ui_state.selected_indices.entry(crate::elm::model::ComponentId::ThresholdConfig).or_insert(0);
                    
                    None
                }
                Screen::ThresholdConfig => {
                    // Get the threshold configuration and start DKG
                    info!("ThresholdConfig confirmed - starting DKG process");
                    
                    // Get the config from custom_config or use defaults
                    let config = if let Some(ref creating_wallet) = model.wallet_state.creating_wallet {
                        if let Some(ref custom_config) = creating_wallet.custom_config {
                            // Use the config that was modified by the arrow keys
                            custom_config.clone()
                        } else {
                            // Fallback to defaults if custom_config wasn't initialized
                            WalletConfig {
                                name: "MPC Wallet".to_string(),
                                threshold: 2,
                                total_participants: 3,
                                curve: creating_wallet.curve.clone().unwrap_or(CurveType::Secp256k1),
                                mode: creating_wallet.mode.clone().unwrap_or(WalletMode::Online),
                            }
                        }
                    } else {
                        // Complete fallback
                        WalletConfig {
                            name: "MPC Wallet".to_string(),
                            threshold: 2,
                            total_participants: 3,
                            curve: CurveType::Secp256k1,
                            mode: WalletMode::Online,
                        }
                    };
                    
                    info!("Starting DKG with config: {:?}", config);
                    Some(Command::SendMessage(Message::CreateWallet { config }))
                }
                Screen::JoinSession => {
                    // Get the selected session index from the JoinSession component
                    let selected_idx = model.ui_state.selected_indices
                        .get(&crate::elm::model::ComponentId::JoinSession)
                        .copied()
                        .unwrap_or(0);
                    
                    let selected_tab = model.ui_state.join_session_tab;
                    info!("JoinSession: Selected session index: {}, tab: {}", selected_idx, 
                          if selected_tab == 0 { "DKG" } else { "Signing" });
                    
                    // Filter sessions by tab type, just like the component does
                    let filtered_sessions: Vec<_> = model.session_invites
                        .iter()
                        .filter(|s| {
                            if selected_tab == 0 {
                                // DKG tab
                                matches!(s.session_type, SessionType::DKG)
                            } else {
                                // Signing tab
                                matches!(s.session_type, SessionType::Signing { .. })
                            }
                        })
                        .cloned()
                        .collect();
                    
                    // Get the session from the filtered list
                    if let Some(session) = filtered_sessions.get(selected_idx).cloned() {
                        info!("Joining DKG session: {}", session.session_id);
                        
                        let session_id = session.session_id.clone();
                        
                        // Set up the session state
                        model.active_session = Some(session);
                        
                        // Navigate to DKG Progress screen
                        model.push_screen(Screen::DKGProgress { session_id: session_id.clone() });
                        
                        // Start joining the DKG session
                        Some(Command::JoinDKG { session_id })
                    } else {
                        warn!("No session available at index {}", selected_idx);
                        None
                    }
                }
                _ => None,
            }
        }
        
        // ============= Modal Management =============
        Message::ShowModal(modal) => {
            model.ui_state.modal = Some(modal);
            None
        }
        
        Message::CloseModal => {
            model.ui_state.modal = None;
            None
        }
        
        Message::ConfirmModal => {
            if let Some(Modal::Confirm { on_confirm, .. }) = &model.ui_state.modal {
                let msg = *on_confirm.clone();
                model.ui_state.modal = None;
                Some(Command::SendMessage(msg))
            } else {
                model.ui_state.modal = None;
                None
            }
        }
        
        Message::CancelModal => {
            if let Some(Modal::Confirm { on_cancel, .. }) = &model.ui_state.modal {
                let msg = *on_cancel.clone();
                model.ui_state.modal = None;
                Some(Command::SendMessage(msg))
            } else {
                model.ui_state.modal = None;
                None
            }
        }
        
        // ============= Notifications =============
        Message::ShowNotification { text, kind } => {
            let notification = Notification {
                id: Uuid::new_v4().to_string(),
                text,
                kind,
                timestamp: Utc::now(),
                dismissible: true,
            };
            
            // Clone the id before moving notification
            let id = notification.id.clone();
            model.ui_state.notifications.push(notification);
            
            // Auto-dismiss after 5 seconds
            Some(Command::ScheduleMessage {
                delay_ms: 5000,
                message: Box::new(Message::ClearNotification { id }),
            })
        }
        
        Message::ClearNotification { id } => {
            model.ui_state.notifications.retain(|n| n.id != id);
            None
        }
        
        // ============= Progress Updates =============
        Message::StartProgress { operation, message } => {
            model.ui_state.progress = Some(ProgressInfo {
                operation,
                progress: 0.0,
                message,
                started_at: Utc::now(),
                estimated_completion: None,
            });
            None
        }
        
        Message::UpdateProgress { progress, message } => {
            if let Some(ref mut info) = model.ui_state.progress {
                info.progress = progress;
                if let Some(msg) = message {
                    info.message = msg;
                }
            }
            None
        }
        
        Message::CompleteProgress => {
            model.ui_state.progress = None;
            None
        }
        
        // ============= System Messages =============
        Message::Initialize => {
            info!("Initializing application");
            
            // Initialize keystore
            let keystore_path = format!("{}/.frost_keystore", 
                std::env::var("HOME").unwrap_or_else(|_| ".".to_string()));
            
            Some(Command::InitializeKeystore { 
                path: keystore_path,
                device_id: model.device_id.clone(),
            })
        }
        
        Message::Quit => {
            info!("Quitting application");
            Some(Command::Quit)
        }
        
        Message::Refresh => {
            info!("Refreshing UI");
            Some(Command::RefreshUI)
        }
        
        Message::Error { message } => {
            error!("Error: {}", message);
            model.ui_state.error_message = Some(message.clone());
            
            let notification = Notification {
                id: Uuid::new_v4().to_string(),
                text: message,
                kind: NotificationKind::Error,
                timestamp: Utc::now(),
                dismissible: true,
            };
            model.ui_state.notifications.push(notification);
            
            None
        }
        
        Message::Success { message } => {
            info!("Success: {}", message);
            model.ui_state.success_message = Some(message.clone());
            
            let notification = Notification {
                id: Uuid::new_v4().to_string(),
                text: message,
                kind: NotificationKind::Success,
                timestamp: Utc::now(),
                dismissible: true,
            };
            model.ui_state.notifications.push(notification);
            
            None
        }
        
        // ============= Keystore Events =============
        Message::KeystoreInitialized { path } => {
            info!("Keystore initialized at: {}", path);
            model.wallet_state.keystore_initialized = true;
            model.wallet_state.keystore_path = path;
            
            // Load wallets after initialization
            Some(Command::LoadWallets)
        }
        
        Message::KeystoreError { error } => {
            error!("Keystore error: {}", error);
            Some(Command::SendMessage(Message::Error { 
                message: format!("Keystore error: {}", error) 
            }))
        }
        
        // ============= Session Discovery Events =============
        Message::SessionsLoaded { sessions } => {
            info!("Loaded {} sessions from discovery", sessions.len());
            // Store the discovered sessions
            model.session_invites = sessions.clone();
            
            // Log session details for debugging
            for session in &sessions {
                info!("Session discovered: {} ({}/{})", session.session_id, session.threshold, session.total);
            }
            
            // Force UI update if we're on the JoinSession screen
            if matches!(model.current_screen, Screen::JoinSession) {
                info!("On JoinSession screen, forcing remount to update session list");
                Some(Command::SendMessage(Message::ForceRemount))
            } else {
                None
            }
        }
        
        // ============= Default =============
        _ => {
            debug!("Unhandled message: {:?}", msg);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elm::model::WalletMode;
    use crossterm::event::KeyEvent;
    
    #[test]
    fn test_navigate_back() {
        let mut model = Model::new("test".to_string());
        model.current_screen = Screen::MainMenu;
        
        // Navigate to wallet list
        update(&mut model, Message::Navigate(Screen::ManageWallets));
        assert_eq!(model.current_screen, Screen::ManageWallets);
        assert_eq!(model.navigation_stack.len(), 1);
        
        // Navigate back
        update(&mut model, Message::NavigateBack);
        assert_eq!(model.current_screen, Screen::MainMenu);
        assert_eq!(model.navigation_stack.len(), 0);
    }
    
    #[test]
    fn test_esc_key_never_exits() {
        let mut model = Model::new("test".to_string());
        model.current_screen = Screen::ManageWallets;
        model.navigation_stack.push(Screen::MainMenu);
        
        // Press Esc
        let cmd = update(&mut model, Message::KeyPressed(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }));
        
        // Should return NavigateBack command, never quit
        assert!(matches!(cmd, Some(Command::SendMessage(Message::NavigateBack))));
    }
    
    #[test]
    fn test_modal_closes_on_esc() {
        let mut model = Model::new("test".to_string());
        model.ui_state.modal = Some(Modal::Error {
            title: "Test".to_string(),
            message: "Test error".to_string(),
        });
        
        // Press Esc with modal open
        let cmd = update(&mut model, Message::KeyPressed(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        }));
        
        // Modal should be closed, no navigation
        assert!(model.ui_state.modal.is_none());
        assert!(cmd.is_none());
    }
}