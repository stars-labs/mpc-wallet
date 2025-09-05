//! Update - The state transition function
//!
//! The update function is the heart of the Elm Architecture. It takes the current
//! model and a message, and returns an updated model along with optional commands
//! to execute side effects.

use crate::elm::model::{Model, Screen, Modal, Notification, NotificationKind, ConnectionStatus, Operation, ProgressInfo, WalletConfig, CurveType, WalletMode};
use crate::elm::message::{Message, DKGRound};
use crate::elm::command::Command;
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
                Screen::ManageWallets => {
                    model.ui_state.focus = crate::elm::model::ComponentId::WalletList;
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
        
        // ============= Wallet Management Messages =============
        Message::CreateWallet { config } => {
            info!("Creating wallet with config: {:?}", config);
            
            // Show progress modal
            model.ui_state.modal = Some(Modal::Progress {
                title: "Creating Wallet".to_string(),
                message: "Initializing DKG protocol...".to_string(),
                progress: 0.0,
            });
            
            // Add to pending operations
            model.pending_operations.push(Operation::CreateWallet(config.clone()));
            
            // Start DKG process
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
        Message::UpdateDKGProgress { round, progress } => {
            let message = match round {
                DKGRound::Initialization => "Initializing DKG protocol...",
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
            
            model.ui_state.modal = Some(Modal::Error {
                title: "DKG Failed".to_string(),
                message: error,
            });
            
            None
        }
        
        // ============= Network Events =============
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
            
            None
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
                _ => {
                    model.ui_state.scroll_position = model.ui_state.scroll_position.saturating_add(1);
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
                            // Create New Wallet (always first)
                            info!("Navigating to Create Wallet");
                            model.push_screen(Screen::CreateWallet(Default::default()));
                            None
                        }
                        (1, _) => {
                            // Join Session (always second)
                            info!("Navigating to Join Session");
                            model.push_screen(Screen::JoinSession);
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
                            info!("Selected: Choose Mode - navigating to mode selection");
                            model.push_screen(Screen::ModeSelection);
                            None
                        }
                        1 => {
                            // Option 2: Select Curve (Secp256k1/Ed25519)  
                            info!("Selected: Select Curve - navigating to curve selection");
                            model.push_screen(Screen::CurveSelection);
                            None
                        }
                        2 => {
                            // Option 3: Configure Threshold
                            info!("Selected: Configure Threshold - navigating to threshold configuration");
                            model.push_screen(Screen::ThresholdConfig);
                            None
                        }
                        3 => {
                            // Option 4: Start DKG Process
                            info!("Selected: Start DKG Process - initiating DKG");
                            // For now, create a basic wallet config and start DKG
                            let config = WalletConfig {
                                name: "Test Wallet".to_string(),
                                threshold: 2,
                                total_participants: 3,
                                curve: CurveType::Secp256k1,
                                mode: WalletMode::Online,
                            };
                            Some(Command::SendMessage(Message::CreateWallet { config }))
                        }
                        _ => {
                            debug!("Invalid selection index: {}", selected_idx);
                            None
                        }
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